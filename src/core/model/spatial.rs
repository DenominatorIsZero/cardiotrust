pub mod nifti;
pub mod sensors;
pub mod voxels;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::{sensors::Sensors, voxels::Voxels};
use crate::core::config::model::Model;

/// Struct containing fields for the heart,
/// voxels and sensors spatial model components.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct SpatialDescription {
    pub voxels: Voxels,
    pub sensors: Sensors,
}

impl SpatialDescription {
    /// Creates an empty `SpatialDescription` struct with the given number of
    /// sensors and voxel dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_sensors: usize,
        voxels_in_dims: [usize; 3],
        sensor_motion_steps: usize,
    ) -> Self {
        debug!("Creating empty spatial description");
        Self {
            voxels: Voxels::empty(voxels_in_dims),
            sensors: Sensors::empty(number_of_sensors, sensor_motion_steps),
        }
    }

    /// Creates a `SpatialDescription` from the given [`Model`] configuration.
    ///
    /// Constructs the `heart`, `voxels`, and `sensors` fields by calling their
    /// respective `from_model_config()` methods.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_model_config(config: &Model) -> Result<Self> {
        debug!("Creating spatial description from model config");
        let voxels = if config.handcrafted.is_some() {
            Voxels::from_handcrafted_model_config(config)
        } else {
            Voxels::from_mri_model_config(config)?
        };

        let sensors = Sensors::from_model_config(&config.common);

        Ok(Self { voxels, sensors })
    }

    /// Saves the spatial description components to .npy files.
    ///
    /// Saves the `heart`, `voxels`, and `sensors` fields to .npy files
    /// in the given `path`.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        trace!("Saving spatial description to npy");
        let path = &path.join("spatial_description");
        self.voxels.save_npy(path);
        self.sensors.save_npy(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use ndarray::Axis;

    use super::*;
    use crate::{
        core::config::model::{Common, Handcrafted, Mri},
        tests::setup_folder,
        vis::plotting::gif::voxel_type::voxel_types_over_slices_plot,
    };

    const COMMON_PATH: &str = "tests/core/model/spatial";

    #[test]
    fn empty_no_crash() {
        let number_of_sensors = 300;
        let voxels_in_dims = [1000, 1, 1];
        let sensor_motion_steps = 10;
        let _spatial_description =
            SpatialDescription::empty(number_of_sensors, voxels_in_dims, sensor_motion_steps);
    }

    #[test]
    fn from_simulation_config_no_crash() -> anyhow::Result<()> {
        let config = Model::default();
        let _spatial_description = SpatialDescription::from_model_config(&config)?;
        Ok(())
    }

    #[test]
    fn from_handcrafted_model_config_no_crash() -> anyhow::Result<()> {
        let config = Model {
            common: Common::default(),
            handcrafted: Some(Handcrafted::default()),
            mri: None,
        };
        let _spatial_description = SpatialDescription::from_model_config(&config)?;
        Ok(())
    }

    #[test]
    fn from_mri_model_config_no_crash() -> anyhow::Result<()> {
        let config = Model {
            common: Common::default(),
            handcrafted: None,
            mri: Some(Mri::default()),
        };
        let _spatial_description = SpatialDescription::from_model_config(&config)?;
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn from_handcrafted_model_config_and_plot() -> anyhow::Result<()> {
        let directory = Path::new(COMMON_PATH).join("handcrafted");
        setup_folder(&directory);
        let config = Model {
            common: Common::default(),
            handcrafted: Some(Handcrafted::default()),
            mri: None,
        };
        let spatial_description = SpatialDescription::from_model_config(&config)?;

        let duration_ms = 5000;
        let path = directory.join("types_over_x.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[0] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(0)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");

        let path = directory.join("types_over_y.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[1] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(1)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");

        let path = directory.join("types_over_z.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[2] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(2)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn from_mri_model_config_and_plot() -> anyhow::Result<()> {
        let directory = Path::new(COMMON_PATH).join("mri");
        setup_folder(&directory);
        let config = Model {
            common: Common::default(),
            handcrafted: None,
            mri: Some(Mri::default()),
        };
        let spatial_description = SpatialDescription::from_model_config(&config)?;

        let duration_ms = 5000;
        let path = directory.join("types_over_x.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[0] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(0)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");

        let path = directory.join("types_over_y.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[1] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(1)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");

        let path = directory.join("types_over_z.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[2] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(2)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn from_mri_model_config_and_plot_coarse() -> anyhow::Result<()> {
        let directory = Path::new(COMMON_PATH).join("mri");
        setup_folder(&directory);
        let mut config = Model {
            common: Common::default(),
            handcrafted: None,
            mri: Some(Mri::default()),
        };
        config.common.voxel_size_mm = 10.0;
        let spatial_description = SpatialDescription::from_model_config(&config)?;

        let duration_ms = 5000;
        let path = directory.join("types_over_x_coarse.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[0] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(0)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");

        let path = directory.join("types_over_y_coarse.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[1] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(1)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");

        let path = directory.join("types_over_z_coarse.gif");
        let time_per_frame_ms = duration_ms / spatial_description.voxels.types.shape()[2] as u32;
        voxel_types_over_slices_plot(
            &spatial_description.voxels.types,
            &spatial_description.voxels.positions_mm,
            spatial_description.voxels.size_mm,
            Some(Axis(2)),
            Some(&path),
            Some(time_per_frame_ms),
        )
        .expect("Failed to create voxel types plot");
        Ok(())
    }
}
