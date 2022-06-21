//! Working with AID.DAT file
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use time::OffsetDateTime;

/// AID structure
///
/// Contains timestamp, two coordinates, and elevation as IEEE754 doubles
/// Layout in file:
/// - `lat:  f64` - latitude
/// - `lon:  f64` - longtitude
/// - `elev: f64` - elevation
/// - `timestamp: i64`   - timestamp
#[derive(Debug, Clone, Copy)]
pub struct AID {
    /// Latitude
    lat: f64,
    /// Longtitude
    lon: f64,
    /// Elevation
    elev: f64,
    /// Timestamp
    timestamp: OffsetDateTime,
}

impl AID {
    pub fn deserialize(rdr: &mut impl Read) -> io::Result<Self> {
        let lat = rdr.read_f64::<LittleEndian>()?;
        let lon = rdr.read_f64::<LittleEndian>()?;
        let elev = rdr.read_f64::<LittleEndian>()?;
        let timeval = rdr.read_i64::<LittleEndian>()?;

        let timestamp = match OffsetDateTime::from_unix_timestamp(timeval) {
            Ok(val) => val,
            _ => OffsetDateTime::now_utc(),
        };
        Ok(AID {
            lat,
            lon,
            elev,
            timestamp,
        })
    }

    pub fn serialize(self, wr: &mut impl Write) -> io::Result<()> {
        wr.write_f64::<LittleEndian>(self.lat)?;
        wr.write_f64::<LittleEndian>(self.lon)?;
        wr.write_f64::<LittleEndian>(self.elev)?;
        wr.write_i64::<LittleEndian>(self.timestamp.unix_timestamp())?;
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
    use time::macros::datetime;

    #[test]
    fn test_deserialize() -> Result<(), io::Error> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/testdata/AID.DAT");
        let mut f = File::open(&d)?;
        let v = AID::deserialize(&mut f)?;
        assert_eq!(v.timestamp, datetime!(2022-06-20 21:13:45 UTC));
        assert_approx_eq!(v.lat, 55.7878, 0.001);
        assert_approx_eq!(v.lon, 37.5387, 0.001);
        assert_approx_eq!(v.elev, 154.7, 0.1);
        Ok(())
    }

    #[test]
    fn test_serialize() -> Result<(), io::Error> {
        let testaid = AID {
            lat: 55.78781266666667,
            lon: 37.5387715,
            elev: 154.7,
            timestamp: datetime!(2022-06-20 21:13:45 UTC),
        };
        let refbytes: [u8; 32] = [
            0x9c, 0x5a, 0xa3, 0x0b, 0xd7, 0xe4, 0x4b, 0x40, 0x29, 0x42, 0xea, 0x76, 0xf6, 0xc4,
            0x42, 0x40, 0x66, 0x66, 0x66, 0x66, 0x66, 0x56, 0x63, 0x40, 0x09, 0xe3, 0xb0, 0x62,
            0x00, 0x00, 0x00, 0x00,
        ];
        let mut bytes = [0; 32];
        {
            let bytes_ref: &mut [u8] = &mut bytes;
            let mut writer = BufWriter::new(bytes_ref);
            testaid.serialize(&mut writer)?;
        }
        assert_eq!(bytes, refbytes);

        Ok(())
    }
}
