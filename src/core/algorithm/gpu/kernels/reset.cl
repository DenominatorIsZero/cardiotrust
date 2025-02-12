__kernel void reset(
    __global float* buffer
){
    int idx = get_global_id(0);
    buffer[idx] = 0.0f;
}