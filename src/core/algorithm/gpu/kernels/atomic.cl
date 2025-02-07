inline void atomic_add_float(volatile __global float *addr, float val) {
    union {
        uint  u32;
        float f32;
    } next, expected, current;
    
    current.f32 = *addr;
    do {
        expected.f32 = current.f32;
        next.f32 = expected.f32 + val;
        current.u32 = atomic_cmpxchg(
            (volatile __global uint *)addr,
            expected.u32,
            next.u32
        );
    } while (current.u32 != expected.u32);
}