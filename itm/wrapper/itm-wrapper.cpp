#include "itm-wrapper.h"
#include "Enums.h"
#include "itm.h"
#include "itm/src/lib.rs.h"

/*=============================================================================
 |
 |  Description: The ITS Irregular Terrain Model (ITM).  This function
 |               exposes point-to-point mode functionality, with variability
 |               specified with time/location/situation (TLS).
 |
 |        Input:  h_tx__meter       - Structural height of the TX, in meters
 |                h_rx__meter       - Structural height of the RX, in meters
 |                pfl[2]            - Terrain data, in PFL format
 |                climate           - Radio climate
 |                                      + 1 : CLIMATE__EQUATORIAL
 |                                      + 2 : CLIMATE__CONTINENTAL_SUBTROPICAL
 |                                      + 3 : CLIMATE__MARITIME_SUBTROPICAL
 |                                      + 4 : CLIMATE__DESERT
 |                                      + 5 : CLIMATE__CONTINENTAL_TEMPERATE
 |                                      + 6 :
 CLIMATE__MARITIME_TEMPERATE_OVER_LAND |                                      +
 7 : CLIMATE__MARITIME_TEMPERATE_OVER_SEA |                N_0               -
 Refractivity, in N-Units |                f__mhz            - Frequency, in MHz
 |                pol               - Polarization
 |                                      + 0 : POLARIZATION__HORIZONTAL
 |                                      + 1 : POLARIZATION__VERTICAL
 |                epsilon           - Relative permittivity
 |                sigma             - Conductivity
 |                mdvar             - Mode of variability
 |                time              - Time percentage, 0 < time < 100
 |                location          - Location percentage, 0 < location < 100
 |                situation         - Situation percentage, 0 < situation < 100
 |
 |      Outputs:  A__db             - Basic transmission loss, in dB
 |                warnings          - Warning flags
 |
 |      Returns:  error             - Error code
 |
 *===========================================================================*/
P2PRes p2p(double h_tx__meter,
           double h_rx__meter,
           rust::Slice<const double> pfl,
           int climate,
           double N_0,
           double f__mhz,
           int pol,
           double epsilon,
           double sigma,
           int mdvar,
           double time,
           double location,
           double situation) {
    P2PRes res;
    long warnings = -1;
    res.ret_code = ITM_P2P_TLS(h_tx__meter,
                               h_rx__meter,
                               const_cast<double *>(pfl.data()),
                               climate,
                               N_0,
                               f__mhz,
                               pol,
                               epsilon,
                               sigma,
                               mdvar,
                               time,
                               location,
                               situation,
                               &res.attenuation_db,
                               &warnings);
    return res;
}
