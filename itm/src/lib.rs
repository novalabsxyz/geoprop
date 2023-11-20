mod error;
mod params;

pub use crate::error::ItmErrCode;
pub use params::{Climate, Mode, ModeVariability, Polarization, SittingCriteria};

#[cxx::bridge]
mod ffi {

    #[derive(Default, Debug)]
    struct P2PRes {
        ret_code: i32,
        attenuation_db: f64,
    }

    unsafe extern "C++" {
        include!("itm/wrapper/itm-wrapper.h");

        #[allow(clippy::too_many_arguments)]
        fn p2p(
            h_tx_meter: f64,
            h_rx_meter: f64,
            pfl: &[f64],
            climate: i32,
            N_0: f64,
            f_mhz: f64,
            pol: i32,
            epsilon: f64,
            sigma: f64,
            mdvar: i32,
            time: f64,
            location: f64,
            situation: f64,
        ) -> P2PRes;
    }
}

/// Returns the attenuation between two points.
///
/// See [C++] docs for info.
///
/// [C++]: https://github.com/dirkcgrunwald/itm/blob/31d068635380f61211e4ba43d50b03f0711b758e/src/itm_p2p.cpp#L5-L39
///
/// # Parameters
///
/// - `h_tx_meter`: transmiter height above ground (meters)
/// - `h_rx_meter`: receiver height above ground (meters)
/// - `step_size_m`: distance between each elevation sample (meters)
/// - `terrain`: elevation samples spaced `step_size_m` apart from eachother (meters)
/// - `climate`: see [`Climate`]
/// - `n0`: refractivity (N-Units, where 301 = 4/3 earth radius)
/// - `freq_hz`: frequency of signal (Hertz)
/// - `pol`: see [`Polarization`]
/// - `epsilon`: relative permittivity (Farads/meter)
/// - `sigma`: ground conductivity (Siemens/meter)
/// - `mdvar`: see [`ModeVariability`]
/// - `time`: time percentage (0.0 < time < 100.0)
/// - `location`: location percentage (0.0 < time < 100.0)
/// - `situation`: situation percentage (0.0 < time < 100.0)
///
/// # Suggested Surface Paramters
///
/// | Ground attribute | Ground Conductivity | Relative ground Permittivity |
/// |------------------|--------------------:|-----------------------------:|
/// | Poor ground      |               0.001 |                            4 |
/// | Average ground   |               0.005 |                           15 |
/// | Good ground      |                0.02 |                           25 |
/// | Fresh water      |                0.01 |                           25 |
/// | Sea water        |                 5.0 |                           25 |
///
/// See [Radio Mobile] for source of this table.
///
/// [Radio Mobile]: http://radiomobile.pe1mew.nl/?Calculations___ITM_model_propagation_settings
#[allow(clippy::too_many_arguments)]
pub fn p2p<T>(
    h_tx_meter: f64,
    h_rx_meter: f64,
    step_size_m: f64,
    terrain: &[T],
    climate: Climate,
    n0: f64,
    f_hz: f64,
    pol: Polarization,
    epsilon: f64,
    sigma: f64,
    mode_variability: ModeVariability,
    time: f64,
    location: f64,
    situation: f64,
) -> Result<f64, ItmErrCode>
where
    T: Copy,
    f64: From<T>,
{
    let pfl = {
        let mut pfl: Vec<f64> = Vec::with_capacity(terrain.len() + 2);
        // Yes, we are pusing two additional non-elevation elemts into
        // the vector, but we only need to compensate for 1.
        #[allow(clippy::cast_precision_loss)]
        pfl.push((terrain.len() - 1) as f64);
        pfl.push(step_size_m);
        pfl.extend(terrain.iter().map(|elev| f64::from(*elev)));
        pfl
    };

    let ffi::P2PRes {
        ret_code,
        attenuation_db,
    } = ffi::p2p(
        h_tx_meter,
        h_rx_meter,
        &pfl,
        climate as i32,
        n0,
        f_hz / 1e6,
        pol as i32,
        epsilon,
        sigma,
        mode_variability as i32,
        time,
        location,
        situation,
    );
    ItmErrCode::from_retcode(ret_code, attenuation_db)
}

#[cfg(test)]
mod tests {
    use super::{p2p, Climate, ModeVariability, Polarization};

    #[test]
    fn test_p2p() {
        // terrain data taken ITM's CLI example file <https://github.com/NTIA/itm/blob/master/cmd_examples/pfl.txt>
        let terrain: &[u16] = &[
            1692, 1692, 1693, 1693, 1693, 1693, 1693, 1693, 1694, 1694, 1694, 1694, 1694, 1694,
            1694, 1694, 1694, 1695, 1695, 1695, 1695, 1695, 1695, 1695, 1695, 1696, 1696, 1696,
            1696, 1696, 1696, 1697, 1697, 1697, 1697, 1697, 1697, 1697, 1697, 1697, 1697, 1698,
            1698, 1698, 1698, 1698, 1698, 1698, 1698, 1698, 1698, 1699, 1699, 1699, 1699, 1699,
            1699, 1700, 1700, 1700, 1700, 1700, 1700, 1700, 1701, 1701, 1701, 1701, 1701, 1701,
            1702, 1702, 1702, 1702, 1702, 1702, 1702, 1702, 1703, 1703, 1703, 1703, 1703, 1703,
            1703, 1703, 1703, 1704, 1704, 1704, 1704, 1704, 1704, 1704, 1704, 1705, 1705, 1705,
            1705, 1705, 1705, 1705, 1705, 1705, 1705, 1706, 1706, 1706, 1706, 1706, 1706, 1706,
            1706, 1706, 1707, 1707, 1707, 1707, 1707, 1707, 1707, 1708, 1708, 1708, 1708, 1708,
            1708, 1708, 1708, 1709, 1709, 1709, 1709, 1709, 1710, 1710, 1710, 1710, 1710, 1710,
            1710, 1710, 1709,
        ];

        // Input: <https://github.com/NTIA/itm/blob/master/cmd_examples/i_p2ptls.txt>
        let h_tx_meter = 15.;
        let h_rx_meter = 3.;
        let climate = Climate::ContinentalTemperate;
        let n0 = 301.;
        let f_hz = 3.5e9;
        let pol = Polarization::Vertical;
        let epsilon = 15.;
        let sigma = 0.005;
        let mdvar = ModeVariability::Accidental;
        let time = 50.0;
        let location = 50.0;
        let situation = 50.0;
        let step_size_m = 25.6;
        let attenuation_db = p2p(
            h_tx_meter,
            h_rx_meter,
            step_size_m,
            terrain,
            climate,
            n0,
            f_hz,
            pol,
            epsilon,
            sigma,
            mdvar,
            time,
            location,
            situation,
        )
        .unwrap();

        // Output: <https://github.com/NTIA/itm/blob/master/cmd_examples/o_p2ptls.txt>
        // Results
        // ITM Warning Flags        0x0000       [No Warnings]
        // ITM Return Code          0            [Success - No Errors]
        // Basic Transmission Loss  114.5        (dB)
        assert!((attenuation_db - 114.536_076_339_885_26).abs() < f64::EPSILON);
    }
}
