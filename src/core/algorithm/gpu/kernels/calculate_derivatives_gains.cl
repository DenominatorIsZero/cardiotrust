__kernel void calculate_derivatives_gains(
    __global float* derivatives_gains,
    __global const float* ap_outputs,
    __global const float* maximum_regularization,
    __global const float* mapped_residuals,
    float mse_scaling,
    float regularization_scaling,
    int num_states
) {
    int state_index = get_global_id(0);
    int offset_index = get_global_id(1);
    int num_offsets = 78;
    
    if (state_index >= num_states || offset_index >= num_offsets) return;
    
    float ap_output = ap_outputs[state_index * num_offsets + offset_index];
    float max_reg = maximum_regularization[state_index];
    float residual = mapped_residuals[state_index];
    
    derivatives_gains[state_index * num_offsets + offset_index] += 
        ap_output * (residual * mse_scaling + max_reg * regularization_scaling);
}