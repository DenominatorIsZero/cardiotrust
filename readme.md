# CardioTrust

CardioTrust is a research tool for cardiac electrophysiological simulation and non-invasive localization of arrhythmogenic tissue, developed as part of a PhD project at Kiel University. The project is funded by the German Research Foundation (Deutsche Forschungsgemeinschaft, DFG) through the Collaborative Research Center CRC 1261.

## Overview

CardioTrust provides a state-space-based framework for:

- Non-invasive electroanatomical mapping (EAM) using magnetic measurements
- Estimation of myocardial current density distributions
- Simulation of cardiac tissue dynamics and pathological conditions
- Analysis of healthy and arrhythmogenic tissue patterns
- Interactive 3D visualization of results

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

- wasm-bindgen-cli (for WASM builds)

```bash
cargo install -f wasm-bindgen-cli
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
# Run the application
just run                 # Debug build
just release             # Release build

# Testing
just test                # Run tests
just test-all            # Run all tests including ignored ones

# Code Quality
just lint                # Run clippy checks
just fmt                 # Format code (requires nightly toolchain)
just work                # Run lint, test and benchmarks

# Benchmarking
just bench               # Run epoch benchmarks
just flamegraph         # Generate flamegraph (requires cargo-flamegraph)

# WebAssembly
just wasm-build         # Build WASM target (debug)
just wasm-run           # Build and run WASM locally
just wasm-deploy        # Build and prepare WASM for deployment (release)
just d                  # Alias for wasm-deploy
just w                  # Alias for wasm-run
```

### Build Configurations

#### Native Build

```bash
cargo build --release
```

#### WebAssembly Build

```bash
cargo build --target wasm32-unknown-unknown --bin client
wasm-bindgen --target web --out-dir ./wasm-client/ ./target/wasm32-unknown-unknown/release/client.wasm
```

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
- Kalman filter-based state estimation
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

## Project Structure

- `src/bin/` - Entry points for local application, web application and experiment planner.
- `src/core/` - Core simulation and estimation algorithms
- `src/ui/` - Graphical user interface components using EGUI
- `src/vis/` - 3D visualization using Bevy and plotting with egui_plot
- `assests` - 3D models and assets, control function data and MRI data.
- `benches` - Benchmarking suite
- `logs` - Log files for debugging and diagnostics
- `results` - Results and analysis outputs (This folder can get very large, make sure to clean it up regularly)
- `tests` - Visual output of unit tests
- `wasm-client` - WebAssembly build artifacts

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
- WebAssembly support through wasm-bindgen

## Performance

The project uses optimized build configurations:

- Development builds use optimization level 3
- Release builds use thin LTO
- Test builds are optimized for performance

## Contributing

While CardioTRust is primarily maintained by one developer as part of ongoing PhD research, contributions are welcome! However, please note that due to the academic nature of the project and its active development state, response times may vary.

### Ways to Contribute

- Report bugs through issues
- Suggest enhancements or new features
- Improve documentation
- Submit pull requests for bug fixes or features

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Make your changes
4. Run the full test suite (`just test-all`)
5. Run formatting and linting (`just fmt && just lint`)
6. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
7. Push to the branch (`git push origin feature/AmazingFeature`)
8. Open a Pull Request

### Quality Guidelines

- Include tests for new functionality
- Maintain or improve code coverage
- Follow existing code style and formatting
- Update documentation as needed
- Keep pull requests focused and specific

Feel free to open an issue first to discuss potential changes or features. For complex changes, it's recommended to reach out before investing significant development time.

For questions about the academic aspects of the project or the underlying research, please refer to the publications or contact the research team.

## Research Team

### Development

- Erik Engelhardt - Digital Signal Processing and System Theory, Kiel University

### Principal Investigators

- Prof. Dr. med. Norbert Frey - University Medical Center Heidelberg
- Prof. Dr.-Ing. Gerhard Schmidt - Kiel University

## Publications

1. Engelhardt, E., et al. (2023). "Non-invasive Electroanatomical Mapping: A State-space Approach for Myocardial Current Density Estimation." Bioengineering, 10(12), p. 1432.

2. Engelhardt, E., et al. (2023). "A Concept for Myocardial Current Density Estimation with Magnetoelectric Sensors." Current Directions in Biomedical Engineering, 9(1), pp. 89-92.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This research was funded by the German Research Foundation (Deutsche Forschungsgemeinschaft, DFG) through the Collaborative Research Center CRC 1261 Magnetoelectric Sensors: From Composite Materials to Biomagnetic Diagnostics.
