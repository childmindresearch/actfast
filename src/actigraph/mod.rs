// actigraph .gt3x file format

mod defs;
mod ssp_codec;

use crate::{actigraph::defs::*, sensors};
use bitreader::BitReader;
use chrono::{TimeDelta, Utc};
use std::io::{BufRead, BufReader, Read};

fn datetime_add_hz(
    dt: chrono::DateTime<Utc>,
    hz: u32,
    sample_counter: u32,
) -> chrono::DateTime<Utc> {
    dt.checked_add_signed(TimeDelta::nanoseconds(
        (1_000_000_000 / hz * sample_counter) as i64,
    ))
    .unwrap_or(dt)
}

pub struct AccelerometerData {
    pub acceleration_time: Vec<i64>,
    pub acceleration: Vec<f32>,

    pub lux_time: Vec<i64>,
    pub lux: Vec<u16>,

    pub capsense_time: Vec<i64>,
    pub capsense: Vec<bool>,

    pub battery_voltage_time: Vec<i64>,
    pub battery_voltage: Vec<u16>,
}

impl AccelerometerData {
    pub fn new() -> AccelerometerData {
        AccelerometerData {
            acceleration_time: Vec::new(),
            acceleration: Vec::new(),
            lux_time: Vec::new(),
            lux: Vec::new(),
            capsense_time: Vec::new(),
            capsense: Vec::new(),
            battery_voltage_time: Vec::new(),
            battery_voltage: Vec::new(),
        }
    }

    fn estimate(sample_rate: usize, date_start: usize, date_end: usize) -> Option<(usize, usize)> {
        if date_start >= date_end {
            return None;
        }
        let date_difference = date_end - date_start;

        // if more than 1 year, return None
        if date_difference > 365 * 24 * 60 * 60 {
            return None;
        }

        // implausible sample rate
        if sample_rate == 0 || sample_rate > 10_000 {
            return None;
        }

        let seconds = date_difference / sample_rate / 100_000;
        let samples = seconds * sample_rate;

        Some((samples, seconds))
    }

    pub fn reserve(&mut self, samples: usize, seconds: usize) {
        // These are estimates, reserving about 1.6 times the actual size in our test data

        self.acceleration_time.reserve(samples);
        self.acceleration.reserve(samples * 3);

        self.lux.reserve(seconds / 4);
        self.lux_time.reserve(seconds / 4);
        self.capsense.reserve(seconds / 60);
        self.capsense_time.reserve(seconds / 60);
        self.battery_voltage.reserve(seconds / 60);
        self.battery_voltage_time.reserve(seconds / 60);
    }

    pub fn reserve_estimate(&mut self, sample_rate: usize, date_start: usize, date_end: usize) {
        let estimate = AccelerometerData::estimate(sample_rate, date_start, date_end);

        if let Some((samples, seconds)) = estimate {
            self.reserve(samples, seconds);
        } else {
            self.reserve_default();
        }
    }

    pub fn reserve_default(&mut self) {
        self.reserve(200_000_000, 50_000_000);
    }
}

pub struct LogRecordHeader {
    pub separator: u8,
    pub record_type: u8,
    pub timestamp: u32,
    pub record_size: u16,
}

impl LogRecordHeader {
    fn from_bytes(bytes: &[u8]) -> LogRecordHeader {
        LogRecordHeader {
            separator: bytes[0],
            record_type: bytes[1],
            timestamp: u32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
            record_size: u16::from_le_bytes([bytes[6], bytes[7]]),
        }
    }

    fn valid_seperator(&self) -> bool {
        self.separator == 0x1E
    }

    fn datetime(&self) -> chrono::DateTime<Utc> {
        chrono::DateTime::<Utc>::from_timestamp(self.timestamp as i64, 0).unwrap_or_default()
    }

    fn datetime_nanos(&self) -> i64 {
        self.timestamp as i64 * 1_000_000_000
    }
}

impl std::fmt::Debug for LogRecordHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Separator: {:x} Record Type: {:?} Timestamp: {:?} Record Size: {}",
            self.separator,
            LogRecordType::from_u8(self.record_type),
            self.datetime(),
            self.record_size
        )
    }
}

struct LogRecordIterator<R: Read> {
    buffer: R,
}

impl<R: Read> LogRecordIterator<R> {
    fn new(buffer: R) -> LogRecordIterator<R> {
        LogRecordIterator { buffer: buffer }
    }
}

impl<R: Read> LogRecordIterator<R> {
    fn next<'a>(
        &mut self,
        data: &'a mut [u8],
    ) -> Option<std::result::Result<(LogRecordHeader, &'a [u8]), &'static str>> {
        let mut header = [0u8; 8];
        match self.buffer.read_exact(&mut header) {
            Ok(_) => {
                let record_header = LogRecordHeader::from_bytes(&header);

                if !record_header.valid_seperator()
                    || (record_header.record_size as usize + 1) >= data.len()
                {
                    return Some(Err("File read aborted after encountering invalid record"));
                }

                let mut data = &mut data[0..record_header.record_size as usize + 1];

                match self.buffer.read_exact(&mut data) {
                    Ok(_) => Some(Ok((record_header, data))),
                    Err(_) => Some(Err("Unexpected end of file")),
                }
            }
            Err(_) => None,
        }
    }
}

pub struct ActigraphReader {
    data: AccelerometerData,
}

impl ActigraphReader {
    pub fn new() -> ActigraphReader {
        ActigraphReader {
            data: AccelerometerData::new(),
        }
    }
}

fn parse_metadata(record_data: &[u8]) -> Option<&str> {
    std::str::from_utf8(&record_data[0..record_data.len() - 1]).ok()
}

struct Parameters {
    sample_rate: u32,
    accel_scale: f32,
    device_features: DeviceFeatures,
}

fn parse_parameters(record_data: &[u8]) -> Parameters {
    let mut params = Parameters {
        sample_rate: 30,
        accel_scale: 1.0,
        device_features: DeviceFeatures::new(),
    };

    for offset in (0..record_data.len() - 1).step_by(8) {
        let param_type = u32::from_le_bytes([
            record_data[offset],
            record_data[offset + 1],
            record_data[offset + 2],
            record_data[offset + 3],
        ]);
        let param_identifier = (param_type >> 16) as u16;
        let param_address_space = (param_type & 0xFFFF) as u16;

        let parameter_type = ParameterType::from_u16(param_address_space, param_identifier);

        match parameter_type {
            ParameterType::SampleRate => {
                params.sample_rate = u32::from_le_bytes([
                    record_data[offset + 4],
                    record_data[offset + 5],
                    record_data[offset + 6],
                    record_data[offset + 7],
                ]);
            }
            ParameterType::AccelScale => {
                let ssp_val = u32::from_le_bytes([
                    record_data[offset + 4],
                    record_data[offset + 5],
                    record_data[offset + 6],
                    record_data[offset + 7],
                ]);
                params.accel_scale = ssp_codec::decode(ssp_val) as f32;
            }
            ParameterType::FeatureEnable => {
                let x = u32::from_le_bytes([
                    record_data[offset + 4],
                    record_data[offset + 5],
                    record_data[offset + 6],
                    record_data[offset + 7],
                ]);
                params.device_features = DeviceFeatures {
                    heart_rate_monitor: x & 1 != 0,
                    data_summary: x & 2 != 0,
                    sleep_mode: x & 4 != 0,
                    proximity_tagging: x & 8 != 0,
                    epoch_data: x & 16 != 0,
                    no_raw_data: x & 32 != 0,
                };
            }
            _ => {
                /*let val = u32::from_le_bytes([
                    data[offset + 4],
                    data[offset + 5],
                    data[offset + 6],
                    data[offset + 7],
                ]);
                println!("Unhandled parameter type: {:?}={}", parameter_type, val);*/
            }
        }
    }
    params
}

fn parse_lux(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}

fn parse_battery_voltage(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}

fn parse_capsense(data: &[u8]) -> bool {
    data[4] != 0
}

impl<'a> sensors::SensorsFormatReader<'a> for ActigraphReader {
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
        let mut archive = zip::ZipArchive::new(reader).or(Err("Could not open zip archive"))?;

        // read metadat
        let mut header_sample_rate: usize = 30;
        let mut header_date_start: usize = 0;
        let mut header_date_end: usize = 0;

        // Read the file line by line and parse into dictionary
        for line in BufReader::new(
            archive
                .by_name(GT3X_FILE_INFO)
                .or(Err("Could not open info file"))?,
        )
        .lines()
        {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.splitn(2, ": ").collect();
                if parts.len() == 2 {
                    metadata_callback(sensors::MetadataEntry {
                        category: "info",
                        key: parts[0],
                        value: parts[1],
                    });

                    match parts[0] {
                        "Sample Rate" => {
                            header_sample_rate = parts[1].parse().unwrap_or(30);
                        }
                        "Start Date" => {
                            header_date_start = parts[1].parse().unwrap_or(0);
                        }
                        "Last Sample Time" => {
                            header_date_end = parts[1].parse().unwrap_or(0);
                        }
                        _ => {}
                    }
                }
            }
        }

        // estimate & reserve data sizes
        self.data
            .reserve_estimate(header_sample_rate, header_date_start, header_date_end);

        // read log data

        let mut log = BufReader::new(
            archive
                .by_name(GT3X_FILE_LOG)
                .or(Err("Could not open log file"))?,
        );

        let mut sample_rate = 30;
        let mut accel_scale = 1.0_f32 / 256.0_f32;

        let mut record_data = [0u8; u16::MAX as usize + 1];

        let mut it = LogRecordIterator::new(&mut log);

        let mut metadata_counter = 0;

        while let Some(record) = it.next(&mut record_data) {
            let (record_header, record_data) = record?;

            if !record_header.valid_seperator() {
                //println!("Invalid separator: {:x}", record_header.separator);
            }

            match LogRecordType::from_u8(record_header.record_type) {
                LogRecordType::Metadata => {
                    if let Some(metadata) = parse_metadata(record_data) {
                        metadata_counter += 1;
                        metadata_callback(sensors::MetadataEntry {
                            category: "metadata",
                            key: &format!("metadata_{}", metadata_counter),
                            value: metadata,
                        });
                    }
                }
                LogRecordType::Parameters => {
                    let params = parse_parameters(record_data);
                    sample_rate = params.sample_rate;
                    accel_scale = params.accel_scale;
                }
                LogRecordType::Activity => {
                    let dt = record_header.datetime();

                    let mut reader = BitReader::new(&record_data[0..record_data.len() - 1]);

                    let mut i = 0;

                    while let Ok(v) = reader.read_i16(12) {
                        let y = v;
                        let x = match reader.read_i16(12) {
                            Ok(val) => val,
                            Err(_) => break,
                        };
                        let z = match reader.read_i16(12) {
                            Ok(val) => val,
                            Err(_) => break,
                        };

                        let timestamp_nanos = datetime_add_hz(dt, sample_rate, i)
                            .timestamp_nanos_opt()
                            .unwrap_or_default();

                        self.data.acceleration_time.push(timestamp_nanos);
                        self.data.acceleration.extend(&[
                            x as f32 / accel_scale,
                            y as f32 / accel_scale,
                            z as f32 / accel_scale,
                        ]);

                        i += 1;
                    }
                }
                LogRecordType::Lux => {
                    let lux = parse_lux(record_data);
                    let timestamp_nanos = record_header.datetime_nanos();
                    self.data.lux.push(lux);
                    self.data.lux_time.push(timestamp_nanos);
                }
                LogRecordType::Battery => {
                    let voltage = parse_battery_voltage(record_data);

                    let timestamp_nanos = record_header.datetime_nanos();
                    self.data.battery_voltage.push(voltage);
                    self.data.battery_voltage_time.push(timestamp_nanos);
                }
                LogRecordType::Capsense => {
                    //let signal = u16::from_le_bytes([data[0], data[1]]);
                    //let reference = u16::from_le_bytes([data[2], data[3]);
                    let state = parse_capsense(record_data);
                    //let bursts = data[5];
                    let timestamp_nanos = record_header.datetime_nanos();
                    self.data.capsense.push(state);
                    self.data.capsense_time.push(timestamp_nanos);
                }
                _ => {
                    //println!("Unhandled record type: {:?}", LogRecordType::from_u8(record_header.record_type));
                }
            }
        }

        sensor_table_callback(sensors::SensorTable {
            name: sensors::SensorKind::Accelerometer.to_str(),
            datetime: &self.data.acceleration_time,
            data: vec![sensors::SensorData {
                kind: sensors::SensorKind::Accelerometer,
                data: sensors::SensorDataDyn::F32(&self.data.acceleration),
            }],
        });

        sensor_table_callback(sensors::SensorTable {
            name: sensors::SensorKind::Light.to_str(),
            datetime: &self.data.lux_time,
            data: vec![sensors::SensorData {
                kind: sensors::SensorKind::Light,
                data: sensors::SensorDataDyn::U16(&self.data.lux),
            }],
        });

        sensor_table_callback(sensors::SensorTable {
            name: sensors::SensorKind::Capacitive.to_str(),
            datetime: &self.data.capsense_time,
            data: vec![sensors::SensorData {
                kind: sensors::SensorKind::Capacitive,
                data: sensors::SensorDataDyn::Bool(&self.data.capsense),
            }],
        });

        sensor_table_callback(sensors::SensorTable {
            name: sensors::SensorKind::BatteryVoltage.to_str(),
            datetime: &self.data.battery_voltage_time,
            data: vec![sensors::SensorData {
                kind: sensors::SensorKind::BatteryVoltage,
                data: sensors::SensorDataDyn::U16(&self.data.battery_voltage),
            }],
        });

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
    fn test_actigraph_reader() {
        let data = include_bytes!("../../test_data/actigraph.gt3x");
        let mut reader = ActigraphReader::new();
        let mut metadata = HashMap::new();
        let mut sensor_table = HashMap::new();
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

        assert_eq!(metadata.len(), 19);
        assert_eq!(sensor_table.len(), 4);

        assert_eq!(metadata[&("info".into(), "Sample Rate".into())], "60");
        assert_eq!(
            metadata[&("info".into(), "Start Date".into())],
            "638500855800000000"
        );
        assert_eq!(
            metadata[&("info".into(), "Last Sample Time".into())],
            "638500857000000000"
        );

        assert_eq!(sensor_table["acceleration"].datetime.len(), 4860);
        assert_eq!(sensor_table["acceleration"].data.len(), 1);
        assert_eq!(sensor_table["acceleration"].data[0].kind, sensors::SensorKind::Accelerometer);
        assert_eq!(sensor_table["acceleration"].datetime[0], 1714488780000000000);
        assert_eq!(sensor_table["acceleration"].datetime[1], 1714488780000000000 + 1_000_000_000 / 60);
        // always F32
        if let sensors::SensorDataDyn::F32(data) = &sensor_table["acceleration"].data[0].data {
            assert_eq!(data.len(), 4860 * 3);
            
            assert_approx_eq!(data[0], -0.519531, 1e-6);
            assert_approx_eq!(data[1], -0.519531, 1e-6);
            assert_approx_eq!(data[2], -0.636719, 1e-6);

            assert_approx_eq!(data[3], -0.296875, 1e-6);
            assert_approx_eq!(data[4], -0.550781, 1e-6);
            assert_approx_eq!(data[5], -0.554688, 1e-6);

            // last entry
            assert_approx_eq!(data[4859 * 3], 0.117188, 1e-6);
            assert_approx_eq!(data[4859 * 3 + 1], 0.003906, 1e-6);
            assert_approx_eq!(data[4859 * 3 + 2], 0.945312, 1e-6);

        } else {
            panic!("Expected F32 data");
        }

        assert_eq!(sensor_table["light"].datetime.len(), 2);
        assert_eq!(sensor_table["light"].data.len(), 1);

        assert_eq!(sensor_table["capsense"].datetime.len(), 3);
        assert_eq!(sensor_table["capsense"].data.len(), 1);

        assert_eq!(sensor_table["battery_voltage"].datetime.len(), 3);
        assert_eq!(sensor_table["battery_voltage"].data.len(), 1);
    }
}
