## Purpose

Governs the declarative layer through which a researcher describes a cardiac electrophysiology experiment before it runs. This spec covers what a valid configuration must contain, what constraints it enforces, and what guarantees callers may rely on when constructing or serializing one.

This spec is distinct from `data-simulation` (which covers what the forward simulation produces once a configuration is applied) and from `scenario-lifecycle` (which covers the experiment execution lifecycle). Configuration is a pure value — it has no side effects and no runtime state.

## ADDED Requirements

### Requirement: Experiment configuration is self-contained

A simulation experiment SHALL be fully described by a single configuration value. Given that configuration value, the experiment is reproducible: the same configuration always produces the same deterministic outputs when run.

#### Scenario: Identical configurations produce identical results

- **WHEN** two experiments are run with configurations that compare equal in all fields
- **THEN** both produce identical ground-truth measurements and model parameter estimates

#### Scenario: Configuration survives serialization round-trip

- **WHEN** a configuration is serialized to text and deserialized back
- **THEN** the deserialized configuration compares equal to the original in all fields

### Requirement: Configuration has a valid default

A default configuration SHALL be constructible without any user input, and the resulting configuration SHALL be valid (usable to run an experiment without modification).

#### Scenario: Default configuration is immediately runnable

- **WHEN** a configuration is constructed using its defaults
- **THEN** it can be passed to the simulation pipeline without errors

### Requirement: Configuration separates simulation from estimation concerns

Configuration SHALL explicitly distinguish between the parameters governing forward simulation (ground-truth data generation) and the parameters governing the inverse estimation algorithm. These two concern groups are independently configurable.

#### Scenario: Simulation parameters do not influence algorithm hyperparameters

- **WHEN** the sensor array geometry is changed in the simulation configuration
- **THEN** the algorithm hyperparameters (learning rate, epochs, optimizer) are unaffected

#### Scenario: Algorithm parameters do not influence forward simulation

- **WHEN** the number of training epochs is changed in the algorithm configuration
- **THEN** the ground-truth sensor measurements produced by forward simulation are unaffected

### Requirement: Propagation velocities cover all connectable tissue types

The configuration SHALL define a propagation velocity for every tissue type that can carry an activation wavefront. The system SHALL never require a velocity for a tissue type that is not connectable, and SHALL never be in a state where a connectable tissue type has no defined velocity.

#### Scenario: All connectable tissue types have a velocity defined

- **WHEN** a configuration is constructed with default values
- **THEN** each connectable cardiac tissue type has a non-negative propagation velocity defined

#### Scenario: Requesting velocity for a non-connectable type returns zero

- **WHEN** the propagation velocity is requested for a tissue type that cannot carry activation
- **THEN** the returned value is zero and no error is produced

### Requirement: Control function selection determines SA node excitation waveform

The configuration SHALL allow selection among a discrete set of excitation waveform shapes for the sinoatrial node driver. The selected waveform is fully determined by configuration at experiment setup time, before any simulation begins.

#### Scenario: Changing the control function changes the excitation waveform

- **WHEN** the control function type is changed between two valid choices
- **THEN** the SA node excitation signal produced for a fixed simulation duration differs between the two configurations

### Requirement: Model anatomy source is mutually exclusive

Configuration SHALL allow either a parametric handcrafted anatomy or an MRI-derived anatomy as the source of the cardiac model, but not both simultaneously. The anatomy source determines how the voxel grid and tissue assignments are constructed.

#### Scenario: Handcrafted and MRI sources cannot coexist

- **WHEN** a configuration is created with both handcrafted and MRI anatomy options active
- **THEN** the system rejects the configuration as invalid before any simulation begins

#### Scenario: MRI source requires a valid path

- **WHEN** the MRI anatomy source is selected and the specified file path does not exist
- **THEN** the system produces an error indicating the missing anatomy file before simulation begins
