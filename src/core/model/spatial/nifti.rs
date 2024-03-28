use ndarray::Ix3;

use nifti::{IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions};

pub(crate) fn load_from_nii(
    path: &str,
) -> ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 3]>> {
    let object = ReaderOptions::new().read_file(path).unwrap();
    let header = object.header();
    let _data_type = header.data_type().unwrap();
    let volume = object.volume();
    let _dims = volume.dim();
    let data = volume.into_ndarray::<f32>().unwrap();
    let data = data.into_dimensionality::<Ix3>().unwrap();
    data
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use ndarray::Axis;

    use crate::{tests::setup_folder, vis::plotting::gif::matrix::matrix_over_slices_plot};

    use super::*;

    const COMMON_PATH: &str = "tests/core/model/spatial/nifti";

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn from_mri_scan() {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let data = load_from_nii("assets/segmentation.nii");
        let duration_ms = 5000;
        let path = Path::new(COMMON_PATH).join("slice_x.gif");
        let time_per_frame_ms = duration_ms / data.shape()[0] as u32;
        matrix_over_slices_plot(
            &data,
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
            &data,
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
            &data,
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
