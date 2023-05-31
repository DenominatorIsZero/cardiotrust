use ndarray::Array1;

#[derive(Debug, PartialEq)]
pub struct Metrics {
    pub loss: ArrayMetricsSample,
    pub loss_epoch: ArrayMetricsEpoch,
}

impl Metrics {
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> Metrics {
        Metrics {
            loss: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            loss_epoch: ArrayMetricsEpoch::new(number_of_epochs),
        }
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
