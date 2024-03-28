use ndarray::{arr1, s, Array3, Array4};
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};
use strum_macros::EnumIter;
use tracing::{debug, trace};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Voxels {
    pub size_mm: f32,
    pub types: VoxelTypes,
    pub numbers: VoxelNumbers,
    pub positions_mm: VoxelPositions,
}

impl Voxels {
    /// Creates an empty Voxels struct with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        debug!("Creating empty voxels");
        Self {
            size_mm: 0.0,
            types: VoxelTypes::empty(voxels_in_dims),
            numbers: VoxelNumbers::empty(voxels_in_dims),
            positions_mm: VoxelPositions::empty(voxels_in_dims),
        }
    }

    /// Creates a Voxels struct from the given Model config.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model) -> Self {
        debug!("Creating voxels from model config");
        let types = VoxelTypes::from_simulation_config(config);
        let numbers = VoxelNumbers::from_voxel_types(&types);
        let positions = VoxelPositions::from_model_config(config, &types);
        Self {
            size_mm: config.voxel_size_mm,
            types,
            numbers,
            positions_mm: positions,
        }
    }

    pub fn load_from_nii(_path: &str) -> Self {
        Self::empty([1, 1, 1])
    }

    /// Returns the total number of voxels.
    ///
    /// This is calculated as the product of the x, y, and z dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count(&self) -> usize {
        trace!("Counting voxels");
        self.count_xyz().iter().product()
    }

    /// Returns the x, y, and z dimensions of the voxels as a 3-element array.
    /// This represents the shape of the voxel grid.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count_xyz(&self) -> [usize; 3] {
        trace!("Counting voxels in xyz");
        let shape = self.types.values.raw_dim();
        [shape[0], shape[1], shape[2]]
    }

    /// Counts the total number of states by iterating through the
    /// voxel types, filtering out voxels of type 'None', and multiplying by 3
    /// (since each voxel has an x, y, and z state).
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count_states(&self) -> usize {
        trace!("Counting states");
        self.types
            .values
            .iter()
            .filter(|voxel| **voxel != VoxelType::None)
            .count()
            * 3
    }

    /// Checks if the given voxel index is within the valid bounds of the voxel grid
    /// and that the voxel type at that index is not `VoxelType::None`.
    ///
    /// Returns `true` if the index is valid, `false` otherwise.
    ///
    /// # Panics
    ///
    /// Panics if number of voxels in any direction
    /// exceed `i32::MAX`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn is_valid_index(&self, index: [i32; 3]) -> bool {
        trace!("Checking if index is valid");
        let [x, y, z] = index;
        let [x_max, y_max, z_max] = self.count_xyz();
        (0 <= x && x < (i32::try_from(x_max).unwrap()))
            && (0 <= y && y < (i32::try_from(y_max).unwrap()))
            && (0 <= z && z < (i32::try_from(z_max).unwrap()))
            && (self.types.values[(
                usize::try_from(x).unwrap(),
                usize::try_from(y).unwrap(),
                usize::try_from(z).unwrap(),
            )] != VoxelType::None)
    }

    /// Returns the index of the first voxel of type `v_type`.
    ///
    /// # Panics
    ///
    /// Panics if no voxel of `v_type` is present in `Voxels`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_first_state_of_type(&self, v_type: VoxelType) -> usize {
        trace!("Getting first state of type {:?}", v_type);
        let query = self
            .types
            .values
            .iter()
            .zip(self.numbers.values.iter())
            .find(|(this_type, _)| **this_type == v_type);
        query.unwrap().1.unwrap()
    }

    /// Saves the voxel grid data to .npy files in the given path.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxels to npy files");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("voxel_size_mm.npy")).unwrap());
        arr1(&[self.size_mm]).write_npy(writer).unwrap();
        self.types.save_npy(path);
        self.numbers.save_npy(path);
        self.positions_mm.save_npy(path);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VoxelTypes {
    pub values: Array3<VoxelType>,
}

impl VoxelTypes {
    /// Creates an empty `VoxelTypes` with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        trace!("Creating empty voxel types");
        Self {
            values: Array3::default(voxels_in_dims),
        }
    }

    /// Creates a `VoxelTypes` struct initialized with voxel types according
    /// to the provided Model configuration. Voxel types are assigned based
    /// on the Model's parameters that define different anatomical regions.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::similar_names
    )]
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn from_simulation_config(config: &Model) -> Self {
        trace!("Creating voxel types from simulation config");
        // Config Parameters
        let voxel_size_mm = config.voxel_size_mm;
        let heart_size_mm = config.heart_size_mm;

        let mut voxels_in_dims = [0, 0, 0];
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        voxels_in_dims
            .iter_mut()
            .zip(heart_size_mm.iter())
            .for_each(|(number, size)| *number = (size / voxel_size_mm) as usize);

        voxels_in_dims
            .iter_mut()
            .for_each(|v| *v = if *v == 0 { 1 } else { *v });

        // Derived Parameters
        let sa_x_center_index = (voxels_in_dims[0] as f32 * config.sa_x_center_percentage) as usize;
        let sa_y_center_index = (voxels_in_dims[1] as f32 * config.sa_y_center_percentage) as usize;
        let atrium_y_stop_index =
            (voxels_in_dims[1] as f32 * config.atrium_y_stop_percentage) as usize;
        let av_x_center_index = (voxels_in_dims[0] as f32 * config.av_x_center_percentage) as usize;
        let hps_y_stop_index = (voxels_in_dims[1] as f32 * config.hps_y_stop_percentage) as usize;
        let hps_x_start_index = (voxels_in_dims[0] as f32 * config.hps_x_start_percentage) as usize;
        let hps_x_stop_index = (voxels_in_dims[0] as f32 * config.hps_x_stop_percentage) as usize;
        let hps_y_up_index = (voxels_in_dims[1] as f32 * config.hps_y_up_percentage) as usize;
        let pathology_x_start_index =
            (voxels_in_dims[0] as f32 * config.pathology_x_start_percentage) as usize;
        let pathology_x_stop_index =
            (voxels_in_dims[0] as f32 * config.pathology_x_stop_percentage) as usize;
        let pathology_y_start_index =
            (voxels_in_dims[1] as f32 * config.pathology_y_start_percentage) as usize;
        let pathology_y_stop_index =
            (voxels_in_dims[1] as f32 * config.pathology_y_stop_percentage) as usize;

        let mut voxel_types = Self::empty(voxels_in_dims);
        voxel_types
            .values
            .indexed_iter_mut()
            .for_each(|((x, y, _z), voxel_type)| {
                if (config.pathological)
                    && (x >= pathology_x_start_index && x <= pathology_x_stop_index)
                    && (pathology_y_start_index <= y && y <= pathology_y_stop_index)
                {
                    *voxel_type = VoxelType::Pathological;
                } else if (x == sa_x_center_index) && (y == sa_y_center_index) {
                    *voxel_type = VoxelType::Sinoatrial;
                } else if x == av_x_center_index && y == atrium_y_stop_index {
                    *voxel_type = VoxelType::Atrioventricular;
                } else if (x == av_x_center_index
                    && y > atrium_y_stop_index
                    && y < hps_y_stop_index)
                    || (x >= hps_x_start_index
                        && x <= hps_x_stop_index
                        && y == hps_y_stop_index - 1)
                    || ((x == hps_x_start_index || x == hps_x_stop_index)
                        && y >= hps_y_up_index
                        && y < hps_y_stop_index)
                {
                    *voxel_type = VoxelType::HPS;
                } else if y < atrium_y_stop_index {
                    *voxel_type = VoxelType::Atrium;
                } else {
                    *voxel_type = VoxelType::Ventricle;
                }
            });
        voxel_types
    }

    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxel types to npy files");
        let writer = BufWriter::new(File::create(path.join("voxel_types.npy")).unwrap());
        self.values
            .map(|v| match v {
                VoxelType::None => 0,
                VoxelType::Sinoatrial => 1,
                VoxelType::Atrium => 2,
                VoxelType::Atrioventricular => 3,
                VoxelType::HPS => 4,
                VoxelType::Ventricle => 5,
                VoxelType::Pathological => 6,
            })
            .write_npy(writer)
            .unwrap();
    }
}

/// Wrapper around a 3d array that contains the state-indices
/// of each voxel.
///
/// If the value is none it means that there is no voxel in this position.
/// In this case, the voxel type at this position is also none.
///
/// Otherwise it is the first component of the current density at this
/// position. In other words the component in the x direction.
/// The next value is then the component in the y direction
/// and finally the offset-2 value is the component in the z
/// direction.
///
/// This struct is often used to iterate over the voxel-tpyes.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VoxelNumbers {
    pub values: Array3<Option<usize>>,
}

impl VoxelNumbers {
    /// Creates a new `VoxelNumbers` instance with the given dimensions,
    /// initializing all voxel values to None.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        trace!("Creating empty voxel numbers");
        Self {
            values: Array3::default(voxels_in_dims),
        }
    }

    /// Creates a new `VoxelNumbers` instance from the given `VoxelTypes`.
    /// initializing the voxel number values based on the voxel types.
    /// Voxels with type `None` will have their number set to `None`.
    /// Other voxels will have their number set to a incrementing integer,
    /// starting from 0 and incrementing by 3 for each voxel.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn from_voxel_types(types: &VoxelTypes) -> Self {
        trace!("Creating voxel numbers from voxel types");
        let mut numbers = Self {
            values: Array3::default(types.values.raw_dim()),
        };

        let mut current_number = 0;
        numbers
            .values
            .iter_mut()
            .zip(types.values.iter())
            .for_each(|(number, voxel_type)| {
                if *voxel_type == VoxelType::None {
                    *number = None;
                } else {
                    *number = Some(current_number);
                    current_number += 3;
                }
            });
        numbers
    }

    /// Saves the voxel numbers to a .npy file at the given path.
    /// The voxel numbers are converted to i32, with -1 representing None.
    /// Uses numpy's .npy format for efficient storage and loading.
    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxel numbers to npy files");
        let writer = BufWriter::new(File::create(path.join("voxel_numbers.npy")).unwrap());
        self.values
            .map(|v| {
                v.as_ref()
                    .map_or(-1, |number| i32::try_from(*number).unwrap())
            })
            .write_npy(writer)
            .unwrap();
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct VoxelPositions {
    pub values: Array4<f32>,
}

impl VoxelPositions {
    /// Creates a new empty `VoxelPositions` instance with the given dimensions.
    /// Initializes the position values to all zeros.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(voxels_in_dims: [usize; 3]) -> Self {
        trace!("Creating empty voxel positions");
        Self {
            values: Array4::zeros((voxels_in_dims[0], voxels_in_dims[1], voxels_in_dims[2], 3)),
        }
    }

    /// Creates a new `VoxelPositions` instance from the given `Model` config
    /// and `VoxelTypes`. Initializes the position values based on the voxel
    /// size and dimensions specified in the `Model`.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn from_model_config(config: &Model, types: &VoxelTypes) -> Self {
        trace!("Creating voxel positions from model config and voxel types");
        let shape = types.values.raw_dim();
        let mut positions = Self::empty([shape[0], shape[1], shape[2]]);
        let offset = config.voxel_size_mm / 2.0;

        #[allow(clippy::cast_precision_loss)]
        types.values.indexed_iter().for_each(|((x, y, z), _)| {
            let position = arr1(&[
                config.voxel_size_mm.mul_add(x as f32, offset),
                config.voxel_size_mm.mul_add(y as f32, offset),
                config.voxel_size_mm.mul_add(z as f32, offset),
            ]);
            positions
                .values
                .slice_mut(s![x, y, z, ..])
                .assign(&position);
        });
        positions
    }

    /// Saves the voxel position values to a .npy file at the given path.
    /// The position values are saved as a 4D float32 array with shape
    /// (x, y, z, 3), where the last dimension contains the x, y, z
    /// coordinates for each voxel position.
    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving voxel positions to npy files");
        let writer = BufWriter::new(File::create(path.join("voxel_positions_mm.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Copy, Clone, EnumIter)]
pub enum VoxelType {
    #[default]
    None,
    Sinoatrial,
    Atrium,
    Atrioventricular,
    HPS,
    Ventricle,
    Pathological,
}

/// Checks if a connection between the given input and output voxel types is allowed
/// based on anatomical constraints. Returns true if allowed, false otherwise.
#[must_use]
#[tracing::instrument(level = "trace")]
pub fn is_connection_allowed(output_voxel_type: &VoxelType, input_voxel_type: &VoxelType) -> bool {
    trace!("Checking if connection is allowed");
    match output_voxel_type {
        VoxelType::None => false,
        VoxelType::Sinoatrial => {
            [VoxelType::Atrium, VoxelType::Pathological].contains(input_voxel_type)
        }
        VoxelType::Atrium => [
            VoxelType::Sinoatrial,
            VoxelType::Atrium,
            VoxelType::Atrioventricular,
            VoxelType::Pathological,
        ]
        .contains(input_voxel_type),
        VoxelType::Atrioventricular => {
            [VoxelType::Atrium, VoxelType::HPS, VoxelType::Pathological].contains(input_voxel_type)
        }
        VoxelType::HPS => [
            VoxelType::HPS,
            VoxelType::Atrioventricular,
            VoxelType::Ventricle,
            VoxelType::Pathological,
        ]
        .contains(input_voxel_type),
        VoxelType::Ventricle => [
            VoxelType::Ventricle,
            VoxelType::HPS,
            VoxelType::Pathological,
        ]
        .contains(input_voxel_type),
        VoxelType::Pathological => true,
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use ndarray::{Axis, Ix3};
    use nifti::{IntoNdArray, NiftiObject, NiftiVolume, ReaderOptions};
    use tracing::info;

    use crate::vis::plotting::gif::matrix::matrix_over_slices_plot;

    use super::*;

    #[test]
    fn count_states_none() {
        let voxels_in_dims = [1000, 1, 1];
        let voxels = Voxels::empty(voxels_in_dims);

        assert_eq!(0, voxels.count_states());
    }

    #[test]
    fn number_of_states_some() {
        let voxels_in_dims = [1000, 1, 1];
        let mut voxels = Voxels::empty(voxels_in_dims);
        voxels.types.values[(0, 0, 0)] = VoxelType::Atrioventricular;

        assert_eq!(3, voxels.count_states());
    }

    #[test]
    fn no_pathology_full_states() {
        let config = Model {
            heart_size_mm: [10.0, 10.0, 10.0],
            voxel_size_mm: 1.0,
            ..Default::default()
        };
        let voxels = Voxels::from_model_config(&config);

        assert_eq!(1000, voxels.count());
        assert_eq!(3000, voxels.count_states());
    }

    #[test]
    fn is_connection_allowed_true() {
        let output_voxel_type = VoxelType::HPS;
        let input_voxel_type = VoxelType::Ventricle;

        let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

        assert!(allowed);
    }

    #[test]
    fn is_connection_allowed_false() {
        let output_voxel_type = VoxelType::Atrium;
        let input_voxel_type = VoxelType::Ventricle;

        let allowed = is_connection_allowed(&output_voxel_type, &input_voxel_type);

        assert!(!allowed);
    }

    #[test]
    fn some_voxel_types_default() {
        let config = Model::default();
        let types = VoxelTypes::from_simulation_config(&config);

        let num_sa = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Sinoatrial)
            .count();

        assert_eq!(num_sa, 1);

        let num_atrium = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrium)
            .count();

        assert!(num_atrium > 0);

        let num_avn = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Atrioventricular)
            .count();

        assert_eq!(num_avn, 1);

        let num_ventricle = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Ventricle)
            .count();

        assert!(num_ventricle > 0);

        let num_hps = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::HPS)
            .count();

        assert!(num_hps > 0);

        let num_pathological = types
            .values
            .iter()
            .filter(|v_type| **v_type == VoxelType::Pathological)
            .count();

        assert_eq!(num_pathological, 0);
    }

    const COMMON_PATH: &str = "tests/core/model/spatial/voxel/mri";

    #[tracing::instrument(level = "trace")]
    fn setup() {
        if !Path::new(COMMON_PATH).exists() {
            std::fs::create_dir_all(COMMON_PATH).unwrap();
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn from_mri_scan() {
        setup();
        let object = ReaderOptions::new()
            .read_file("assets/Segmentation.nii")
            .unwrap();
        let header = object.header();
        let data_type = header.data_type().unwrap();
        info!("Data type: {data_type:?}");
        let volume = object.volume();
        let dims = volume.dim();
        info!("Dims: {dims:?}");
        let data = volume.into_ndarray::<f32>().unwrap();
        let data = data.into_dimensionality::<Ix3>().unwrap();
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
