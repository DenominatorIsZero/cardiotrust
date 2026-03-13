## Purpose

Governs the mathematical and anatomical heart model that both the forward simulation and the inverse algorithm operate on. This spec covers the voxel grid structure, tissue connectivity rules, the all-pass filter network encoding propagation delays, the Biot-Savart measurement physics, and the GPU mirroring contract.

This spec is distinct from `data-simulation` (which describes what the model produces when driven through a simulation) and from `inverse-algorithm` (which describes how the model's parameters are updated during estimation). The cardiac model is the shared substrate: it is constructed once and then consumed, read, or mutated by the simulation and algorithm domains respectively.

## ADDED Requirements

### Requirement: Each connectable voxel has a unique, stable state index

Every voxel that can carry an electrical activation SHALL be assigned a unique non-negative integer index into the state vector. This index SHALL be a multiple of three, reserving three consecutive state slots for the Cartesian components (x, y, z) of that voxel's current density. Voxels that cannot carry activation SHALL have no index. This indexing convention is an absolute invariant that all other subsystems rely on.

#### Scenario: State index is always a multiple of three

- **WHEN** the state index of any connectable voxel is retrieved
- **THEN** the index is divisible by three with no remainder

#### Scenario: Non-connectable voxels have no state index

- **WHEN** the state index is queried for a voxel of a non-connectable type (such as torso or chamber)
- **THEN** the result indicates no index is assigned

#### Scenario: No two voxels share a state index

- **WHEN** the state indices of all connectable voxels in a model are collected
- **THEN** all indices are distinct

### Requirement: Anatomical connectivity rules govern wavefront propagation paths

The model SHALL enforce tissue-specific rules about which tissue types may directly activate neighboring tissue types. These rules encode anatomical constraints: for example, atrial tissue SHALL NOT directly activate ventricular tissue — activation must route through the atrioventricular junction and His-Purkinje system. A connection between two adjacent voxels is only established when both their types satisfy the connectivity rules.

#### Scenario: Atrium-to-ventricle direct connection is forbidden

- **WHEN** a model is built where an atrial voxel is adjacent to a ventricular voxel with no AV junction voxel between them
- **THEN** no direct activation connection is established from atrium to ventricle

#### Scenario: Connections are established when rules permit

- **WHEN** a sinoatrial voxel is adjacent to an atrial voxel
- **THEN** an activation connection from the sinoatrial node to the atrium is established

### Requirement: Sinoatrial node is the unique activation origin

The model SHALL designate exactly one region as the sinoatrial node, and activation SHALL originate from this region at time zero. All other voxels receive their activation time by propagation from the sinoatrial node, with delays determined by propagation velocity and inter-voxel distance.

#### Scenario: Sinoatrial node activates at time zero

- **WHEN** a model is constructed and activation times are computed
- **THEN** the sinoatrial node voxels have an activation time of zero

#### Scenario: Downstream voxels activate after the sinoatrial node

- **WHEN** activation times are computed across all voxels
- **THEN** every non-sinoatrial connectable voxel has an activation time strictly greater than zero

### Requirement: All-pass filter delays are always stable

Each inter-voxel connection is represented as an all-pass filter whose delay coefficient controls the propagation timing. The coefficient SHALL always be clamped to a range that guarantees filter stability. A coefficient at the boundary values is never permitted.

#### Scenario: Filter coefficient is strictly between zero and one

- **WHEN** any all-pass filter coefficient in the model is retrieved
- **THEN** it is strictly greater than zero and strictly less than one

#### Scenario: Delay changes do not push coefficient outside stable range

- **WHEN** a propagation delay is updated to an extreme value
- **THEN** the resulting filter coefficient is clamped to remain within the stable range

### Requirement: Measurement matrix encodes Biot-Savart physics

The model SHALL maintain a measurement matrix that maps the 3-D cardiac current density state to predicted magnetic field readings at each sensor position and orientation. The matrix is computed once from the sensor geometry and remains fixed throughout the lifetime of the model. For a multi-position sensor array, there is one measurement matrix slice per acquisition position.

#### Scenario: Measurement matrix has one slice per sensor position

- **WHEN** a model is built with a sensor array that moves through N positions
- **THEN** the measurement matrix has N position slices

#### Scenario: Single-position array has a single measurement matrix slice

- **WHEN** a model is built with a stationary sensor array
- **THEN** the measurement matrix has exactly one slice

### Requirement: GPU model is always consistent with CPU model after synchronization

The model may be mirrored to GPU memory for accelerated computation. After any explicit synchronization operation (CPU-to-GPU or GPU-to-CPU), the learned parameters (gains and delay coefficients) SHALL agree bit-for-bit between the CPU and GPU representations. Static fields that are set at construction time and never modified by the algorithm SHALL reside solely on the CPU.

#### Scenario: CPU-to-GPU copy yields matching learned parameters

- **WHEN** the current CPU model parameters are copied to the GPU
- **THEN** a subsequent GPU-to-CPU copy produces learned parameters equal to the original CPU values

#### Scenario: GPU update is reflected on CPU after synchronization

- **WHEN** the GPU model's learned parameters are updated by the estimation algorithm and then synchronized back to CPU
- **THEN** the CPU model's learned parameters equal the updated GPU values

### Requirement: MRI-derived anatomy is accepted when a segmentation source is provided

When the model configuration specifies an MRI-based anatomy source, the model SHALL be constructed from that segmentation, assigning voxel tissue types based on the segmentation labels. If the specified source is absent or unreadable, model construction SHALL fail with a diagnostic error rather than silently producing an incorrect model.

#### Scenario: Valid MRI source produces a model with tissue-typed voxels

- **WHEN** a valid MRI segmentation source is provided
- **THEN** the constructed model has voxels with tissue types derived from that segmentation

#### Scenario: Missing MRI source is reported as an error

- **WHEN** the MRI anatomy source is specified but the source cannot be read
- **THEN** model construction fails with a descriptive error and no partial model is returned
