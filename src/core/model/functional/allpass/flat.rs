use std::error::Error;

use ndarray::Dim;

use serde::{Deserialize, Serialize};

use crate::core::{config::model::Model, model::spatial::SpatialDescription};

use super::{
    delay::calculate_delay_samples_array,
    from_samples_to_coef, from_samples_to_usize,
    shapes::{
        flat::{ArrayDelaysFlat, ArrayGainsFlat, ArrayIndicesGainsFlat},
        ArrayActivationTime,
    },
};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct APParametersFlat {
    pub gains: ArrayGainsFlat<f32>,
    pub output_state_indices: ArrayIndicesGainsFlat,
    pub coefs: ArrayDelaysFlat<f32>,
    pub delays: ArrayDelaysFlat<usize>,
    pub activation_time_ms: ArrayActivationTime,
}

impl APParametersFlat {
    #[must_use]
    pub fn empty(number_of_states: usize, voxels_in_dims: Dim<[usize; 3]>) -> Self {
        Self {
            gains: ArrayGainsFlat::empty(number_of_states),
            output_state_indices: ArrayIndicesGainsFlat::empty(number_of_states),
            coefs: ArrayDelaysFlat::empty(number_of_states),
            delays: ArrayDelaysFlat::empty(number_of_states),
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

fn init_output_state_indicies(_spatial_description: &SpatialDescription) -> ArrayIndicesGainsFlat {
    todo!()
}

fn connect_voxels(
    _spatial_description: &SpatialDescription,
    _config: &Model,
    _ap_params: &mut APParametersFlat,
) {
    todo!()
}
