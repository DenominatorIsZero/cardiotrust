pub mod prediction;

#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

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
        let mut initial_value = vec![1.0f32];
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
            .src(format!("{}\n{}", kernel_src, test_src))
            .build(&context)
            .unwrap();

        let kernel = Kernel::builder()
            .program(&program)
            .name("test_atomic_add")
            .queue(queue.clone())
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

        assert_eq!(result[0], 101.0); // Initial 1.0 + 100 additions of 1.0
    }
}
