pub mod measurement;
pub mod shapes;
pub mod simulation;

use std::error::Error;

use nalgebra::Dyn;
use ndarray::Dim;
use serde::{Deserialize, Serialize};

use self::measurement::Measurement;
use self::shapes::ArraySystemStates;
use self::simulation::Simulation;

use crate::core::config::simulation::Simulation as SimulationConfig;
use crate::core::data::shapes::ArrayMeasurements;

use super::model::{
    functional::{
        allpass::shapes::{
            normal::{ArrayDelaysNormal, ArrayGainsNormal},
            ArrayActivationTime,
        },
        control::ControlFunction,
    },
    spatial::voxels::VoxelTypes,
    Model,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Data {
    simulation: Option<Simulation>,
    measurement: Option<Measurement>,
}
impl Data {
    #[must_use]
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Self {
        Self {
            simulation: Some(Simulation::empty(
                number_of_sensors,
                number_of_states,
                number_of_steps,
                voxels_in_dims,
            )),
            measurement: None,
        }
    }

    /// .
    ///
    /// # Errors
    ///
    /// Errors if model parameters to not result in valid delays.
    pub fn from_simulation_config(config: &SimulationConfig) -> Result<Self, Box<dyn Error>> {
        let mut simulation = Simulation::from_config(config)?;
        simulation.run();
        Ok(Self {
            simulation: Some(simulation),
            measurement: None,
        })
    }

    /// Returns a reference to the get measurements of this [`Data`].
    ///
    /// # Panics
    ///
    /// Panics if simulation and measurement are both None.
    #[must_use]
    pub fn get_measurements(&self) -> &ArrayMeasurements {
        self.simulation.as_ref().map_or_else(
            || {
                &(self
                    .measurement
                    .as_ref()
                    .expect("Measurement to be some")
                    .measurements)
            },
            |simulation| &(simulation.measurements),
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_system_states(&self) -> &ArraySystemStates {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.system_states,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_control_function_values(&self) -> &ControlFunction {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .control_function_values
            },
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_gains(&self) -> &ArrayGainsNormal<f32> {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("AP params to be some.")
                    .gains
            },
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_coefs(&self) -> &ArrayDelaysNormal<f32> {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("AP params to be some.")
                    .coefs
            },
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_voxel_types(&self) -> &VoxelTypes {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model.spatial_description.voxels.types,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_model(&self) -> &Model {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_activation_time_ms(&self) -> &ArrayActivationTime {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("AP params to be some.")
                    .activation_time_ms
            },
        )
    }

    /// Returns a reference to the get delays of this [`Data`].
    ///
    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    pub fn get_delays(&self) -> &ArrayDelaysNormal<usize> {
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .ap_params_normal
                    .as_ref()
                    .expect("AP params to be some.")
                    .delays
            },
        )
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if simulation is none.
    pub fn save_npy(&self, path: &std::path::Path) {
        self.simulation
            .as_ref()
            .expect("Simulation to be some.")
            .save_npy(&path.join("simulation"));
    }
}
