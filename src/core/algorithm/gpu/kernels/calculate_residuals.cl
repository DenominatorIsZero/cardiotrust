__kernel void calculate_residuals(
    __global float* residuals,
    __global const float* predicted_measurements,
    __global const float* actual_measurements,
    __global int* step,
    __global int* beat,
    int num_sensors,
    int num_steps
) {
    int sensor_idx = get_global_id(0);
    if (sensor_idx >= num_sensors) return;
    int step_idx = step[0];
    int beat_idx = beat[0];
    
    residuals[sensor_idx] = predicted_measurements[beat_idx * num_sensors * num_steps + step_idx * num_sensors + sensor_idx] - actual_measurements[beat_idx * num_sensors * num_steps + step_idx * num_sensors + sensor_idx];
}