__kernel void add_control_function(
    __global float* system_states,
    __global const float* control_matrix,
    __global const int* step,
    __global float* control_values,
    const int num_states
) {
    int state_idx = get_global_id(0);
    if (state_idx >= num_states) return;
    int step_idx = step[0];
    
    system_states[step_idx * num_states + state_idx] += 
        control_values[step_idx] * control_matrix[state_idx];
}