// GENEActiv .bin file format

mod defs;

use crate::error::{ActfastError, FileLocation, Result};
use crate::geneactiv::defs::*;
use crate::sensors;

use std::io::{BufRead, BufReader};

pub struct SampleDataUncalibrated {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub light: u16,
    pub button_state: bool,
}

impl SampleDataUncalibrated {
    pub fn read(
        bitreader: &mut bitreader::BitReader,
        location: &FileLocation,
    ) -> Result<SampleDataUncalibrated> {
        let make_err = |context: &'static str| ActfastError::BitRead {
            context: context.to_string(),
            location: location.clone(),
        };

        let x = bitreader
            .read_i16(12)
            .map_err(|_| make_err("accelerometer X"))?;
        let y = bitreader
            .read_i16(12)
            .map_err(|_| make_err("accelerometer Y"))?;
        let z = bitreader
            .read_i16(12)
            .map_err(|_| make_err("accelerometer Z"))?;
        let light = bitreader
            .read_u16(10)
            .map_err(|_| make_err("light sensor"))?;
        let button_state = bitreader
            .read_bool()
            .map_err(|_| make_err("button state"))?;
        bitreader.skip(1).map_err(|_| make_err("padding bit"))?;

        Ok(SampleDataUncalibrated {
            x,
            y,
            z,
            light,
            button_state,
        })
    }

    pub fn calibrate(&self, cal: &CalibrationData) -> SampleDataCalibrated {
        SampleDataCalibrated {
            x: ((self.x as f32 * 100.0) - cal.x_offset as f32) / cal.x_gain as f32,
            y: ((self.y as f32 * 100.0) - cal.y_offset as f32) / cal.y_gain as f32,
            z: ((self.z as f32 * 100.0) - cal.z_offset as f32) / cal.z_gain as f32,
            light: self.light as f32 * cal.lux as f32 / cal.volts as f32,
            button_state: self.button_state,
        }
    }
}

pub struct SampleDataCalibrated {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub light: f32,
    pub button_state: bool,
}

fn read_prefixed<'a>(s: &'a str, prefix: &str, spacing: usize) -> Option<&'a str> {
    if s.starts_with(prefix) {
        Some(&s[prefix.len() + spacing..])
    } else {
        None
    }
}

fn parse_value<T>(s: &str, prefix: &str, spacing: usize) -> Option<T>
where
    T: std::str::FromStr,
{
    read_prefixed(s, prefix, spacing).and_then(|v| v.trim().parse::<T>().ok())
}

/// Read exactly N lines from a reader, returning the line count actually read
pub fn read_n_lines<R: BufRead>(
    reader: &mut R,
    lines: &mut [String],
    start_line: usize,
) -> Result<usize> {
    for (i, line) in lines.iter_mut().enumerate() {
        line.clear();
        match reader.read_line(line) {
            Ok(0) => return Ok(i), // EOF reached
            Ok(_) => {}
            Err(e) => {
                return Err(ActfastError::Io {
                    source: e,
                    context: format!("reading line {}", start_line + i),
                });
            }
        }
    }
    Ok(lines.len())
}

pub fn decode_hex(s: &str, location: FileLocation) -> Result<Vec<u8>> {
    let s = s.trim();
    (0..(s.len() - (s.len() % 2)))
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ActfastError::InvalidHex {
                value: s.to_string(),
                location: location.clone(),
            })
        })
        .collect()
}

#[derive(Debug)]
pub struct CalibrationData {
    pub x_gain: i32,
    pub x_offset: i32,
    pub y_gain: i32,
    pub y_offset: i32,
    pub z_gain: i32,
    pub z_offset: i32,
    pub volts: i32,
    pub lux: i32,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            x_gain: 1,
            x_offset: 0,
            y_gain: 1,
            y_offset: 0,
            z_gain: 1,
            z_offset: 0,
            volts: 1,
            lux: 1,
        }
    }
}

#[derive(Default)]
pub struct HighFrequencySensorData {
    pub time: Vec<i64>,
    pub acceleration: Vec<f32>,
    pub light: Vec<f32>,
    pub button_state: Vec<bool>,
}

impl HighFrequencySensorData {
    pub fn reserve(&mut self, num_measurements: usize) {
        self.time.reserve(num_measurements);
        self.acceleration.reserve(num_measurements * 3);
        self.light.reserve(num_measurements);
        self.button_state.reserve(num_measurements);
    }

    pub fn push(&mut self, time: i64, sample: SampleDataCalibrated) {
        self.time.push(time);
        self.acceleration.push(sample.x);
        self.acceleration.push(sample.y);
        self.acceleration.push(sample.z);
        self.light.push(sample.light);
        self.button_state.push(sample.button_state);
    }

    pub fn sensor_table(&self) -> sensors::SensorTable<'_> {
        sensors::SensorTable {
            name: "high_frequency",
            datetime: &self.time,
            data: vec![
                sensors::SensorData {
                    kind: sensors::SensorKind::Accelerometer,
                    data: sensors::SensorDataDyn::F32(&self.acceleration),
                },
                sensors::SensorData {
                    kind: sensors::SensorKind::Light,
                    data: sensors::SensorDataDyn::F32(&self.light),
                },
                sensors::SensorData {
                    kind: sensors::SensorKind::ButtonState,
                    data: sensors::SensorDataDyn::Bool(&self.button_state),
                },
            ],
        }
    }
}

#[derive(Default)]
pub struct LowFrequencySensorData {
    pub time: Vec<i64>,
    pub temperature: Vec<f32>,
    pub battery_voltage: Vec<f32>,
}

impl LowFrequencySensorData {
    pub fn reserve(&mut self, num_measurements: usize) {
        self.time.reserve(num_measurements);
        self.temperature.reserve(num_measurements);
        self.battery_voltage.reserve(num_measurements);
    }

    pub fn push(&mut self, time: i64, temperature: f32, battery_voltage: f32) {
        self.time.push(time);
        self.temperature.push(temperature);
        self.battery_voltage.push(battery_voltage);
    }

    pub fn sensor_table(&self) -> sensors::SensorTable<'_> {
        sensors::SensorTable {
            name: "low_frequency",
            datetime: &self.time,
            data: vec![
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

#[derive(Default)]
pub struct GeneActivReader {
    pub high_frequency_data: HighFrequencySensorData,
    pub low_frequency_data: LowFrequencySensorData,
}

impl GeneActivReader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reserve(&mut self, num_records: usize, measurements_per_record: usize) {
        let num_measurements = num_records * measurements_per_record;
        self.high_frequency_data.reserve(num_measurements);
        self.low_frequency_data.reserve(num_records);
    }
}

const HEADER_LINES: usize = 59;
const RECORD_LINES: usize = 10;

impl<'a> sensors::SensorsFormatReader<'a> for GeneActivReader {
    fn read<R: std::io::Read + std::io::Seek, M, S>(
        &'a mut self,
        reader: R,
        mut metadata_callback: M,
        mut sensor_table_callback: S,
    ) -> Result<()>
    where
        M: FnMut(sensors::MetadataEntry),
        S: FnMut(sensors::SensorTable<'a>),
    {
        let mut buf_reader = BufReader::new(reader);

        let mut number_of_pages: usize = 0;
        let mut data_reserved = false;
        let mut calibration_data = CalibrationData::default();

        // Read header (59 lines)
        let mut lines_header = vec![String::new(); HEADER_LINES];
        let lines_read = read_n_lines(&mut buf_reader, &mut lines_header, 1)?;

        if lines_read < HEADER_LINES {
            return Err(ActfastError::UnexpectedEof {
                context: format!(
                    "while reading header (expected {} lines, got {})",
                    HEADER_LINES, lines_read
                ),
                location: FileLocation::at_line(lines_read),
            });
        }

        let mut last_category = String::new();
        for line in &lines_header {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let colon = match line.find(':') {
                Some(pos) => pos,
                None => {
                    last_category = line.to_string();
                    continue;
                }
            };

            let entry = sensors::MetadataEntry {
                category: &last_category,
                key: &line[..colon],
                value: &line[colon + 1..],
            };

            // Extract number of pages for data reservation
            if entry.category == defs::id::memory::HEADER {
                if let Some(pages) = parse_value(line, id::memory::PAGES, 1) {
                    number_of_pages = pages;
                }
            }
            // Extract calibration data
            else if entry.category == defs::id::calibration::HEADER {
                if let Some(v) = parse_value(line, id::calibration::X_GAIN, 1) {
                    calibration_data.x_gain = v;
                } else if let Some(v) = parse_value(line, id::calibration::X_OFFSET, 1) {
                    calibration_data.x_offset = v;
                } else if let Some(v) = parse_value(line, id::calibration::Y_GAIN, 1) {
                    calibration_data.y_gain = v;
                } else if let Some(v) = parse_value(line, id::calibration::Y_OFFSET, 1) {
                    calibration_data.y_offset = v;
                } else if let Some(v) = parse_value(line, id::calibration::Z_GAIN, 1) {
                    calibration_data.z_gain = v;
                } else if let Some(v) = parse_value(line, id::calibration::Z_OFFSET, 1) {
                    calibration_data.z_offset = v;
                } else if let Some(v) = parse_value(line, id::calibration::VOLTS, 1) {
                    calibration_data.volts = v;
                } else if let Some(v) = parse_value(line, id::calibration::LUX, 1) {
                    calibration_data.lux = v;
                }
            }

            metadata_callback(entry);
        }

        // Read data records
        let mut lines_record = vec![String::new(); RECORD_LINES];
        let mut current_line = HEADER_LINES + 1;
        let mut record_index: usize = 0;

        loop {
            let lines_read = read_n_lines(&mut buf_reader, &mut lines_record, current_line)?;
            if lines_read == 0 {
                break; // Normal EOF
            }
            if lines_read < RECORD_LINES {
                return Err(ActfastError::UnexpectedEof {
                    context: format!(
                        "incomplete record (expected {} lines, got {})",
                        RECORD_LINES, lines_read
                    ),
                    location: FileLocation::at_record(record_index).with_sample(0),
                });
            }

            let record_location = FileLocation::at_record(record_index);

            if !data_reserved {
                let samples_per_record = lines_record[9].trim().len() / 12; // 6 bytes = 12 hex chars
                self.reserve(number_of_pages, samples_per_record);
                data_reserved = true;
            }

            // Parse record header
            let mut measurement_frequency: f32 = 1.0;
            let mut page_time = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap();
            let mut temperature: f32 = 0.0;
            let mut battery_voltage: f32 = 0.0;

            for line in lines_record.iter().take(9) {
                let line = line.trim();
                if let Some(freq) = parse_value(line, id::record::MEASUREMENT_FREQUENCY, 1) {
                    measurement_frequency = freq;
                } else if let Some(time_str) = read_prefixed(line, id::record::PAGE_TIME, 1) {
                    page_time = defs::parse_date_time(time_str.trim(), record_location.clone())?;
                } else if let Some(temp) = parse_value(line, id::record::TEMPERATURE, 1) {
                    temperature = temp;
                } else if let Some(bv) = parse_value(line, id::record::BATTERY_VOLTAGE, 1) {
                    battery_voltage = bv;
                }
            }

            let page_time_nanos =
                page_time
                    .timestamp_nanos_opt()
                    .ok_or_else(|| ActfastError::InvalidDateTime {
                        value: page_time.to_string(),
                        format: "timestamp out of nanosecond range",
                        location: record_location.clone(),
                    })?;

            self.low_frequency_data
                .push(page_time_nanos, temperature, battery_voltage);

            // Parse sample data (hex-encoded binary)
            let hex_data = lines_record[9].trim();
            let buf = decode_hex(hex_data, record_location.clone())?;
            let mut bitreader = bitreader::BitReader::new(buf.as_slice());

            let num_samples = buf.len() / 6;
            for sample_idx in 0..num_samples {
                let sample_location = record_location.clone().with_sample(sample_idx);
                let sample = SampleDataUncalibrated::read(&mut bitreader, &sample_location)?
                    .calibrate(&calibration_data);

                let sample_offset_nanos = (1_000_000_000.0 / measurement_frequency) as i64;
                let sample_time_nanos = page_time_nanos + sample_offset_nanos * sample_idx as i64;

                self.high_frequency_data.push(sample_time_nanos, sample);
            }

            current_line += RECORD_LINES;
            record_index += 1;
        }

        sensor_table_callback(self.low_frequency_data.sensor_table());
        sensor_table_callback(self.high_frequency_data.sensor_table());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::SensorsFormatReader;
    use assert_approx_eq::assert_approx_eq;
    use std::{collections::HashMap, io::Cursor};

    #[test]
    fn test_read_n_lines() {
        let s = "line1\nline2\nline3\n";
        let mut reader = BufReader::new(s.as_bytes());
        let mut lines = vec![String::new(); 2];
        let _ = read_n_lines(&mut reader, &mut lines, 0);
        assert_eq!(lines, vec!["line1\n", "line2\n"]);
    }

    #[test]
    fn test_decode_hex() {
        assert_eq!(
            decode_hex("010203FFAC", FileLocation::new()).ok(),
            Some(vec![0x01, 0x02, 0x03, 0xFF, 0xAC])
        );
    }

    #[test]
    fn test_geneactiv_reader() {
        let mut reader = GeneActivReader::new();
        let mut metadata = HashMap::new();
        let mut sensor_table = HashMap::new();
        let data = include_bytes!("../../test_data/cmi/geneactiv.bin");
        assert!(
            reader
                .read(
                    Cursor::new(data),
                    |entry| {
                        metadata.insert(
                            (entry.category.to_owned(), entry.key.to_owned()),
                            entry.value.to_owned(),
                        );
                    },
                    |table| {
                        sensor_table.insert(table.name, table);
                    }
                )
                .is_ok()
        );

        assert_eq!(metadata.len(), 45);
        assert_eq!(sensor_table.len(), 2);

        let low_frequency = sensor_table.get("low_frequency").unwrap();
        assert_eq!(low_frequency.datetime.len(), 20);
        assert_eq!(low_frequency.data.len(), 2);

        let high_frequency = sensor_table.get("high_frequency").unwrap();
        assert_eq!(high_frequency.datetime.len(), 6000);
        assert_eq!(high_frequency.data.len(), 3);

        // Datetime

        assert_eq!(
            high_frequency.datetime[0],
            chrono::DateTime::<chrono::Utc>::from_timestamp(1714490010, 0)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap()
        );
        assert_approx_eq!(
            high_frequency.datetime[5999],
            chrono::DateTime::<chrono::Utc>::from_timestamp(1714490110, 0)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap(),
            1_000_000_000
        );

        assert_eq!(
            low_frequency.datetime[0],
            chrono::DateTime::<chrono::Utc>::from_timestamp(1714490010, 0)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap()
        );
        assert_eq!(
            low_frequency.datetime[19],
            chrono::DateTime::<chrono::Utc>::from_timestamp(1714490105, 0)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap()
        );

        // Temperature

        let temperature = low_frequency
            .data
            .iter()
            .find(|d| d.kind == sensors::SensorKind::Temperature)
            .unwrap();
        if let sensors::SensorDataDyn::F32(data) = &temperature.data {
            assert_eq!(data.len(), 20);
            assert_approx_eq!(data[0], 35.8, 1e-6);
            assert_approx_eq!(data[19], 31.2, 1e-6);
        } else {
            panic!("Expected f32 data");
        }

        // Light

        let light = high_frequency
            .data
            .iter()
            .find(|d| d.kind == sensors::SensorKind::Light)
            .unwrap();
        if let sensors::SensorDataDyn::F32(data) = &light.data {
            assert_eq!(data.len(), 6000);
            assert_approx_eq!(data[0], 0.0, 1e-6);
            assert_approx_eq!(data[5999], 55.38889, 1e-6);
        } else {
            panic!("Expected f32 data");
        }

        // Accelerometer

        let acceleration = high_frequency
            .data
            .iter()
            .find(|d| d.kind == sensors::SensorKind::Accelerometer)
            .unwrap();
        if let sensors::SensorDataDyn::F32(data) = &acceleration.data {
            assert_eq!(data.len(), 6000 * 3);
            assert_approx_eq!(data[0], 0.943648595, 1e-6);
            assert_approx_eq!(data[1], 0.038804781, 1e-6);
            assert_approx_eq!(data[2], 0.093962705, 1e-6);

            assert_approx_eq!(data[5999 * 3], 0.084922833, 1e-6);
            assert_approx_eq!(data[5999 * 3 + 1], -0.8376892, 1e-6);
            assert_approx_eq!(data[5999 * 3 + 2], 0.06174232, 1e-6);
        } else {
            panic!("Expected f32 data");
        }

        // Button state

        let button_state = high_frequency
            .data
            .iter()
            .find(|d| d.kind == sensors::SensorKind::ButtonState)
            .unwrap();
        if let sensors::SensorDataDyn::Bool(data) = &button_state.data {
            assert_eq!(data.len(), 6000);
            assert_eq!(data[0], false);
            assert_eq!(data[5999], false);
        } else {
            panic!("Expected bool data");
        }

        // Battery voltage

        let battery_voltage = low_frequency
            .data
            .iter()
            .find(|d| d.kind == sensors::SensorKind::BatteryVoltage)
            .unwrap();
        if let sensors::SensorDataDyn::F32(data) = &battery_voltage.data {
            assert_eq!(data.len(), 20);
            assert_approx_eq!(data[0], 4.00, 1e-6);
            assert_approx_eq!(data[19], 3.99, 1e-6);
        } else {
            panic!("Expected f32 data");
        }
    }

    #[test]
    fn test_invalid_hex() {
        let result = decode_hex("GGGG", FileLocation::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::ActfastError::InvalidHex { .. }));
    }

    #[test]
    fn test_truncated_header() {
        let mut reader = GeneActivReader::new();
        // Only 10 lines instead of required 59
        let data = b"Device Identity\nSerial:123\n\n\n\n\n\n\n\n\n";
        let result = reader.read(std::io::Cursor::new(data), |_| {}, |_| {});
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::error::ActfastError::UnexpectedEof { .. }
        ));
    }
}
