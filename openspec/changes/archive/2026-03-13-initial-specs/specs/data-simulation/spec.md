## Purpose

Governs the forward cardiac simulation — the process that takes a fully specified configuration and produces synthetic ground-truth data: time-series magnetic field measurements, cardiac current density states, and derived activation times. The outputs of this domain are the reference against which the inverse algorithm's estimates are evaluated.

This spec is distinct from `cardiac-model` (which governs the construction and structure of the heart model used during simulation) and from `inverse-algorithm` (which governs the estimation process that inverts these measurements). Data simulation is a one-way, read-only computation: it consumes a configuration and produces data; it does not update any model parameters.

## ADDED Requirements

### Requirement: Forward simulation is deterministic

Given identical simulation configuration, the forward simulation SHALL always produce bit-for-bit identical outputs, including noise-corrupted measurements. Determinism SHALL be unconditional: it SHALL hold across runs on the same hardware and across fresh process invocations.

#### Scenario: Same configuration always yields same noisy measurements

- **WHEN** the forward simulation is run twice with the same configuration
- **THEN** both runs produce sensor measurement arrays that are element-wise identical, including noise

#### Scenario: Different configurations yield different measurements

- **WHEN** the forward simulation is run with two configurations that differ in at least one cardiac geometry parameter
- **THEN** the resulting noise-free measurement arrays are not identical

### Requirement: Simulated measurements have well-defined shape and semantics

The output measurement array SHALL be organized by beat, time step, and sensor. The total number of beats SHALL equal the number of sensor-array positions (at least one for a stationary array). Each entry represents the magnetic field measured by one sensor at one sensor position and one time step.

#### Scenario: Static sensor array produces a single-beat measurement array

- **WHEN** the simulation is configured with a stationary sensor array
- **THEN** the measurement output contains exactly one beat

#### Scenario: Moving sensor array produces one beat per position

- **WHEN** the simulation is configured with a sensor array that moves through N positions
- **THEN** the measurement output contains exactly N beats

#### Scenario: Measurement array dimensions are consistent with configuration

- **WHEN** a simulation is run with S sensors and T time steps per beat
- **THEN** each beat's measurement slice has exactly S sensors and T time steps

### Requirement: Simulated system states represent cardiac current density

The forward simulation SHALL produce a time-series of 3-D cardiac current density vectors — one Cartesian (x, y, z) triplet per connectable voxel per time step. The Cartesian representation is the primary output; spherical (magnitude, elevation, azimuth) representations are always derived from it and are always consistent with it.

#### Scenario: Spherical representation is consistent with Cartesian

- **WHEN** a forward simulation is run
- **THEN** for every voxel at every time step, the spherical magnitude equals the Euclidean norm of the Cartesian components

#### Scenario: State array dimensionality matches voxel count

- **WHEN** a simulation is run over a model with V connectable voxels and T time steps
- **THEN** the system states array has T rows and 3·V columns (three Cartesian components per voxel)

### Requirement: Activation times are derived from peak current density

For each connectable voxel, the forward simulation SHALL compute the time step at which that voxel's current density magnitude reaches its maximum. This activation time is the primary derived scalar quantity for the spatial analysis of cardiac conduction.

#### Scenario: Each voxel has an activation time

- **WHEN** a forward simulation is run
- **THEN** every connectable voxel has an associated peak activation time expressed in milliseconds

#### Scenario: Sinoatrial node activates first

- **WHEN** a forward simulation is run on a healthy anatomy
- **THEN** the sinoatrial node voxel has the earliest (minimum) activation time of all connectable voxels

### Requirement: Noise-free and noise-corrupted measurements are both available

The simulation SHALL produce both a noise-free version and a noise-corrupted version of the sensor measurements. The noise level is governed by the measurement covariance specified in the configuration. Both versions are retained and accessible after the simulation completes.

#### Scenario: Noise-corrupted measurements differ from noise-free

- **WHEN** the simulation is run with a nonzero measurement noise configuration
- **THEN** the noise-corrupted measurement array is not element-wise equal to the noise-free array

#### Scenario: Zero noise produces identical noise-free and noisy measurements

- **WHEN** the measurement noise covariance is set to zero
- **THEN** the noise-corrupted measurement array is element-wise equal to the noise-free array

### Requirement: Simulation output is persistable and reloadable

The complete simulation output SHALL be serializable to disk and deserializable back without loss of information. After a round-trip, all arrays and derived quantities SHALL be equal to their pre-serialization values.

#### Scenario: Round-trip preserves all measurement data

- **WHEN** simulation output is saved to disk and reloaded
- **THEN** the reloaded measurement arrays are element-wise equal to the originals

#### Scenario: Round-trip preserves activation times

- **WHEN** simulation output is saved to disk and reloaded
- **THEN** the reloaded per-voxel activation times are equal to the originals
