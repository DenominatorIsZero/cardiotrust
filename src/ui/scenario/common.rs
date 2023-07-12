use egui_extras::TableBody;

use crate::core::{
    config::model::{ControlFunction, Model},
    model::spatial::voxels::VoxelType,
};

pub fn draw_ui_scenario_common(body: &mut TableBody, model: &mut Model) {
    // Control function
    let control_function = &mut model.control_function;
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Contorl function");
        });
        row.col(|ui| {
            egui::ComboBox::new("cb_control_function", "")
                .selected_text(format!("{:?}", control_function))
                .show_ui(ui, |ui| {
                    ui.selectable_value(control_function, ControlFunction::Sinosodal, "Sinosodal");
                    ui.selectable_value(control_function, ControlFunction::Ohara, "Ohara");
                });
        });
        row.col(|ui| {
            ui.label(
                "The control function used as the input tthe system \
                                    / The shape of the assumed current density curve.",
            );
        });
    });
    // Pathological
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Pathological");
        });
        row.col(|ui| {
            ui.checkbox(&mut model.pathological, "");
        });
        row.col(|ui| {
            ui.label("Whether or not to place pathological tissue in the model.");
        });
    });
    // Current Factor in Pathology
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Current Factor\nin Pathology");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.current_factor_in_pathology,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "A factor describing how much to reduce the \
                                    current densities in pathological voxels.",
            );
        });
    });
    // Propagation velocity sinoatrial node
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Propagaion Velocity\nSinoatrial Node");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(
                    model
                        .propagation_velocities_m_per_s
                        .get_mut(&VoxelType::Sinoatrial)
                        .unwrap(),
                    0.01..=10.0,
                )
                .suffix(" m/s"),
            );
        });
        row.col(|ui| {
            ui.label(
                "Desired propagation velocity in the \
                                    sinoatrial node in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
            );
        });
    });
    // Propagation velocity Atrium
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Propagaion Velocity\nAtrium");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(
                    model
                        .propagation_velocities_m_per_s
                        .get_mut(&VoxelType::Atrium)
                        .unwrap(),
                    0.01..=10.0,
                )
                .suffix(" m/s"),
            );
        });
        row.col(|ui| {
            ui.label(
                "Desired propagation velocity in the \
                                    atrium in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
            );
        });
    });
    // Propagation velocity atrioventricular node
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Propagaion Velocity\nAtrioventricular Node");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(
                    model
                        .propagation_velocities_m_per_s
                        .get_mut(&VoxelType::Atrioventricular)
                        .unwrap(),
                    0.01..=10.0,
                )
                .suffix(" m/s"),
            );
        });
        row.col(|ui| {
            ui.label(
                "Desired propagation velocity in the \
                                    atrioventricular node in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
            );
        });
    });
    // Propagation velocity HPS
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Propagaion Velocity\nHPS");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(
                    model
                        .propagation_velocities_m_per_s
                        .get_mut(&VoxelType::HPS)
                        .unwrap(),
                    0.01..=10.0,
                )
                .suffix(" m/s"),
            );
        });
        row.col(|ui| {
            ui.label(
                "Desired propagation velocity in the \
                                    His-Purkinje system in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
            );
        });
    });
    // Propagation velocity ventricle
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Propagaion Velocity\nVentricle");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(
                    model
                        .propagation_velocities_m_per_s
                        .get_mut(&VoxelType::Ventricle)
                        .unwrap(),
                    0.01..=10.0,
                )
                .suffix(" m/s"),
            );
        });
        row.col(|ui| {
            ui.label(
                "Desired propagation velocity in the \
                                    ventricle in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
            );
        });
    });
    // Propagation velocity pathological
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Propagaion Velocity\nPathological");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(
                    model
                        .propagation_velocities_m_per_s
                        .get_mut(&VoxelType::Pathological)
                        .unwrap(),
                    0.01..=10.0,
                )
                .suffix(" m/s"),
            );
        });
        row.col(|ui| {
            ui.label(
                "Desired propagation velocity in the \
                                    pathological tissue in m/s. Note that the \
                                    maximum propagation velocity is limited \
                                    by the voxel size and sample rate.",
            );
        });
    });
    // sinoatrial node x center percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("X Center\nSinoatrial Node");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.sa_x_center_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The center of the sinoatrial node \
                                    in x-direction in percent.",
            );
        });
    });
    // sinoatrial node y center percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Y Center\nSinoatrial Node");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.sa_y_center_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The center of the sinoatrial node \
                                    in y-direction in percent.",
            );
        });
    });
    // atrium y stop percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Y Stop\nAtrium");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.atrium_y_stop_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The end of the atrium \
                                    / start of the ventricles
                                    in y-direction in percent.",
            );
        });
    });
    // atrioventricular node x center percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("X Center\nAtrioventricular Node");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.av_x_center_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The center of the atrioventricular node \
                                    in x-direction in percent.",
            );
        });
    });
    // hps y stop percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Y Stop\nHPS");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.hps_y_stop_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The end of the His-Purkinje-System \
                                    in y-direction in percent.",
            );
        });
    });
    // hps x start percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("X Start\nHPS");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.hps_x_start_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The start of the His-Purkinje-System \
                                    in x-direction in percent.",
            );
        });
    });
    // hps x stop percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("X Stop\nHPS");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.hps_x_stop_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The end of the His-Purkinje-System \
                                    in x-direction in percent.",
            );
        });
    });
    // hps y up percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Y Up\nHPS");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(&mut model.hps_y_up_percentage, 0.0..=1.0));
        });
        row.col(|ui| {
            ui.label(
                "The end of the upwards portion \
                                    of the His-Purkinje-System \
                                    in x-direction in percent.",
            );
        });
    });
    // pathology x start percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("X Start\nPathology");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.pathology_x_start_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The start of the pathology \
                                    in x-direction in percent.",
            );
        });
    });
    // pathology x stop percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("X Stop\nPathology");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.hps_x_stop_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The end of the pathology \
                                    in x-direction in percent.",
            );
        });
    });
    // pathology y start percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Y Start\nPathology");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.pathology_y_start_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The start of the pathology \
                                    in y-direction in percent.",
            );
        });
    });
    // pathology y stop percentage
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Y Stop\nPathology");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.pathology_y_stop_percentage,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The end of the pathology \
                                    in y-direction in percent.",
            );
        });
    });
    // measurement covariance mean
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Measurement\ncovariance mean");
        });
        row.col(|ui| {
            ui.add(
                egui::Slider::new(&mut model.measurement_covariance_mean, 1e-10..=1e10)
                    .logarithmic(true)
                    .custom_formatter(|n, _| format!("{:+.4e}", n)),
            );
        });
        row.col(|ui| {
            ui.label(
                "The mean value of the measurement \
                                noise covariance matrix.",
            );
        });
    });
    // measurement covariance std
    body.row(30.0, |mut row| {
        row.col(|ui| {
            ui.label("Measurement\ncovariance std");
        });
        row.col(|ui| {
            ui.add(egui::Slider::new(
                &mut model.measurement_covariance_std,
                0.0..=1.0,
            ));
        });
        row.col(|ui| {
            ui.label(
                "The standard deviation of the \
                                measurement noise covariance matrix. \
                                If this is zero, all diagonal values will \
                                be choosen as the mean. \
                                Otherwise they will be drawn from a normal \
                                distribution according \
                                to the mean value and standard deviation.",
            );
        });
    });
}
