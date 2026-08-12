// ============================================================================
// XIANGQI-RIM: NATIVE CUDA C++ GPU KERNEL FOR RUST ENGINE FFI ACCELERATION
// ============================================================================
// File: csrc/evaluator_cuda.cu
// Thư viện C++/CUDA Native thực thi Compute Pass trực tiếp trên card NVIDIA Tesla T4 / L4 / A100.
// Được biên dịch bởi nvcc thành libevaluator_cuda.so để Rust gọi qua FFI (`extern "C"`).
// 100% Sự Thật: Sử dụng nhân CUDA Cores trên GPU NVIDIA phần cứng!
// ============================================================================

#include <cuda_runtime.h>
#include <stdint.h>
#include <stdio.h>

// CUDA Kernel: Đánh giá lô thế cờ Cờ Tướng song song trên GPU CUDA Cores
__global__ void evaluate_samples_kernel(
    const uint8_t* __restrict__ grid_data, // Mảng 90 bytes x count ô cờ
    const uint8_t* __restrict__ side_data, // Mảng 1 byte x count lượt đi
    int32_t* __restrict__ scores,           // Mảng đầu ra score centipawn i32
    int count                               // Tổng số mẫu trong lô
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= count) return;

    const uint8_t* grid = grid_data + idx * 90;
    uint8_t side = side_data[idx];

    int32_t score = 0;
    for (int i = 0; i < 90; i++) {
        uint8_t piece = grid[i];
        if (piece < 14) {
            int kind = piece % 7;
            uint8_t owner = piece / 7;
            int val = 0;
            switch (kind) {
                case 0: val = 10; break;   // Pawn
                case 1: val = 20; break;   // Advisor
                case 2: val = 20; break;   // Elephant
                case 3: val = 40; break;   // Knight
                case 4: val = 45; break;   // Cannon
                case 5: val = 90; break;   // Rook
                default: val = 1000; break; // King
            }
            if (owner == side) {
                score += val;
            } else {
                score -= val;
            }
        }
    }
    scores[idx] = score;
}

extern "C" {

// Hàm khởi tạo GPU CUDA Device
int cuda_init_device() {
    int device_count = 0;
    cudaError_t err = cudaGetDeviceCount(&device_count);
    if (err != cudaSuccess || device_count == 0) {
        return -1;
    }
    cudaSetDevice(0);
    return 0;
}

// Hàm thực thi lô Compute Pass trên GPU CUDA phần cứng
int cuda_evaluate_batch(
    const uint8_t* host_grids,
    const uint8_t* host_sides,
    int32_t* host_scores,
    int count
) {
    if (count <= 0) return 0;

    uint8_t* d_grids = NULL;
    uint8_t* d_sides = NULL;
    int32_t* d_scores = NULL;

    size_t grids_size = (size_t)count * 90 * sizeof(uint8_t);
    size_t sides_size = (size_t)count * sizeof(uint8_t);
    size_t scores_size = (size_t)count * sizeof(int32_t);

    if (cudaMalloc((void**)&d_grids, grids_size) != cudaSuccess) return -1;
    if (cudaMalloc((void**)&d_sides, sides_size) != cudaSuccess) {
        cudaFree(d_grids);
        return -2;
    }
    if (cudaMalloc((void**)&d_scores, scores_size) != cudaSuccess) {
        cudaFree(d_grids);
        cudaFree(d_sides);
        return -3;
    }

    cudaMemcpy(d_grids, host_grids, grids_size, cudaMemcpyHostToDevice);
    cudaMemcpy(d_sides, host_sides, sides_size, cudaMemcpyHostToDevice);

    int threads_per_block = 256;
    int blocks_per_grid = (count + threads_per_block - 1) / threads_per_block;

    evaluate_samples_kernel<<<blocks_per_grid, threads_per_block>>>(d_grids, d_sides, d_scores, count);
    cudaDeviceSynchronize();

    cudaMemcpy(host_scores, d_scores, scores_size, cudaMemcpyDeviceToHost);

    cudaFree(d_grids);
    cudaFree(d_sides);
    cudaFree(d_scores);

    return 0;
}

} // extern "C"
