# Error Handling Patterns for CardioTrust

This document describes the actual error handling patterns implemented throughout the CardioTrust codebase during the error handling standardization project.

## Core Principles

1. **Preserve Algorithm Correctness** - Error handling changes must not alter computational behavior
2. **Provide Rich Context** - Errors should include actionable debugging information
3. **Strategic expect() Usage** - expect() only in closures/Bevy systems with error logging
4. **Research-Friendly** - Error messages should help with scenario debugging and analysis

## Implementation Strategy

### Rule 1: expect() Only in Closures and Bevy Systems

**Pattern**: Use `expect()` only in thread spawns, closures, and Bevy systems where returning errors is not possible. Always log the error.

```rust
// Thread spawn closure - cannot return Result
let handle = thread::spawn(move || {
    if let Err(e) = run(send_scenario, &epoch_tx, &summary_tx) {
        tracing::error!("Scenario failed: {:?}", e);
    }
});

// Bevy system - cannot return Result
let ctx = match contexts.ctx_mut() {
    Ok(ctx) => ctx,
    Err(e) => {
        error!("EGUI context not available for volumetric panel: {}", e);
        return;
    }
};

// Visualization systems that must complete
plot_voxel_types(&path, &spatial_model, voxel_nums)
    .expect("Failed to create voxel types plot");
```

### Rule 2: Everything Else Returns Results

**Pattern**: All other functions return `anyhow::Result<T>` and propagate errors upward.

```rust
pub fn load(path: &Path) -> Result<Self> {
    let contents = fs::read_to_string(&scenario_path).with_context(|| {
        format!("Failed to read scenario file: {}", scenario_path.display())
    })?;

    let scenario: Self = toml::from_str(&contents).with_context(|| {
        format!("Failed to parse scenario file: {}", scenario_path.display())
    })?;

    Ok(scenario)
}
```

## Standard Patterns

### 1. File I/O Operations

**Pattern**: File operations use `with_context()` for rich error information

```rust
use anyhow::{Context, Result};

// Implemented pattern:
let contents = fs::read_to_string(&scenario_path).with_context(|| {
    format!("Failed to read scenario file: {}", scenario_path.display())
})?;

let scenario: Self = toml::from_str(&contents).with_context(|| {
    format!("Failed to parse scenario file: {}", scenario_path.display())
})?;
```

### 2. Directory Operations

**Pattern**: Directory operations with formatted context

```rust
// Implemented pattern:
std::fs::create_dir_all(&path).with_context(|| {
    format!("Failed to create directory: {}", path.display())
})?;

// Simple context for basic operations:
std::fs::create_dir_all(&path).context("Failed to create test directory")?;
```

### 3. Context Usage Patterns

**Pattern**: Choose between `.context()` and `.with_context()` based on formatting needs

```rust
// Simple static context:
Data::from_simulation_config(&simulation_config)
    .context("Failed to create simulation data for delay plot test")?;

// Formatted context with variables:
scenario.schedule().with_context(|| {
    format!("Failed to schedule scenario '{}'", scenario.get_id())
})?;

// Complex formatting:
.with_context(|| format!(
    "Failed to schedule Y-motion scenario for experiment '{experiment_name}', {y_step} steps"
))?;
```

### 4. Strategic expect() in Limited Contexts

**Pattern**: expect() only in UI systems and closures where error propagation is impossible

```rust
// UI systems where error propagation is impossible:
let mut new_scenario = Scenario::build(None)
    .expect("Failed to create new scenario");

// In mathematical functions with known valid parameters (tests):
let actual = offset_to_gain_index(-1, -1, -1, 2)
    .expect("Offsets to be valid");

// Visualization operations that must complete:
plot_voxel_types(&path, &spatial_model, voxel_nums)
    .expect("Failed to create voxel types plot");
```

### 5. Consistent Error Propagation in Algorithm Chains

**Pattern**: All algorithms now use proper error propagation with context

```rust
// GPU algorithm with proper error handling:
run_model_based_gpu(&scenario, &model, &mut results, &data, &mut summary, epoch_tx, summary_tx)
    .context("Failed to execute model-based GPU algorithm")?;

// Alternative algorithms with proper error propagation:
AlgorithmType::PseudoInverse => {
    run_pseudo_inverse(&scenario, &model, &mut results, &data, &mut summary)
        .context("Failed to execute pseudo inverse algorithm")?;
    results.model = Some(model);
}

// Final scenario save with error propagation:
scenario.save().context("Failed to save completed scenario results")?;
```

### 6. Error Logging in Systems

**Pattern**: Bevy systems and UI code log errors and gracefully degrade

```rust
// Bevy system error handling:
let scenario = scenario_list.entries.get(index).map_or_else(|| {
    error!("Selected scenario index {} is out of bounds", index);
    None
}, |entry| Some(&entry.scenario));

// Visualization error handling:
if let Err(e) = some_operation() {
    error!("Visualization failed: {}", e);
    return; // Graceful degradation
}
```

### 7. Thread and Scheduler Error Handling

**Pattern**: Thread spawns handle errors internally with logging

```rust
// Implemented pattern in scheduler:
let handle = thread::spawn(move || {
    if let Err(e) = run(send_scenario, &epoch_tx, &summary_tx) {
        tracing::error!("Scenario failed: {:?}", e);
    }
});

// Save operations in scheduler also log errors:
if let Err(e) = entry.scenario.save() {
    error!("Failed to save scenario {}: {}", entry.scenario.get_id(), e);
}
```

### 8. Test and Development Support

**Pattern**: Tests and benchmarks use expect() with clear messages

```rust
// Test data creation:
Data::from_simulation_config(&simulation_config)
    .expect("Model parameters to be valid.");

// Test cleanup operations:
let model = Model::get_default()
    .expect("Failed to create default model for results");

// Benchmark execution:
prectiction_benches(&mut group)
    .expect("Benchmark execution should succeed");
```

## Actual Function Signature Patterns

### Core Functions Return Results

Most functions in the codebase return `anyhow::Result<T>` for proper error propagation:

```rust
// Data creation and simulation
pub fn from_simulation_config(config: &SimulationConfig) -> Result<Self> {
    // Implementation with error contexts
}

// File I/O operations
pub fn load(path: &Path) -> Result<Self> {
    // File loading with context
}

// Algorithm implementations
pub fn run_pseudo_inverse(/* args */) -> Result<()> {
    // Algorithm execution with error propagation
}
```

### Entry Points and Critical Operations

Entry points handle errors appropriately - some use expect() for unrecoverable failures:

```rust
// Scenario execution where failure is unrecoverable
pub fn run_scenario(scenario: Scenario) -> Result<()> {
    // Most operations propagate errors
    let data = Data::from_config(&scenario.config)?;

    // Critical GPU operations use expect()
    run_model_based_gpu(/* args */)
        .expect("Failed to run model-based GPU algorithm");

    // Final save uses expect() - scenario creation failure is unrecoverable
    scenario.save().expect("Could not save scenario");
    Ok(())
}
```

## Implemented Error Context Patterns

### Scenario and Experiment Context

Actual patterns for research debugging:

```rust
// Scenario operations with ID context
scenario.schedule().with_context(|| {
    format!("Failed to schedule scenario '{}'", scenario.get_id())
})?;

// Experiment context for batch operations
.with_context(|| format!(
    "Failed to schedule Y-motion scenario for experiment '{experiment_name}', {y_step} steps"
))?;
```

### File and Path Context

File operations include full path information:

```rust
// File operations with path context
fs::read_to_string(&scenario_path).with_context(|| {
    format!("Failed to read scenario file: {}", scenario_path.display())
})?;

// Directory creation with path
std::fs::create_dir_all(&path).with_context(|| {
    format!("Failed to create directory: {}", path.display())
})?;
```

### Measurement and Simulation Context

Data processing includes relevant parameters:

```rust
// Noise distribution creation with sensor context
Normal::new(0.0, self.measurement_noise).with_context(|| {
    format!("Failed to create measurement noise distribution for sensor {sensor_index}")
})?;

// Data operations with dimensional context
.with_context(|| format!("Failed to find maximum index for state {state}"))?;
```

## Implementation Results

The error handling migration has been completed with these outcomes:

1. **Comprehensive Result Types** - Core functions now return `anyhow::Result<T>` with proper error propagation
2. **Strategic expect() Usage** - Limited to closures, systems, and unrecoverable failures
3. **Rich Error Context** - File operations, scenarios, and algorithms include debugging context
4. **Preserved Algorithm Integrity** - No computational behavior changes during conversion
5. **Improved Debugging** - Error messages include actionable information for research scenarios

## Guidelines for Future Development

### When to Use expect()

1. **Thread spawns and closures** where error propagation is impossible
2. **Bevy systems and UI code** that must complete and can only log errors
3. **Mathematical functions** with known valid parameters in tests
4. **Visualization operations** that must complete for debugging/analysis
5. **Test data creation** with controlled parameters

### When to Use Result Types

1. **All other functions** should return `anyhow::Result<T>`
2. **File I/O operations** with path context
3. **Data processing** with parameter context
4. **Algorithm implementations** with debugging context
5. **Configuration parsing** with format validation
6. **Constructor functions** like `Scenario::build()` that can fail
7. **Test functions** that should propagate errors properly

### Context Guidelines

- Use `.context()` for simple static messages
- Use `.with_context()` when formatting variables into the error message
- Include file paths, scenario IDs, and parameter values in context
- Provide actionable debugging information for research scenarios

### Error Logging

- Use `tracing::error!()` in thread spawns and scheduler operations
- Use `error!()` macro in Bevy systems for UI-related failures
- Always log the error before graceful degradation in systems