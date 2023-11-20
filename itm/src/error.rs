use std::ffi::c_int;

#[derive(Debug, thiserror::Error)]
pub enum ItmErrCode {
    #[error("TX terminal height is out of range")]
    TxTerminalHeight,
    #[error("RX terminal height is out of range")]
    RxTerminalHeight,
    #[error("Invalid value for radio climate")]
    InvalidRadioClimate,
    #[error("Time percentage is out of range")]
    InvalidTime,
    #[error("Location percentage is out of range")]
    InvalidLocation,
    #[error("Situation percentage is out of range")]
    InvalidSituation,
    #[error("Confidence percentage is out of range")]
    InvalidConfidence,
    #[error("Reliability percentage is out of range")]
    InvalidReliability,
    #[error("Refractivity is out of range")]
    Refractivity,
    #[error("Frequency is out of range")]
    Frequency,
    #[error("Invalid value for polarization")]
    Polarization,
    #[error("Epsilon is out of range")]
    Epsilon,
    #[error("Sigma is out of range")]
    Sigma,
    #[error("The imaginary portion of the complex impedance is larger than the real portion")]
    GroundImpedance,
    #[error("Invalid value for mode of variability")]
    Mdvar,
    #[error("Internally computed effective earth radius is invalid")]
    EffectiveEarth,
    #[error("Path distance is out of range")]
    PathDistance,
    #[error("Delta H (terrain irregularity parameter) is out of range")]
    DeltaH,
    #[error("Invalid value for TX siting criteria")]
    TxSitingCriteria,
    #[error("Invalid value for RX siting criteria")]
    RxSitingCriteria,
    #[error("Internally computed surface refractivity value is too small")]
    SurfaceRefractivitySmall,
    #[error("Internally computed surface refractivity value is too large")]
    SurfaceRefractivityLarge,
}

impl ItmErrCode {
    pub fn from_retcode<T>(err_code: c_int, val: T) -> Result<T, ItmErrCode> {
        let err = match err_code {
            0 | 1 => return Ok(val),
            1000 => ItmErrCode::TxTerminalHeight,
            1001 => ItmErrCode::RxTerminalHeight,
            1002 => ItmErrCode::InvalidRadioClimate,
            1003 => ItmErrCode::InvalidTime,
            1004 => ItmErrCode::InvalidLocation,
            1005 => ItmErrCode::InvalidSituation,
            1006 => ItmErrCode::InvalidConfidence,
            1007 => ItmErrCode::InvalidReliability,
            1008 => ItmErrCode::Refractivity,
            1009 => ItmErrCode::Frequency,
            1010 => ItmErrCode::Polarization,
            1011 => ItmErrCode::Epsilon,
            1012 => ItmErrCode::Sigma,
            1013 => ItmErrCode::GroundImpedance,
            1014 => ItmErrCode::Mdvar,
            1016 => ItmErrCode::EffectiveEarth,
            1017 => ItmErrCode::PathDistance,
            1018 => ItmErrCode::DeltaH,
            1019 => ItmErrCode::TxSitingCriteria,
            1020 => ItmErrCode::RxSitingCriteria,
            1021 => ItmErrCode::SurfaceRefractivitySmall,
            1022 => ItmErrCode::SurfaceRefractivityLarge,
            _ => unreachable!(),
        };
        Err(err)
    }
}
