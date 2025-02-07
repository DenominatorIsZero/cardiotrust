void atomic_add_float(volatile __global float *addr, float val);

__kernel void innovate_system_states(
    __global float* ap_outputs_now,
    __global float* ap_outputs_last,
    __global float* system_states,
    __global const float* ap_coefs,
    __global const int* ap_delays,
    __global const float* ap_gains,
    __global const int* output_state_indices,
    const int step,
    const int num_states,
    const int num_offsets
) {
    int index_state = get_global_id(0);
    int index_offset = get_global_id(1);
    
    // Boundary checks
    if (index_state >= num_states || index_offset >= num_offsets)
        return;
        
    // Copy last outputs for derivative calculation
    int output_idx = index_state * num_offsets + index_offset;
    ap_outputs_last[output_idx] = ap_outputs_now[output_idx];
    
    // Get output state index
    int output_state_idx = output_state_indices[output_idx];
    if (output_state_idx == -1) // Handle None case
        return;
        
    // Calculate indices
    int coef_index = (index_state / 3) * (num_offsets / 3) + (index_offset / 3);
    
    // Get parameters
    float coef = ap_coefs[coef_index];
    int delay = ap_delays[coef_index];
    
    // Calculate delayed inputs
    float input = (delay <= step) ? 
        system_states[(step - delay) * num_states + output_state_idx] : 0.0f;
    float input_delayed = (delay < step) ?
        system_states[(step - delay - 1) * num_states + output_state_idx] : 0.0f;
    
    // Update ap output
    float ap_output = coef * (input - ap_outputs_now[output_idx]) + input_delayed;
    ap_outputs_now[output_idx] = ap_output;
    
    // Update system state with gain
    float gain = ap_gains[output_idx];
    atomic_add_float(&system_states[step * num_states + index_state], gain * ap_output);
}