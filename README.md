[![Workflow Status](https://github.com/rayslava/minigps/workflows/ci/badge.svg)](https://github.com/rayslava/minigps/actions?query=workflow%3A%22ci%22)
[![Coverage Status](https://codecov.io/gh/rayslava/minigps/branch/master/graph/badge.svg)](https://codecov.io/gh/rayslava/minigps)
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# minigps

MiniGPS file format support library

This library contains support for file formats of noname MiniGPS from
Aliexpress, like https://aliexpress.com/item/1005003479481773.html

Currently following files are supported:
- POI.DAT

The library allows conversion of POIs from and into `gpx::Waypoint` to work
with GPX files.

Usage example could be found at https://github.com/rayslava/minigps-conv

License: MIT
