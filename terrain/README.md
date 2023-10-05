# Terrain profiling

`terrain` aggregates and sources NASADEM tiles, and generates 1d elevation profiles between any two points on earth.

## Motivating example

We want to know the terrain obstruction a ray will encounter if it followed the line on this map from east to west:

[![lake-tahoe-google-maps](https://github.com/JayKickliter/geoprof/assets/2551201/d8e0bd0d-3fcc-4860-a152-29c90c3222f4)]("https://www.google.com/maps/d/u/0/embed?mid=1Q4TbMv-ZmAa4Uf6FizvkhQD3Ww2A498&ehbc=2E312F)

We can use `terrain` to get data used to create this plot (`terrain` does not perform plotting itself):

![lake-tahoe](https://github.com/JayKickliter/geoprof/assets/2551201/b8c94b4b-017c-4dd1-8a87-37c808ccea2b)
