void atomic_add_float(volatile __global float *addr, float val);

__kernel void predict_measurements(
        __global float* measurements,
    __global const float* measurement_matrix,
    __global const float* system_states,
    __global const int* beat,
    __global const int* step,
    __local float* partial_sums,
    int num_sensors,
    int num_states,
    int num_steps
) {
    int sensor_idx = get_global_id(0);
    int state = get_global_id(1);
    int lid = get_local_id(1);
    int local_size = get_local_size(1);
    
    
    int step_idx = step[0];
    int beat_idx = beat[0];
    
    float contribution = 0.0f;
    if (state < num_states) {
        contribution = measurement_matrix[beat_idx * num_sensors * num_states + sensor_idx * num_states + state] 
                    * system_states[step_idx * num_states + state];
    }
    
    partial_sums[lid] = contribution;
    barrier(CLK_LOCAL_MEM_FENCE);
    
    for(int stride = local_size>>1; stride > 0; stride >>= 1) {
        if(lid < stride) {
            partial_sums[lid] += partial_sums[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    if(lid == 0) {
        atomic_add_float(&measurements[beat_idx * num_sensors * num_steps + step_idx * num_sensors + sensor_idx], partial_sums[0]);
    }
}