## Purpose

Governs the inverse estimation algorithm that infers cardiac model parameters from observed sensor measurements. This spec covers the iterative training loop (epoch structure, gradient accumulation, parameter updates), state bounding, segmentation metric evaluation, the closed-form pseudo-inverse alternative, and the requirement that GPU and CPU execution paths produce equivalent results.

This spec is distinct from `cardiac-model` (which governs the structure of the model being estimated) and from `data-simulation` (which governs the ground-truth measurements the algorithm inverts). The inverse algorithm is the only domain that mutates model parameters; all other domains treat the model as read-only.

## ADDED Requirements

### Requirement: Epoch zero is always a measurement-only epoch

The first epoch of any iterative estimation run SHALL be executed with a learning rate of zero, leaving all model parameters unchanged. This produces a baseline performance measurement that reflects the model's initial state before any learning occurs.

#### Scenario: Model parameters are unchanged after epoch zero

- **WHEN** epoch zero completes
- **THEN** all model gains and delay coefficients are equal to their pre-epoch values

#### Scenario: Epoch-zero metrics reflect initial model state

- **WHEN** epoch zero completes
- **THEN** the recorded loss metrics correspond to the prediction error of the initial (pre-training) model

### Requirement: Beat order within each epoch is randomized

During each epoch, the set of acquisition beats SHALL be visited in a randomized order. Different epochs SHALL use different random orderings, producing stochastic gradient descent at the beat level.

#### Scenario: Beat processing order varies between epochs

- **WHEN** two consecutive epochs are run over the same set of beats
- **THEN** the order in which beats contribute to gradient accumulation differs between the two epochs

### Requirement: Divergent training halts without consuming remaining epochs

If the total loss becomes non-finite (not-a-number or infinite) during training, the algorithm SHALL stop the epoch loop immediately and not attempt further updates. The model parameters at the point of divergence are retained as the final state.

#### Scenario: Non-finite loss terminates the training loop

- **WHEN** the total loss becomes not-a-number during an epoch
- **THEN** no subsequent epochs are executed and the algorithm terminates

### Requirement: Excessive system states incur a regularization penalty

During each epoch, voxels whose L1 norm (sum of absolute values of the three Cartesian components) exceeds a configured threshold SHALL incur a maximum regularization penalty that is added to the training loss. The penalty is proportional to the amount by which the norm exceeds the threshold, and the regularization gradient is computed component-wise with sign matching the state direction, discouraging the state from growing beyond the threshold.

#### Scenario: Voxels within threshold incur no regularization penalty

- **WHEN** the L1 norm of a voxel's Cartesian state triplet is at or below the configured threshold
- **THEN** the maximum regularization contribution from that voxel is zero

#### Scenario: Voxels exceeding threshold produce a positive penalty

- **WHEN** the L1 norm of a voxel's Cartesian state triplet exceeds the configured threshold
- **THEN** a positive regularization penalty proportional to the excess is added to the total loss for that time step

### Requirement: Segmentation metrics are evaluated over a full threshold sweep

At the end of a training run, the algorithm SHALL evaluate binary voxel classification performance — distinguishing pathological from healthy tissue — across 101 evenly spaced threshold values from 0.0 to 1.0 inclusive. For each threshold, Dice coefficient, Intersection-over-Union, precision, and recall SHALL be computed.

#### Scenario: Final metrics cover exactly 101 thresholds

- **WHEN** the final segmentation metrics are computed after training
- **THEN** metric arrays contain exactly 101 entries corresponding to thresholds 0.00, 0.01, 0.02, …, 1.00

#### Scenario: Classification is threshold-based on peak activation magnitude

- **WHEN** a threshold is applied to classify voxels
- **THEN** voxels whose peak activation magnitude is below the threshold are classified as pathological, and those at or above are classified as healthy

### Requirement: Pseudo-inverse solution requires no iteration

The pseudo-inverse algorithm SHALL compute a closed-form estimate of the system states from the measurement matrix in a single pass, with no epoch loop. It uses only the first acquisition beat's measurements. The result is the least-squares solution in the presence of near-zero singular values, which are treated as zero below a fixed numerical threshold.

#### Scenario: Pseudo-inverse completes in a single pass

- **WHEN** the pseudo-inverse algorithm is selected and executed
- **THEN** no epoch loop runs and the result is available immediately after one matrix operation

#### Scenario: Pseudo-inverse result is unaffected by epoch or learning-rate settings

- **WHEN** the pseudo-inverse algorithm runs with different epoch counts or learning rates in the configuration
- **THEN** the estimated system states are identical regardless of those settings

### Requirement: GPU and CPU estimation produce equivalent results

For any scenario that can be run on either the CPU or GPU execution path, the estimation outcomes SHALL be numerically equivalent. The GPU path is an acceleration of the CPU path, not a different algorithm. Any divergence in numerical results between the two paths is a defect.

#### Scenario: GPU and CPU produce the same loss progression

- **WHEN** the same scenario is run twice — once on CPU, once on GPU — with identical random seeds
- **THEN** the loss at every epoch is equal between the two runs

#### Scenario: GPU and CPU produce the same final model parameters

- **WHEN** the same scenario is run to completion on CPU and GPU
- **THEN** the final gain and delay coefficient arrays are equal between the two runs
