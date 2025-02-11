__kernel void calculate_derivatives_coefs_fir(
    __global float* derivatives_fir,
    __global const float* system_states,
    __global const int* output_state_indices,
    __global const float* ap_coefs,
    __global const int* ap_delays,
    __global const int* step,
    int num_states
) {
    int state_index = get_global_id(0);
    int offset_index = get_global_id(1);
    int num_offsets = 78;
    int step_idx = step[0];
    
    if (state_index >= num_states || offset_index >= num_offsets) return;
    
    int output_state = output_state_indices[state_index * num_offsets + offset_index];
    if (output_state == -1) return;
    
    int coef_index = (state_index / 3) * (num_offsets / 3) + (offset_index / 3);
    int delay = ap_delays[coef_index];
    float coef = ap_coefs[coef_index];
    if (step_idx >= delay) {
        float state_val = system_states[(step_idx - delay) * num_states + output_state];
        float derivative_old = derivatives_fir[state_index * num_offsets + offset_index];
        float derivative_new = (-coef) * derivative_old + state_val;
        derivatives_fir[state_index * num_offsets + offset_index] = derivative_new;
    }
}

__kernel void calculate_derivatives_coefs_iir(
    __global float* derivatives_iir,
    __global const float* ap_outputs_last,
    __global const float* ap_coefs,
    __global const int* ap_delays,
    __global const int* step,
    int num_states
) {
    int state_index = get_global_id(0);
    int offset_index = get_global_id(1);
    int num_offsets = 78;
    int step_idx = step[0];
    
    if (state_index >= num_states || offset_index >= num_offsets) return;
    
    int coef_index = (state_index / 3) * (num_offsets / 3) + (offset_index / 3);
    int delay = ap_delays[coef_index];
    float coef = ap_coefs[coef_index];
    
    if (step_idx >= delay) {
        float ap_output_last = ap_outputs_last[state_index * num_offsets + offset_index];
        derivatives_iir[state_index * num_offsets + offset_index] = (-coef) * derivatives_iir[state_index * num_offsets + offset_index] + ap_output_last;
    }
}

__kernel void calculate_derivatives_coefs_combine(
    __global float* derivatives_coefs,
    __global const float* derivatives_iir,
    __global const float* derivatives_fir,
    __global const float* ap_gains,
    __global const float* mapped_residuals,
    __global const float* ap_coefs,
    __global const int* ap_delays,
    __local float* partial_sums,
    float mse_scaling,
    int num_states
) {
    int state_index = get_global_id(0);
    int offset_index = get_global_id(1);
    int lid_x = get_local_id(0);
    int lid_y = get_local_id(1);
    int local_idx = lid_y * 3 + lid_x;
    int num_offsets = 78;
    int coef_index = (state_index / 3) * (num_offsets / 3) + (offset_index / 3);
    
    float contribution = 0.0f;
    if (state_index < num_states && offset_index < num_offsets) {  
        float iir = derivatives_iir[state_index * num_offsets + offset_index];
        float fir = derivatives_fir[state_index * num_offsets + offset_index];
        float ap_gain = ap_gains[state_index * num_offsets + offset_index];
        float mapped_residual = mapped_residuals[state_index];
        
        contribution = ((fir - iir) * ap_gain * mapped_residual) * mse_scaling;
    }
    
    partial_sums[local_idx] = contribution;
    barrier(CLK_LOCAL_MEM_FENCE);

    // First step: 8 -> 4
    if (local_idx < 4) {
        partial_sums[local_idx] += partial_sums[local_idx + 4];
    }
    barrier(CLK_LOCAL_MEM_FENCE);

    // Second step: 4 -> 2
    if (local_idx < 2) {
        partial_sums[local_idx] += partial_sums[local_idx + 2];
    }
    barrier(CLK_LOCAL_MEM_FENCE);

    // Final step: 2 -> 1 (+ 9)
    if (local_idx == 0) {
        partial_sums[0] += partial_sums[1] + partial_sums[8];
        derivatives_coefs[coef_index] += partial_sums[0];
    }
}
