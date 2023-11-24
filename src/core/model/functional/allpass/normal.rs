use std::error::Error;

use approx::relative_eq;

use itertools::Itertools;
use ndarray::{arr1, s, Array1, Array3, Array4, Dim};
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};

use crate::core::{
    config::model::Model,
    model::spatial::{voxels, voxels::VoxelType, SpatialDescription},
};

use super::{
    delay::{self, calculate_delay_samples_array},
    direction, find_candidate_voxels, from_samples_to_coef, from_samples_to_usize, gain,
    shapes::{
        normal::{ArrayDelaysNormal, ArrayGainsNormal, ArrayIndicesGainsNormal},
        ArrayActivationTime,
    },
};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct APParametersNormal {
    pub gains: ArrayGainsNormal<f32>,
    pub output_state_indices: ArrayIndicesGainsNormal,
    pub coefs: ArrayDelaysNormal<f32>,
    pub delays: ArrayDelaysNormal<usize>,
    pub activation_time_ms: ArrayActivationTime,
}

impl APParametersNormal {
    #[must_use]
    pub fn empty(number_of_states: usize, voxels_in_dims: Dim<[usize; 3]>) -> Self {
        Self {
            gains: ArrayGainsNormal::empty(number_of_states),
            output_state_indices: ArrayIndicesGainsNormal::empty(number_of_states),
            coefs: ArrayDelaysNormal::empty(number_of_states),
            delays: ArrayDelaysNormal::empty(number_of_states),
            activation_time_ms: ArrayActivationTime::empty(voxels_in_dims),
        }
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if model cant be build with
    /// given config.
    pub fn from_model_config(
        config: &Model,
        spatial_description: &SpatialDescription,
        sample_rate_hz: f32,
    ) -> Result<Self, Box<dyn Error>> {
        let mut ap_params = Self::empty(
            spatial_description.voxels.count_states(),
            spatial_description.voxels.types.values.raw_dim(),
        );

        connect_voxels(spatial_description, config, &mut ap_params);

        let delays_samples = calculate_delay_samples_array(
            spatial_description,
            &config.propagation_velocities_m_per_s,
            sample_rate_hz,
        )?;

        ap_params.output_state_indices = init_output_state_indicies(spatial_description);

        ap_params
            .delays
            .values
            .iter_mut()
            .zip(delays_samples.values.iter())
            .for_each(|(delay, samples)| *delay = from_samples_to_usize(*samples));

        ap_params
            .coefs
            .values
            .iter_mut()
            .zip(delays_samples.values.iter())
            .for_each(|(coef, samples)| *coef = from_samples_to_coef(*samples));

        Ok(ap_params)
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        let path = &path.join("allpass");
        self.gains.save_npy(path, "gains.npy");
        self.output_state_indices.save_npy(path);
        self.coefs.save_npy(path);
        self.delays.save_npy(path);
        self.activation_time_ms.save_npy(path);
    }
}

fn connect_voxels(
    spatial_description: &SpatialDescription,
    config: &Model,
    ap_params: &mut APParametersNormal,
) {
    let mut activation_time_s =
        Array3::<Option<f32>>::from_elem(spatial_description.voxels.types.values.raw_dim(), None);
    let mut current_directions =
        Array4::<f32>::zeros(spatial_description.voxels.positions_mm.values.raw_dim());

    let v_types = &spatial_description.voxels.types.values;

    let mut current_time_s: f32 = 0.0;
    // Handle Sinoatrial node
    v_types
        .indexed_iter()
        .filter(|(_, v_type)| **v_type == VoxelType::Sinoatrial)
        .for_each(|(index, _)| {
            activation_time_s[index] = Some(current_time_s);
            current_directions
                .slice_mut(s![index.0, index.1, index.2, ..])
                .assign(&arr1(&[1.0, 0.0, 0.0]));
        });
    let mut connected_something = true;

    while connected_something {
        // reset the connected something variable so we don't get stuck here forever
        // have to check the activation times because there might be come connection possible
        // with a voxel that is not yet activated.
        if !activation_time_s
            .iter()
            .filter(|time_s| time_s.is_some())
            .any(|time_s| time_s.unwrap() > current_time_s)
        {
            connected_something = false;
        }
        // find all voxels with an activation time equal to the current time
        // i.e., currently activated voxels
        let output_voxel_indices = find_candidate_voxels(&activation_time_s, current_time_s);

        for output_voxel_index in output_voxel_indices {
            for x_offset in -1..=1 {
                for y_offset in -1..=1 {
                    for z_offset in -1..=1 {
                        connected_something |= try_to_connect(
                            (x_offset, y_offset, z_offset),
                            output_voxel_index,
                            spatial_description,
                            &mut activation_time_s,
                            config,
                            &mut current_directions,
                            ap_params,
                        );
                    }
                }
            }
        }
        let candidate_times_s: Vec<f32> = activation_time_s
            .iter()
            .filter(|t| t.is_some() && t.unwrap() > current_time_s)
            .map(|t| t.unwrap())
            .collect();
        let candidate_times_s = Array1::from_vec(candidate_times_s);
        current_time_s = *candidate_times_s.min_skipnan();
    }
    ap_params
        .activation_time_ms
        .values
        .iter_mut()
        .zip(activation_time_s)
        .filter(|(_, s)| s.is_some())
        .for_each(|(ms, s)| *ms = Some(s.unwrap() * 1000.0));
}

fn try_to_connect(
    voxel_offset: (i32, i32, i32),
    output_voxel_index: (usize, usize, usize),
    spatial_description: &SpatialDescription,
    activation_time_s: &mut ndarray::ArrayBase<ndarray::OwnedRepr<Option<f32>>, Dim<[usize; 3]>>,
    config: &Model,
    current_directions: &mut ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 4]>>,
    ap_params: &mut APParametersNormal,
) -> bool {
    let v_types = &spatial_description.voxels.types.values;
    let v_position_mm = &spatial_description.voxels.positions_mm.values;
    let v_numbers = &spatial_description.voxels.numbers.values;
    let (x_offset, y_offset, z_offset) = voxel_offset;

    // no self connection allowed
    if x_offset == 0 && y_offset == 0 && z_offset == 0 {
        return false;
    }
    let (x_out, y_out, z_out) = output_voxel_index;
    let input_voxel_index = [
        i32::try_from(x_out).unwrap() - x_offset,
        i32::try_from(y_out).unwrap() - y_offset,
        i32::try_from(z_out).unwrap() - z_offset,
    ];
    // Skip if the input voxel doesn't exist
    if !spatial_description.voxels.is_valid_index(input_voxel_index) {
        return false;
    }
    let input_voxel_index = [
        usize::try_from(i32::try_from(x_out).unwrap() - x_offset).unwrap(),
        usize::try_from(i32::try_from(y_out).unwrap() - y_offset).unwrap(),
        usize::try_from(i32::try_from(z_out).unwrap() - z_offset).unwrap(),
    ];
    // SKip if the input voxel is already connected
    if activation_time_s[input_voxel_index].is_some() {
        return false;
    }
    let output_voxel_type = &v_types[output_voxel_index];
    let input_voxel_type = &v_types[input_voxel_index];
    // Skip if connection is not alowed
    if !voxels::is_connection_allowed(output_voxel_type, input_voxel_type) {
        return false;
    }
    // Skip pathologies if the propagation factor is zero
    if input_voxel_type == &VoxelType::Pathological
        && relative_eq!(config.current_factor_in_pathology, 0.0)
    {
        return false;
    }
    // Now we finally found something that we want to connect.
    let input_state_number = v_numbers[input_voxel_index].unwrap();
    let output_position_mm = &v_position_mm.slice(s![x_out, y_out, z_out, ..]);
    let [x_in, y_in, z_in] = input_voxel_index;
    let input_position_mm = &v_position_mm.slice(s![x_in, y_in, z_in, ..]);
    let propagation_velocity_m_per_s = config
        .propagation_velocities_m_per_s
        .get(input_voxel_type)
        .unwrap();
    let delay_s = delay::calculate_delay_s(
        input_position_mm,
        output_position_mm,
        *propagation_velocity_m_per_s,
    );
    // update activation time of input voxel, marking them as connected
    activation_time_s[input_voxel_index] =
        Some(activation_time_s[output_voxel_index].unwrap() + delay_s);
    let direction = direction::calculate(input_position_mm, output_position_mm);
    current_directions
        .slice_mut(s![x_in, y_in, z_in, ..])
        .assign(&direction);
    let mut gain = gain::calculate(
        &direction,
        current_directions.slice(s![x_out, y_out, z_out, ..]),
    );
    if *input_voxel_type == VoxelType::Pathological && *output_voxel_type != VoxelType::Pathological
    {
        gain *= config.current_factor_in_pathology;
    }
    if *output_voxel_type == VoxelType::Pathological && *input_voxel_type != VoxelType::Pathological
    {
        gain *= 1.0 / config.current_factor_in_pathology;
    }
    assign_gain(
        ap_params,
        input_state_number,
        x_offset,
        y_offset,
        z_offset,
        &gain,
    );
    true
}

fn assign_gain(
    ap_params: &mut APParametersNormal,
    input_state_number: usize,
    x_offset: i32,
    y_offset: i32,
    z_offset: i32,
    gain: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>>,
) {
    for input_dimension in 0..3 {
        for output_dimension in 0..3 {
            ap_params.gains.values[(
                input_state_number + input_dimension,
                usize::try_from(1 + x_offset).unwrap(),
                usize::try_from(1 + y_offset).unwrap(),
                usize::try_from(1 + z_offset).unwrap(),
                output_dimension,
            )] = gain[(input_dimension, output_dimension)];
        }
    }
}

fn init_output_state_indicies(spatial_description: &SpatialDescription) -> ArrayIndicesGainsNormal {
    let mut output_state_indices =
        ArrayIndicesGainsNormal::empty(spatial_description.voxels.count_states());
    let v_types = &spatial_description.voxels.types.values;
    let v_numbers = &spatial_description.voxels.numbers.values;
    // TODO: write tests
    v_types
        .indexed_iter()
        .filter(|(_, v_type)| **v_type != VoxelType::None)
        .for_each(|(input_voxel_index, _)| {
            let (x_in, y_in, z_in) = input_voxel_index;
            for ((x_offset, y_offset), z_offset) in
                (-1..=1).cartesian_product(-1..=1).cartesian_product(-1..=1)
            {
                let ouput_voxel_index_candidate = [
                    i32::try_from(x_in).unwrap() + x_offset,
                    i32::try_from(y_in).unwrap() + y_offset,
                    i32::try_from(z_in).unwrap() + z_offset,
                ];
                if !spatial_description
                    .voxels
                    .is_valid_index(ouput_voxel_index_candidate)
                {
                    continue;
                }
                let output_voxel_index = [
                    usize::try_from(ouput_voxel_index_candidate[0]).unwrap(),
                    usize::try_from(ouput_voxel_index_candidate[1]).unwrap(),
                    usize::try_from(ouput_voxel_index_candidate[2]).unwrap(),
                ];
                for input_direction in 0..3 {
                    let input_state_number =
                        v_numbers[input_voxel_index].unwrap() + input_direction;
                    for output_direction in 0..3 {
                        let output_state_index =
                            v_numbers[output_voxel_index].unwrap() + output_direction;
                        output_state_indices.values[(
                            input_state_number,
                            usize::try_from(1 + x_offset).unwrap(),
                            usize::try_from(1 + y_offset).unwrap(),
                            usize::try_from(1 + z_offset).unwrap(),
                            output_direction,
                        )] = Some(output_state_index);
                    }
                }
            }
        });
    output_state_indices
}

#[cfg(test)]
mod test {

    use crate::{
        core::{
            config::model::Model,
            model::spatial::{voxels::VoxelType, SpatialDescription},
        },
        vis::plotting::matrix::plot_activation_time,
    };

    use super::{init_output_state_indicies, APParametersNormal};

    #[test]
    fn activation_time_is_some() {
        let config = &Model::default();
        let spatial_description = &SpatialDescription::from_model_config(config);
        let sample_rate_hz = 2000.0;
        let ap_params =
            APParametersNormal::from_model_config(config, spatial_description, sample_rate_hz)
                .unwrap();

        for (index, activation_time_ms) in ap_params.activation_time_ms.values.indexed_iter() {
            assert!(
                activation_time_ms.is_some(),
                "Activation time at {index:?} was none."
            );
            assert!(activation_time_ms.unwrap() >= 0.0);
        }
    }

    #[test]
    #[ignore]
    fn activation_time_is_some_and_plot() {
        let config = &Model::default();
        let spatial_description = &SpatialDescription::from_model_config(config);
        let sample_rate_hz = 2000.0;
        let ap_params =
            APParametersNormal::from_model_config(config, spatial_description, sample_rate_hz)
                .unwrap();

        for (index, activation_time_ms) in ap_params.activation_time_ms.values.indexed_iter() {
            assert!(
                activation_time_ms.is_some(),
                "Activation time at {index:?} was none."
            );
            assert!(activation_time_ms.unwrap() >= 0.0);
        }
        plot_activation_time(
            &ap_params.activation_time_ms,
            "tests/ap_params_activation_times_default",
            "Activation times [ms]",
        );
    }

    #[test]
    fn activation_time_fast_av() {
        let mut config = Model::default();
        config
            .propagation_velocities_m_per_s
            .insert(VoxelType::Atrioventricular, 0.8);
        let spatial_description = &SpatialDescription::from_model_config(&config);
        let sample_rate_hz = 2000.0;
        let ap_params =
            APParametersNormal::from_model_config(&config, spatial_description, sample_rate_hz)
                .unwrap();

        for (index, activation_time_ms) in ap_params.activation_time_ms.values.indexed_iter() {
            assert!(
                activation_time_ms.is_some(),
                "Activation time at {index:?} was none."
            );
            assert!(activation_time_ms.unwrap() >= 0.0);
        }
    }

    #[test]
    #[ignore]
    fn activation_time_fast_av_and_plot() {
        let mut config = Model::default();
        config
            .propagation_velocities_m_per_s
            .insert(VoxelType::Atrioventricular, 0.8);
        let spatial_description = &SpatialDescription::from_model_config(&config);
        let sample_rate_hz = 2000.0;
        let ap_params =
            APParametersNormal::from_model_config(&config, spatial_description, sample_rate_hz)
                .unwrap();

        for (index, activation_time_ms) in ap_params.activation_time_ms.values.indexed_iter() {
            assert!(
                activation_time_ms.is_some(),
                "Activation time at {index:?} was none."
            );
            assert!(activation_time_ms.unwrap() >= 0.0);
        }
        plot_activation_time(
            &ap_params.activation_time_ms,
            "tests/ap_params_activation_times_fast_av",
            "Activation times [ms]",
        );
    }

    #[test]
    fn init_output_state_indicies_works() {
        let config = &Model::default();
        let spatial_description = &SpatialDescription::from_model_config(config);
        let output_state_indicies = init_output_state_indicies(spatial_description);

        assert!(output_state_indicies.values[(0, 1, 1, 1, 0)].unwrap() == 0);
        assert!(output_state_indicies.values[(0, 1, 1, 1, 1)].unwrap() == 1);
        assert!(output_state_indicies.values[(0, 1, 1, 1, 2)].unwrap() == 2);
        assert!(output_state_indicies.values[(0, 1, 1, 0, 0)].is_none());
    }
}
