use std::path::Path;

use anyhow::{anyhow, Context};
use ndarray::{s, Ix3};
use nifti::{IntoNdArray, NiftiObject, ReaderOptions};
use strum::EnumCount;
use tracing::{debug, trace};

use super::voxels::VoxelType;
use crate::core::config::model::Model;

#[derive(Debug)]
pub struct MriData {
    pub segmentation: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 3]>>,
    pub voxel_size_mm: [f32; 3],
}

#[tracing::instrument(level = "debug")]
pub(crate) fn load_from_nii<P>(path: P) -> anyhow::Result<MriData>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    debug!("Loading nifti file from {path:?}");
    let object = ReaderOptions::new()
        .read_file(&path)
        .with_context(|| format!("Failed to read NIFTI file: {path:?}"))?;
    let header = object.header();
    debug!("Nifti header: {header:?}");
    let volume = object.volume();
    let data = volume.into_ndarray::<f32>().with_context(|| {
        format!("Failed to convert NIFTI volume to f32 array for file: {path:?}")
    })?;
    let mut segmentation = data.into_dimensionality::<Ix3>().with_context(|| {
        format!("Failed to convert array to 3D dimensionality for file: {path:?}")
    })?;
    segmentation.swap_axes(1, 2);
    let segmentation = segmentation.slice(s![.., .., ..;-1]).to_owned();
    let voxel_size_mm = [header.pixdim[1], header.pixdim[3], header.pixdim[2]];
    Ok(MriData {
        segmentation,
        voxel_size_mm,
    })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn determine_voxel_type(
    config: &Model,
    position: ndarray::ArrayBase<ndarray::ViewRepr<&f32>, ndarray::Dim<[usize; 1]>>,
    mri_data: &MriData,
    sinoatrial_placed: bool,
) -> anyhow::Result<VoxelType> {
    let mut count = [0; VoxelType::COUNT];
    trace!("Determining voxel type at position {position:?}");

    // calculate the search area
    let x_start_mm =
        position[0] - config.common.heart_offset_mm[0] - config.common.voxel_size_mm / 2.0;
    let x_stop_mm =
        position[0] - config.common.heart_offset_mm[0] + config.common.voxel_size_mm / 2.0;
    let y_start_mm =
        position[1] - config.common.heart_offset_mm[1] - config.common.voxel_size_mm / 2.0;
    let y_stop_mm =
        position[1] - config.common.heart_offset_mm[1] + config.common.voxel_size_mm / 2.0;
    let z_start_mm =
        position[2] - config.common.heart_offset_mm[2] - config.common.voxel_size_mm / 2.0;
    let z_stop_mm =
        position[2] - config.common.heart_offset_mm[2] + config.common.voxel_size_mm / 2.0;

    let x_start_index = (x_start_mm / mri_data.voxel_size_mm[0]).floor() as usize;
    let x_stop_index = (x_stop_mm / mri_data.voxel_size_mm[0]).ceil() as usize;
    let y_start_index = (y_start_mm / mri_data.voxel_size_mm[1]).floor() as usize;
    let y_stop_index = (y_stop_mm / mri_data.voxel_size_mm[1]).ceil() as usize;
    let z_start_index = (z_start_mm / mri_data.voxel_size_mm[2]).floor() as usize;
    let z_stop_index = (z_stop_mm / mri_data.voxel_size_mm[2]).ceil() as usize;

    for x in x_start_index..x_stop_index {
        for y in y_start_index..y_stop_index {
            for z in z_start_index..z_stop_index {
                let voxel_type =
                    VoxelType::from_mri_data(mri_data.segmentation[[x, y, z]] as usize);
                count[voxel_type as usize] += 1;
            }
        }
    }

    if !sinoatrial_placed && count[VoxelType::Sinoatrial as usize] > 0 {
        return Ok(VoxelType::Sinoatrial);
    }

    let (index, _) = count
        .iter()
        .enumerate()
        .max_by_key(|&(_, &value)| value)
        .ok_or_else(|| anyhow!("No voxel types found in count array - this should not happen"))?;
    let mut voxel_type = num_traits::FromPrimitive::from_usize(index).ok_or_else(|| {
        anyhow!("Failed to convert index {index} to VoxelType - invalid enum value")
    })?;
    if voxel_type == VoxelType::Sinoatrial {
        count[VoxelType::Sinoatrial as usize] = 0;
        let (index, _) = count
            .iter()
            .enumerate()
            .max_by_key(|&(_, &value)| value)
            .ok_or_else(|| anyhow!("No non-sinoatrial voxel types found in count array"))?;
        voxel_type = num_traits::FromPrimitive::from_usize(index).ok_or_else(|| {
            anyhow!("Failed to convert fallback index {index} to VoxelType - invalid enum value")
        })?;
    }
    trace!("Placing Voxel type: {index:?} ({voxel_type:?}), count: {count:?}");
    Ok(voxel_type)
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use ndarray::Axis;

    use super::*;
    use crate::{tests::setup_folder, vis::plotting::gif::matrix::matrix_over_slices_plot};

    const COMMON_PATH: &str = "tests/core/model/spatial/nifti";

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_load_file() -> anyhow::Result<()> {
        let _result = load_from_nii("assets/Segmentation.nii")?;
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    #[ignore = "expensive integration test"]
    fn from_mri_scan() -> anyhow::Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let mri_data = load_from_nii("assets/Segmentation.nii")?;
        let data = &mri_data.segmentation;
        let sizes = &mri_data.voxel_size_mm;
        let duration_ms = 5000;
        let path = Path::new(COMMON_PATH).join("slice_x.gif");
        let time_per_frame_ms = duration_ms / data.shape()[0] as u32;
        matrix_over_slices_plot(
            data,
            Some(Axis(0)),
            None,
            Some((sizes[1], sizes[2])),
            None,
            Some(path.as_path()),
            Some("MRI scan along X-Axis. "),
            Some("z [mm]"),
            Some("y [mm]"),
            Some("Label"),
            None,
            None,
            Some(time_per_frame_ms),
        )
        .expect("Failed to create matrix plot");
        let path = Path::new(COMMON_PATH).join("slice_y.gif");
        let time_per_frame_ms = duration_ms / data.shape()[1] as u32;
        matrix_over_slices_plot(
            data,
            Some(Axis(1)),
            None,
            Some((sizes[0], sizes[2])),
            None,
            Some(path.as_path()),
            Some("MRI scan along Y-Axis. "),
            Some("z [mm]"),
            Some("x [mm]"),
            Some("Label"),
            None,
            None,
            Some(time_per_frame_ms),
        )
        .expect("Failed to create matrix plot");
        let path = Path::new(COMMON_PATH).join("slice_z.gif");
        let time_per_frame_ms = duration_ms / data.shape()[2] as u32;
        matrix_over_slices_plot(
            data,
            Some(Axis(2)),
            None,
            Some((sizes[0], sizes[1])),
            None,
            Some(path.as_path()),
            Some("MRI scan along Z-Axis. "),
            Some("y [mm]"),
            Some("x [mm]"),
            Some("Label"),
            None,
            None,
            Some(time_per_frame_ms),
        )
        .expect("Failed to create matrix plot");
        Ok(())
    }
}
