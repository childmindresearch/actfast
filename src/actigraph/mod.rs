mod defs;

use bitreader::BitReader;
use chrono::TimeDelta;
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read},
};

use crate::actigraph::defs::*;

fn datetime_add_hz(
    dt: chrono::NaiveDateTime,
    hz: u32,
    sample_counter: u32,
) -> chrono::NaiveDateTime {
    dt.checked_add_signed(TimeDelta::nanoseconds(
        (1_000_000_000 / hz * sample_counter) as i64,
    ))
    .unwrap()
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
    pub metadata_json: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub device_features: DeviceFeatures,
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
            metadata_json: Vec::new(),
            metadata: HashMap::new(),
            device_features: DeviceFeatures::new(),
        }
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

    fn datetime(&self) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::from_timestamp_opt(self.timestamp as i64, 0).unwrap()
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
    fn next<'a>(&mut self, data: &'a mut [u8]) -> Option<(LogRecordHeader, &'a [u8])> {
        let mut header = [0u8; 8];
        match self.buffer.read_exact(&mut header) {
            Ok(_) => {
                let record_header = LogRecordHeader::from_bytes(&header);

                let mut data = &mut data[0..record_header.record_size as usize + 1];

                self.buffer.read_exact(&mut data).unwrap();

                Some((record_header, data))
            }
            Err(_) => None,
        }
    }
}

/// Decode SSP floating point format
fn decode_ssp_f32(data: &[u8]) -> f32 {
    let mut reader = BitReader::new(data);
    let fraction = reader.read_i32(24).unwrap();
    let exponent = reader.read_i8(8).unwrap();
    (fraction as f32 / 8_388_608.0) * 2.0_f32.powi(exponent as i32)
}

fn estimate_data_size(metadata: &HashMap<String, String>) -> Option<(usize, usize)> {
    let sample_rate: usize = metadata.get("Sample Rate")?.parse().ok()?;
    let date_start: usize = metadata.get("Start Date")?.parse().ok()?;
    let date_end: usize = metadata.get("Last Sample Time")?.parse().ok()?;

    let duration = (date_end - date_start) / sample_rate / 100_000;
    let num_samples = duration * sample_rate;

    Some((duration, num_samples))
}

pub fn load_data(path: String) -> AccelerometerData {
    let fname = std::path::Path::new(&path);
    let file = fs::File::open(fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut data = AccelerometerData::new();

    // read metadat

    // Read the file line by line and parse into dictionary
    for line in BufReader::new(archive.by_name(GT3X_FILE_INFO).unwrap()).lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() == 2 {
                data.metadata
                    .insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    // estimate data sizes

    let (estimate_seconds, estimate_samples) =
        estimate_data_size(&data.metadata).unwrap_or((50_000_000, 200_000_000));

    // These are estimates, reserving about 1.6 times the actual size in our test data
    data.acceleration_time.reserve(estimate_samples);
    data.acceleration.reserve(estimate_samples * 3);
    data.lux.reserve(estimate_seconds / 4);
    data.lux_time.reserve(estimate_seconds / 4);
    data.capsense.reserve(estimate_seconds / 60);
    data.capsense_time.reserve(estimate_seconds / 60);
    data.battery_voltage.reserve(estimate_seconds / 60);
    data.battery_voltage_time.reserve(estimate_seconds / 60);

    // read log data

    let mut log = BufReader::new(archive.by_name(GT3X_FILE_LOG).unwrap());

    let mut sample_rate = 30;
    let mut accel_scale = 1.0_f32 / 256.0_f32;

    let mut record_data = [0u8; u16::MAX as usize + 1];

    let mut it = LogRecordIterator::new(&mut log);

    while let Some((record_header, record_data)) = it.next(&mut record_data) {
        if !record_header.valid_seperator() {
            //println!("Invalid separator: {:x}", record_header.separator);
        }

        match LogRecordType::from_u8(record_header.record_type) {
            LogRecordType::Metadata => {
                let metadata = std::str::from_utf8(&record_data[0..record_data.len() - 1]).unwrap();
                data.metadata_json.push(metadata.to_owned());
            }
            LogRecordType::Parameters => {
                for offset in (0..record_data.len() - 1).step_by(8) {
                    let param_type = u32::from_le_bytes([
                        record_data[offset],
                        record_data[offset + 1],
                        record_data[offset + 2],
                        record_data[offset + 3],
                    ]);
                    let param_identifier = (param_type >> 16) as u16;
                    let param_address_space = (param_type & 0xFFFF) as u16;

                    let parameter_type =
                        ParameterType::from_u16(param_address_space, param_identifier);

                    match parameter_type {
                        ParameterType::SampleRate => {
                            sample_rate = u32::from_le_bytes([
                                record_data[offset + 4],
                                record_data[offset + 5],
                                record_data[offset + 6],
                                record_data[offset + 7],
                            ]);
                        }
                        ParameterType::AccelScale => {
                            accel_scale = decode_ssp_f32(&record_data[offset + 4..offset + 8]);
                        }
                        ParameterType::FeatureEnable => {
                            let x = u32::from_le_bytes([
                                record_data[offset + 4],
                                record_data[offset + 5],
                                record_data[offset + 6],
                                record_data[offset + 7],
                            ]);
                            data.device_features = DeviceFeatures {
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

                    let timestamp_nanos =
                        datetime_add_hz(dt, sample_rate, i)
                            .timestamp_nanos_opt()
                            .unwrap();

                    data.acceleration_time.push(timestamp_nanos);
                    data.acceleration.extend(&[
                        x as f32 * accel_scale,
                        y as f32 * accel_scale,
                        z as f32 * accel_scale,
                    ]);

                    i += 1;
                }
            }
            LogRecordType::Lux => {
                let lux = u16::from_le_bytes([record_data[0], record_data[1]]);
                let timestamp_nanos = record_header.datetime().timestamp_nanos_opt().unwrap();
                data.lux.push(lux);
                data.lux_time.push(timestamp_nanos);
            }
            LogRecordType::Battery => {
                let voltage = u16::from_le_bytes([record_data[0], record_data[1]]);

                let timestamp_nanos = record_header.datetime().timestamp_nanos_opt().unwrap();
                data.battery_voltage.push(voltage);
                data.battery_voltage_time.push(timestamp_nanos);
            }
            LogRecordType::Capsense => {
                //let signal = u16::from_le_bytes([data[0], data[1]]);
                //let reference = u16::from_le_bytes([data[2], data[3]);
                let state = record_data[4] != 0;
                //let bursts = data[5];
                let timestamp_nanos = record_header.datetime().timestamp_nanos_opt().unwrap();
                data.capsense.push(state);
                data.capsense_time.push(timestamp_nanos);
            }
            _ => {
                //println!("Unhandled record type: {:?}", LogRecordType::from_u8(record_header.record_type));
            }
        }
    }

    data
}
