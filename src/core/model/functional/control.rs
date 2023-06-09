use ndarray::Array1;
use ndarray_npy::read_npy;
use samplerate::{self, ConverterType};
use std::fs;

use crate::core::{
    config::model::Model,
    model::spatial::{voxels::VoxelType, SpatialDescription},
};

#[derive(Debug, PartialEq)]
pub struct ControlMatrix {
    pub values: Array1<f32>,
}

impl ControlMatrix {
    pub fn empty(number_of_states: usize) -> ControlMatrix {
        ControlMatrix {
            values: Array1::zeros(number_of_states),
        }
    }

    pub fn from_model_config(
        _config: &Model,
        spatial_description: &SpatialDescription,
    ) -> ControlMatrix {
        let mut control_matrix = ControlMatrix::empty(spatial_description.voxels.count_states());
        spatial_description
            .voxels
            .types
            .values
            .iter()
            .zip(spatial_description.voxels.numbers.values.iter())
            .for_each(|(v_type, v_number)| {
                if *v_type == VoxelType::Sinoatrial {
                    control_matrix.values[v_number.unwrap()] = 1.0;
                }
            });
        control_matrix
    }
}

#[derive(Debug, PartialEq)]
pub struct ControlFunction {
    pub values: Array1<f32>,
}

impl ControlFunction {
    pub fn empty(number_of_steps: usize) -> ControlFunction {
        ControlFunction {
            values: Array1::zeros(number_of_steps),
        }
    }

    pub fn from_model_config(
        _config: &Model,
        sample_rate_hz: f32,
        duration_s: f32,
    ) -> ControlFunction {
        let sample_rate_hz_in = 2000.0;
        let control_function_raw: Array1<f32> =
            read_npy("assets/control_function_ohara.npy").unwrap();

        let desired_length_samples = (duration_s * sample_rate_hz) as usize;

        let control_function_converted = samplerate::convert(
            sample_rate_hz_in as u32,
            sample_rate_hz as u32,
            1,
            ConverterType::SincBestQuality,
            &control_function_raw.to_vec(),
        )
        .unwrap();

        let control_function_values: Vec<f32> = (0..desired_length_samples)
            .map(|i| {
                let index = i % control_function_converted.len();
                control_function_converted[index]
            })
            .collect();

        ControlFunction {
            values: Array1::from(control_function_values),
        }
    }
}

#[cfg(test)]
mod test {
    use plotters::prelude::*;

    use super::*;

    #[test]
    fn matrix_from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);

        let _control_matrix = ControlMatrix::from_model_config(&config, &spatial_description);
    }

    #[test]
    fn function_from_model_config_no_crash() -> Result<(), Box<dyn std::error::Error>> {
        let sample_rate_hz = 3000.0;
        let duration_s = 1.0;
        let expected_length_samples = (sample_rate_hz * duration_s) as usize;
        let config = Model::default();

        let control_function =
            ControlFunction::from_model_config(&config, sample_rate_hz, duration_s);
        assert_eq!(expected_length_samples, control_function.values.shape()[0]);

        let x: Vec<f32> = (0..control_function.values.shape()[0])
            .map(|i| i as f32 / sample_rate_hz)
            .collect();
        let x_min = *x
            .iter()
            .reduce(|min, e| if e < min { e } else { min })
            .unwrap_or(&f32::MAX);
        let x_max = *x
            .iter()
            .reduce(|max, e| if e > max { e } else { max })
            .unwrap_or(&f32::MIN);
        let y = control_function.values.to_vec();
        let y: Vec<f32> = x.iter().map(|x| *x * *x).collect();
        let mut y_min = *y
            .iter()
            .reduce(|min, e| if e < min { e } else { min })
            .unwrap_or(&f32::MAX);
        let mut y_max = *y
            .iter()
            .reduce(|max, e| if e > max { e } else { max })
            .unwrap_or(&f32::MIN);
        let y_range = y_max - y_min;
        let y_margin = 0.1;
        y_min = y_min - y_margin * y_range;
        y_max = y_max + y_margin * y_range;

        let root =
            BitMapBackend::new("tests/control_function.png", (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(100)
            .y_label_area_size(120)
            .top_x_label_area_size(10)
            .right_y_label_area_size(10)
            .caption("Control Function", ("computer-modern", 75).into_font())
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?
            .set_secondary_coord(x_min..x_max, y_min..y_max);

        let style = ("computer-modern", 50).into_font();
        chart
            .configure_mesh()
            .x_label_style(style.clone())
            .y_label_style(style.clone())
            .x_desc("Time [s]")
            .y_desc("Current density [A/m^2]")
            .draw()?;

        chart
            .configure_secondary_axes()
            .x_labels(0)
            .y_labels(0)
            .draw()?;

        chart.draw_series(LineSeries::new(
            x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
            Into::<ShapeStyle>::into(&RED).stroke_width(5).filled(),
        ))?;

        root.present()?;

        let root = SVGBackend::new("tests/control_function.svg", (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(100)
            .y_label_area_size(120)
            .top_x_label_area_size(10)
            .right_y_label_area_size(10)
            .caption("Control Function", ("computer-modern", 75).into_font())
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?
            .set_secondary_coord(x_min..x_max, y_min..y_max);

        let style = ("computer-modern", 50).into_font();
        chart
            .configure_mesh()
            .x_label_style(style.clone())
            .y_label_style(style.clone())
            .x_desc("Time [s]")
            .y_desc("Current density [A/m^2]")
            .draw()?;

        chart
            .configure_secondary_axes()
            .x_labels(0)
            .y_labels(0)
            .draw()?;

        chart.draw_series(LineSeries::new(
            x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
            Into::<ShapeStyle>::into(&RED).stroke_width(5),
        ))?;

        Ok(())
    }
}
