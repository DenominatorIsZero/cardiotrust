use anyhow::{Context as AnyhowContext, Result};
use ocl::{Context, Device, Platform, Queue};

pub mod derivation;
pub mod epoch;
pub mod helper;
pub mod metrics;
pub mod prediction;
pub mod reset;
pub mod update;

#[derive(Debug, Clone)]
pub struct GPU {
    pub queue: Queue,
    pub context: Context,
    pub device: Device,
}

impl GPU {
    /// Creates a new `GPU` instance.
    ///
    /// This function initializes the `OpenCL` platform, device, context, and queue
    /// required for GPU-accelerated computations. The resulting `GPU` struct
    /// contains references to these `OpenCL` objects, which can be used to execute
    /// GPU kernels.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU device detection, context creation, or queue initialization fails.
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new() -> Result<Self> {
        let platform = Platform::default();
        let device = Device::first(platform)
            .with_context(|| "Failed to find first GPU device - no OpenCL devices available")?;

        let device_type = device
            .info(ocl::core::DeviceInfo::Type)
            .with_context(|| "Failed to query GPU device type information")?;

        if device_type.to_string() != ocl::core::DeviceInfoResult::Type(ocl::core::DeviceType::GPU).to_string() {
            return Err(anyhow::anyhow!(
                "Device is not a GPU - found device type: {}",
                device_type.to_string()
            ));
        }

        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .with_context(|| "Failed to create OpenCL context for GPU device")?;

        let queue = Queue::new(&context, device, None)
            .with_context(|| "Failed to create OpenCL command queue")?;

        Ok(Self {
            queue,
            context,
            device,
        })
    }
}


#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use anyhow::Context as AnyhowContext;
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    use crate::core::algorithm::gpu::GPU;

    #[test]
    fn test_atomic_add_float() -> anyhow::Result<()> {
        // Setup OpenCL
        let platform = Platform::default();
        let device = Device::first(platform)
            .with_context(|| "Failed to find OpenCL device for atomic test")?;
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .with_context(|| "Failed to create OpenCL context for atomic test")?;
        let queue = Queue::new(&context, device, None)
            .with_context(|| "Failed to create OpenCL queue for atomic test")?;

        // Create test buffer
        let initial_value = vec![1.0f32];
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .flags(ocl::MemFlags::new().read_write())
            .len(1)
            .copy_host_slice(&initial_value)
            .build()
            .with_context(|| "Failed to create test buffer for atomic operations")?;

        // Load and build kernel
        let kernel_src = std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl")
            .with_context(|| "Failed to read atomic kernel source file")?;
        let test_src = r"
            __kernel void test_atomic_add(__global float* value) {
                atomic_add_float(value, 1.0f);
            }
        ";

        let program = Program::builder()
            .src(format!("{kernel_src}\n{test_src}"))
            .build(&context)
            .with_context(|| "Failed to build OpenCL program for atomic test")?;

        let kernel = Kernel::builder()
            .program(&program)
            .name("test_atomic_add")
            .queue(queue)
            .global_work_size(100) // Run 100 parallel additions
            .arg(&buffer)
            .build()
            .with_context(|| "Failed to build atomic test kernel")?;

        // Run kernel
        unsafe {
            kernel.enq()
                .with_context(|| "Failed to execute atomic test kernel")?;
        }
        // Verify result
        let mut result = vec![0.0f32];
        buffer.read(&mut result).enq()
            .with_context(|| "Failed to read atomic test results from GPU buffer")?;

        assert_relative_eq!(result[0], 101.0, epsilon = 1e-6); // Initial 1.0 + 100 additions of 1.0
        Ok(())
    }

    #[test]
    fn test_new_gpu() -> anyhow::Result<()> {
        let gpu = GPU::new()?;
        println!("GPU: {gpu:?}");
        Ok(())
    }
}
