# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CardioTrust is a research tool for cardiac electrophysiological simulation and non-invasive localization of arrhythmogenic tissue. It provides a state-space-based framework for electroanatomical mapping using magnetic measurements, implemented in Rust with 3D visualization using Bevy engine.

### Project Context

**Status**: Completed PhD thesis work, now maintained as personal project
**License**: Apache 2.0 - provides flexibility for improvements and commercial use
**Repository**: Personal fork for ongoing improvements and polish
**Goal**: Transform research code into professional-quality software suitable for presentation

**Key Objectives**:

- Address technical debt accumulated during PhD development
- Improve code organization, error handling, and maintainability
- Polish UI/UX for both native and WASM versions
- Create deployable WASM version for web demonstration
- Maintain research integrity while enhancing software quality

## Commands

### Build & Run

- `just run` - Debug build
- `just release` - Release build
- `cargo build --release` - Standard release build

### Testing

- `just test` - Run tests with nextest (recommended)
- `just test-all` - Run all tests including ignored ones
- `cargo nextest run --no-fail-fast` - Direct nextest command

### Code Quality

- `just lint` - Run clippy checks (includes clippy-tracing)
- `just fmt` - Format code (requires nightly toolchain)
- `just work` - Run lint, test and benchmarks

### WebAssembly Build

- `just wasm-build` - Build WASM target (debug)
- `just wasm-run` or `just w` - Build and run WASM locally
- `just wasm-deploy` or `just d` - Build and prepare WASM for deployment (release)

### Benchmarking

- `just bench` - Run epoch benchmarks
- `just flamegraph` - Generate flamegraph (requires cargo-flamegraph)

## Architecture

### Core Components

- **`src/core/`** - Core simulation and estimation algorithms

  - `algorithm/` - Main algorithms (model-based, pseudo-inverse, GPU implementations)
  - `config/` - Configuration structs for algorithm, model, and simulation
  - `data/` - Data handling and simulation data structures
  - `model/` - Spatial and functional model descriptions
  - `scenario/` - Scenario execution, results management, and status tracking

- **`src/ui/`** - EGUI-based user interface components

  - Scenario configuration and management
  - Results visualization and exploration
  - Volume rendering controls

- **`src/vis/`** - 3D visualization using Bevy engine
  - Heart model rendering
  - Sensor array visualization
  - Plotting utilities (PNG/GIF export)
  - Cutting plane visualization

### Key Data Structures

- **`Scenario`** - Central struct managing configuration, execution status, and results

  - Supports state management: Planning → Scheduled → Running → Done
  - Serializes to TOML configuration files in `./results/{id}/`
  - Binary serialization of data/results for performance

- **`Model`** - Contains spatial (voxels, sensors) and functional (Kalman filters, all-pass filters) descriptions
- **`Data`** - Simulation data and measurements
- **`Results`** - Algorithm outputs, estimations, and metrics

### Execution Modes

1. **Native Application** (`src/bin/main.rs`) - Full desktop application with file I/O
2. **WebAssembly Client** (`src/bin/client.rs`) - Browser-based version with websocket communication
3. **Experiment Planner** (`src/bin/planner.rs`) - Batch scenario generation

### GPU Acceleration

GPU kernels implemented in `src/core/algorithm/gpu/` using OpenCL:

- Epoch processing, prediction, update, and derivation calculations
- Automatic fallback to CPU implementations

### File Formats

- **NIFTI** - Medical imaging format (MRI data)
- **NPY** - NumPy array format for data exchange
- **TOML** - Human-readable configuration files
- **Binary** - Performance-optimized serialization with bincode

## Development Notes

### Build Configuration

- Development builds use optimization level 3 for performance during debugging
- Release builds use thin LTO
- Test builds are optimized

### Dependencies

- `bevy` - 3D engine and ECS
- `nalgebra` - Linear algebra
- `ndarray` - N-dimensional arrays
- `egui` - Immediate mode GUI
- `tracing` - Structured logging

### Testing Strategy

- Tests may fail on first run - run application first to generate test data
- Visual test outputs saved to `tests/` directory
- Benchmarking suite covers all major algorithms

### Directory Structure

- `assets/` - 3D models, control functions, MRI data
- `results/` - Scenario outputs (can become very large)
- `logs/` - Application logs
- `wasm-client/` - WebAssembly build artifacts
- `docs/` - Project documentation and development planning

## AI-Assisted Development Workflow

This project follows a systematic AI-assisted development workflow designed for efficient collaboration with Claude Code. The workflow ensures well-documented, systematic improvements from initial analysis to final implementation.

### Documentation Structure

The `docs/` folder supports the AI-assisted development process:

- **`docs/projects/`** - Project-based documentation organization
  - **`archive/`** - Completed improvement projects with their specifications and implementation plans
  - **`template/`** - Reusable specification and plan templates for CardioTrust improvements
    - `spec-template.md` - Research software improvement specification template
    - `plan-template.md` - Phase-based implementation plan template
    - `README.md` - Template usage guidelines for research software development

### Three-Phase Development Process

#### Phase 1: Specification

Work with Claude Code to develop comprehensive improvement specifications:

1. **Problem Analysis** - Identify specific technical debt, performance issues, or quality improvements needed
2. **Requirements Gathering** - Define functional and technical requirements for improvements
3. **Success Criteria** - Establish measurable outcomes and quality gates
4. **Technical Approach** - Document implementation strategy within existing architecture
5. **Impact Assessment** - Evaluate effects on algorithms, GPU implementations, and WASM builds

The specification phase is complete when:

- Technical debt or improvement opportunity is clearly defined
- Implementation approach respects existing algorithm integrity
- Success criteria are measurable and testable
- GPU/CPU compatibility is considered
- WASM deployment implications are addressed

#### Phase 2: Planning

Transform specifications into actionable implementation plans:

1. **Task Breakdown** - Decompose improvements into discrete, atomic todos
2. **Dependency Mapping** - Identify task dependencies and optimal sequencing
3. **Algorithm Safety** - Plan changes to preserve research algorithm correctness
4. **Testing Strategy** - Define validation approaches for complex algorithms
5. **Rollback Safety** - Ensure each step leaves code in working state

Plans should contain:

- Numbered, sequential tasks that can be completed independently
- Clear definition of "done" for each task including algorithm validation
- Commit points that represent stable, testable states
- Consideration for the two-run testing requirement
- Use `[ ]` checkboxes for trackable tasks, `[x]` for completed tasks

#### Phase 3: Execution

Implement plans step-by-step with Claude Code assistance:

1. **Task-by-Task Implementation** - Work through planned todos systematically
2. **Algorithm Preservation** - Maintain research algorithm integrity throughout changes
3. **Continuous Validation** - Test changes with existing benchmarks and scenarios
4. **Documentation Updates** - Keep technical documentation in sync with improvements
5. **Performance Monitoring** - Verify optimizations don't degrade algorithm performance

### Research Software Specific Guidelines

#### Algorithm Modification Safety

- **Preserve Research Integrity** - Changes should not alter algorithm behavior unless explicitly intended
- **GPU/CPU Parity** - Maintain consistency between GPU and CPU implementations
- **Benchmark Validation** - Use existing benchmarks to verify improvements don't degrade performance
- **Scenario Testing** - Test changes against multiple scenario types (basic, sheet_ap, etc.)

#### Testing Strategy for Research Code

- **Two-Run Requirement** - Tests may fail on first run, requiring application execution to generate test data
- **Visual Output Validation** - Check visual outputs in `tests/` directory for algorithm correctness
- **Benchmark Verification** - Run relevant benchmarks to ensure performance isn't degraded
- **End-to-End Validation** - Test complete scenarios to verify algorithm chain integrity

#### Code Quality Improvements

- **Error Handling** - Replace panics with proper Result types where appropriate
- **Code Organization** - Improve module structure without breaking algorithm dependencies
- **Documentation** - Add inline documentation for complex algorithm implementations
- **Performance** - Optimize hot paths identified through profiling and benchmarking

#### AI Integration Guidelines

##### Effective Collaboration with Claude Code

- **Context Management** - Reference specifications and plans to maintain context across sessions
- **Incremental Development** - Work on one discrete task at a time
- **Code Review Partnership** - Use Claude Code to review implementations before committing
- **Problem-Solving** - Leverage Claude Code's analysis for debugging and optimization
- **Research Software Patterns** - Follow existing algorithm architecture and coding conventions

##### Best Practices for AI-Assisted Research Software Development

- **Clear Communication** - Provide specific, actionable requests
- **Iterative Refinement** - Use multiple rounds of feedback to improve specifications and implementations
- **Documentation First** - Always document decisions and rationale for future reference
- **Validation Focus** - Ask Claude Code to help validate implementations against specifications
- **Algorithm Safety** - Ensure research integrity considerations are addressed
- **Performance Awareness** - Monitor benchmark results for performance-affecting changes

#### CLAUDE.md Maintenance

**IMPORTANT**: Keep this file current to improve future AI interactions.

**When to update CLAUDE.md:**
- After major architectural changes or refactoring
- When new development patterns or workflows are established
- After discovering common issues and their solutions
- When user has to correct AI assumptions or provide missing context
- After adding new tools, dependencies, or build processes
- After significant algorithm improvements or research software updates

**Proactive Update Protocol:**
- Claude Code should offer to update CLAUDE.md after significant changes
- Focus on generic guidance that benefits future interactions
- Document new patterns, workflows, or common corrections
- Update file paths, command references, or architectural descriptions
- Add lessons learned from debugging or problem-solving sessions

Example trigger situations:
- "You should use X instead of Y" → Add to development guidelines
- "The files are actually located in Z" → Update repository structure
- "This command doesn't work, use this instead" → Update development commands
- Major refactoring or new features → Update relevant sections

### Commit Strategy

#### Commit as Natural Checkpoints

Commits should represent meaningful progress points, not arbitrary code changes:

- **Algorithm Milestones** - Complete implementation of planned algorithm improvements
- **Stable States** - Code compiles, tests pass, and algorithms work correctly
- **Logical Units** - Related changes grouped together (e.g., CPU + GPU implementation updates)
- **Rollback Points** - States you could confidently return to if needed
- **Research Integrity** - Each commit preserves algorithm correctness and reproducibility

#### Commit Review Requirement

**IMPORTANT**: Always ask the user to review changes before committing. Present a summary of what will be committed and wait for approval before executing any git commit commands.

#### Commit Message Conventions

Follow this format for AI workflow commits:

```
[PHASE] Brief description of change

- Specific changes made
- Algorithm impact assessment
- Performance implications
- Reference to docs/projects/ files

Closes: #issue-number (if applicable)
Refs: docs/projects/improvement-name/spec.md, docs/projects/improvement-name/plan.md
```

Examples:
```
[SPEC] Define error handling standardization requirements

- Analyzed panic usage throughout codebase
- Identified critical vs non-critical failure points
- Defined Result type strategy for algorithm chains
- Assessed GPU implementation compatibility

Refs: docs/projects/error-handling/spec.md
```

```
[PLAN] Break down GPU performance optimization implementation

- Created 6 discrete tasks for kernel optimization
- Identified dependencies on existing benchmark infrastructure
- Planned CPU implementation updates for consistency
- Assessed performance validation strategy

Refs: docs/projects/gpu-optimization/plan.md
```

```
[IMPL] Replace panics with Result types in scenario loading

- Converted scenario.toml parsing to use Result types
- Added comprehensive error messages for configuration issues
- Maintained backward compatibility for existing scenarios
- Verified no performance impact on algorithm execution

Refs: docs/projects/error-handling/spec.md, docs/projects/error-handling/plan.md
```

```
[DOCS] Update algorithm documentation for GPU kernel changes

- Added inline documentation for optimized GPU kernels
- Updated architecture documentation with performance improvements
- Documented GPU/CPU consistency validation procedures
- Added troubleshooting section for common GPU issues

Refs: CLAUDE.md
```

```
[MAINT] Update dependencies and fix clippy warnings

- Updated nalgebra to latest version for performance improvements
- Fixed deprecated OpenCL function calls in GPU kernels
- Enhanced benchmark reporting for better performance tracking
- Updated WASM build configuration for optimized builds

Refs: Cargo.toml, justfile
```

#### Branching Strategy

- **Feature Branches** - One branch per specification for organized development (e.g., `feature/error-handling`)
- **Direct Merges** - Merge to main after local validation and testing (solo project workflow)
- **Clean History** - Use meaningful commit messages with phase tags for clear development history
- **Rollback Safety** - Each commit represents a stable checkpoint you can confidently return to

### Quality Gates

#### Research Software Quality Standards

- **Algorithm Correctness** - All existing scenarios produce expected results
- **Performance Preservation** - Benchmarks show no unexpected regressions
- **GPU Compatibility** - Changes work correctly on both CPU and GPU implementations
- **WASM Deployment** - Improvements are compatible with WebAssembly builds
- **Documentation Currency** - Technical documentation reflects current implementation

#### Development Workflow Integration

- Use existing development commands (`just work`, `just test`, etc.)
- Validate changes across all execution modes (native, WASM, planner)
- Test with multiple scenario types to ensure broad compatibility
- Monitor `results/` directory size during development (can grow very large)

This systematic approach ensures that research software improvements maintain scientific integrity while achieving professional software quality standards.
