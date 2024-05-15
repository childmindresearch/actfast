// GENEActiv .bin file format

mod defs;

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
    pub fn read(bitreader: &mut bitreader::BitReader) -> SampleDataUncalibrated {
        let x = bitreader.read_i16(12).unwrap();
        let y = bitreader.read_i16(12).unwrap();
        let z = bitreader.read_i16(12).unwrap();
        let light = bitreader.read_u16(10).unwrap();
        let button_state = bitreader.read_bool().unwrap();
        bitreader.skip(1).unwrap();

        SampleDataUncalibrated {
            x,
            y,
            z,
            light,
            button_state,
        }
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
        Some(&s[prefix.len()+spacing..])
    } else {
        None
    }
}

fn parse_value<'a, T>(s: &str, prefix: &str, spacing: usize) -> Option<T>
where
    T: std::str::FromStr,
{
    read_prefixed(s, prefix, spacing).and_then(|v| v.trim().parse::<T>().ok())
}

pub fn read_n_lines<R: BufRead>(reader: &mut R, lines: &mut [String]) -> Option<()> {
    for i in 0..lines.len() {
        let l = &mut lines[i];
        l.clear();
        let r = reader.read_line(l);
        // if r is None or Some(0), we're done
        if r.ok()? == 0 {
            return None;
        }
    }
    Some(())
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..(s.len() - (s.len() % 2)))  // ignore last byte if odd
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
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

impl CalibrationData {
    pub fn new() -> CalibrationData {
        CalibrationData {
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

pub struct HighFrequencySensorData {
    pub time: Vec<i64>,
    pub acceleration: Vec<f32>,
    pub light: Vec<f32>,
    pub button_state: Vec<bool>,
}

impl HighFrequencySensorData {
    pub fn new() -> HighFrequencySensorData {
        HighFrequencySensorData {
            time: Vec::new(),
            acceleration: Vec::new(),
            light: Vec::new(),
            button_state: Vec::new(),
        }
    }

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

    pub fn sensor_table(&self) -> sensors::SensorTable {
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

pub struct LowFrequencySensorData {
    pub time: Vec<i64>,
    pub temperature: Vec<f32>,
    pub battery_voltage: Vec<f32>,
}

impl LowFrequencySensorData {
    pub fn new() -> LowFrequencySensorData {
        LowFrequencySensorData {
            time: Vec::new(),
            temperature: Vec::new(),
            battery_voltage: Vec::new(),
        }
    }

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

    pub fn sensor_table(&self) -> sensors::SensorTable {
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

pub struct GeneActivReader {
    pub high_frequency_data: HighFrequencySensorData,
    pub low_frequency_data: LowFrequencySensorData,
}

impl GeneActivReader {
    pub fn new() -> GeneActivReader {
        GeneActivReader {
            high_frequency_data: HighFrequencySensorData::new(),
            low_frequency_data: LowFrequencySensorData::new(),
        }
    }

    pub fn reserve(&mut self, num_records: usize, measurements_per_record: usize) {
        let num_measurements = num_records * measurements_per_record;
        self.high_frequency_data.reserve(num_measurements);
        self.low_frequency_data.reserve(num_records);
    }
}

impl<'a> sensors::SensorsFormatReader<'a> for GeneActivReader {
    fn read<R: std::io::Read + std::io::Seek, M, S>(
        &'a mut self,
        reader: R,
        mut metadata_callback: M,
        mut sensor_table_callback: S,
    ) -> Result<(), String>
    where
        M: FnMut(sensors::MetadataEntry),
        S: FnMut(sensors::SensorTable<'a>),
    {
        let mut buf_reader = BufReader::new(reader);

        let mut number_of_pages: usize = 0;
        let mut data_reserved = false;
        let mut calibration_data = CalibrationData::new();

        // the header is 59 lines long
        let mut lines_header = vec![String::new(); 59];
        read_n_lines(&mut buf_reader, &mut lines_header);

        let mut last_category = String::new();
        for line in lines_header.iter() {
            let line = line.trim();
            // continue if line is empty
            if line.is_empty() {
                continue;
            }
            // find colon position
            let colon = line.find(':');

            if colon.is_none() {
                last_category = line.to_string();
                continue;
            }

            let entry = sensors::MetadataEntry {
                category: &last_category,
                key: &line[..colon.unwrap()],
                value: &line[colon.unwrap() + 1..],
            };

            // Extract number of pages for data reservation
            if entry.category == defs::id::memory::HEADER {
                if let Some(pages) = parse_value(line, id::memory::PAGES, 1) {
                    number_of_pages = pages;
                }
            }
            // Extract calibration data
            else if entry.category == defs::id::calibration::HEADER {
                if let Some(x_gain) = parse_value(line, id::calibration::X_GAIN, 1) {
                    calibration_data.x_gain = x_gain;
                } else if let Some(x_offset) = parse_value(line, id::calibration::X_OFFSET, 1) {
                    calibration_data.x_offset = x_offset;
                } else if let Some(y_gain) = parse_value(line, id::calibration::Y_GAIN, 1) {
                    calibration_data.y_gain = y_gain;
                } else if let Some(y_offset) = parse_value(line, id::calibration::Y_OFFSET, 1) {
                    calibration_data.y_offset = y_offset;
                } else if let Some(z_gain) = parse_value(line, id::calibration::Z_GAIN, 1) {
                    calibration_data.z_gain = z_gain;
                } else if let Some(z_offset) = parse_value(line, id::calibration::Z_OFFSET, 1) {
                    calibration_data.z_offset = z_offset;
                } else if let Some(volts) = parse_value(line, id::calibration::VOLTS, 1) {
                    calibration_data.volts = volts;
                } else if let Some(lux) = parse_value(line, id::calibration::LUX, 1) {
                    calibration_data.lux = lux;
                }
            }

            metadata_callback(entry);
        }

        let mut lines_record = vec![String::new(); 10];

        while Some(()) == read_n_lines(&mut buf_reader, &mut lines_record) {
            if !data_reserved {
                // need to look at the first record to know how much data to reserve
                self.reserve(number_of_pages, lines_record[9].as_bytes().len() / 6);
                data_reserved = true;
            }

            // read record header
            let mut measurement_frequency: f32 = 1.0;
            let mut page_time = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap();
            let mut temperature: f32 = 0.0;
            let mut battery_voltage: f32 = 0.0;

            for i in 0..9 {
                let line = lines_record[i].trim();
                if let Some(measurement_frequency_) =
                    parse_value(line, id::record::MEASUREMENT_FREQUENCY, 1)
                {
                    measurement_frequency = measurement_frequency_;
                } else if let Some(page_time_str) = parse_value(line, id::record::PAGE_TIME, 1) {
                    let _: String = page_time_str;
                    page_time = defs::parse_date_time(&page_time_str);
                } else if let Some(temperature_) = parse_value(line, id::record::TEMPERATURE, 1) {
                    temperature = temperature_;
                } else if let Some(battery_voltage_) =
                    parse_value(line, id::record::BATTERY_VOLTAGE, 1)
                {
                    battery_voltage = battery_voltage_;
                }
            }

            self.low_frequency_data.push(
                page_time.timestamp_nanos_opt().unwrap_or(0),
                temperature,
                battery_voltage,
            );

            // read record data

            let buf = decode_hex(&lines_record[9].trim()).unwrap_or_else(|_| {
                //println!("Warning: Error decoding hex string");
                Vec::new()
            });
            let mut bitreader = bitreader::BitReader::new(buf.as_slice());

            for i in 0..buf.len() / 6 {
                let sample =
                    SampleDataUncalibrated::read(&mut bitreader).calibrate(&calibration_data);

                let sample_time = page_time
                    + chrono::Duration::nanoseconds(
                        (1_000_000_000.0 / measurement_frequency) as i64 * i as i64,
                    );

                self.high_frequency_data
                    .push(sample_time.timestamp_nanos_opt().unwrap(), sample);
            }
        }

        // callback for sensor tables
        sensor_table_callback(self.low_frequency_data.sensor_table());
        sensor_table_callback(self.high_frequency_data.sensor_table());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::SensorsFormatReader;
    use std::{collections::HashMap, io::Cursor};
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_read_n_lines() {
        let s = "line1\nline2\nline3\n";
        let mut reader = BufReader::new(s.as_bytes());
        let mut lines = vec![String::new(); 2];
        read_n_lines(&mut reader, &mut lines);
        assert_eq!(lines, vec!["line1\n", "line2\n"]);
    }

    #[test]
    fn test_decode_hex() {
        assert_eq!(decode_hex("010203FFAC"), Ok(vec![0x01, 0x02, 0x03, 0xFF, 0xAC]));
    }

    #[test]
    fn test_geneactiv_reader() {
        let mut reader = GeneActivReader::new();
        let mut metadata = HashMap::new();
        let mut sensor_table = HashMap::new();
        let data = include_bytes!("../../test_data/cmi/geneactiv.bin");
        assert!(reader
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
            .is_ok());

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

        let temperature = low_frequency.data.iter().find(|d| d.kind == sensors::SensorKind::Temperature).unwrap();
        if let sensors::SensorDataDyn::F32(data) = &temperature.data {
            assert_eq!(data.len(), 20);
            assert_approx_eq!(data[0], 35.8, 1e-6);
            assert_approx_eq!(data[19], 31.2, 1e-6);
        } else {
            panic!("Expected f32 data");
        }

        // Light

        let light = high_frequency.data.iter().find(|d| d.kind == sensors::SensorKind::Light).unwrap();
        if let sensors::SensorDataDyn::F32(data) = &light.data {
            assert_eq!(data.len(), 6000);
            assert_approx_eq!(data[0], 0.0, 1e-6);
            assert_approx_eq!(data[5999], 55.38889, 1e-6);
        } else {
            panic!("Expected f32 data");
        }

        // Accelerometer

        let acceleration = high_frequency.data.iter().find(|d| d.kind == sensors::SensorKind::Accelerometer).unwrap();
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

        let button_state = high_frequency.data.iter().find(|d| d.kind == sensors::SensorKind::ButtonState).unwrap();
        if let sensors::SensorDataDyn::Bool(data) = &button_state.data {
            assert_eq!(data.len(), 6000);
            assert_eq!(data[0], false);
            assert_eq!(data[5999], false);
        } else {
            panic!("Expected bool data");
        }

        // Battery voltage

        let battery_voltage = low_frequency.data.iter().find(|d| d.kind == sensors::SensorKind::BatteryVoltage).unwrap();
        if let sensors::SensorDataDyn::F32(data) = &battery_voltage.data {
            assert_eq!(data.len(), 20);
            assert_approx_eq!(data[0], 4.00, 1e-6);
            assert_approx_eq!(data[19], 3.99, 1e-6);
        } else {
            panic!("Expected f32 data");
        }

    }
}