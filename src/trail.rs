//! Working with TRAIL.DAT file
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

/// TRAIL structure
///
/// Contains timestamp, two coordinates, and elevation as IEEE754 doubles
/// Layout in file:
/// - `_:     u64` - zeros, padding?
/// - `lat:   f64` - latitude of last track point
/// - `lon:   f64` - longtitude of last track point
/// - `_:     u32` - zero padding
/// = `points:u32` - number of track points?
/// - `dist:  f64` - distance in meters
/// - `time:  u32` - seconds that path took
/// - `speed: f32` - Looks like speed, but not sure if average or just in the last track point
#[derive(Debug, Clone, Copy)]
pub struct Trail {
    /// Latitude
    lat: f64,
    /// Longtitude
    lon: f64,
    /// Distance
    dist: f64,
    /// Points
    points: u32,
    /// Time
    time: u32,
    /// Speed
    speed: f32,
}

impl Trail {
    pub fn deserialize(rdr: &mut impl Read) -> io::Result<Self> {
        rdr.read_u64::<LittleEndian>()?; // Padding?
        let lat = rdr.read_f64::<LittleEndian>()?;
        let lon = rdr.read_f64::<LittleEndian>()?;
        rdr.read_u32::<LittleEndian>()?; // Padding?
        let points = rdr.read_u32::<LittleEndian>()?;
        let dist = rdr.read_f64::<LittleEndian>()?;
        let time = rdr.read_u32::<LittleEndian>()?;
        let speed = rdr.read_f32::<LittleEndian>()?; // Padding?

        Ok(Trail {
            lat,
            lon,
            dist,
            points,
            time,
            speed,
        })
    }

    pub fn serialize(self, wr: &mut impl Write) -> io::Result<()> {
        wr.write_u64::<LittleEndian>(0)?;
        wr.write_f64::<LittleEndian>(self.lat)?;
        wr.write_f64::<LittleEndian>(self.lon)?;
        wr.write_u32::<LittleEndian>(0)?;
        wr.write_u32::<LittleEndian>(self.points)?;
        wr.write_f64::<LittleEndian>(self.dist)?;
        wr.write_u32::<LittleEndian>(self.time)?;
        wr.write_f32::<LittleEndian>(self.speed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::PathBuf;

    #[test]
    fn test_deserialize() -> Result<(), io::Error> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/testdata/TRAIL.DAT");
        let mut f = File::open(&d)?;
        let v = Trail::deserialize(&mut f)?;
        assert_approx_eq!(v.lat, 55.7880, 0.001);
        assert_approx_eq!(v.lon, 37.5388, 0.001);
        assert_approx_eq!(v.dist, 3011.489, 0.001);
        assert_eq!(v.time, 1983);
        Ok(())
    }

    #[test]
    fn test_serialize() -> Result<(), io::Error> {
        let testtrail = Trail {
            lat: 55.788067,
            lon: 37.538875833333336,
            points: 1972,
            dist: 3011.489339109006,
            time: 1983,
            speed: 3.57,
        };
        let refbytes: [u8; 48] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46, 0x07, 0x24, 0x61, 0xdf, 0xe4,
            0x4b, 0x40, 0xbe, 0x62, 0x20, 0xe2, 0xf9, 0xc4, 0x42, 0x40, 0x00, 0x00, 0x00, 0x00,
            0xb4, 0x07, 0x00, 0x00, 0xab, 0xdb, 0xa7, 0x8a, 0xfa, 0x86, 0xa7, 0x40, 0xbf, 0x07,
            0x00, 0x00, 0xe1, 0x7a, 0x64, 0x40,
            // 0xe1, 0x7a, 0x64, 0x40, - last bytes
        ];
        let mut bytes = [0; 48];
        {
            let bytes_ref: &mut [u8] = &mut bytes;
            let mut writer = BufWriter::new(bytes_ref);
            testtrail.serialize(&mut writer)?;
        }
        assert_eq!(bytes, refbytes);

        Ok(())
    }
}
