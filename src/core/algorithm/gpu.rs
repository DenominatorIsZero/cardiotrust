use ocl::{Context, Device, Platform, Queue};

pub mod derivation;
pub mod prediction;

#[derive(Debug, Clone)]
pub struct GPU {
    pub queue: Queue,
    pub context: Context,
    pub device: Device,
}

impl GPU {
    #[must_use]
    /// Creates a new `GPU` instance.
    ///
    /// This function initializes the `OpenCL` platform, device, context, and queue
    /// required for GPU-accelerated computations. The resulting `GPU` struct
    /// contains references to these `OpenCL` objects, which can be used to execute
    /// GPU kernels.
    ///
    /// # Panics
    /// If there is an error during the initialization process, this function will panic.
    pub fn new() -> Self {
        let platform = Platform::default();
        let device = Device::first(platform).unwrap();
        assert_eq!(
            device
                .info(ocl::core::DeviceInfo::Type)
                .unwrap()
                .to_string(),
            ocl::core::DeviceInfoResult::Type(ocl::core::DeviceType::GPU).to_string()
        );
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .unwrap();
        let queue = Queue::new(&context, device, None).unwrap();
        Self {
            queue,
            context,
            device,
        }
    }
}

impl Default for GPU {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    use crate::core::algorithm::gpu::GPU;

    #[test]
    fn test_atomic_add_float() {
        // Setup OpenCL
        let platform = Platform::default();
        let device = Device::first(platform).unwrap();
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .unwrap();
        let queue = Queue::new(&context, device, None).unwrap();

        // Create test buffer
        let initial_value = vec![1.0f32];
        let buffer = Buffer::builder()
            .queue(queue.clone())
            .flags(ocl::MemFlags::new().read_write())
            .len(1)
            .copy_host_slice(&initial_value)
            .build()
            .unwrap();

        // Load and build kernel
        let kernel_src =
            std::fs::read_to_string("src/core/algorithm/gpu/kernels/atomic.cl").unwrap();
        let test_src = r"
            __kernel void test_atomic_add(__global float* value) {
                atomic_add_float(value, 1.0f);
            }
        ";

        let program = Program::builder()
            .src(format!("{kernel_src}\n{test_src}"))
            .build(&context)
            .unwrap();

        let kernel = Kernel::builder()
            .program(&program)
            .name("test_atomic_add")
            .queue(queue)
            .global_work_size(100) // Run 100 parallel additions
            .arg(&buffer)
            .build()
            .unwrap();

        // Run kernel
        unsafe {
            kernel.enq().unwrap();
        }
        // Verify result
        let mut result = vec![0.0f32];
        buffer.read(&mut result).enq().unwrap();

        assert_relative_eq!(result[0], 101.0, epsilon = 1e-6); // Initial 1.0 + 100 additions of 1.0
    }

    #[test]
    fn test_new_gpu() {
        let gpu = GPU::new();
        println!("GPU: {gpu:?}");
    }
}
