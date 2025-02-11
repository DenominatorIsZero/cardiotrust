void atomic_add_float(volatile __global float *addr, float val);

__kernel void calculate_mapped_residuals(
    __global float* mapped_residuals,
    __global const float* measurement_matrix,
    __global const float* residuals,
    __global const int* beat,
    __local float* partial_sums,
    int num_states,
    int num_sensors
) {
    int state_idx = get_global_id(0);
    int sensor_idx = get_global_id(1);
    int lid = get_local_id(1);
    int local_size = get_local_size(1);

    if (state_idx >= num_states) return;

    float contribution = 0.0f;
    int beat_idx = beat[0];
    
    if (sensor_idx < num_sensors){
        int idx = beat_idx * num_sensors * num_states + sensor_idx * num_states + state_idx;
        float mat_entry = measurement_matrix[idx];
        float res = residuals[sensor_idx];
        contribution = mat_entry * res;
    }
    
    partial_sums[lid] = contribution;
    barrier(CLK_LOCAL_MEM_FENCE);
    
    for(int stride = local_size/2; stride > 0; stride >>= 1) {
        if(lid < stride) {
            partial_sums[lid] += partial_sums[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    if(lid == 0) {
        atomic_add_float(&mapped_residuals[state_idx], partial_sums[0]);
    }
}