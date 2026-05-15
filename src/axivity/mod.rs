// Axivity .cwa file format (AX3 / AX6)
//
// Reference:
//   https://github.com/digitalinteraction/openmovement/blob/master/Docs/ax3/ax3-technical.md
//   wadpac/GGIRread `readAxivity.R` (Mirkes / Jackson)

mod defs;

use crate::axivity::defs::*;
use crate::error::{ActfastError, FileLocation, Result};
use crate::file_format::FileFormat;
use crate::sensors;

use std::io::Read;

#[derive(Default)]
pub struct HighFrequencyData {
    pub time: Vec<i64>,
    pub acceleration: Vec<f32>,
    pub gyroscope: Vec<f32>,
}

impl HighFrequencyData {
    pub fn reserve(&mut self, samples: usize, has_gyro: bool) {
        self.time.reserve(samples);
        self.acceleration.reserve(samples * 3);
        if has_gyro {
            self.gyroscope.reserve(samples * 3);
        }
    }

    pub fn sensor_table(&self) -> sensors::SensorTable<'_> {
        let mut data = vec![sensors::SensorData {
            kind: sensors::SensorKind::Accelerometer,
            data: sensors::SensorDataDyn::F32(&self.acceleration),
        }];
        if !self.gyroscope.is_empty() {
            data.push(sensors::SensorData {
                kind: sensors::SensorKind::Gyroscope,
                data: sensors::SensorDataDyn::F32(&self.gyroscope),
            });
        }
        sensors::SensorTable {
            name: "high_frequency",
            datetime: &self.time,
            data,
        }
    }
}

#[derive(Default)]
pub struct LowFrequencyData {
    pub time: Vec<i64>,
    pub light: Vec<u16>,
    pub temperature: Vec<f32>,
    pub battery_voltage: Vec<f32>,
}

impl LowFrequencyData {
    pub fn reserve(&mut self, sectors: usize) {
        self.time.reserve(sectors);
        self.light.reserve(sectors);
        self.temperature.reserve(sectors);
        self.battery_voltage.reserve(sectors);
    }

    pub fn sensor_table(&self) -> sensors::SensorTable<'_> {
        sensors::SensorTable {
            name: "low_frequency",
            datetime: &self.time,
            data: vec![
                sensors::SensorData {
                    kind: sensors::SensorKind::Light,
                    data: sensors::SensorDataDyn::U16(&self.light),
                },
                sensors::SensorData {
                    kind: sensors::SensorKind::Temperature,
                    data: sensors::SensorDataDyn::F32(&self.temperature),
                },
                sensors::SensorData {
                    kind: sensors::SensorKind::BatteryVoltage,
                    data: sensors::SensorDataDyn::F32(&self.battery_voltage),
                },
            ],
        }
    }
}

/// Per-block scaling parameters that are constant for the whole recording.
/// Cached from the first valid data block and reused.
#[derive(Debug, Clone, Copy)]
struct BlockParameters {
    num_axes: u8,
    packed: bool,
    accel_scale: f32,
    gyro_scale: f32,
    nanos_per_sample: i64,
    /// `true` if `tsOffset` top bit is set (newer fractional-time format).
    fractional_format: bool,
}

#[derive(Default)]
pub struct AxivityReader {
    high_frequency_data: HighFrequencyData,
    low_frequency_data: LowFrequencyData,
}

impl AxivityReader {
    pub fn new() -> Self {
        Self::default()
    }
}

fn parse_header<M: FnMut(sensors::MetadataEntry)>(
    header: &[u8; HEADER_SIZE],
    mut metadata_callback: M,
) -> Result<()> {
    if &header[0..2] != HEADER_MAGIC {
        return Err(ActfastError::Parse {
            format: FileFormat::AxivityCwa,
            message: format!(
                "invalid header magic: expected 'MD', got 0x{:02x}{:02x}",
                header[0], header[1]
            ),
            location: FileLocation {
                byte_offset: Some(0),
                ..FileLocation::new()
            },
        });
    }

    let hardware_byte = header[4];
    let lower_device_id = u16::from_le_bytes([header[5], header[6]]);
    let session_id = u32::from_le_bytes([header[7], header[8], header[9], header[10]]);
    let mut upper_device_id = u16::from_le_bytes([header[11], header[12]]);
    if upper_device_id == 0xFFFF {
        upper_device_id = 0;
    }
    let device_id = ((upper_device_id as u32) << 16) | (lower_device_id as u32);

    let logging_start = u32::from_le_bytes([header[13], header[14], header[15], header[16]]);
    let logging_end = u32::from_le_bytes([header[17], header[18], header[19], header[20]]);
    let logging_capacity = u32::from_le_bytes([header[21], header[22], header[23], header[24]]);
    let sample_rate_byte = header[36];
    let firmware_revision = header[41];
    let time_zone = i16::from_le_bytes([header[42], header[43]]);

    let hardware_type = match hardware_byte {
        HW_AX6 => "AX6",
        HW_AX3_DEFAULT | HW_AX3_ALT => "AX3",
        _ => "Unknown",
    };
    let (rate_hz, range_g) = decode_sample_rate(sample_rate_byte);

    metadata_callback(sensors::MetadataEntry {
        category: "device",
        key: "hardware_type",
        value: hardware_type,
    });
    metadata_callback(sensors::MetadataEntry {
        category: "device",
        key: "device_id",
        value: &device_id.to_string(),
    });
    metadata_callback(sensors::MetadataEntry {
        category: "device",
        key: "session_id",
        value: &session_id.to_string(),
    });
    metadata_callback(sensors::MetadataEntry {
        category: "device",
        key: "firmware_revision",
        value: &firmware_revision.to_string(),
    });
    metadata_callback(sensors::MetadataEntry {
        category: "device",
        key: "time_zone",
        value: &time_zone.to_string(),
    });
    metadata_callback(sensors::MetadataEntry {
        category: "configuration",
        key: "sample_rate_hz",
        value: &rate_hz.to_string(),
    });
    metadata_callback(sensors::MetadataEntry {
        category: "configuration",
        key: "accelerometer_range_g",
        value: &range_g.to_string(),
    });

    // Logging start/end are packed timestamps; emit as raw values if decodable.
    if let Ok(ts) = decode_timestamp(logging_start, FileLocation::new()) {
        metadata_callback(sensors::MetadataEntry {
            category: "configuration",
            key: "logging_start",
            value: &ts.to_rfc3339(),
        });
    }
    if let Ok(ts) = decode_timestamp(logging_end, FileLocation::new()) {
        metadata_callback(sensors::MetadataEntry {
            category: "configuration",
            key: "logging_end",
            value: &ts.to_rfc3339(),
        });
    }
    if logging_capacity != 0 {
        metadata_callback(sensors::MetadataEntry {
            category: "configuration",
            key: "logging_capacity",
            value: &logging_capacity.to_string(),
        });
    }

    // Annotation: 448 bytes of free-text starting at offset 64.
    // Padded with 0x00 / 0xFF / spaces.
    let annot_raw = &header[64..512];
    let end = annot_raw
        .iter()
        .position(|&b| b == 0 || b == 0xFF)
        .unwrap_or(annot_raw.len());
    if let Ok(s) = std::str::from_utf8(&annot_raw[..end]) {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            metadata_callback(sensors::MetadataEntry {
                category: "session",
                key: "annotation",
                value: trimmed,
            });
        }
    }

    Ok(())
}

/// Validate a sector's 16-bit checksum: the sum (mod 2^16) of all 256 little-endian
/// u16 words must equal zero. Returns `Ok(())` if valid or skipped (very old files
/// with zero rate byte don't have a checksum).
fn check_sector_checksum(sector: &[u8; SECTOR_SIZE]) -> bool {
    if sector[24] == 0 {
        return true;
    }
    let mut sum: u16 = 0;
    for chunk in sector.chunks_exact(2) {
        let word = u16::from_le_bytes([chunk[0], chunk[1]]);
        sum = sum.wrapping_add(word);
    }
    sum == 0
}

impl AxivityReader {
    fn parse_first_data_block_parameters(
        sector: &[u8; SECTOR_SIZE],
        location: &FileLocation,
    ) -> Result<BlockParameters> {
        let offset18 = u16::from_le_bytes([sector[18], sector[19]]);
        let offset25 = sector[25];
        let sample_rate_byte = sector[24];

        let num_axes = (offset25 >> 4) & 0x0F;
        let packed = (offset25 & 0x0F) == 0;
        if !packed && (offset25 & 0x0F) != 2 {
            return Err(ActfastError::InvalidField {
                field: "numAxesBPS",
                value: format!("0x{:02x}", offset25),
                expected: "low nibble 0 (packed 10-bit) or 2 (unpacked 16-bit)",
                location: location.clone(),
            });
        }

        let accel_scale_code = (offset18 >> 13) & 0x07;
        // accel_scale = 1 / 2^(8 + code), so raw_int / 256 for code=0
        let accel_scale = 1.0f32 / ((1u32 << (8 + accel_scale_code)) as f32);

        let gyro_range_code = (offset18 >> 10) & 0x07;
        // gyro_range_dps = 8000 / 2^code; raw / 2^15 * gyro_range
        let gyro_range_dps = 8000.0f32 / ((1u32 << gyro_range_code) as f32);
        let gyro_scale = gyro_range_dps / 32768.0;

        let tsoffset = u16::from_le_bytes([sector[4], sector[5]]);
        let fractional_format = (tsoffset & 0x8000) != 0;

        Ok(BlockParameters {
            num_axes,
            packed,
            accel_scale,
            gyro_scale,
            nanos_per_sample: nanos_per_sample(sample_rate_byte),
            fractional_format,
        })
    }

    fn parse_data_sector(
        &mut self,
        sector: &[u8; SECTOR_SIZE],
        params: &BlockParameters,
        location: &FileLocation,
    ) -> Result<()> {
        let tsoffset = u16::from_le_bytes([sector[4], sector[5]]);
        let timestamp_packed =
            u32::from_le_bytes([sector[14], sector[15], sector[16], sector[17]]);
        let offset18 = u16::from_le_bytes([sector[18], sector[19]]);
        let light = offset18 & 0x03FF;
        let temperature_raw = u16::from_le_bytes([sector[20], sector[21]]) & 0x03FF;
        let battery_byte = sector[23];
        let offset26 = i16::from_le_bytes([sector[26], sector[27]]);
        let sample_count = u16::from_le_bytes([sector[28], sector[29]]) as usize;

        let timestamp = decode_timestamp(timestamp_packed, location.clone())?;
        let timestamp_nanos =
            timestamp
                .timestamp_nanos_opt()
                .ok_or_else(|| ActfastError::InvalidDateTime {
                    value: timestamp.to_string(),
                    format: "timestamp out of nanosecond range",
                    location: location.clone(),
                })?;

        // Block start time offset within the buffer:
        //   shift = offset26 + (fractional * frequency) >> 16   (if fractional format)
        // and the actual whole-second timestamp applies to sample[shift].
        let mut shift = offset26 as i64;
        let mut fractional_ns: i64 = 0;
        if params.fractional_format {
            let fractional = ((tsoffset & 0x7FFF) as u32) << 1;
            // Whole-sample equivalent of the fractional second.
            let freq = 1_000_000_000i64 / params.nanos_per_sample.max(1);
            shift += ((fractional as i64) * freq) >> 16;
            // Fractional offset in nanoseconds: fractional / 65536 of a second.
            fractional_ns = ((fractional as i64) * 1_000_000_000) >> 16;
        }
        // sample[shift].time = timestamp_nanos + fractional_ns
        // sample[i].time     = timestamp_nanos + fractional_ns + (i - shift) * nanos_per_sample
        let block_origin_nanos = timestamp_nanos + fractional_ns;

        // Battery: voltage = 3.0 * (byte / 256 + 1)
        let battery_voltage = 3.0 * (battery_byte as f32 / 256.0 + 1.0);
        // Temperature: °C = (raw & 0x3FF) * 75 / 256 - 50
        let temperature_c = (temperature_raw as f32) * 75.0 / 256.0 - 50.0;

        self.low_frequency_data
            .time
            .push(block_origin_nanos - shift * params.nanos_per_sample);
        self.low_frequency_data.light.push(light);
        self.low_frequency_data.temperature.push(temperature_c);
        self.low_frequency_data
            .battery_voltage
            .push(battery_voltage);

        // Sample data spans bytes 30..510 (480 bytes), with checksum at 510-511.
        let sample_data = &sector[30..510];
        let bytes_per_sample: usize = if params.packed {
            4
        } else {
            (params.num_axes as usize) * 2
        };
        if bytes_per_sample == 0 {
            return Err(ActfastError::InvalidField {
                field: "numAxesBPS",
                value: format!("packed={} num_axes={}", params.packed, params.num_axes),
                expected: "non-zero bytes per sample",
                location: location.clone(),
            });
        }
        let max_samples = sample_data.len() / bytes_per_sample;
        let actual_samples = sample_count.min(max_samples);

        let has_gyro = !params.packed && params.num_axes >= 6;

        for i in 0..actual_samples {
            let off = i * bytes_per_sample;
            let buf = &sample_data[off..off + bytes_per_sample];

            let (ax, ay, az, gxyz) = if params.packed {
                let word = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let (x, y, z) = decode_packed_sample(word);
                (x, y, z, None)
            } else if has_gyro {
                // AX6 layout: gx, gy, gz, ax, ay, az (six int16, gyro first).
                let gx = i16::from_le_bytes([buf[0], buf[1]]);
                let gy = i16::from_le_bytes([buf[2], buf[3]]);
                let gz = i16::from_le_bytes([buf[4], buf[5]]);
                let ax = i16::from_le_bytes([buf[6], buf[7]]) as i32;
                let ay = i16::from_le_bytes([buf[8], buf[9]]) as i32;
                let az = i16::from_le_bytes([buf[10], buf[11]]) as i32;
                (ax, ay, az, Some((gx, gy, gz)))
            } else {
                let x = i16::from_le_bytes([buf[0], buf[1]]) as i32;
                let y = i16::from_le_bytes([buf[2], buf[3]]) as i32;
                let z = i16::from_le_bytes([buf[4], buf[5]]) as i32;
                (x, y, z, None)
            };

            let sample_time =
                block_origin_nanos + (i as i64 - shift) * params.nanos_per_sample;
            self.high_frequency_data.time.push(sample_time);
            self.high_frequency_data
                .acceleration
                .push(ax as f32 * params.accel_scale);
            self.high_frequency_data
                .acceleration
                .push(ay as f32 * params.accel_scale);
            self.high_frequency_data
                .acceleration
                .push(az as f32 * params.accel_scale);

            if let Some((gx, gy, gz)) = gxyz {
                self.high_frequency_data
                    .gyroscope
                    .push(gx as f32 * params.gyro_scale);
                self.high_frequency_data
                    .gyroscope
                    .push(gy as f32 * params.gyro_scale);
                self.high_frequency_data
                    .gyroscope
                    .push(gz as f32 * params.gyro_scale);
            }
        }

        Ok(())
    }
}

impl<'a> sensors::SensorsFormatReader<'a> for AxivityReader {
    fn read<R: Read + std::io::Seek, M, S>(
        &'a mut self,
        mut reader: R,
        mut metadata_callback: M,
        mut sensor_table_callback: S,
        lenient: bool,
    ) -> Result<sensors::ReadResult>
    where
        M: FnMut(sensors::MetadataEntry),
        S: FnMut(sensors::SensorTable<'a>),
    {
        let mut result = sensors::ReadResult::new();

        // --- Header (1024 bytes) ---
        let mut header = [0u8; HEADER_SIZE];
        reader
            .read_exact(&mut header)
            .map_err(|e| ActfastError::Io {
                source: e,
                context: "reading CWA header".to_string(),
            })?;
        parse_header(&header, &mut metadata_callback)?;

        // Pre-reserve memory based on file size, assuming ~120 samples / sector
        // (worst-case 480 for AX3 unpacked, but allocations are amortised so the
        // overshoot doesn't matter for correctness).
        let total_len = reader
            .seek(std::io::SeekFrom::End(0))
            .and_then(|n| reader.seek(std::io::SeekFrom::Start(HEADER_SIZE as u64)).map(|_| n))
            .unwrap_or(0);
        let estimated_sectors = (total_len as usize).saturating_sub(HEADER_SIZE) / SECTOR_SIZE;

        // --- Data sectors ---
        let mut sector = [0u8; SECTOR_SIZE];
        let mut sector_index: usize = 0;
        let mut byte_offset: u64 = HEADER_SIZE as u64;
        let mut params: Option<BlockParameters> = None;
        let mut data_reserved = false;

        loop {
            match reader.read_exact(&mut sector) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    return Err(ActfastError::Io {
                        source: e,
                        context: format!("reading data sector {}", sector_index),
                    });
                }
            }

            let location = FileLocation {
                byte_offset: Some(byte_offset),
                record_index: Some(sector_index),
                sample_index: None,
                line_number: None,
            };

            if &sector[0..2] != DATA_MAGIC {
                let err = ActfastError::Parse {
                    format: FileFormat::AxivityCwa,
                    message: format!(
                        "invalid data sector magic: expected 'AX', got 0x{:02x}{:02x}",
                        sector[0], sector[1]
                    ),
                    location,
                };
                if lenient {
                    result.warnings.push(err.to_string());
                    sector_index += 1;
                    byte_offset += SECTOR_SIZE as u64;
                    continue;
                } else {
                    return Err(err);
                }
            }

            if !check_sector_checksum(&sector) {
                let warning = format!(
                    "sector {} (byte offset {}) failed checksum",
                    sector_index, byte_offset
                );
                if lenient {
                    result.warnings.push(warning);
                    sector_index += 1;
                    byte_offset += SECTOR_SIZE as u64;
                    continue;
                } else {
                    return Err(ActfastError::Parse {
                        format: FileFormat::AxivityCwa,
                        message: warning,
                        location,
                    });
                }
            }

            // First valid block sets the format parameters.
            if params.is_none() {
                params = Some(Self::parse_first_data_block_parameters(&sector, &location)?);
            }
            let p = params.unwrap();

            if !data_reserved {
                let est_samples_per_sector = if p.packed { 120 } else { 480 / (p.num_axes as usize).max(1) / 2 };
                self.high_frequency_data
                    .reserve(estimated_sectors * est_samples_per_sector, p.num_axes >= 6);
                self.low_frequency_data.reserve(estimated_sectors);
                data_reserved = true;
            }

            match self.parse_data_sector(&sector, &p, &location) {
                Ok(()) => {}
                Err(e) => {
                    if lenient {
                        result.warnings.push(e.to_string());
                    } else {
                        return Err(e);
                    }
                }
            }

            sector_index += 1;
            byte_offset += SECTOR_SIZE as u64;
        }

        sensor_table_callback(self.low_frequency_data.sensor_table());
        sensor_table_callback(self.high_frequency_data.sensor_table());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::SensorsFormatReader;
    use assert_approx_eq::assert_approx_eq;
    use std::{collections::HashMap, io::Cursor};

    /// Truncated to 30 data sectors from GGIRread's `ax3_testfile.cwa` (Apache 2.0).
    const AX3_BYTES: &[u8] = include_bytes!("../../test_data/cmi/axivity_ax3.cwa");
    /// Truncated to 30 data sectors from GGIRread's `ax6_testfile.cwa` (Apache 2.0).
    const AX6_BYTES: &[u8] = include_bytes!("../../test_data/cmi/axivity_ax6.cwa");

    #[test]
    fn test_axivity_reader_ax3() {
        let data = AX3_BYTES;
        let mut reader = AxivityReader::new();
        let mut metadata = HashMap::new();
        let mut sensor_table = HashMap::new();
        let result = reader.read(
            Cursor::new(data),
            |entry| {
                metadata.insert(
                    (entry.category.to_owned(), entry.key.to_owned()),
                    entry.value.to_owned(),
                );
            },
            |table| {
                sensor_table.insert(table.name, table);
            },
            false,
        );
        assert!(result.is_ok(), "read failed: {:?}", result.err());
        assert!(result.unwrap().warnings.is_empty());

        // Header metadata
        assert_eq!(metadata[&("device".into(), "hardware_type".into())], "AX3");
        assert_eq!(
            metadata[&("configuration".into(), "sample_rate_hz".into())],
            "100"
        );
        // First sector's accelScaleCode=0 means 8g range encoded in sample rate byte
        assert_eq!(
            metadata[&("configuration".into(), "accelerometer_range_g".into())],
            "8"
        );

        // Sensor tables
        let low = sensor_table.get("low_frequency").unwrap();
        let high = sensor_table.get("high_frequency").unwrap();

        // 16384-byte file = 1024 byte header + 30 sectors × 512
        assert_eq!(low.datetime.len(), 30);
        assert_eq!(low.data.len(), 3); // light, temperature, battery

        // 100 Hz × 120 samples/sector × 30 sectors
        assert_eq!(high.datetime.len(), 30 * 120);
        // AX3 → no gyro entry
        assert_eq!(high.data.len(), 1);
        assert_eq!(high.data[0].kind, sensors::SensorKind::Accelerometer);

        // First sample acceleration values: x=84/256, y=252/256, z=52/256
        if let sensors::SensorDataDyn::F32(accel) = &high.data[0].data {
            assert_eq!(accel.len(), 30 * 120 * 3);
            assert_approx_eq!(accel[0], 84.0 / 256.0, 1e-6);
            assert_approx_eq!(accel[1], 252.0 / 256.0, 1e-6);
            assert_approx_eq!(accel[2], 52.0 / 256.0, 1e-6);
        } else {
            panic!("expected F32 acceleration data");
        }

        // First sector temperature: (258 * 75 / 256) - 50 = 25.5859 °C
        if let sensors::SensorDataDyn::F32(temp) = &low.data[1].data {
            assert_eq!(temp.len(), 30);
            assert_approx_eq!(temp[0], (258.0 * 75.0 / 256.0) - 50.0, 1e-4);
        } else {
            panic!("expected F32 temperature data");
        }

        // First sector light raw = 0x011b & 0x3FF = 283
        if let sensors::SensorDataDyn::U16(light) = &low.data[0].data {
            assert_eq!(light[0], 283);
        } else {
            panic!("expected U16 light data");
        }

        // First sector battery byte 0xBE = 190 → 3 * (190/256 + 1) ≈ 5.2266 V
        if let sensors::SensorDataDyn::F32(bv) = &low.data[2].data {
            assert_approx_eq!(bv[0], 3.0 * (190.0 / 256.0 + 1.0), 1e-4);
        } else {
            panic!("expected F32 battery data");
        }

        // Timestamps are monotonic
        for i in 1..high.datetime.len() {
            assert!(high.datetime[i] > high.datetime[i - 1]);
        }
    }

    #[test]
    fn test_axivity_reader_ax6() {
        let data = AX6_BYTES;
        let mut reader = AxivityReader::new();
        let mut metadata = HashMap::new();
        let mut sensor_table = HashMap::new();
        let result = reader.read(
            Cursor::new(data),
            |entry| {
                metadata.insert(
                    (entry.category.to_owned(), entry.key.to_owned()),
                    entry.value.to_owned(),
                );
            },
            |table| {
                sensor_table.insert(table.name, table);
            },
            false,
        );
        assert!(result.is_ok(), "read failed: {:?}", result.err());
        assert!(result.unwrap().warnings.is_empty());

        assert_eq!(metadata[&("device".into(), "hardware_type".into())], "AX6");

        let low = sensor_table.get("low_frequency").unwrap();
        let high = sensor_table.get("high_frequency").unwrap();

        // 16384 = 1024 header + 30 sectors × 512
        assert_eq!(low.datetime.len(), 30);

        // AX6 unpacked: 40 samples / sector × 30 sectors
        assert_eq!(high.datetime.len(), 30 * 40);

        // Both accelerometer and gyroscope present
        assert_eq!(high.data.len(), 2);
        assert_eq!(high.data[0].kind, sensors::SensorKind::Accelerometer);
        assert_eq!(high.data[1].kind, sensors::SensorKind::Gyroscope);

        // First sample: gx=36, gy=-66, gz=2067 raw; ax=15, ay=146, az=18 raw
        // accelScaleCode=3 → 1/2048; gyroRangeCode=5 → 250 dps / 32768
        if let sensors::SensorDataDyn::F32(accel) = &high.data[0].data {
            assert_eq!(accel.len(), 30 * 40 * 3);
            assert_approx_eq!(accel[0], 15.0 / 2048.0, 1e-6);
            assert_approx_eq!(accel[1], 146.0 / 2048.0, 1e-6);
            assert_approx_eq!(accel[2], 18.0 / 2048.0, 1e-6);
        } else {
            panic!("expected F32 acceleration data");
        }
        if let sensors::SensorDataDyn::F32(gyro) = &high.data[1].data {
            assert_eq!(gyro.len(), 30 * 40 * 3);
            assert_approx_eq!(gyro[0], 36.0 * 250.0 / 32768.0, 1e-6);
            assert_approx_eq!(gyro[1], -66.0 * 250.0 / 32768.0, 1e-6);
            assert_approx_eq!(gyro[2], 2067.0 * 250.0 / 32768.0, 1e-6);
        } else {
            panic!("expected F32 gyroscope data");
        }
    }

    /// Synthesise a corrupted CWA: copy the AX3 bytes and zero the packed timestamp
    /// of selected sectors, which breaks both the checksum and the timestamp decode.
    fn corrupt_ax3_bytes(corrupt_sectors: &[usize]) -> Vec<u8> {
        let mut data = AX3_BYTES.to_vec();
        for &idx in corrupt_sectors {
            let base = HEADER_SIZE + idx * SECTOR_SIZE;
            // Wipe timestamp (offset 14..18) and offset18 (light/scale codes).
            // This invalidates the 16-bit checksum at the end of the sector.
            for off in 14..20 {
                data[base + off] = 0;
            }
        }
        data
    }

    #[test]
    fn test_axivity_reader_corrupt_lenient() {
        // Corrupt blocks 0, 13, 14, 28 — checksum fails, 26 sectors survive.
        // Block 0 corruption verifies that BlockParameters is captured from the
        // first *valid* sector, not blindly the first sector.
        let data = corrupt_ax3_bytes(&[0, 13, 14, 28]);
        let mut reader = AxivityReader::new();
        let mut sensor_table = HashMap::new();
        let result = reader.read(
            Cursor::new(&data[..]),
            |_| {},
            |table| {
                sensor_table.insert(table.name, table);
            },
            true,
        );
        let read_result = result.expect("lenient read should succeed");
        assert_eq!(
            read_result.warnings.len(),
            4,
            "expected 4 corrupt-block warnings, got: {:?}",
            read_result.warnings
        );

        let low = sensor_table.get("low_frequency").unwrap();
        let high = sensor_table.get("high_frequency").unwrap();
        // 30 sectors - 4 corrupt = 26 surviving
        assert_eq!(low.datetime.len(), 26);
        assert_eq!(high.datetime.len(), 26 * 120);
    }

    #[test]
    fn test_axivity_reader_corrupt_strict() {
        let data = corrupt_ax3_bytes(&[0, 13]);
        let mut reader = AxivityReader::new();
        let result = reader.read(Cursor::new(&data[..]), |_| {}, |_| {}, false);
        assert!(
            matches!(result.unwrap_err(), ActfastError::Parse { .. }),
            "strict mode should error on first corrupt block"
        );
    }

    #[test]
    fn test_invalid_header_magic() {
        let mut reader = AxivityReader::new();
        let mut data = vec![0u8; HEADER_SIZE];
        data[0] = b'X';
        data[1] = b'X';
        let result = reader.read(Cursor::new(data), |_| {}, |_| {}, false);
        assert!(matches!(result.unwrap_err(), ActfastError::Parse { .. }));
    }

    #[test]
    fn test_truncated_header() {
        let mut reader = AxivityReader::new();
        let data = b"MD";
        let result = reader.read(Cursor::new(data.as_slice()), |_| {}, |_| {}, false);
        assert!(matches!(result.unwrap_err(), ActfastError::Io { .. }));
    }
}
