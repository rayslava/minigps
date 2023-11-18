//! Working with POI.DAT file
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use geo_types::Point;
use gpx::Time;
use gpx::Waypoint;
use std::convert::{From, Into};
use std::io::{self, ErrorKind, Read, Write};
use time::OffsetDateTime;

/// Single POI
///
/// Only contains creation timestamp and two coordinates as IEEE754 doubles
/// Layout in file:
/// - `timestamp: i64` - unix timestamp
/// - `lat: f64` - latitude
/// - `lon: f64` - longtitude
/// - `_: u64`   - zeros, apparently just padding to 32 bit size
#[derive(Debug, Clone, Copy)]
pub struct POI {
    /// POI timestamp
    timestamp: OffsetDateTime,
    /// Latitude
    lat: f64,
    /// Longtitude
    lon: f64,
}

impl POI {
    fn deserialize(rdr: &mut impl Read) -> io::Result<Self> {
        let timeval = rdr.read_i64::<LittleEndian>()?;
        let lat = rdr.read_f64::<LittleEndian>()?;
        let lon = rdr.read_f64::<LittleEndian>()?;
        rdr.read_u64::<LittleEndian>()?; // Padding?

        let timestamp = match OffsetDateTime::from_unix_timestamp(timeval) {
            Ok(val) => val,
            _ => OffsetDateTime::now_utc(),
        };
        Ok(POI {
            timestamp,
            lat,
            lon,
        })
    }

    fn serialize(self, wr: &mut impl Write) -> io::Result<()> {
        wr.write_i64::<LittleEndian>(self.timestamp.unix_timestamp())?;
        wr.write_f64::<LittleEndian>(self.lat)?;
        wr.write_f64::<LittleEndian>(self.lon)?;
        wr.write_u64::<LittleEndian>(0)?;
        Ok(())
    }
}

impl From<Waypoint> for POI {
    fn from(wpt: Waypoint) -> Self {
        POI {
            timestamp: match wpt.time {
                Some(t) => t.into(),
                _ => OffsetDateTime::now_utc(),
            },
            lat: wpt.point().y(),
            lon: wpt.point().x(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Waypoint> for POI {
    fn into(self) -> Waypoint {
        let mut result = Waypoint::new(Point::new(self.lon, self.lat));
        result.time = Some(Time::from(self.timestamp));
        result
    }
}

/// Read a typical POI.DAT file and convert it into `Vec<POI>`
pub fn read_pois(mut rdr: impl Read) -> io::Result<Vec<POI>> {
    let mut pois: Vec<POI> = Vec::new();
    loop {
        match POI::deserialize(&mut rdr) {
            Ok(poi) => pois.push(poi),
            Err(io_error) => match io_error.kind() {
                io::ErrorKind::UnexpectedEof => break,
                _ => return Err(io_error),
            },
        }
    }
    Ok(pois)
}

/// Write a POI.DAT file with up to 16 POIs
pub fn write_pois(pois: Vec<POI>, wr: &mut impl Write) -> io::Result<()> {
    if pois.len() > 16 {
        return Err(io::Error::new(ErrorKind::Other, "Too main POIs"));
    }
    let pad_size = 16 - pois.len(); // We have to pad file up to 512 bytes
    for p in pois {
        p.serialize(wr)?;
    }
    for _ in 0..pad_size * 4 {
        wr.write_u64::<LittleEndian>(0)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use gpx::{read, Gpx};
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use std::path::PathBuf;
    use time::macros::datetime;

    #[test]
    fn test_deserialize() -> Result<(), io::Error> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/testdata/POI.DAT");
        let mut f = File::open(&d)?;
        let v = POI::deserialize(&mut f)?;
        assert_eq!(v.timestamp, datetime!(2022-01-15 06:59:15 UTC));
        assert_approx_eq!(v.lat, 55.789, 0.001);
        assert_approx_eq!(v.lon, 37.536, 0.001);

        Ok(())
    }

    #[test]
    fn test_read_many() -> Result<(), io::Error> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/testdata/POI.DAT");
        let f = File::open(&d)?;
        let v = read_pois(&f)?;
        assert_eq!(v.len(), 16);
        assert_approx_eq!(v[0].lat, 55.789, 0.001);
        assert_approx_eq!(v[0].lon, 37.536, 0.001);
        assert_approx_eq!(v[15].lat, 10.000, 0.0001);
        assert_approx_eq!(v[15].lon, 0.0, 0.001);

        Ok(())
    }

    #[test]
    fn test_serialize() -> Result<(), io::Error> {
        let testpoi = POI {
            timestamp: datetime!(2022-01-15 06:59:15 UTC),
            lat: 55.78938888888889,
            lon: 37.536833333333334,
        };
        let refbytes: [u8; 32] = [
            0xC3, 0x70, 0xE2, 0x61, 0x00, 0x00, 0x00, 0x00, 0x41, 0xCD, 0xF2, 0xB1, 0x0A, 0xE5,
            0x4B, 0x40, 0xE0, 0x08, 0x65, 0xF4, 0xB6, 0xC4, 0x42, 0x40, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let mut bytes = [0; 32];
        {
            let bytes_ref: &mut [u8] = &mut bytes;
            let mut writer = BufWriter::new(bytes_ref);
            testpoi.serialize(&mut writer)?;
        }
        assert_eq!(bytes, refbytes);

        Ok(())
    }

    #[test]
    fn test_write_many() -> Result<(), io::Error> {
        let testpoi = POI {
            timestamp: datetime!(2022-01-15 06:59:15 UTC),
            lat: 55.78938888888889,
            lon: 37.536833333333334,
        };
        let testvec = vec![testpoi; 16];

        let mut bytes = [0; 512];
        {
            let bytes_ref: &mut [u8] = &mut bytes;
            let mut writer = BufWriter::new(bytes_ref);
            write_pois(testvec, &mut writer)?;
        }
        assert_eq!(bytes[0x23], 0x61);
        assert_eq!(bytes[0x1f7], 0x40);

        let testvec = vec![testpoi; 1];
        {
            let bytes_ref: &mut [u8] = &mut bytes;
            let mut writer = BufWriter::new(bytes_ref);
            write_pois(testvec, &mut writer)?;
        }
        assert_eq!(bytes[0x23], 0x0);
        assert_eq!(bytes[0x1f7], 0x0);

        Ok(())
    }

    #[test]
    fn test_write_many_fail() -> Result<(), io::Error> {
        let testpoi = POI {
            timestamp: datetime!(2022-01-15 06:59:15 UTC),
            lat: 55.78938888888889,
            lon: 37.536833333333334,
        };
        let testvec = vec![testpoi; 19];

        let mut bytes = [0; 512];
        {
            let bytes_ref: &mut [u8] = &mut bytes;
            let mut writer = BufWriter::new(bytes_ref);
            let result = write_pois(testvec, &mut writer).map_err(|e| e.kind());
            let expected = Err(io::ErrorKind::Other);
            assert_eq!(expected, result);
        }

        Ok(())
    }

    #[test]
    fn test_convert_poi_to_gpx() -> Result<(), io::Error> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/testdata/test.gpx");
        let f = File::open(&d)?;
        let reader = BufReader::new(f);

        // read takes any io::Read and gives a Result<Gpx, Error>.
        {
            let mut gpx: Gpx = read(reader).unwrap();
            let newpoi = POI::from(gpx.waypoints.remove(0));
            assert_approx_eq!(newpoi.lat, 55.789, 0.001);
            assert_approx_eq!(newpoi.lon, 37.536, 0.001);
            assert_eq!(newpoi.timestamp, datetime!(2022-05-22 20:36:54.804 UTC));
        }
        Ok(())
    }

    #[test]
    fn test_convert_gpx_to_poi() -> Result<(), io::Error> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/testdata/POI.DAT");
        let mut f = File::open(&d)?;
        let v = POI::deserialize(&mut f)?;
        let w: Waypoint = v.into();

        assert_eq!(
            "2022-01-15T06:59:15.000000000Z",
            w.time.unwrap().format().unwrap()
        );
        assert_approx_eq!(w.point().y(), 55.789, 0.001);
        assert_approx_eq!(w.point().x(), 37.536, 0.001);

        Ok(())
    }
}
