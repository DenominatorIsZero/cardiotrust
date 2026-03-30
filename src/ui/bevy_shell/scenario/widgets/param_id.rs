//! `ParamId` — stable identifier for each config parameter.

// ── ParamId ───────────────────────────────────────────────────────────────────

/// Stable identifier used to correlate widgets with their config values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamId {
    // Simulation - Core Setup
    SampleRate,
    Duration,
    // Simulation - Sensor Configuration
    SensorGeometry,
    SensorMotion,
    ThreeDSensors,
    ArrayOriginX,
    ArrayOriginY,
    ArrayOriginZ,
    SensorsPerAxis,
    SensorArraySizeX,
    SensorArraySizeY,
    SensorArraySizeZ,
    SensorRadius,
    NumberOfSensors,
    MotionRangeX,
    MotionRangeY,
    MotionRangeZ,
    MotionStepsX,
    MotionStepsY,
    MotionStepsZ,
    // Simulation - Measurement Data
    CovarianceMean,
    CovarianceStd,
    // Algorithm - Algorithm Settings
    AlgorithmType,
    Epochs,
    BatchSize,
    FreezeGains,
    FreezeDelays,
    // Algorithm - Optimizer Settings
    OptimizerType,
    LearningRate,
    LrReductionInterval,
    LrReductionFactor,
    // Algorithm - Regularization
    MaxRegThreshold,
    MaxRegStrength,
    // Algorithm - Metrics
    SnapshotInterval,
    // Model - Heart Geometry
    VoxelSize,
    HeartOffsetX,
    HeartOffsetY,
    HeartOffsetZ,
    HeartSizeX,
    HeartSizeY,
    HeartSizeZ,
    // Model - Functional Settings
    ControlFunction,
    Pathological,
    CurrentFactor,
    // Model - Propagation Velocity
    PropVelSA,
    PropVelAtrium,
    PropVelAV,
    PropVelHPS,
    PropVelVentricle,
    PropVelPathological,
    // Model - Handcrafted
    SaCenterX,
    SaCenterY,
    AtriumYStart,
    AvCenterX,
    HpsYStop,
    HpsXStart,
    HpsXStop,
    HpsYUp,
    PathXStart,
    PathXStop,
    PathYStart,
    PathYStop,
    IncludeAtrium,
    IncludeAv,
    IncludeHps,
    // Model - MRI
    MriPath,
    // Comment
    Comment,
}

impl ParamId {
    /// Returns true for params that should display as integers (no decimal point).
    #[tracing::instrument(level = "trace")]
    pub fn is_integer_display(self) -> bool {
        matches!(
            self,
            Self::SensorsPerAxis
                | Self::NumberOfSensors
                | Self::MotionStepsX
                | Self::MotionStepsY
                | Self::MotionStepsZ
                | Self::Epochs
                | Self::BatchSize
                | Self::LrReductionInterval
                | Self::SnapshotInterval
        )
    }

    /// Returns true for params that should display with two decimal places.
    #[tracing::instrument(level = "trace")]
    pub fn is_two_decimal(self) -> bool {
        matches!(self, Self::SaCenterX | Self::SaCenterY)
    }
}
