# Project: XiangRust Round 6 — Integrated GPU Acceleration Platform (Intel iGPU 512MB)

## Architecture
- Clean Room Design (0 external crates in `src/`, std-only Rust).
- Single-Word English Identifiers for all code constructs (`Device`, `Backend`, `Guard`, `Buffer`, `Kernel`, `Evaluator`, `Sample`, `Status`).
- Hardware Alignment: `#[repr(C, align(64))]` for main structs to prevent false sharing; `align(16)` / `align(32)` for compact records.
- 100% Vietnamese comments in code and 100% Vietnamese in technical documentation.
- Ports & Adapters Architecture (`GpuAdapter` / `Device`, `Backend`, `Guard`, `Buffer`, `Evaluator`, `Kernel`).
- Unified Memory 0-Copy (`MTLResourceStorageModeShared` / `CL_MEM_USE_HOST_PTR` for Intel iGPU macOS).
- Asynchronous Lock-Free Ring Buffer Queue for non-blocking batch submission.
- Autonomous GPU Evaluator Kernel for NNUE matrix multiplication & accumulator updates.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Multi-Backend GPU Adapter (`Device`) | Metal API Native on macOS (Intel iGPU 512MB) with OpenCL/CPU SIMD fallback | M1 | survey |
| 2 | VRAM OOM Protection (`Guard`) | Strict 512MB VRAM allocation ceiling & zero-copy buffer pool | M1 | survey |
| 3 | VRAM Aligned Buffer (`Buffer`) | 64-byte hardware aligned VRAM allocation & host-device transfer | M1 | survey |
| 4 | NNUE Batch Evaluator (`Evaluator`) | Parallel matrix multiplication for 1k-8k positions on GPU | M2 | survey |
| 5 | Parallel Search Kernel (`Kernel`) | Leaf node evaluation offloading for PVS search tree | M2 | survey |
| 6 | GYM Depth 12 GPU Integration (`Gym`) | Acceleration pipeline integration into `src/learn/gym.rs` | M3 | survey |
| 7 | Executable GPU Example (`examples/15_gpu_acceleration.rs`) | Demonstration script for GPU detection, VRAM profiling & GYM benchmarks | M4 | survey |
| 8 | Technical Architecture Specs (`docs/gpu_acceleration_architecture.md`) | 100% Vietnamese GPU architecture, Metal FFI specs & benchmarks | M4 | survey |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | GPU Adapter & VRAM Guard | `src/gpu/mod.rs`, `device.rs`, `backend.rs`, `guard.rs`, `buffer.rs` | none | DONE |
| M2 | NNUE Batch Evaluator & Parallel Search Kernel | `src/gpu/sample.rs`, `batch.rs`, `kernel.rs`, `evaluator.rs`, `status.rs` | M1 | DONE |
| M3 | GYM Depth 12 Integration | `src/gpu/gym.rs`, `src/learn/gym.rs` | M1, M2 | DONE |
| M4 | Example Script, Architecture Docs & Test Suite | `examples/15_gpu_acceleration.rs`, `docs/gpu_acceleration_architecture.md`, tests | M1, M2, M3 | DONE |

## Interface Contracts
### `gpu` ↔ `learn` & `search`
- `Device`: `#[repr(C, align(64))]`, init device, detect backend, allocate VRAM buffer
- `Guard`: `#[repr(C, align(64))]`, check 512MB limit, enforce VRAM allocation safety
- `Buffer`: `#[repr(C, align(64))]`, continuous VRAM slice host-device transfer
- `Evaluator`: `#[repr(C, align(64))]`, evaluate position batch, matrix multiplication
- `Kernel`: `#[repr(C, align(64))]`, compute shader pipeline for leaf nodes

## Code Layout
- `src/gpu/mod.rs`
- `src/gpu/device.rs`
- `src/gpu/backend.rs`
- `src/gpu/guard.rs`
- `src/gpu/buffer.rs`
- `src/gpu/sample.rs`
- `src/gpu/batch.rs`
- `src/gpu/kernel.rs`
- `src/gpu/evaluator.rs`
- `src/gpu/status.rs`
- `src/gpu/gym.rs`
- `examples/15_gpu_acceleration.rs`
- `docs/gpu_acceleration_architecture.md`
