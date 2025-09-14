# Error Handling Patterns for CardioTrust

This document defines standard error handling patterns for systematic conversion from unwrap/expect to anyhow::Result throughout the CardioTrust codebase.

## Core Principles

1. **Preserve Algorithm Correctness** - Error handling changes must not alter computational behavior
2. **Provide Rich Context** - Errors should include actionable debugging information
3. **Graceful Degradation** - Non-critical failures should not crash the application
4. **Research-Friendly** - Error messages should help with scenario debugging and analysis

## Standard Patterns

### 1. File I/O Operations

**Pattern**: File operations (loading scenarios, reading configurations, saving results)

```rust
use anyhow::{Context, Result};

// Before:
let contents = fs::read_to_string(path).expect("Failed to read file");

// After:
let contents = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;
```

### 2. Directory Operations

**Pattern**: Directory creation and traversal

```rust
// Before:
fs::create_dir_all(dir).expect("Permission to create directory.");

// After:
fs::create_dir_all(&dir)
    .with_context(|| format!("Failed to create results directory: {}", dir.display()))?;
```

### 3. Parsing and Deserialization

**Pattern**: Configuration parsing, TOML/JSON deserialization

```rust
// Before:
let config: Config = toml::from_str(&contents).unwrap();

// After:
let config: Config = toml::from_str(&contents)
    .with_context(|| format!("Invalid configuration format in: {}", path.display()))?;
```

### 4. Array/Vector Indexing

**Pattern**: Safe indexing operations

```rust
// Before:
let item = vec[index].unwrap();

// After:
let item = vec.get(index)
    .with_context(|| format!("Index {} out of bounds for array of length {}", index, vec.len()))?;
```

### 5. GPU Operations with Explicit Failure

**Pattern**: GPU initialization and kernel execution with clear error reporting

```rust
// Before:
let gpu_result = gpu_operation().expect("GPU operation failed");

// After:
let result = gpu_operation()
    .with_context(|| "GPU operation failed - check OpenCL installation and GPU drivers")?;
```

**Pattern**: For optional GPU acceleration with user control

```rust
// When GPU is explicitly requested but fails:
if config.use_gpu {
    let result = gpu_operation()
        .with_context(|| "GPU operation failed - disable GPU acceleration or fix OpenCL setup")?;
} else {
    let result = cpu_operation()
        .context("CPU operation failed")?;
}
```

### 6. Numerical Operations

**Pattern**: Mathematical operations that can fail

```rust
// Before:
let result = matrix.try_inverse().unwrap();

// After:
let result = matrix.try_inverse()
    .with_context(|| "Matrix inversion failed - matrix may be singular or ill-conditioned")?;
```

### 7. Thread and Communication

**Pattern**: Thread spawning and channel operations

```rust
// Before:
let handle = thread::spawn(move || computation()).join().unwrap();

// After:
let handle = thread::spawn(move || computation())
    .join()
    .map_err(|e| anyhow::anyhow!("Worker thread panicked: {:?}", e))
    .context("Failed to execute computation in worker thread")?;
```

### 8. Resource Management

**Pattern**: Resource allocation and cleanup

```rust
// Before:
let resource = acquire_resource().expect("Failed to acquire resource");

// After:
let resource = acquire_resource()
    .context("Failed to acquire computational resource - check system requirements")?;
```

## Function Signature Patterns

### Core Algorithm Functions

Functions in the algorithm chain should return `anyhow::Result<T>`:

```rust
pub fn run_estimation(&self, data: &Data) -> Result<EstimationResult> {
    // Implementation with proper error propagation
}
```

### Entry Point Functions

Top-level functions should handle errors and provide user-friendly messages:

```rust
pub fn execute_scenario(scenario: &mut Scenario) -> Result<()> {
    // Implementation with comprehensive error handling
    // Include scenario ID and configuration context in errors
}
```

### GPU/CPU Compatibility

Maintain consistent error types between GPU and CPU implementations:

```rust
// GPU module
pub fn run_gpu_computation(input: &Input) -> Result<Output> {
    // OpenCL kernel execution
    execute_kernel(input)
        .context("GPU computation failed")?
}

// CPU module
pub fn run_cpu_computation(input: &Input) -> Result<Output> {
    // CPU-based computation
    compute_on_cpu(input)
        .context("CPU computation failed")?
}

// Calling code chooses explicitly
let result = if config.use_gpu {
    gpu::run_gpu_computation(&input)?
} else {
    cpu::run_cpu_computation(&input)?
};
```

## Error Context Guidelines

### Scenario Context

Include scenario information in errors for research debugging:

```rust
.with_context(|| format!("Error in scenario '{}' (ID: {})", scenario.name, scenario.id))
```

### Algorithm Context

Include algorithm step and parameters:

```rust
.with_context(|| format!("Failed during {} step with {} iterations", algorithm_step, iterations))
```

### Data Context

Include data dimensions and types:

```rust
.with_context(|| format!("Data processing failed for {}x{} matrix", rows, cols))
```

## Migration Strategy

1. **Bottom-Up Conversion** - Start with core data structures and model components
2. **Preserve API Compatibility** - Keep public interfaces stable during conversion
3. **Test Algorithm Correctness** - Verify computational results remain unchanged
4. **Gradual Propagation** - Let compiler guide error propagation through call chains

## Testing Error Handling

### Unit Tests

Add error case testing:

```rust
#[test]
fn test_invalid_scenario_loading() {
    let result = Scenario::load("nonexistent/path");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("scenario file"));
}
```

### Integration Tests

Test error propagation through algorithm chains:

```rust
#[test]
fn test_algorithm_error_propagation() {
    let invalid_scenario = create_invalid_scenario();
    let result = run_full_algorithm(&invalid_scenario);
    assert!(result.is_err());
    // Verify error contains useful context
}
```