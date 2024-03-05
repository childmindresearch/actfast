mod defs;

use bitreader::BitReader;
use chrono::TimeDelta;
use std::{
    fs,
    io::{BufReader, Read},
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
    pub metadata: Vec<String>,
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

struct LogRecord {
    header: LogRecordHeader,
    data: Vec<u8>,
}

struct LogRecordIterator<R: Read> {
    buffer: R,
}

impl<R: Read> LogRecordIterator<R> {
    fn new(buffer: R) -> LogRecordIterator<R> {
        LogRecordIterator { buffer: buffer }
    }
}

impl<R: Read> Iterator for LogRecordIterator<R> {
    type Item = LogRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header = [0u8; 8];
        match self.buffer.read_exact(&mut header) {
            Ok(_) => {
                let record_header = LogRecordHeader::from_bytes(&header);

                let mut data = vec![0u8; record_header.record_size as usize + 1];
                self.buffer.read_exact(&mut data).unwrap();

                Some(LogRecord {
                    header: record_header,
                    data: data,
                })
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

pub fn load_data(path: String) -> AccelerometerData {
    let fname = std::path::Path::new(&path);
    let file = fs::File::open(fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    // measure execution time start
    //use std::time::Instant;
    //let now = Instant::now();

    // read metadata

    /*let mut info: HashMap<String, String> = HashMap::new();

    // Read the file line by line and parse into dictionary
    for line in BufReader::new(archive.by_name(GT3X_FILE_INFO).unwrap()).lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() == 2 {
                info.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }
    // print dictionary
    println!("{:?}", info);*/

    // read log data

    // Read buffered stream

    let mut log = BufReader::new(archive.by_name(GT3X_FILE_LOG).unwrap());

    // Loop through entries

    // count records by type
    //let mut record_counts: std::collections::HashMap<u8, u32> = std::collections::HashMap::new();

    let mut data = AccelerometerData {
        acceleration_time: Vec::with_capacity(50_000_000),
        acceleration: Vec::with_capacity(200_000_000),
        lux: Vec::with_capacity(50_000),
        lux_time: Vec::with_capacity(50_000),
        capsense: Vec::with_capacity(50_000),
        capsense_time: Vec::with_capacity(50_000),
        battery_voltage: Vec::with_capacity(50_000),
        battery_voltage_time: Vec::with_capacity(50_000),
        metadata: Vec::new(),
    };

    //let mut counter = 0;

    let mut sample_rate = 30;
    let mut accel_scale = 1.0_f32 / 256.0_f32;

    for record in LogRecordIterator::new(&mut log) {
        if !record.header.valid_seperator() {
            //println!("Invalid separator: {:x}", record.header.separator);
        }

        match LogRecordType::from_u8(record.header.record_type) {
            LogRecordType::Metadata => {
                let metadata = std::str::from_utf8(&record.data[0..record.data.len() - 1]).unwrap();
                data.metadata.push(metadata.to_owned());
            }
            LogRecordType::Parameters => {
                for offset in (0..record.data.len() - 1).step_by(8) {
                    let param_type = u32::from_le_bytes([
                        record.data[offset],
                        record.data[offset + 1],
                        record.data[offset + 2],
                        record.data[offset + 3],
                    ]);
                    let param_identifier = (param_type >> 16) as u16;
                    let param_address_space = (param_type & 0xFFFF) as u16;

                    let parameter_type =
                        ParameterType::from_u16(param_address_space, param_identifier);

                    match parameter_type {
                        ParameterType::SampleRate => {
                            sample_rate = u32::from_le_bytes([
                                record.data[offset + 4],
                                record.data[offset + 5],
                                record.data[offset + 6],
                                record.data[offset + 7],
                            ]);
                        }
                        ParameterType::AccelScale => {
                            accel_scale = decode_ssp_f32(&record.data[offset + 4..offset + 8]);
                        }
                        _ => {
                            /*let val = u32::from_le_bytes([
                                record.data[offset + 4],
                                record.data[offset + 5],
                                record.data[offset + 6],
                                record.data[offset + 7],
                            ]);
                            println!("Unhandled parameter type: {:?}={}", parameter_type, val);*/
                        }
                    }
                }
            }
            LogRecordType::Activity => {
                let dt = record.header.datetime();

                let mut reader = BitReader::new(&record.data[0..record.data.len() - 1]);

                let mut field = Vec::<i16>::with_capacity(31 * 3);

                while let Ok(v) = reader.read_i16(12) {
                    field.push(v);
                }

                for i in (0..field.len()).step_by(3) {
                    let y = field[i];
                    let x = field[i + 1];
                    let z = field[i + 2];

                    let timestamp_nanos = datetime_add_hz(dt, sample_rate, i as u32 / 3)
                        .timestamp_nanos_opt()
                        .unwrap();

                    data.acceleration_time.push(timestamp_nanos);
                    data.acceleration.extend(&[
                        x as f32 * accel_scale,
                        y as f32 * accel_scale,
                        z as f32 * accel_scale,
                    ]);
                }
            }
            LogRecordType::Lux => {
                let lux = u16::from_le_bytes([record.data[0], record.data[1]]);
                let timestamp_nanos = record.header.datetime().timestamp_nanos_opt().unwrap();
                data.lux.push(lux);
                data.lux_time.push(timestamp_nanos);
            }
            LogRecordType::Battery => {
                let voltage = u16::from_le_bytes([record.data[0], record.data[1]]);

                let timestamp_nanos = record.header.datetime().timestamp_nanos_opt().unwrap();
                data.battery_voltage.push(voltage);
                data.battery_voltage_time.push(timestamp_nanos);
            }
            LogRecordType::Capsense => {
                //let signal = u16::from_le_bytes([record.data[0], record.data[1]]);
                //let reference = u16::from_le_bytes([record.data[2], record.data[3]);
                let state = record.data[4] != 0;
                //let bursts = record.data[5];
                let timestamp_nanos = record.header.datetime().timestamp_nanos_opt().unwrap();
                data.capsense.push(state);
                data.capsense_time.push(timestamp_nanos);
            }
            _ => {}
        }
    }

    data
}
