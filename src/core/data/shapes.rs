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
pub struct ArraySystemStates {
    pub values: Array2<f32>,
}

impl ArraySystemStates {
    /// Creates an empty `ArraySystemStates` with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_steps: usize, number_of_states: usize) -> Self {
        trace!("Creating empty system states");
        Self {
            values: Array2::zeros((number_of_steps, number_of_states)),
        }
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
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArraySystemStatesSpherical {
    pub magnitude: Array2<f32>,
    pub theta: Array2<f32>,
    pub phi: Array2<f32>,
}

impl ArraySystemStatesSpherical {
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
    pub fn calculate(&mut self, states: &ArraySystemStates) {
        trace!("Calculating spherical states");
        self.magnitude
            .indexed_iter_mut()
            .for_each(|((time_index, state_index), value)| {
                *value = states.values[(time_index, 3 * state_index)].abs()
                    + states.values[(time_index, 3 * state_index + 1)].abs()
                    + states.values[(time_index, 3 * state_index + 2)].abs();
            });
        self.theta
            .indexed_iter_mut()
            .for_each(|((time_index, state_index), value)| {
                *value = (states.values[(time_index, 3 * state_index + 2)]
                    / self.magnitude[(time_index, state_index)])
                    .acos();
            });
        self.phi
            .indexed_iter_mut()
            .for_each(|((time_index, state_index), value)| {
                *value = states.values[(time_index, 3 * state_index + 1)]
                    .atan2(states.values[(time_index, 3 * state_index)]);
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

impl<'a> std::ops::Sub for &'a ArraySystemStatesSpherical {
    type Output = ArraySystemStatesSpherical;

    #[tracing::instrument(level = "trace")]
    fn sub(self, rhs: Self) -> Self::Output {
        trace!("Subtracting spherical states");
        ArraySystemStatesSpherical {
            magnitude: &self.magnitude - &rhs.magnitude,
            theta: &self.theta - &rhs.theta,
            phi: &self.phi - &rhs.phi,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArraySystemStatesSphericalMax {
    pub magnitude: Array1<f32>,
    pub theta: Array1<f32>,
    pub phi: Array1<f32>,
}

impl ArraySystemStatesSphericalMax {
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
    pub fn calculate(&mut self, spehrical: &ArraySystemStatesSpherical) {
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

impl<'a> std::ops::Sub for &'a ArraySystemStatesSphericalMax {
    type Output = ArraySystemStatesSphericalMax;

    #[tracing::instrument(level = "trace")]
    fn sub(self, rhs: Self) -> Self::Output {
        trace!("Subtracting spherical states max");
        ArraySystemStatesSphericalMax {
            magnitude: &self.magnitude - &rhs.magnitude,
            theta: &self.theta - &rhs.theta,
            phi: &self.phi - &rhs.phi,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayActivationTimePerState {
    pub time_ms: Array1<f32>,
}

impl ArrayActivationTimePerState {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        Self {
            time_ms: Array1::zeros(number_of_states / 3),
        }
    }

    #[tracing::instrument(level = "trace")]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate(&mut self, spehrical: &ArraySystemStatesSpherical, sample_rate_hz: f32) {
        for state in 0..self.time_ms.len() {
            let index = spehrical
                .magnitude
                .index_axis(Axis(1), state)
                .argmax_skipnan()
                .unwrap();
            self.time_ms[state] = index as f32 / sample_rate_hz * 1000.0;
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving system states spherical max");
        fs::create_dir_all(path).unwrap();

        let writer =
            BufWriter::new(File::create(path.join("system_states_activation_time.npy")).unwrap());
        self.time_ms.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayMeasurements(Array3<f32>);

impl ArrayMeasurements {
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
        self.shape()[0]
    }

    #[must_use]
    pub fn num_steps(&self) -> usize {
        self.shape()[1]
    }

    #[must_use]
    pub fn num_sensors(&self) -> usize {
        self.shape()[2]
    }
}

impl Deref for ArrayMeasurements {
    type Target = Array3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ArrayMeasurements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayResiduals(Array1<f32>);

impl ArrayResiduals {
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

impl Deref for ArrayResiduals {
    type Target = Array1<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ArrayResiduals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
