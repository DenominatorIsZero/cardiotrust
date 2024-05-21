use ndarray::Dim;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{debug, info, trace};

use super::shapes::{
    ActivationTimePerStateMs, SystemStates, SystemStatesSpherical, SystemStatesSphericalMax,
};
use crate::core::{
    algorithm::estimation::prediction::calculate_system_prediction,
    config::{model::SensorArrayMotion, simulation::Simulation as SimulationConfig},
    data::Measurements,
    model::{functional::allpass::shapes::Gains, Model},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub measurements: Measurements,
    pub system_states: SystemStates,
    pub system_states_spherical: SystemStatesSpherical,
    pub system_states_spherical_max: SystemStatesSphericalMax,
    pub activation_times: ActivationTimePerStateMs,
    pub sample_rate_hz: f32,
    pub model: Model,
}
impl Simulation {
    /// Creates an empty Simulation with the given dimensions and number of
    /// sensors, states, and steps.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
        sensor_motion_steps: usize,
    ) -> Self {
        debug!("Creating empty simulation");
        Self {
            measurements: Measurements::empty(
                sensor_motion_steps,
                number_of_steps,
                number_of_sensors,
            ),
            system_states: SystemStates::empty(number_of_steps, number_of_states),
            system_states_spherical: SystemStatesSpherical::empty(
                number_of_steps,
                number_of_states,
            ),
            system_states_spherical_max: SystemStatesSphericalMax::empty(number_of_states),
            activation_times: ActivationTimePerStateMs::empty(number_of_states),
            sample_rate_hz: 1.0,
            model: Model::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
                voxels_in_dims,
                sensor_motion_steps,
            ),
        }
    }

    /// Creates a new Simulation instance from the provided `SimulationConfig`.
    ///
    /// Initializes an empty Simulation with the model, number of sensors, states,
    /// and time steps specified in the config. The model is validated before
    /// creating the Simulation.
    ///
    /// # Errors
    ///
    /// Returns an error if the model fails to initialize from the config.
    #[tracing::instrument(level = "debug")]
    pub fn from_config(config: &SimulationConfig) -> Result<Self, Box<dyn Error>> {
        debug!("Creating simulation from config");
        let model =
            Model::from_model_config(&config.model, config.sample_rate_hz, config.duration_s)?;
        let number_of_sensors = model.spatial_description.sensors.count();
        let number_of_states = model.spatial_description.voxels.count_states();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let number_of_steps = (config.sample_rate_hz * config.duration_s) as usize;
        let number_of_beats = match config.model.common.sensor_array_motion {
            SensorArrayMotion::Static => 1,
            SensorArrayMotion::Grid => config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
        };

        let measurements = Measurements::empty(number_of_beats, number_of_steps, number_of_sensors);
        let system_states = SystemStates::empty(number_of_steps, number_of_states);
        let system_states_spherical =
            SystemStatesSpherical::empty(number_of_steps, number_of_states);
        let system_states_spherical_max = SystemStatesSphericalMax::empty(number_of_states);
        let activation_times = ActivationTimePerStateMs::empty(number_of_states);

        Ok(Self {
            measurements,
            system_states,
            system_states_spherical,
            system_states_spherical_max,
            activation_times,
            sample_rate_hz: config.sample_rate_hz,
            model,
        })
    }

    /// Runs a simulation by calculating system predictions, adding measurement
    /// noise, and storing results in the measurements and `system_states` fields.
    ///
    /// # Panics
    ///
    /// if there are negative values in the measurement covariance matrix.
    #[tracing::instrument(level = "info", skip_all)]
    pub fn run(&mut self) {
        info!("Running simulation");
        let measurements = &mut self.measurements;
        let system_states = &mut self.system_states;
        let model = &self.model;

        let mut ap_outputs: Gains = Gains::empty(system_states.shape()[1]);
        for beat in 0..measurements.num_beats() {
            ap_outputs.fill(0.0);
            system_states.fill(0.0);
            let measurement_matrix = &model
                .functional_description
                .measurement_matrix
                .at_beat(beat);
            let mut measurements = measurements.at_beat_mut(beat);
            for step in 0..system_states.shape()[0] {
                let mut measurements = measurements.at_step_mut(step);
                calculate_system_prediction(
                    &mut ap_outputs,
                    system_states,
                    &mut measurements,
                    &model.functional_description.ap_params,
                    measurement_matrix,
                    model.functional_description.control_function_values[step],
                    &model.functional_description.control_matrix,
                    step,
                );
            }
        }
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for sensor_index in 0..measurements.num_sensors() {
            let dist = Normal::new(
                0.0,
                model.functional_description.measurement_covariance[[sensor_index, sensor_index]],
            )
            .unwrap();
            for beat_index in 0..measurements.num_beats() {
                for time_index in 0..measurements.num_sensors() {
                    measurements[[beat_index, time_index, sensor_index]] += dist.sample(&mut rng);
                }
            }
        }
        self.calculate_plotting_arrays();
    }

    pub(crate) fn calculate_plotting_arrays(&mut self) {
        let system_states = &mut self.system_states;
        self.system_states_spherical.calculate(system_states);
        self.system_states_spherical_max
            .calculate(&self.system_states_spherical);
        self.activation_times
            .calculate(&self.system_states_spherical, self.sample_rate_hz);
    }

    /// Saves the simulation data (measurements, system states, model) to `NumPy` files at the given path.
    /// The measurements, system states, and model are saved to separate .npy files.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving simulation data to npy");
        self.measurements.save_npy(path);
        self.system_states.save_npy(path);
        self.model.save_npy(path);
    }

    pub(crate) fn update_activation_time(&mut self) {
        self.model.update_activation_time(&self.activation_times);
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use approx::{assert_relative_eq, RelativeEq};

    use ndarray::s;
    use ndarray_stats::QuantileExt;

    use crate::{
        core::{config::model::Mri, model::spatial::voxels::VoxelType},
        tests::setup_folder,
        vis::plotting::{
            gif::states::states_spherical_plot_over_time,
            png::{
                line::{plot_state_xyz, standard_time_plot},
                states::states_spherical_plot,
            },
            PlotSlice, StateSphericalPlotMode,
        },
    };

    use super::*;

    const COMMON_PATH: &str = "tests/core/data/simulation";

    #[test]
    fn create_simulation_no_crash() {
        let config = &SimulationConfig::default();
        let simulation = Simulation::from_config(config);
        assert!(simulation.is_ok());
        let simulation = simulation.unwrap();
        let max = *simulation.system_states.max_skipnan();
        assert_relative_eq!(max, 0.0);
        let max = *simulation.measurements.max_skipnan();
        assert_relative_eq!(max, 0.0);
    }

    #[test]
    fn run_simulation_default() {
        let config = &SimulationConfig::default();
        let mut simulation = Simulation::from_config(config).unwrap();
        simulation.run();
        let max = *simulation.system_states.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.max_skipnan();
        assert!(max > 0.0);
    }

    #[test]
    #[ignore]
    #[allow(clippy::too_many_lines)]
    fn run_simulation_default_and_plot() {
        let folder = Path::new(COMMON_PATH).join("healthy");
        setup_folder(&folder);
        let config = &SimulationConfig::default();
        let mut simulation = Simulation::from_config(config).unwrap();
        simulation.run();
        let max = *simulation.system_states.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.max_skipnan();
        assert!(max > 0.0);

        let sa_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Sinoatrial);

        let path = folder.join("sa.png");
        plot_state_xyz(
            &simulation.system_states,
            sa_index,
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Current Density Sinoatrial Node",
        )
        .unwrap();

        let av_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Atrioventricular);

        let path = folder.join("av.png");
        plot_state_xyz(
            &simulation.system_states,
            av_index,
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Current Density Atrioventricular Node",
        )
        .unwrap();

        let path = folder.join("sensor_0_x.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 0]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - x",
            "H [pT]",
        )
        .unwrap();

        let path = folder.join("sensor_0_y.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 1]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - y",
            "H [pT]",
        )
        .unwrap();

        let path = folder.join("sensor_0_z.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 2]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - z",
            "H [pT]",
        )
        .unwrap();

        let time_index = simulation.system_states.shape()[0] / 3;

        let path = folder.join(format!("states{time_index}.png"));
        states_spherical_plot(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(time_index),
            Some((0.0, 1.0)),
        )
        .unwrap();

        let path = folder.join("states_max.png");
        states_spherical_plot(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        let fps = 20;
        let playback_speed = 0.1;

        let path = folder.join("states.gif");
        states_spherical_plot_over_time(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            config.sample_rate_hz,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(fps),
        )
        .unwrap();
    }

    #[test]
    fn run_simulation_pathological() {
        let mut config = SimulationConfig::default();
        config.model.common.pathological = true;
        let mut simulation = Simulation::from_config(&config).unwrap();
        simulation.run();
        let max = *simulation.system_states.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.max_skipnan();
        assert!(max > 0.0);
    }

    #[test]
    #[ignore]
    #[allow(clippy::too_many_lines)]
    fn run_simulation_pathological_and_plot() {
        let folder = Path::new(COMMON_PATH).join("pathological");
        setup_folder(&folder);
        let mut config = SimulationConfig::default();
        config.model.common.pathological = true;
        let mut simulation = Simulation::from_config(&config).unwrap();
        simulation.run();
        let max = *simulation.system_states.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.max_skipnan();
        assert!(max > 0.0);

        let sa_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Sinoatrial);

        let path = folder.join("sa.png");
        plot_state_xyz(
            &simulation.system_states,
            sa_index,
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Current Density Sinoatrial Node",
        )
        .unwrap();

        let av_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Atrioventricular);

        let path = folder.join("av.png");
        plot_state_xyz(
            &simulation.system_states,
            av_index,
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Current Density Atrioventricular Node",
        )
        .unwrap();

        let path = folder.join("sensor_0_x.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 0]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - x",
            "H [pT]",
        )
        .unwrap();

        let path = folder.join("sensor_0_y.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 1]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - y",
            "H [pT]",
        )
        .unwrap();

        let path = folder.join("sensor_0_z.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 2]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - z",
            "H [pT]",
        )
        .unwrap();

        let time_index = simulation.system_states.shape()[0] / 3;

        let path = folder.join(format!("states{time_index}.png"));
        states_spherical_plot(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(time_index),
            None,
        )
        .unwrap();

        let path = folder.join("states_max.png");
        states_spherical_plot(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        let fps = 20;
        let playback_speed = 0.1;
        let path = folder.join("states.gif");
        states_spherical_plot_over_time(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            config.sample_rate_hz,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(fps),
        )
        .unwrap();
    }

    #[test]
    fn run_simulation_mri() {
        let mut config = SimulationConfig::default();
        config.model.handcrafted = None;
        config.model.mri = Some(Mri::default());
        let mut simulation = Simulation::from_config(&config).unwrap();
        simulation.run();
        let max = *simulation.system_states.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.002, 0.002), "max: {max}");
        let max = *simulation.measurements.max_skipnan();
        assert!(max > 0.0);
    }

    #[test]
    #[ignore]
    #[allow(clippy::too_many_lines)]
    fn run_simulation_mri_and_plot() {
        let folder = Path::new(COMMON_PATH).join("mri");
        setup_folder(&folder);
        let mut config = SimulationConfig::default();
        config.model.handcrafted = None;
        config.model.mri = Some(Mri::default());
        let mut simulation = Simulation::from_config(&config).unwrap();
        simulation.run();
        let max = *simulation.system_states.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.002, 0.002));
        let max = *simulation.measurements.max_skipnan();
        assert!(max > 0.0);

        let sa_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Sinoatrial);

        let path = folder.join("sa.png");
        plot_state_xyz(
            &simulation.system_states,
            sa_index,
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Current Density Sinoatrial Node",
        )
        .unwrap();

        let path = folder.join("sensor_0_x.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 0]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - x",
            "H [pT]",
        )
        .unwrap();

        let path = folder.join("sensor_0_y.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 1]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - y",
            "H [pT]",
        )
        .unwrap();

        let path = folder.join("sensor_0_z.png");
        standard_time_plot(
            &simulation.measurements.slice(s![0, .., 2]).to_owned(),
            config.sample_rate_hz,
            path.as_path(),
            "Simulated Measurement Sensor 0 - z",
            "H [pT]",
        )
        .unwrap();

        let time_index = simulation.system_states.shape()[0] / 3;

        let path = folder.join(format!("states{time_index}.png"));
        states_spherical_plot(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(25)),
            Some(StateSphericalPlotMode::ABS),
            Some(time_index),
            None,
        )
        .unwrap();

        let path = folder.join("states_max.png");
        states_spherical_plot(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(25)),
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        let fps = 20;
        let playback_speed = 0.1;
        let path = folder.join("states.gif");
        states_spherical_plot_over_time(
            &simulation.system_states_spherical,
            &simulation.system_states_spherical_max,
            &simulation.model.spatial_description.voxels.positions_mm,
            simulation.model.spatial_description.voxels.size_mm,
            config.sample_rate_hz,
            &simulation.model.spatial_description.voxels.numbers,
            Some(path.as_path()),
            Some(PlotSlice::Z(25)),
            Some(StateSphericalPlotMode::ABS),
            Some(playback_speed),
            Some(fps),
        )
        .unwrap();
    }
}
