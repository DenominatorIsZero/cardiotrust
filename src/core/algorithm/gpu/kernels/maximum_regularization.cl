void atomic_add_float(volatile __global float *addr, float val);

__kernel void calculate_maximum_regularization(
    __global float* maximum_regularization,
    __global float* maximum_regularization_sum,
    __global const float* system_states,
    __local float* partial_sums,
    float regularization_threshold,
    int num_voxels
) {
    int voxel_idx = get_global_id(0);
    int lid = get_local_id(0);
    int local_size = get_local_size(0);

    float factor_squared = 0.0f;
    
    if (voxel_idx < num_voxels) {
        int state_idx = voxel_idx * 3;
        float sum = fabs(system_states[state_idx]) + 
                    fabs(system_states[state_idx + 1]) + 
                    fabs(system_states[state_idx + 2]);
                    
        if (sum > regularization_threshold) {
            float factor = sum - regularization_threshold;
            factor_squared = factor * factor;

            maximum_regularization[state_idx] = factor * sign(system_states[state_idx]);
            maximum_regularization[state_idx + 1] = factor * sign(system_states[state_idx + 1]);
            maximum_regularization[state_idx + 2] = factor * sign(system_states[state_idx + 2]);
        } else {
            maximum_regularization[state_idx] = 0.0f;
            maximum_regularization[state_idx + 1] = 0.0f;
            maximum_regularization[state_idx + 2] = 0.0f;
        }
    }

    partial_sums[lid] = factor_squared;
    barrier(CLK_LOCAL_MEM_FENCE);
    
    for(int stride = local_size>>1; stride > 0; stride >>= 1) {
        if(lid < stride) {
            partial_sums[lid] += partial_sums[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    if(lid == 0) {
        atomic_add_float(maximum_regularization_sum, partial_sums[0]);
    }
}