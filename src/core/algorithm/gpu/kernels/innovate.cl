void atomic_add_float(volatile __global float *addr, float val);

__kernel void innovate_system_states(
    __global float* ap_outputs_now,
    __global float* ap_outputs_last,
    __global float* system_states,
    __global const float* ap_coefs,
    __global const int* ap_delays,
    __global const float* ap_gains,
    __global const int* output_state_indices,
    __global int* step,
    __local float* partial_sums,
    const int num_states
) {
    int index_state = get_global_id(0);
    int index_offset = get_local_id(1);
    int num_offsets = 78;

    
    // Boundary checks
    if (index_state >= num_states) return;

    int step_idx = step[0];

    // Copy last outputs for derivative calculation
    int ap_idx = index_state * num_offsets + index_offset;
    float contribution = 0.0f;
    
    // Get output state index
    int output_state_idx = output_state_indices[ap_idx];
    if (output_state_idx != -1 && index_offset < 78){ 
        // Calculate indices
        int coef_index = (index_state / 3) * (num_offsets / 3) + (index_offset / 3);
        ap_outputs_last[ap_idx] = ap_outputs_now[ap_idx];
        
        // Get parameters
        float coef = ap_coefs[coef_index];
        int delay = ap_delays[coef_index];
        
        // Calculate delayed inputs
        float input = (delay <= step_idx) ? 
            system_states[(step_idx - delay) * num_states + output_state_idx] : 0.0f;
        float input_delayed = (delay < step_idx) ?
            system_states[(step_idx - delay - 1) * num_states + output_state_idx] : 0.0f;
        
        // Update ap output
        float ap_output = coef * (input - ap_outputs_last[ap_idx]) + input_delayed;
        ap_outputs_now[ap_idx] = ap_output;
        
        // Update system state with gain
        float gain = ap_gains[ap_idx];
        contribution = gain * ap_output;
    }
    partial_sums[index_offset] = contribution;
    barrier(CLK_LOCAL_MEM_FENCE);

    for(int stride = num_offsets>>1; stride > 0; stride >>= 1) {
        if(index_offset < stride) {
            partial_sums[index_offset] += partial_sums[index_offset + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    if(index_offset == 0) {
        system_states[step_idx * num_states + index_state] = partial_sums[0];
    }
}