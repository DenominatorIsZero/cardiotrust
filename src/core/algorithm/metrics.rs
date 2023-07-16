use ndarray::{s, Array1};

use crate::core::data::shapes::{ArrayMeasurements, ArraySystemStates};

#[derive(Debug, PartialEq, Clone)]
pub struct Metrics {
    pub loss: ArrayMetricsSample,
    pub loss_epoch: ArrayMetricsEpoch,
    pub delta_states_mean: ArrayMetricsSample,
    pub delta_states_mean_epoch: ArrayMetricsEpoch,
}

impl Metrics {
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> Metrics {
        Metrics {
            loss: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            loss_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_states_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_states_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
        }
    }

    pub fn calculate_step(
        &mut self,
        residuals: &ArrayMeasurements,
        system_states_delta: &ArraySystemStates,
        time_index: usize,
        epoch_index: usize,
    ) {
        let index = time_index
            + epoch_index * (self.loss.values.shape()[0] / self.loss_epoch.values.shape()[0]);
        self.loss.values[index] = residuals.values.mapv(|v| v.powi(2)).sum();
        self.delta_states_mean.values[index] =
            system_states_delta.values.mapv(|v| v.abs()).mean().unwrap();
    }

    pub fn calculate_epoch(&mut self, epoch_index: usize) {
        let number_of_steps = self.loss.values.shape()[0] / self.loss_epoch.values.shape()[0];
        let start_index = epoch_index * number_of_steps;
        let stop_index = (epoch_index + 1) * number_of_steps;

        self.loss_epoch.values[epoch_index] =
            self.loss.values.slice(s![start_index..stop_index]).sum();
        self.delta_states_mean_epoch.values[epoch_index] = self
            .delta_states_mean
            .values
            .slice(s![start_index..stop_index])
            .sum();
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
