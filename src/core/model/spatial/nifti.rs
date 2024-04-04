use std::path::Path;

use ndarray::Ix3;

use nifti::{IntoNdArray, NiftiObject, ReaderOptions};
use strum::EnumCount;
use tracing::{debug, trace};

use crate::core::config::model::Model;

use super::voxels::VoxelType;

#[derive(Debug)]
pub struct MriData {
    pub segmentation: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 3]>>,
    pub voxel_size_mm: [f32; 3],
    pub offset_mm: [f32; 3],
}

#[tracing::instrument(level = "debug")]
pub(crate) fn load_from_nii<P>(path: P) -> MriData
where
    P: AsRef<Path> + std::fmt::Debug,
{
    debug!("Loading nifti file from {path:?}");
    let object = ReaderOptions::new().read_file(path).unwrap();
    let header = object.header();
    debug!("Nifti header: {header:?}");
    let volume = object.volume();
    let data = volume.into_ndarray::<f32>().unwrap();
    let segmentation = data.into_dimensionality::<Ix3>().unwrap();
    let voxel_size_mm = [header.pixdim[1], header.pixdim[2], header.pixdim[3]];
    let offset_mm = [header.quatern_x, header.quatern_y, header.quatern_z];
    MriData {
        segmentation,
        voxel_size_mm,
        offset_mm,
    }
}

#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn determine_voxel_type(
    config: &Model,
    position: ndarray::ArrayBase<ndarray::ViewRepr<&f32>, ndarray::Dim<[usize; 1]>>,
    mri_data: &MriData,
    sinoatrial_placed: bool,
) -> VoxelType {
    let mut count = [0; VoxelType::COUNT];
    trace!("Determining voxel type at position {position:?}");

    // calculate the search area
    let x_start_mm = position[0]
        - config.common.heart_offset_mm[0]
        - mri_data.offset_mm[0]
        - config.common.voxel_size_mm / 2.0;
    let x_stop_mm = position[0] - config.common.heart_offset_mm[0] - mri_data.offset_mm[0]
        + config.common.voxel_size_mm / 2.0;
    let y_start_mm = position[1]
        - config.common.heart_offset_mm[1]
        - mri_data.offset_mm[1]
        - config.common.voxel_size_mm / 2.0;
    let y_stop_mm = position[1] - config.common.heart_offset_mm[1] - mri_data.offset_mm[1]
        + config.common.voxel_size_mm / 2.0;
    let z_start_mm = position[2]
        - config.common.heart_offset_mm[2]
        - mri_data.offset_mm[2]
        - config.common.voxel_size_mm / 2.0;
    let z_stop_mm = position[2] - config.common.heart_offset_mm[2] - mri_data.offset_mm[2]
        + config.common.voxel_size_mm / 2.0;

    trace!(
        "Searching for voxel type in range [{}, {}, {}, {}, {}, {}]",
        x_start_mm,
        x_stop_mm,
        y_start_mm,
        y_stop_mm,
        z_start_mm,
        z_stop_mm
    );

    let x_start_index = (x_start_mm / mri_data.voxel_size_mm[0]).floor() as usize;
    let x_stop_index = (x_stop_mm / mri_data.voxel_size_mm[0]).ceil() as usize;
    let y_start_index = (y_start_mm / mri_data.voxel_size_mm[1]).floor() as usize;
    let y_stop_index = (y_stop_mm / mri_data.voxel_size_mm[1]).ceil() as usize;
    let z_start_index = (z_start_mm / mri_data.voxel_size_mm[2]).floor() as usize;
    let z_stop_index = (z_stop_mm / mri_data.voxel_size_mm[2]).ceil() as usize;

    trace!(
        "Searching for voxel type in range [{}, {}, {}, {}, {}, {}]",
        x_start_index,
        x_stop_index,
        y_start_index,
        y_stop_index,
        z_start_index,
        z_stop_index
    );

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
        return VoxelType::Sinoatrial;
    }

    let (index, _) = count
        .iter()
        .enumerate()
        .max_by_key(|&(_, &value)| value)
        .unwrap();
    let mut voxel_type = num_traits::FromPrimitive::from_usize(index).unwrap();
    if voxel_type == VoxelType::Sinoatrial {
        count[VoxelType::Sinoatrial as usize] = 0;
        let (index, _) = count
            .iter()
            .enumerate()
            .max_by_key(|&(_, &value)| value)
            .unwrap();
        voxel_type = num_traits::FromPrimitive::from_usize(index).unwrap();
    }
    trace!("Placing Voxel type: {index:?} ({voxel_type:?}), count: {count:?}");
    voxel_type
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use ndarray::Axis;
    use tracing_test::traced_test;

    use crate::{tests::setup_folder, vis::plotting::gif::matrix::matrix_over_slices_plot};

    use super::*;

    const COMMON_PATH: &str = "tests/core/model/spatial/nifti";

    #[test]
    #[traced_test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_load_file() {
        let _ = load_from_nii("assets/segmentation.nii");
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    #[ignore]
    fn from_mri_scan() {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let mri_data = load_from_nii("assets/segmentation.nii");
        let data = &mri_data.segmentation;
        let duration_ms = 5000;
        let path = Path::new(COMMON_PATH).join("slice_x.gif");
        let time_per_frame_ms = duration_ms / data.shape()[0] as u32;
        matrix_over_slices_plot(
            data,
            Some(Axis(0)),
            None,
            Some((1.0, 2.25)),
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
        .unwrap();
        let path = Path::new(COMMON_PATH).join("slice_y.gif");
        let time_per_frame_ms = duration_ms / data.shape()[1] as u32;
        matrix_over_slices_plot(
            data,
            Some(Axis(1)),
            None,
            Some((1.0, 2.25)),
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
        .unwrap();
        let path = Path::new(COMMON_PATH).join("slice_z.gif");
        let time_per_frame_ms = duration_ms / data.shape()[2] as u32;
        matrix_over_slices_plot(
            data,
            Some(Axis(2)),
            None,
            Some((1.0, 1.0)),
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
        .unwrap();
    }
}
