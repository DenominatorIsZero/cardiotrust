void atomic_add_float(volatile __global float *addr, float val);

__kernel void predict_measurements(
    __global float* measurements,
    __global const float* measurement_matrix,
    __global const float* system_states,
    __global const int* beat,
    __global const int* step,
    int num_sensors,
    int num_states,
    int num_steps
) {
    int sensor_idx = get_global_id(0);
    int state = get_global_id(1);
    
    if (sensor_idx >= num_sensors || state >= num_states) return;
    
    int step_idx = step[0];
    int beat_idx = beat[0];
    
    float contribution = measurement_matrix[beat_idx * num_sensors * num_states + sensor_idx * num_states + state] 
                      * system_states[step_idx * num_states + state];
    
    atomic_add_float(&measurements[beat_idx * num_sensors * num_steps + step_idx * num_sensors + sensor_idx], contribution);
}