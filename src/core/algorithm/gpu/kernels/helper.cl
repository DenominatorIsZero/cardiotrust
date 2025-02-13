__kernel void increase_int(
    __global int* buffer
){
    int idx = get_global_id(0);
    buffer[idx] += 1;
}