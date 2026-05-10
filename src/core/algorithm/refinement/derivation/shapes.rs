use std::ops::{Deref, DerefMut, Sub};

use anyhow::{Context, Result};
use ndarray::Array1;
#[cfg(feature = "native")]
use ocl::Buffer;
use serde::{Deserialize, Serialize};
use tracing::trace;

/// Shape for the mapped residuals.
///
/// Has dimensions (`number_of_states`)
///
/// The residuals (measurements) of the state estimation
/// get mapped onto the system states.
/// These values are then used for the calcualtion of the derivatives
///
/// The mapped residuals are calculated as
/// `H_T` * y
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MappedResiduals(Array1<f32>);

impl MappedResiduals {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_states: usize) -> Self {
        trace!("Creating ArrayMappedResiduals");
        Self(Array1::zeros(number_of_states))
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn to_gpu(&self, queue: &ocl::Queue) -> Result<Buffer<f32>> {
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(
                self.as_slice()
                    .context("Failed to get array slice for GPU copy")?,
            )
            .build()
            .context("Failed to build GPU buffer for mapped residuals")?;
        Ok(buffer)
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn update_from_gpu(&mut self, mapped_residuals: &Buffer<f32>) -> Result<()> {
        mapped_residuals
            .read(
                self.as_slice_mut()
                    .context("Failed to get mutable array slice for GPU read")?,
            )
            .enq()
            .context("Failed to read mapped residuals from GPU buffer")?;
        Ok(())
    }
}

impl Deref for MappedResiduals {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MappedResiduals {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Shape for the average delays in each voxel.
///
/// Has dimensions (`number_of_states / 3`)
///
/// The average delays are calculated as a
/// weighted sum of the delays by the gains in that direction.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AverageDelays(Array1<Option<f32>>);

impl AverageDelays {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn empty(number_of_states: usize) -> Self {
        trace!("Creating AverageDelays");
        Self(Array1::from_elem(number_of_states / 3, None))
    }
}

impl<'b> Sub<&'b AverageDelays> for &AverageDelays {
    type Output = AverageDelays;

    #[tracing::instrument(level = "trace")]
    fn sub(self, rhs: &'b AverageDelays) -> Self::Output {
        let result = self
            .0
            .iter()
            .zip(rhs.0.iter())
            .map(|(a, b)| match (a, b) {
                (Some(x), Some(y)) => Some(x - y),
                _ => None,
            })
            .collect();
        AverageDelays(result)
    }
}

impl Deref for AverageDelays {
    type Target = Array1<Option<f32>>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AverageDelays {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Shape for the maximum system states regularization.
///
/// Has dimensions (`number_of_states`)
///
/// The maximum current density in a single voxel should not exceed one.
/// For this we have to add up all three absoutle values of
/// components in each voxel.
/// If this sum is greater than one, the system state get's copied into
/// this array. Otherwise the component get's set to zero.
///
/// You can think about it like a kind of relu activation.
/// Only if all three components added up are greater than one,
/// do we want to dercease the components, otherwise the
/// magnitude should not influence the loss and therefore
/// the derivatives.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MaximumRegularization(Array1<f32>);

impl MaximumRegularization {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_states: usize) -> Self {
        trace!("Creating ArrayMaximumRegularization");
        Self(Array1::zeros(number_of_states))
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn to_gpu(&self, queue: &ocl::Queue) -> Result<Buffer<f32>> {
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(
                self.as_slice()
                    .context("Failed to get array slice for GPU copy")?,
            )
            .build()
            .context("Failed to build GPU buffer for maximum regularization")?;
        Ok(buffer)
    }

    #[cfg(feature = "native")]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn update_from_gpu(&mut self, maximum_regularization: &Buffer<f32>) -> Result<()> {
        maximum_regularization
            .read(
                self.as_slice_mut()
                    .context("Failed to get mutable array slice for GPU read")?,
            )
            .enq()
            .context("Failed to read maximum regularization from GPU buffer")?;
        Ok(())
    }
}

impl Deref for MaximumRegularization {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaximumRegularization {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
