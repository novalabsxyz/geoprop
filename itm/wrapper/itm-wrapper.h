#pragma once

#include "rust/cxx.h"

struct P2PRes;

int poc(rust::slice<const double> terrain);

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
           double situation);
