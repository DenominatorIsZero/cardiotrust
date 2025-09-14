# CardioTrust Architecture

This document provides a high-level technical overview of the CardioTrust system architecture, application workflow, and core algorithms.

## System Overview

CardioTrust is a research tool for cardiac electrophysiological simulation and non-invasive localization of arrhythmogenic tissue. The system uses a state-space-based framework to estimate myocardial current density distributions from magnetic field measurements.

### Core Components

The architecture is organized into three main layers:

- **`src/core/`** - Core simulation and estimation algorithms
- **`src/ui/`** - EGUI-based user interface for configuration and results
- **`src/vis/`** - Bevy-powered 3D visualization and plotting

## Application Usage Workflow

### 1. Scenario Creation

Users create simulation scenarios by configuring:

- **Ground Truth Model** - The "actual" cardiac tissue to simulate
- **Initial Model** - Starting point for the estimation algorithm
- **Algorithm Parameters** - Optimization settings and constraints

_Key files: `src/core/scenario.rs`, `src/core/config/`_

### 2. Simulation Execution

- Start simulation runs through the UI
- Multiple scenarios can run in parallel
- Progress tracking and status monitoring

_Key files: `src/scheduler.rs`, `src/core/algorithm/`_

### 3. Results Analysis

- **2D Plotting** - Time series, parameter evolution, error metrics
- **3D Visualization** - Interactive heart model with current density overlays
- **Data Export** - Results in various formats (NPY, PNG, GIF)

_Key files: `src/ui/results.rs`, `src/vis/plotting/`, `src/vis/heart.rs`_

## Core Algorithm

### Mathematical Foundation

The system implements a state-space approach for non-invasive electroanatomical mapping:

**Model Structure:**

- **Voxels** - 3D spatial discretization of cardiac tissue
- **All-Pass Filters** - Connect voxels with continuous, differentiable group delay
- **State Propagation** - Current density propagates through neighboring voxels

**Measurement Calculation:**

- **Biot-Savart Law** - Magnetic field computation from current densities
- **Sensor Arrays** - Configurable magnetometer positions and orientations

**Optimization Process:**

- **Objective Function** - Mean squared error between measured and simulated magnetic signals
- **Gradient Descent** - Parameter optimization using differentiation
- **Regularization** - Smoothness constraints and anatomical priors

_For detailed mathematical formulations, see the publications referenced in [README.md](../README.md)._

### Technology Choices

**Core Dependencies:**

- **Bevy** - 3D visualization engine and ECS framework
- **nalgebra** - Linear algebra and mathematical operations
- **ndarray** - N-dimensional array operations for scientific computing
- **egui** - Immediate mode GUI for interactive controls
- **OpenCL** - GPU acceleration with CPU fallback

**File Format Support:**

- **NIFTI** - Medical imaging data (MRI, anatomical models)
- **NPY** - NumPy arrays for data exchange
- **TOML** - Human-readable configuration files
- **Binary** - High-performance serialization with bincode

### GPU Acceleration Strategy

The system provides user-selectable execution paths:

**GPU Implementation:**

- OpenCL kernels for high-performance computation
- Fewer configuration options (implementation in progress)

_Key files: `src/core/algorithm/gpu/`_

**CPU Implementation:**

- Full-featured algorithm implementations
- Complete configuration options available
- Consistent results with GPU implementations
- Used when GPU not selected or unavailable

_Key files: `src/core/algorithm/refinement/`, `src/core/algorithm/estimation/`_

## Data Flow

```
Scenario Configuration (TOML)
    ↓
Model Creation (Voxels + Sensors)
    ↓
Data Generation/Loading (Ground Truth)
    ↓
Algorithm Execution (GPU/CPU)
    ↓
Results Storage (Binary + Summary)
    ↓
Visualization & Analysis (2D/3D)
```

## Module Organization

### Core (`src/core/`)

- **`algorithm/`** - Main estimation algorithms (pseudo-inverse, model-based, GPU)
- **`config/`** - Configuration structures for all system components
- **`data/`** - Simulation data handling and generation
- **`model/`** - Spatial (voxels, sensors) and functional (filters) model descriptions
- **`scenario/`** - Scenario execution, results management, and status tracking

### User Interface (`src/ui/`)

- **`scenario/`** - Scenario configuration and management interface
- **`results.rs`** - Results visualization and exploration
- **`explorer.rs`** - File and scenario browsing
- **`vol.rs`** - Volume rendering controls

### Visualization (`src/vis/`)

- **`plotting/`** - 2D plotting utilities (PNG/GIF export)
- **`heart.rs`** - 3D heart model rendering
- **`sensors.rs`** - Sensor array visualization
- **`cutting_plane.rs`** - Cross-sectional visualization tools

## Execution Modes

### Native Application (`src/bin/main.rs`)

Full desktop application with:

- Complete file system access
- GPU acceleration support
- Interactive 3D visualization
- Comprehensive logging

### Experiment Planner (`src/bin/planner.rs`)

Batch scenario generation for:

- Parameter sweeps
- Statistical studies
- Automated experiment design

## Performance Considerations

### Build Optimization

- Development builds use O3 optimization for debugging performance (required for useful model sizes)
- Release builds employ thin LTO for size and speed
- Profiling support with flamegraph generation
- Efficient serialization with bincode
- Configurable parallel processing at scenario level (optimized for throughput)
- GPU implementation optimized for single-scenario latency

### Testing Strategy

- Two-phase testing (application run required for data generation) - to be improved for faster/easier testing
- Visual output verification
- Comprehensive benchmarking suite
- Algorithm consistency validation

_Key files: `benches/`, `tests/`_

---

_For implementation details and mathematical foundations, refer to the publications listed in [README.md](../README.md). For development guidelines, see [CONTRIBUTING.md](../CONTRIBUTING.md)._
