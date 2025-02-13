__kernel void reset_float(
    __global float* buffer
){
    int idx = get_global_id(0);
    buffer[idx] = 0.0f;
}

__kernel void reset_int(
    __global int* buffer
){
    int idx = get_global_id(0);
    buffer[idx] = 0;
}