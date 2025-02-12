__kernel void update_gains(
    __global float* gains,
    __global const float* derivatives_gains,
    float learning_rate_over_batch_size,
    int num_states
    ){
        int state_idx = get_global_id(0);
        int offset_idx = get_global_id(1);
        int num_offsets = 78;

        if (state_idx >= num_states || offset_idx >= num_offsets) return;

        gains[state_idx * num_offsets + offset_idx] -= derivatives_gains[state_idx * num_offsets + offset_idx] * learning_rate_over_batch_size;
    }