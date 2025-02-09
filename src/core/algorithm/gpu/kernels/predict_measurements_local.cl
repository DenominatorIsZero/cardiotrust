__kernel void predict_measurements(
    __global float* measurements,
    __global const float* measurement_matrix,
    __global const float* system_states,
    __global const int* beat,
    __global const int* step,
    __local float* partial_sums,
    const int num_sensors,
    const int num_states,
    const int num_steps
) {
    int sensor_idx = get_global_id(0);
    int lid = get_local_id(0);
    int local_size = get_local_size(0);
    
    if (sensor_idx >= num_sensors) return;
    int step_idx = step[0];
    int beat_idx = beat[0];
    
    // Compute partial dot products
    float temp = 0.0f;
    for (int state = lid; state < num_states; state += local_size) {
        temp += measurement_matrix[beat_idx * num_sensors * num_states + sensor_idx * num_states + state] 
              * system_states[step_idx * num_states + state];
    }
    
    // Store in local memory
    partial_sums[lid] = temp;
    barrier(CLK_LOCAL_MEM_FENCE);
    
    // Reduction in local memory
    for(int stride = local_size/2; stride > 0; stride >>= 1) {
        if(lid < stride) {
            partial_sums[lid] += partial_sums[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    // Write result
    if(lid == 0) {
        measurements[beat_idx * num_sensors * num_steps + step_idx * num_sensors + sensor_idx] = partial_sums[0];
    }
}