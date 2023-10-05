# Terrain profiling

`terrain` aggregates and sources NASADEM tiles, and generates 1d
elevation profiles between any two points on earth.

## Motivation

Imaging we want to know what kind of terrain we'll encounter if we followed the line on this map from right to left:


<iframe src="https://www.google.com/maps/d/u/0/embed?mid=1Q4TbMv-ZmAa4Uf6FizvkhQD3Ww2A498&ehbc=2E312F" width="640" height="480"></iframe>

```rust
use terrain::{Tiles, Profile};

let tiles = 
