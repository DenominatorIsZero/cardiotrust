__kernel void update_coefs(
    __global float* coefs,
    __global int* delays,
    __global const float* derivatives_coefs,
    float learning_rate_over_batch_size,
    int num_voxels
    ){
        int voxel_idx = get_global_id(0);
        int offset_idx = get_global_id(1);
        int num_offsets = 26;
        float margin = 1e-4;

        if (voxel_idx >= num_voxels || offset_idx >= num_offsets) return;

        float new_coef = coefs[voxel_idx * num_offsets + offset_idx] - derivatives_coefs[voxel_idx * num_offsets + offset_idx] * learning_rate_over_batch_size;
        int delay = delays[voxel_idx * num_offsets + offset_idx];

        if (new_coef < margin){
            if (delay < 1000){
                coefs[voxel_idx * num_offsets + offset_idx] = 1.0f - 2.0f* margin;
                delays[voxel_idx * num_offsets + offset_idx] = delay + 1;
            }else{
                coefs[voxel_idx * num_offsets + offset_idx] = margin;
            }
        }else if(new_coef > 1.0f - margin){
            if (delay > 1){
                coefs[voxel_idx * num_offsets + offset_idx] = 2.0f * margin;
                delays[voxel_idx * num_offsets + offset_idx] = delay - 1;
            } else{
                coefs[voxel_idx * num_offsets + offset_idx] = 1.0f - margin;
            }
        }
    }