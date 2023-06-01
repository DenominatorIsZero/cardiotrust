pub mod shapes;

use ndarray::{Array1, Array2};

use self::shapes::{
    ArrayControlFunction, ArrayCtlMat, ArrayDelays, ArrayGains, ArrayIndicesGains, ArrayKalmanGain,
    ArrayMeasMat,
};

use super::config::simulation::Simulation;

#[derive(Debug, PartialEq)]
pub struct APParameters {
    pub gains: ArrayGains<f32>,
    pub output_state_indices: ArrayIndicesGains,
    pub coefs: ArrayDelays<f32>,
    pub delays: ArrayDelays<usize>,
}

impl APParameters {
    pub fn empty(number_of_states: usize) -> APParameters {
        APParameters {
            gains: ArrayGains::empty(number_of_states),
            output_state_indices: ArrayIndicesGains::empty(number_of_states),
            coefs: ArrayDelays::empty(number_of_states),
            delays: ArrayDelays::empty(number_of_states),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionalDescription {
    pub ap_params: APParameters,
    pub measurement_matrix: ArrayMeasMat,
    pub control_matrix: ArrayCtlMat,
    pub kalman_gain: ArrayKalmanGain,
    pub control_function_values: ArrayControlFunction,
}

impl FunctionalDescription {
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> FunctionalDescription {
        FunctionalDescription {
            ap_params: APParameters::empty(number_of_states),
            measurement_matrix: ArrayMeasMat::empty(number_of_states, number_of_sensors),
            control_matrix: ArrayCtlMat::empty(number_of_states),
            kalman_gain: ArrayKalmanGain::empty(number_of_states, number_of_sensors),
            control_function_values: ArrayControlFunction::empty(number_of_steps),
        }
    }
    pub fn from_simulation_config(
        config: &Simulation,
        spatial_description: &SpatialDescription,
    ) -> FunctionalDescription {
        todo!();
    }
}

#[derive(Debug, PartialEq)]
pub struct SpatialDescription {
    pub voxel_size_mm: f32,
    pub heart_size_mm: [f32; 3],
    pub heart_origin_mm: [f32; 3],
    pub voxel_types: Array1<VoxelType>,
    pub sensor_positions: Array2<f32>,
    pub sensors_orientations: Array2<f32>,
}

impl SpatialDescription {
    pub fn empty(number_of_sensors: usize, number_of_states: usize) -> SpatialDescription {
        SpatialDescription {
            voxel_size_mm: 0.0,
            heart_size_mm: [0.0, 0.0, 0.0],
            heart_origin_mm: [0.0, 0.0, 0.0],
            voxel_types: Array1::default(number_of_states / 3),
            sensor_positions: Array2::zeros((number_of_sensors, 3)),
            sensors_orientations: Array2::zeros((number_of_sensors, 3)),
        }
    }

    pub fn from_simulation_config(config: &Simulation) -> SpatialDescription {
        todo!();
    }

    pub fn get_number_of_states(&self) -> usize {
        todo!();
    }

    pub fn get_number_of_sensors(&self) -> usize {
        todo!();
    }
}

#[derive(Debug, PartialEq)]
pub struct Model {
    pub functional_description: FunctionalDescription,
    pub spatial_description: SpatialDescription,
}

impl Model {
    pub fn empty(
        number_of_states: usize,
        number_of_sensors: usize,
        number_of_steps: usize,
    ) -> Model {
        Model {
            functional_description: FunctionalDescription::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
            ),
            spatial_description: SpatialDescription::empty(number_of_sensors, number_of_states),
        }
    }
    pub fn from_simulation_config(config: &Simulation) -> Model {
        let spatial_description = SpatialDescription::from_simulation_config(config);
        let functional_description =
            FunctionalDescription::from_simulation_config(config, &spatial_description);
        Model {
            functional_description,
            spatial_description,
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub enum VoxelType {
    #[default]
    None,
    Sinoatrial,
    Atrium,
    Atrioventricular,
    HPS,
    Ventricle,
    Pathological,
}
