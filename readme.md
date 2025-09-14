# CardioTrust

A research tool for cardiac electrophysiological simulation and non-invasive localization of arrhythmogenic tissue. This project serves as both a showcase of advanced cardiac modeling algorithms and a practical exploration of high-performance scientific computing in Rust.

**Original Repository**: [CRC1261/cardiotrust](https://github.com/CRC1261/cardiotrust)  
**Personal Fork**: [DenominatorIsZero/cardiotrust](https://github.com/DenominatorIsZero/cardiotrust)

> **Note**: This is a personal continuation of PhD research originally developed at Kiel University and funded by the German Research Foundation (DFG) through CRC 1261. The original version represents completed academic work, while this fork focuses on code quality improvements, professionalization, and ongoing enhancements.

## Overview

CardioTrust provides a state-space-based framework for:

- Non-invasive electroanatomical mapping (EAM) using magnetic measurements
- Estimation of myocardial current density distributions
- Simulation of cardiac tissue dynamics and pathological conditions
- Analysis of healthy and arrhythmogenic tissue patterns
- Interactive 3D visualization of results

## Key Features

### Interactive Visualization

- 3D visualization using Bevy engine
- Interactive GUI with egui
- Real-time parameter adjustment
- Plot generation with egui_plot
- Support for various data formats (NIFTI, NPY, PNG)

### State-Space Model

- Voxel-based anatomical structure modeling
- All-pass filters for current density propagation simulation
- Nested optimization procedure for model refinement
- Support for both healthy and pathological tissue states

### Magnetic Field Processing

- Integration with magnetocardiography (MCG) data
- Biot-Savart law calculations for magnetic field estimation
- Support for dynamic sensor arrays

### Performance Metrics

- Dice score calculation for tissue classification
- Precision and recall analysis
- Current density error estimation
- Comprehensive benchmarking suite

## Installation

### Prerequisites

- Rust (latest stable version) (https://www.rust-lang.org/tools/install)
- Cargo (comes with Rust)
- Just (command runner)

```bash
cargo install just
```

- nextest (test runner)

```bash
cargo install cargo-nextest
```

- Nightly Rust toolchain (for formatting)

```bash
rustup toolchain install nightly
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/DenominatorIsZero/cardiotrust.git
cd cardiotrust

# Build in release mode
cargo build --release

# Run tests
cargo nextest run --no-fail-fast # This will fail the first time
cargo run --release # Run test scenarios by hitting start, adjust the number of parallel jobs as needed
cargo nextest run --no-fail-fast # This should pass the second time
```

## Development

### Common Commands

```bash
# Development
just run                 # Debug build
just release             # Release build
just planner             # Run experiment planner

# Testing
just test                # Run tests with nextest
just test-all            # Run all tests including ignored ones

# Code Quality
just lint                # Run clippy checks (includes clippy-tracing)
just fmt                 # Format code (requires nightly toolchain)
just fmt-check           # Check formatting without making changes

# Build
just build               # Debug build
just build-release       # Release build

# Documentation
just doc                 # Build and open docs (no dependencies)
just doc-all             # Build and open docs (with dependencies)

# Benchmarking
just bench               # Run epoch benchmarks
just bench-all           # Run all benchmarks
just flamegraph         # Generate flamegraph (requires cargo-flamegraph)

# Maintenance
just clean               # Clean build artifacts and results
just check               # Run cargo check on all targets

# Combined Workflows
just work                # Run lint, test and benchmarks
just ci                  # Run fmt-check, lint, test (CI pipeline)
just dev                 # Run build and test (development workflow)
```

### Build Configurations

```bash
# Native release build
cargo build --release

# Development build (optimized for debugging)
cargo build
```

### AI-Assisted Development

As I continued working on this as a personal project, I started using Claude Code for refactoring and code quality improvements. For details on this workflow, see [`CLAUDE.md`](CLAUDE.md).

## Project Structure

- `src/bin/` - Entry points for main application and experiment planner.
- `src/core/` - Core simulation and estimation algorithms
- `src/ui/` - Graphical user interface components using egui
- `src/vis/` - 3D visualization using Bevy and plotting with egui_plot
- `assets/` - 3D models and assets, control function data and MRI data.
- `benches` - Benchmarking suite
- `logs` - Log files for debugging and diagnostics
- `results` - Results and analysis outputs (This folder can get very large, make sure to clean it up regularly)
- `tests` - Visual output of unit tests

## Technology Stack

### Core Dependencies

- `bevy` - 3D visualization and game engine
- `nalgebra` - Linear algebra computations
- `ndarray` - N-dimensional array operations
- `egui` - Immediate mode GUI
- `tracing` - Logging and diagnostics

### File Formats

- NIFTI medical imaging format support
- NPY array format support
- PNG image export
- GIF animation generation
- Binary serialization with bincode

### Development Tools

- Criterion for benchmarking
- Nextest for testing
- Just for command management
- Clippy and rustfmt for code quality

## Performance

The project uses optimized build configurations:

- Development builds use optimization level 3
- Release builds use thin LTO
- Test builds are optimized for performance

## Documentation

For detailed technical information and development guidelines:

- **[Architecture Overview](docs/architecture.md)** - System design, algorithm foundations, and technology choices
- **[Contributing Guide](CONTRIBUTING.md)** - Development guidelines, setup instructions, and contribution process
- **[AI Development Workflow](CLAUDE.md)** - Claude Code integration and development patterns

## Publications

1. Engelhardt, E., et al. (2025). "Gradient-Based Optimization of All-Pass Filter Networks for Non-Invasive Electroanatomical Mapping." IEEE Access, 13.

2. Engelhardt, E., et al. (2023). "Non-invasive Electroanatomical Mapping: A State-space Approach for Myocardial Current Density Estimation." Bioengineering, 10(12), p. 1432.

3. Engelhardt, E., et al. (2023). "A Concept for Myocardial Current Density Estimation with Magnetoelectric Sensors." Current Directions in Biomedical Engineering, 9(1), pp. 89-92.

## Related Work

The CRC 1261 project is currently ongoing with another PhD student. For more information about the current research, visit: [Project B10 - Biomagnetic Sensing](https://biomagnetic-sensing.de/index.php/research/projects/project-b10)

## Contributing

I don't expect contributions to this personal project, but if you find errors please feel free to create an issue or send me a message. I'm happy to answer any questions you might have.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

The original research (available at [CRC1261/cardiotrust](https://github.com/CRC1261/cardiotrust)) was funded by the German Research Foundation (Deutsche Forschungsgemeinschaft, DFG) through the Collaborative Research Center CRC 1261 Magnetoelectric Sensors: From Composite Materials to Biomagnetic Diagnostics.

**Original Research Team:**
- Erik Engelhardt - Digital Signal Processing and System Theory, Kiel University ([erik-engelhardt.com](https://erik-engelhardt.com/))
- Prof. Dr. med. Norbert Frey - University Medical Center Heidelberg  
- Prof. Dr.-Ing. Gerhard Schmidt - Kiel University
