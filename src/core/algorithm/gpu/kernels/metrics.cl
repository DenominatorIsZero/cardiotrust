void atomic_add_float(volatile __global float *addr, float val);

__kernel void calculate_mse_step(
    __global const float* residuals,
    __global float* loss_mse,
    __local float* partial_sums,
    __global const int* step,
    const int num_sensors
) {
    int idx = get_global_id(0);
    int lid = get_local_id(0);
    int step_idx = step[0];
    
    // Load and square into local memory
    float contribution = 0.0f;
    if (idx < num_sensors) {
        float val = residuals[idx];
        contribution = val * val;
    }
    partial_sums[lid] = contribution;
    
    barrier(CLK_LOCAL_MEM_FENCE);
    
    // Parallel reduction in local memory
    for (int stride = get_local_size(0)>>1; stride > 0; stride >>= 1) {
        if (lid < stride) {
            partial_sums[lid] += partial_sums[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    // Write result to global memory
    if (lid == 0) {
        atomic_add_float(&loss_mse[step_idx], partial_sums[0] / num_sensors);
    }
}

__kernel void store_max_reg(
    __global float* loss_maximum_regularization,
    __global const float* maximum_regularization_sum,
    __global const int* step
) {
    int step_idx = step[0];
    if (get_global_id(0) == 0) {
        loss_maximum_regularization[step_idx] = maximum_regularization_sum[0];
    }
}

__kernel void calculate_final_loss(
    __global float* loss,
    __global const float* loss_mse,
    __global const float* loss_maximum_regularization,
    __global const int* step,
    const float regularization_strength
) {
    int step_idx = step[0];
    if (get_global_id(0) == 0) {
        loss[step_idx] = regularization_strength * loss_maximum_regularization[step_idx] + loss_mse[step_idx];
    }
}

__kernel void calculate_metrics_batch(
    __global float* loss_mse_batch,
    __global float* loss_maximum_regularization_batch,
    __global float* loss_batch,
    __global const float* loss_mse,
    __global const float* loss_maximum_regularization,
    __global const float* loss,
    __local float* partial_sums_mse,
    __local float* partial_sums_max_reg,
    __local float* partial_sums_loss,
    __global const int* epoch,
    int num_steps
) {
    int gid = get_global_id(0);
    int lid = get_local_id(0);
    int local_size = get_local_size(0);
    int epoch_idx = epoch[0];
    
    float sum_mse = 0.0f;
    float sum_max_reg = 0.0f;
    float sum_loss = 0.0f;
    
    if (gid < num_steps) {
        sum_mse = loss_mse[gid];
        sum_max_reg = loss_maximum_regularization[gid];
        sum_loss = loss[gid];
    }
    
    partial_sums_mse[lid] = sum_mse;
    partial_sums_max_reg[lid] = sum_max_reg;
    partial_sums_loss[lid] = sum_loss;
    barrier(CLK_LOCAL_MEM_FENCE);
    
    for(int stride = local_size/2; stride > 0; stride >>= 1) {
        if(lid < stride) {
            partial_sums_mse[lid] += partial_sums_mse[lid + stride];
            partial_sums_max_reg[lid] += partial_sums_max_reg[lid + stride];
            partial_sums_loss[lid] += partial_sums_loss[lid + stride];
        }
        barrier(CLK_LOCAL_MEM_FENCE);
    }
    
    if(lid == 0) {
        float mean_mse = partial_sums_mse[0] / num_steps;
        float mean_max_reg = partial_sums_max_reg[0] / num_steps;
        float mean_loss = partial_sums_loss[0] / num_steps;
        
        atomic_add_float(&loss_mse_batch[epoch_idx], mean_mse);
        atomic_add_float(&loss_maximum_regularization_batch[epoch_idx], mean_max_reg);
        atomic_add_float(&loss_batch[epoch_idx], mean_loss);
    }
}