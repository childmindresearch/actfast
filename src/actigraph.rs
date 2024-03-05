use bitreader::BitReader;
use chrono::TimeDelta;
use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Read},
};

//const GT3X_FILE_INFO: &str = "info.txt";
const GT3X_FILE_LOG: &str = "log.bin";

#[derive(Debug)]
enum LogRecordType {
    Unknown,
    Activity,
    Battery,
    Event,
    HeartRateBPM,
    Lux,
    Metadata,
    Tag,
    Epoch,
    HeartRateAnt,
    Epoch2,
    Capsense,
    HeartRateBle,
    Epoch3,
    Epoch4,
    FifoError,
    FifoDump,
    Parameters,
    SensorSchema,
    SensorData,
    Activity2,
}

impl LogRecordType {
    fn from_u8(val: u8) -> LogRecordType {
        match val {
            0x00 => LogRecordType::Activity,
            0x02 => LogRecordType::Battery,
            0x03 => LogRecordType::Event,
            0x04 => LogRecordType::HeartRateBPM,
            0x05 => LogRecordType::Lux,
            0x06 => LogRecordType::Metadata,
            0x07 => LogRecordType::Tag,
            0x09 => LogRecordType::Epoch,
            0x0B => LogRecordType::HeartRateAnt,
            0x0C => LogRecordType::Epoch2,
            0x0D => LogRecordType::Capsense,
            0x0E => LogRecordType::HeartRateBle,
            0x0F => LogRecordType::Epoch3,
            0x10 => LogRecordType::Epoch4,
            0x13 => LogRecordType::FifoError,
            0x14 => LogRecordType::FifoDump,
            0x15 => LogRecordType::Parameters,
            0x18 => LogRecordType::SensorSchema,
            0x19 => LogRecordType::SensorData,
            0x1A => LogRecordType::Activity2,
            _ => LogRecordType::Unknown,
        }
    }
}

#[derive(Debug)]
enum ParameterType {
    Unknown,
    BatteryState,
    BatteryVoltage,
    BoardRevision,
    CalibrationTime,
    FirmwareVersion,
    MemorySize,
    FeatureCapabilities,
    DisplayCapabilities,
    WirelessFirmwareVersion,
    IMUAccelScale,
    IMUGyroScale,
    IMUMagScale,
    AccelScale,
    IMUTempScale,
    IMUTempOffset,
    WirelessMode,
    WirelessSerialNumber,
    FeatureEnable,
    DisplayConfiguration,
    NegativeGOffsetX,
    NegativeGOffsetY,
    NegativeGOffsetZ,
    PositiveGOffsetX,
    PositiveGOffsetY,
    PositiveGOffsetZ,
    SampleRate,
    TargetStartTime,
    TargetStopTime,
    TimeOfDay,
    ZeroGOffsetX,
    ZeroGOffsetY,
    ZeroGOffsetZ,
    HRMSerialNumberH,
    HRMSerialNumberL,
    ProximityInterval,
    IMUNegativeGOffsetX,
    IMUNegativeGOffsetY,
    IMUNegativeGOffsetZ,
    IMUPositiveGOffsetX,
    IMUPositiveGOffsetY,
    IMUPositiveGOffsetZ,
    UTCOffset,
    IMUZeroGOffsetX,
    IMUZeroGOffsetY,
    IMUZeroGOffsetZ,
    SensorConfiguration,
}

impl ParameterType {
    fn from_u16(address_space: u16, identifier: u16) -> ParameterType {
        match address_space {
            0 => match identifier {
                6 => ParameterType::BatteryState,
                7 => ParameterType::BatteryVoltage,
                8 => ParameterType::BoardRevision,
                9 => ParameterType::CalibrationTime,
                13 => ParameterType::FirmwareVersion,
                16 => ParameterType::MemorySize,
                28 => ParameterType::FeatureCapabilities,
                29 => ParameterType::DisplayCapabilities,
                32 => ParameterType::WirelessFirmwareVersion,
                49 => ParameterType::IMUAccelScale,
                50 => ParameterType::IMUGyroScale,
                51 => ParameterType::IMUMagScale,
                55 => ParameterType::AccelScale,
                57 => ParameterType::IMUTempScale,
                58 => ParameterType::IMUTempOffset,
                _ => ParameterType::Unknown,
            },
            1 => match identifier {
                0 => ParameterType::WirelessMode,
                1 => ParameterType::WirelessSerialNumber,
                2 => ParameterType::FeatureEnable,
                3 => ParameterType::DisplayConfiguration,
                4 => ParameterType::NegativeGOffsetX,
                5 => ParameterType::NegativeGOffsetY,
                6 => ParameterType::NegativeGOffsetZ,
                7 => ParameterType::PositiveGOffsetX,
                8 => ParameterType::PositiveGOffsetY,
                9 => ParameterType::PositiveGOffsetZ,
                10 => ParameterType::SampleRate,
                12 => ParameterType::TargetStartTime,
                13 => ParameterType::TargetStopTime,
                14 => ParameterType::TimeOfDay,
                15 => ParameterType::ZeroGOffsetX,
                16 => ParameterType::ZeroGOffsetY,
                17 => ParameterType::ZeroGOffsetZ,
                20 => ParameterType::HRMSerialNumberH,
                21 => ParameterType::HRMSerialNumberL,
                33 => ParameterType::ProximityInterval,
                34 => ParameterType::IMUNegativeGOffsetX,
                35 => ParameterType::IMUNegativeGOffsetY,
                36 => ParameterType::IMUNegativeGOffsetZ,
                37 => ParameterType::IMUPositiveGOffsetX,
                38 => ParameterType::IMUPositiveGOffsetY,
                39 => ParameterType::IMUPositiveGOffsetZ,
                40 => ParameterType::UTCOffset,
                41 => ParameterType::IMUZeroGOffsetX,
                42 => ParameterType::IMUZeroGOffsetY,
                43 => ParameterType::IMUZeroGOffsetZ,
                44 => ParameterType::SensorConfiguration,
                _ => ParameterType::Unknown,
            },
            _ => ParameterType::Unknown,
        }
    }
}

impl std::fmt::Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct LogRecordHeader {
    separator: u8,
    record_type: u8,
    timestamp: u32,
    record_size: u16,
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
    pub time: Vec<i64>,
    pub acceleration: Vec<f32>,
    pub lux: Vec<u16>,
    pub metadata: HashMap<String, u32>,
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
        time: Vec::with_capacity(50_000_000),
        acceleration: Vec::with_capacity(200_000_000),
        lux: Vec::with_capacity(50_000),
        metadata: HashMap::new(),
    };

    //let mut counter = 0;

    let mut sample_rate = 30;

    for record in LogRecordIterator::new(&mut log) {
        if !record.header.valid_seperator() {
            println!("Invalid separator: {:x}", record.header.separator);
        }

        match LogRecordType::from_u8(record.header.record_type) {
            LogRecordType::Unknown => {
                println!("Unknown record type: {:?}", record.header.record_type);
            }
            /*LogRecordType::Metadata => {

                // last byte needs to be skipped
                let metadata = std::str::from_utf8(&buffer[0..buffer.len() - 1]).unwrap();
                println!("Metadata: {}", metadata);
            }*/
            LogRecordType::Parameters => {
                // last byte needs to be skipped
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
                        _ => {}
                    }

                    match parameter_type {
                        ParameterType::Unknown => {}
                        _ => {
                            // add to metadata dict
                            data.metadata.insert(
                                parameter_type.to_string(),
                                u32::from_le_bytes([
                                    record.data[offset + 4],
                                    record.data[offset + 5],
                                    record.data[offset + 6],
                                    record.data[offset + 7],
                                ]),
                            );
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

                    data.time.push(timestamp_nanos);
                    data.acceleration.extend(&[
                        x as f32 / 256.0,
                        y as f32 / 256.0,
                        z as f32 / 256.0,
                    ]);
                }
            },
            LogRecordType::Lux => {
                //let dt = record.header.datetime();
                let lux = u16::from_le_bytes([record.data[0], record.data[1]]);
                //let timestamp_nanos = dt.timestamp_nanos_opt().unwrap();
                data.lux.push(lux);
            },
            _ => {}
        }
    }

    data
}