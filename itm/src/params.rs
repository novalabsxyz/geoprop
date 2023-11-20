/// Antenna polarization.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarization {
    Horizontal = 0,
    Vertical = 1,
}

/// Siting criteria.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SittingCriteria {
    Random = 0,
    Careful = 1,
    VeryCareful = 2,
}

/// Radio climate.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Climate {
    Equatorial = 1,
    ContinentalSubtropical = 2,
    MaritimeSubtropical = 3,
    Desert = 4,
    ContinentalTemperate = 5,
    MaritimeTemperateOverLand = 6,
    MaritimeTemperateOverSea = 7,
}

/// Propagation mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    NotSet = 0,
    LineOfSight = 1,
    Diffraction = 2,
    Troposcatter = 3,
}

/// Mode of variability.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeVariability {
    SingleMessage = 0,
    Accidental = 1,
    Mobile = 2,
    Broadcast = 3,
}
