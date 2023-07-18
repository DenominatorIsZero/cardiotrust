use ndarray::{s, Array1};
use ndarray_stats::QuantileExt;

use crate::core::{
    data::shapes::{ArrayMeasurements, ArraySystemStates},
    model::functional::allpass::shapes::{ArrayDelays, ArrayGains},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Metrics {
    pub loss: ArrayMetricsSample,
    pub loss_epoch: ArrayMetricsEpoch,

    pub delta_states_mean: ArrayMetricsSample,
    pub delta_states_mean_epoch: ArrayMetricsEpoch,
    pub delta_states_max: ArrayMetricsSample,
    pub delta_states_max_epoch: ArrayMetricsEpoch,

    pub delta_measurements_mean: ArrayMetricsSample,
    pub delta_measurements_mean_epoch: ArrayMetricsEpoch,
    pub delta_measurements_max: ArrayMetricsSample,
    pub delta_measurements_max_epoch: ArrayMetricsEpoch,

    pub delta_gains_mean: ArrayMetricsSample,
    pub delta_gains_mean_epoch: ArrayMetricsEpoch,
    pub delta_gains_max: ArrayMetricsSample,
    pub delta_gains_max_epoch: ArrayMetricsEpoch,

    pub delta_delays_mean: ArrayMetricsSample,
    pub delta_delays_mean_epoch: ArrayMetricsEpoch,
    pub delta_delays_max: ArrayMetricsSample,
    pub delta_delays_max_epoch: ArrayMetricsEpoch,
}

impl Metrics {
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> Metrics {
        Metrics {
            loss: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            loss_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_states_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_states_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_states_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_states_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_measurements_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_measurements_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_measurements_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_measurements_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_gains_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_gains_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_gains_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_gains_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_delays_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_delays_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_delays_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_delays_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),
        }
    }

    pub fn calculate_step(
        &mut self,
        residuals: &ArrayMeasurements,
        system_states_delta: &ArraySystemStates,
        gains_delta: &ArrayGains<f32>,
        delays_delta: &ArrayDelays<f32>,
        time_index: usize,
        epoch_index: usize,
    ) {
        let index = time_index
            + epoch_index * (self.loss.values.shape()[0] / self.loss_epoch.values.shape()[0]);
        self.loss.values[index] = residuals.values.mapv(|v| v.powi(2)).sum();

        let states_delta_abs = system_states_delta.values.mapv(|v| v.abs());
        self.delta_states_mean.values[index] = states_delta_abs.mean().unwrap();
        self.delta_states_max.values[index] = *states_delta_abs.max_skipnan();

        let measurements_delta_abs = residuals.values.mapv(|v| v.abs());
        self.delta_measurements_mean.values[index] = measurements_delta_abs.mean().unwrap();
        self.delta_measurements_max.values[index] = *measurements_delta_abs.max_skipnan();

        let gains_delta_abs = gains_delta.values.mapv(|v| v.abs());
        self.delta_gains_mean.values[index] = gains_delta_abs.mean().unwrap();
        self.delta_gains_max.values[index] = *gains_delta_abs.max_skipnan();

        let delays_delta_abs = delays_delta.values.mapv(|v| v.abs());
        self.delta_delays_mean.values[index] = delays_delta_abs.mean().unwrap();
        self.delta_delays_max.values[index] = *delays_delta_abs.max_skipnan();
    }

    pub fn calculate_epoch(&mut self, epoch_index: usize) {
        let number_of_steps = self.loss.values.shape()[0] / self.loss_epoch.values.shape()[0];
        let start_index = epoch_index * number_of_steps;
        let stop_index = (epoch_index + 1) * number_of_steps;
        let slice = s![start_index..stop_index];

        self.loss_epoch.values[epoch_index] = self.loss.values.slice(slice).mean().unwrap();

        self.delta_states_mean_epoch.values[epoch_index] =
            self.delta_states_mean.values.slice(slice).mean().unwrap();
        self.delta_states_max_epoch.values[epoch_index] =
            *self.delta_states_max.values.slice(slice).max_skipnan();

        self.delta_measurements_mean_epoch.values[epoch_index] = self
            .delta_measurements_mean
            .values
            .slice(slice)
            .mean()
            .unwrap();
        self.delta_measurements_max_epoch.values[epoch_index] = *self
            .delta_measurements_max
            .values
            .slice(slice)
            .max_skipnan();

        self.delta_gains_mean_epoch.values[epoch_index] =
            self.delta_gains_mean.values[stop_index - 1];
        self.delta_gains_max_epoch.values[epoch_index] =
            self.delta_gains_max.values[stop_index - 1];

        self.delta_delays_mean_epoch.values[epoch_index] =
            self.delta_delays_mean.values[stop_index - 1];
        self.delta_delays_max_epoch.values[epoch_index] =
            self.delta_delays_max.values[stop_index - 1];
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayMetricsSample {
    pub values: Array1<f32>,
}

impl ArrayMetricsSample {
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> ArrayMetricsSample {
        ArrayMetricsSample {
            values: Array1::zeros(number_of_epochs * number_of_steps),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayMetricsEpoch {
    pub values: Array1<f32>,
}

impl ArrayMetricsEpoch {
    pub fn new(number_of_epochs: usize) -> ArrayMetricsEpoch {
        ArrayMetricsEpoch {
            values: Array1::zeros(number_of_epochs),
        }
    }
}
