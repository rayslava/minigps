//! MiniGPS file format support library
//!
//! This library contains support for file formats of noname MiniGPS from
//! Aliexpress, like https://aliexpress.com/item/1005003479481773.html
//!
//! Currently following files are supported:
//! - POI.DAT
//!
//! The library allows conversion of POIs from and into `gpx::Waypoint` to work
//! with GPX files.
//!
//! Usage example could be found at https://github.com/rayslava/minigps-conv

pub mod poi;
