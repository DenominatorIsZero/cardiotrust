use std::path::PathBuf;

use egui_extras::{Column, TableBuilder};
use tracing::error;

use super::super::{FIRST_COLUMN_WIDTH, PADDING, ROW_HEIGHT, SECOND_COLUMN_WIDTH};
use crate::core::config::model::Mri;

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip_all, level = "trace")]
pub fn draw_mri_settings(ui: &mut egui::Ui, mri: &mut Mri, _patholoical: bool) {
    ui.label(egui::RichText::new("MRI Model Settings").underline());
    ui.group(|ui| {
        let width = ui.available_width();
        TableBuilder::new(ui)
            .column(Column::exact(FIRST_COLUMN_WIDTH))
            .column(Column::exact(SECOND_COLUMN_WIDTH))
            .column(Column::exact(
                width - FIRST_COLUMN_WIDTH - SECOND_COLUMN_WIDTH - PADDING,
            ))
            .striped(true)
            .header(ROW_HEIGHT, |mut header| {
                header.col(|ui| {
                    ui.heading("Parameter");
                });
                header.col(|ui| {
                    ui.heading("Value");
                });
                header.col(|ui| {
                    ui.heading("Description");
                });
            })
            .body(|mut body| {
                // Path
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label("Path");
                    });
                    row.col(|ui| {
                        let mut path = mri
                            .path
                            .to_str()
                            .unwrap_or_else(|| {
                                error!("MRI path contains invalid UTF-8: {:?}", mri.path);
                                "<invalid path>"
                            })
                            .to_string();
                        ui.add(egui::TextEdit::singleline(&mut path));
                        mri.path = PathBuf::from(path);
                    });
                    row.col(|ui| {
                        ui.add(egui::Label::new("The path to the .nii file.").truncate());
                    });
                });
            });
    });
}
