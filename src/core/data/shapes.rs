use ndarray::{Array1, Array2, Array3, Axis};
use ndarray_npy::WriteNpyExt;
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};
use tracing::trace;

/// Shape for the simulated/estimated system states
///
/// Has dimensions (`number_of_steps` `number_of_states`)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SystemStates(Array2<f32>);

impl SystemStates {
    /// Creates an empty `ArraySystemStates` with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_steps: usize, number_of_states: usize) -> Self {
        trace!("Creating empty system states");
        Self(Array2::zeros((number_of_steps, number_of_states)))
    }

    /// Saves the `ArraySystemStates` to a .npy file at the given path.
    ///
    /// Creates any missing directories in the path, opens a file at that path,
    /// and writes the underlying `values` array to it in .npy format.
    ///
    /// # Panics
    ///
    /// Panics if directory of file cant be created.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving system states");
        fs::create_dir_all(path).unwrap();

        let writer = BufWriter::new(File::create(path.join("system_states.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }

    #[must_use]
    pub fn num_steps(&self) -> usize {
        self.raw_dim()[0]
    }

    #[must_use]
    pub fn num_states(&self) -> usize {
        self.raw_dim()[1]
    }
}

impl Deref for SystemStates {
    type Target = Array2<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SystemStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SystemStatesSpherical {
    pub magnitude: Array2<f32>,
    pub theta: Array2<f32>,
    pub phi: Array2<f32>,
}

impl SystemStatesSpherical {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_steps: usize, number_of_states: usize) -> Self {
        trace!("Creating empty system states abs");
        Self {
            magnitude: Array2::zeros((number_of_steps, number_of_states / 3)),
            theta: Array2::zeros((number_of_steps, number_of_states / 3)),
            phi: Array2::zeros((number_of_steps, number_of_states / 3)),
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn calculate(&mut self, states: &SystemStates) {
        trace!("Calculating spherical states");
        self.magnitude
            .indexed_iter_mut()
            .for_each(|((time_index, state_index), value)| {
                *value = states[(time_index, 3 * state_index)].abs()
                    + states[(time_index, 3 * state_index + 1)].abs()
                    + states[(time_index, 3 * state_index + 2)].abs();
            });
        self.theta
            .indexed_iter_mut()
            .for_each(|((time_index, state_index), value)| {
                *value = (states[(time_index, 3 * state_index + 2)]
                    / self.magnitude[(time_index, state_index)])
                    .acos();
            });
        self.phi
            .indexed_iter_mut()
            .for_each(|((time_index, state_index), value)| {
                *value = states[(time_index, 3 * state_index + 1)]
                    .atan2(states[(time_index, 3 * state_index)]);
            });
    }

    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving system states spherical");
        fs::create_dir_all(path).unwrap();

        let writer =
            BufWriter::new(File::create(path.join("system_states_magnitude.npy")).unwrap());
        self.magnitude.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("system_states_theta.npy")).unwrap());
        self.theta.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("system_states_phi.npy")).unwrap());
        self.phi.write_npy(writer).unwrap();
    }
}

impl<'a> std::ops::Sub for &'a SystemStatesSpherical {
    type Output = SystemStatesSpherical;

    #[tracing::instrument(level = "trace")]
    fn sub(self, rhs: Self) -> Self::Output {
        trace!("Subtracting spherical states");
        SystemStatesSpherical {
            magnitude: &self.magnitude - &rhs.magnitude,
            theta: &self.theta - &rhs.theta,
            phi: &self.phi - &rhs.phi,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SystemStatesSphericalMax {
    pub magnitude: Array1<f32>,
    pub theta: Array1<f32>,
    pub phi: Array1<f32>,
}

impl SystemStatesSphericalMax {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        trace!("Creating empty system states abs");
        Self {
            magnitude: Array1::zeros(number_of_states / 3),
            theta: Array1::zeros(number_of_states / 3),
            phi: Array1::zeros(number_of_states / 3),
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn calculate(&mut self, spehrical: &SystemStatesSpherical) {
        trace!("Calculating max spherical states");
        for state in 0..self.magnitude.len() {
            let index = spehrical
                .magnitude
                .index_axis(Axis(1), state)
                .argmax_skipnan()
                .unwrap();
            self.magnitude[state] = spehrical.magnitude[(index, state)];
            self.theta[state] = spehrical.theta[(index, state)];
            self.phi[state] = spehrical.phi[(index, state)];
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving system states spherical max");
        fs::create_dir_all(path).unwrap();

        let writer =
            BufWriter::new(File::create(path.join("system_states_magnitude_max.npy")).unwrap());
        self.magnitude.write_npy(writer).unwrap();
        let writer =
            BufWriter::new(File::create(path.join("system_states_theta_max.npy")).unwrap());
        self.theta.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("system_states_phi_max.npy")).unwrap());
        self.phi.write_npy(writer).unwrap();
    }
}

impl<'a> std::ops::Sub for &'a SystemStatesSphericalMax {
    type Output = SystemStatesSphericalMax;

    #[tracing::instrument(level = "trace")]
    fn sub(self, rhs: Self) -> Self::Output {
        trace!("Subtracting spherical states max");
        SystemStatesSphericalMax {
            magnitude: &self.magnitude - &rhs.magnitude,
            theta: &self.theta - &rhs.theta,
            phi: &self.phi - &rhs.phi,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ActivationTimePerStateMs(Array1<f32>);

impl ActivationTimePerStateMs {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        Self(Array1::zeros(number_of_states / 3))
    }

    #[tracing::instrument(level = "trace")]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate(&mut self, spehrical: &SystemStatesSpherical, sample_rate_hz: f32) {
        for state in 0..self.len() {
            let index = spehrical
                .magnitude
                .index_axis(Axis(1), state)
                .argmax_skipnan()
                .unwrap();
            self[state] = index as f32 / sample_rate_hz * 1000.0;
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving system states spherical max");
        fs::create_dir_all(path).unwrap();

        let writer =
            BufWriter::new(File::create(path.join("system_states_activation_time.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }
}

impl Deref for ActivationTimePerStateMs {
    type Target = Array1<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActivationTimePerStateMs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Measurements(Array3<f32>);

impl Measurements {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    /// Creates an empty `ArrayMeasurements` with the given dimensions.
    pub fn empty(beats: usize, steps: usize, sensors: usize) -> Self {
        trace!("Creating empty measurements");
        Self(Array3::zeros((beats, steps, sensors)))
    }

    /// Panics if file or directory can't be written.
    /// Saves the `ArrayMeasurements` to a .npy file at the given path.
    ///
    /// Creates any missing directories in the path, opens a file at that path,
    /// and writes the underlying `values` array to it in .npy format.
    ///
    /// # Panics
    ///
    /// Panics if directory of file cant be created.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving measurements");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("measurements.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }

    #[must_use]
    pub fn num_beats(&self) -> usize {
        self.raw_dim()[0]
    }

    #[must_use]
    pub fn num_steps(&self) -> usize {
        self.raw_dim()[1]
    }

    #[must_use]
    pub fn num_sensors(&self) -> usize {
        self.raw_dim()[2]
    }
}

impl Deref for Measurements {
    type Target = Array3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Measurements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Residuals(Array1<f32>);

impl Residuals {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    /// Creates an empty `ArrayMeasurements` with the given dimensions.
    pub fn empty(sensors: usize) -> Self {
        trace!("Creating empty measurements");
        Self(Array1::zeros(sensors))
    }

    /// Panics if file or directory can't be written.
    /// Saves the `ArrayMeasurements` to a .npy file at the given path.
    ///
    /// Creates any missing directories in the path, opens a file at that path,
    /// and writes the underlying `values` array to it in .npy format.
    ///
    /// # Panics
    ///
    /// Panics if directory of file cant be created.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving measurements");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("measurements.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }
}

impl Deref for Residuals {
    type Target = Array1<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Residuals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
