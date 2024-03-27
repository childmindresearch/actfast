/**
 * Device Identity
Device Unique Serial Code:101774
Device Type:GENEActiv
Device Model:1.2
Device Firmware Version:Ver06.17 15June23
Calibration Date:2023-09-07 15:04:34:000

Device Capabilities
Accelerometer Range:-8 to 8
Accelerometer Resolution:0.0039
Accelerometer Units:g
Light Meter Range:0 to 20000
Light Meter Resolution:3 to 48
Light Meter Units:lux
Temperature Sensor Range:0 to 70
Temperature Sensor Resolution:0.1
Temperature Sensor Units:deg. C

Configuration Info
Measurement Frequency:50 Hz
Measurement Period:720 Hours
Start Time:2024-01-11 15:35:30:000
Time Zone:GMT -05:00

Trial Info
Study Centre:
Study Code:
Investigator ID:
Exercise Type:
Config Operator ID:
Config Time:2024-01-11 14:07:24:471
Config Notes:
Extract Operator ID:
Extract Time:2024-01-31 12:22:13:153
Extract Notes:(device clock drift -33,078,823.924s)

Subject Info
Device Location Code:
Subject Code:5063690
Date of Birth:1900-01-01
Sex:
Height:
Weight:
Handedness Code:
Subject Notes:

Calibration Data
x gain:24949
x offset:-813
y gain:24936
y offset:-1262
z gain:25168
z offset:492
Volts:60
Lux:988

Memory Status
Number of Pages:103717


Recorded Data
Device Unique Serial Code:101774
Sequence Number:0
Page Time:2024-01-11 15:35:31:000
Unassigned:
Temperature:37.2
Battery voltage:4.1400
Device Status:Recording
Measurement Frequency:50.0
 */

#[allow(dead_code)]
pub mod id {
    pub mod identity {
        pub const HEADER: &str = "Device Identity";
        pub const SERIAL: &str = "Device Unique Serial Code:";
        pub const TYPE: &str = "Device Type:";
        pub const MODEL: &str = "Device Model:";
        pub const FIRMWARE: &str = "Device Firmware Version:";
        pub const CALIBRATION_DATE: &str = "Calibration Date:";
    }

    pub mod capabilities {
        pub const HEADER: &str = "Device Capabilities";
        pub const ACCELEROMETER_RANGE: &str = "Accelerometer Range:";
        pub const ACCELEROMETER_RESOLUTION: &str = "Accelerometer Resolution:";
        pub const ACCELEROMETER_UNITS: &str = "Accelerometer Units:";
        pub const LIGHT_METER_RANGE: &str = "Light Meter Range:";
        pub const LIGHT_METER_RESOLUTION: &str = "Light Meter Resolution:";
        pub const LIGHT_METER_UNITS: &str = "Light Meter Units:";
        pub const TEMPERATURE_SENSOR_RANGE: &str = "Temperature Sensor Range:";
        pub const TEMPERATURE_SENSOR_RESOLUTION: &str = "Temperature Sensor Resolution:";
        pub const TEMPERATURE_SENSOR_UNITS: &str = "Temperature Sensor Units:";
    }

    pub mod configuration {
        pub const HEADER: &str = "Configuration Info";
        pub const MEASUREMENT_FREQUENCY: &str = "Measurement Frequency:";
        pub const MEASUREMENT_PERIOD: &str = "Measurement Period:";
        pub const START_TIME: &str = "Start Time:";
        pub const TIME_ZONE: &str = "Time Zone:";
    }

    pub mod trial {
        pub const HEADER: &str = "Trial Info";
        pub const STUDY_CENTRE: &str = "Study Centre:";
        pub const STUDY_CODE: &str = "Study Code:";
        pub const INVESTIGATOR_ID: &str = "Investigator ID:";
        pub const EXERCISE_TYPE: &str = "Exercise Type:";
        pub const CONFIG_OPERATOR_ID: &str = "Config Operator ID:";
        pub const CONFIG_TIME: &str = "Config Time:";
        pub const CONFIG_NOTES: &str = "Config Notes:";
        pub const EXTRACT_OPERATOR_ID: &str = "Extract Operator ID:";
        pub const EXTRACT_TIME: &str = "Extract Time:";
        pub const EXTRACT_NOTES: &str = "Extract Notes:";
    }

    pub mod subject {
        pub const HEADER: &str = "Subject Info";
        pub const LOCATION_CODE: &str = "Device Location Code:";
        pub const CODE: &str = "Subject Code:";
        pub const DATE_OF_BIRTH: &str = "Date of Birth:";
        pub const SEX: &str = "Sex:";
        pub const HEIGHT: &str = "Height:";
        pub const WEIGHT: &str = "Weight:";
        pub const HANDEDNESS_CODE: &str = "Handedness Code:";
        pub const NOTES: &str = "Subject Notes:";
    }

    pub mod calibration {
        pub const HEADER: &str = "Calibration Data";
        pub const X_GAIN: &str = "x gain:";
        pub const X_OFFSET: &str = "x offset:";
        pub const Y_GAIN: &str = "y gain:";
        pub const Y_OFFSET: &str = "y offset:";
        pub const Z_GAIN: &str = "z gain:";
        pub const Z_OFFSET: &str = "z offset:";
        pub const VOLTS: &str = "Volts:";
        pub const LUX: &str = "Lux:";
    }

    pub mod memory {
        pub const HEADER: &str = "Memory Status";
        pub const PAGES: &str = "Number of Pages:";
    }

    pub mod record {
        pub const HEADER: &str = "Recorded Data";
        pub const SERIAL: &str = "Device Unique Serial Code:";
        pub const SEQUENCE: &str = "Sequence Number:";
        pub const PAGE_TIME: &str = "Page Time:";
        pub const UNASSIGNED: &str = "Unassigned:";
        pub const TEMPERATURE: &str = "Temperature:";
        pub const BATTERY_VOLTAGE: &str = "Battery voltage:";
        pub const DEVICE_STATUS: &str = "Device Status:";
        pub const MEASUREMENT_FREQUENCY: &str = "Measurement Frequency:";
    }
}

#[allow(dead_code)]
pub mod data {
    use crate::geneactiv::id;
    use chrono::{DateTime, NaiveDate, Utc};
    use struct_iterable::Iterable;

    #[derive(Debug, Iterable)]
    pub struct DeviceIdentity {
        pub serial: String,
        pub device_type: String,
        pub model: String,
        pub firmware: String,
        pub calibration_date: DateTime<Utc>,
    }

    impl DeviceIdentity {
        pub fn new() -> DeviceIdentity {
            DeviceIdentity {
                serial: String::new(),
                device_type: String::new(),
                model: String::new(),
                firmware: String::new(),
                calibration_date: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::identity::SERIAL => self.serial = value.to_string(),
                id::identity::TYPE => self.device_type = value.to_string(),
                id::identity::MODEL => self.model = value.to_string(),
                id::identity::FIRMWARE => self.firmware = value.to_string(),
                id::identity::CALIBRATION_DATE => {
                    self.calibration_date =
                        DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S:%3f")
                            .unwrap()
                            .to_utc();
                }
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct DeviceCapabilities {
        pub accelerometer_range: String,
        pub accelerometer_resolution: String,
        pub accelerometer_units: String,
        pub light_meter_range: String,
        pub light_meter_resolution: String,
        pub light_meter_units: String,
        pub temperature_sensor_range: String,
        pub temperature_sensor_resolution: String,
        pub temperature_sensor_units: String,
    }

    impl DeviceCapabilities {
        pub fn new() -> DeviceCapabilities {
            DeviceCapabilities {
                accelerometer_range: String::new(),
                accelerometer_resolution: String::new(),
                accelerometer_units: String::new(),
                light_meter_range: String::new(),
                light_meter_resolution: String::new(),
                light_meter_units: String::new(),
                temperature_sensor_range: String::new(),
                temperature_sensor_resolution: String::new(),
                temperature_sensor_units: String::new(),
            }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::capabilities::ACCELEROMETER_RANGE => {
                    self.accelerometer_range = value.to_string()
                }
                id::capabilities::ACCELEROMETER_RESOLUTION => {
                    self.accelerometer_resolution = value.to_string()
                }
                id::capabilities::ACCELEROMETER_UNITS => {
                    self.accelerometer_units = value.to_string()
                }
                id::capabilities::LIGHT_METER_RANGE => self.light_meter_range = value.to_string(),
                id::capabilities::LIGHT_METER_RESOLUTION => {
                    self.light_meter_resolution = value.to_string()
                }
                id::capabilities::LIGHT_METER_UNITS => self.light_meter_units = value.to_string(),
                id::capabilities::TEMPERATURE_SENSOR_RANGE => {
                    self.temperature_sensor_range = value.to_string()
                }
                id::capabilities::TEMPERATURE_SENSOR_RESOLUTION => {
                    self.temperature_sensor_resolution = value.to_string()
                }
                id::capabilities::TEMPERATURE_SENSOR_UNITS => {
                    self.temperature_sensor_units = value.to_string()
                }
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct ConfigurationInfo {
        pub measurement_frequency: String,
        pub measurement_period: String,
        pub start_time: DateTime<Utc>,
        pub time_zone: String,
    }

    impl ConfigurationInfo {
        pub fn new() -> ConfigurationInfo {
            ConfigurationInfo {
                measurement_frequency: String::new(),
                measurement_period: String::new(),
                start_time: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                time_zone: String::new(),
            }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::configuration::MEASUREMENT_FREQUENCY => {
                    self.measurement_frequency = value.to_string()
                }
                id::configuration::MEASUREMENT_PERIOD => {
                    self.measurement_period = value.to_string()
                }
                id::configuration::START_TIME => {
                    self.start_time = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S:%3f")
                        .unwrap()
                        .to_utc();
                }
                id::configuration::TIME_ZONE => self.time_zone = value.to_string(),
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct TrialInfo {
        pub study_centre: String,
        pub study_code: String,
        pub investigator_id: String,
        pub exercise_type: String,
        pub config_operator_id: String,
        pub config_time: DateTime<Utc>,
        pub config_notes: String,
        pub extract_operator_id: String,
        pub extract_time: DateTime<Utc>,
        pub extract_notes: String,
    }

    impl TrialInfo {
        pub fn new() -> TrialInfo {
            TrialInfo {
                study_centre: String::new(),
                study_code: String::new(),
                investigator_id: String::new(),
                exercise_type: String::new(),
                config_operator_id: String::new(),
                config_time: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                config_notes: String::new(),
                extract_operator_id: String::new(),
                extract_time: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                extract_notes: String::new(),
            }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::trial::STUDY_CENTRE => self.study_centre = value.to_string(),
                id::trial::STUDY_CODE => self.study_code = value.to_string(),
                id::trial::INVESTIGATOR_ID => self.investigator_id = value.to_string(),
                id::trial::EXERCISE_TYPE => self.exercise_type = value.to_string(),
                id::trial::CONFIG_OPERATOR_ID => self.config_operator_id = value.to_string(),
                id::trial::CONFIG_TIME => {
                    self.config_time = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S:%3f")
                        .unwrap()
                        .to_utc();
                }
                id::trial::CONFIG_NOTES => self.config_notes = value.to_string(),
                id::trial::EXTRACT_OPERATOR_ID => self.extract_operator_id = value.to_string(),
                id::trial::EXTRACT_TIME => {
                    self.extract_time = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S:%3f")
                        .unwrap()
                        .to_utc();
                }
                id::trial::EXTRACT_NOTES => self.extract_notes = value.to_string(),
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct SubjectInfo {
        pub location_code: String,
        pub code: String,
        pub date_of_birth: NaiveDate,
        pub sex: String,
        pub height: String,
        pub weight: String,
        pub handedness_code: String,
        pub notes: String,
    }

    impl SubjectInfo {
        pub fn new() -> SubjectInfo {
            SubjectInfo {
                location_code: String::new(),
                code: String::new(),
                date_of_birth: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
                sex: String::new(),
                height: String::new(),
                weight: String::new(),
                handedness_code: String::new(),
                notes: String::new(),
            }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::subject::LOCATION_CODE => self.location_code = value.to_string(),
                id::subject::CODE => self.code = value.to_string(),
                id::subject::DATE_OF_BIRTH => {
                    self.date_of_birth = NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap();
                }
                id::subject::SEX => self.sex = value.to_string(),
                id::subject::HEIGHT => self.height = value.to_string(),
                id::subject::WEIGHT => self.weight = value.to_string(),
                id::subject::HANDEDNESS_CODE => self.handedness_code = value.to_string(),
                id::subject::NOTES => self.notes = value.to_string(),
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
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

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::calibration::X_GAIN => self.x_gain = value.parse::<i32>().unwrap(),
                id::calibration::X_OFFSET => self.x_offset = value.parse::<i32>().unwrap(),
                id::calibration::Y_GAIN => self.y_gain = value.parse::<i32>().unwrap(),
                id::calibration::Y_OFFSET => self.y_offset = value.parse::<i32>().unwrap(),
                id::calibration::Z_GAIN => self.z_gain = value.parse::<i32>().unwrap(),
                id::calibration::Z_OFFSET => self.z_offset = value.parse::<i32>().unwrap(),
                id::calibration::VOLTS => self.volts = value.parse::<i32>().unwrap(),
                id::calibration::LUX => self.lux = value.parse::<i32>().unwrap(),
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct MemoryStatus {
        pub number_of_pages: usize,
    }

    impl MemoryStatus {
        pub fn new() -> MemoryStatus {
            MemoryStatus { number_of_pages: 0 }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::memory::PAGES => self.number_of_pages = value.parse::<usize>().unwrap(),
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct MainHeader {
        pub identity: DeviceIdentity,
        pub capabilities: DeviceCapabilities,
        pub configuration: ConfigurationInfo,
        pub trial: TrialInfo,
        pub subject: SubjectInfo,
        pub calibration: CalibrationData,
        pub memory: MemoryStatus,
    }

    impl MainHeader {
        pub fn new() -> MainHeader {
            MainHeader {
                identity: DeviceIdentity::new(),
                capabilities: DeviceCapabilities::new(),
                configuration: ConfigurationInfo::new(),
                trial: TrialInfo::new(),
                subject: SubjectInfo::new(),
                calibration: CalibrationData::new(),
                memory: MemoryStatus::new(),
            }
        }

        pub fn set_field(&mut self, category: &str, field: &str, value: &str) {
            match category {
                id::identity::HEADER => self.identity.set_field(field, value),
                id::capabilities::HEADER => self.capabilities.set_field(field, value),
                id::configuration::HEADER => self.configuration.set_field(field, value),
                id::trial::HEADER => self.trial.set_field(field, value),
                id::subject::HEADER => self.subject.set_field(field, value),
                id::calibration::HEADER => self.calibration.set_field(field, value),
                id::memory::HEADER => self.memory.set_field(field, value),
                _ => {}
            }
        }
    }

    #[derive(Debug, Iterable)]
    pub struct RecordedData {
        pub device_unique_serial_code: String,
        pub sequence_number: i32,
        pub page_time: DateTime<Utc>,
        pub unassigned: String,
        pub temperature: f32,
        pub battery_voltage: f32,
        pub device_status: String,
        pub measurement_frequency: f32,
    }

    impl RecordedData {
        pub fn new() -> RecordedData {
            RecordedData {
                device_unique_serial_code: String::new(),
                sequence_number: 0,
                page_time: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                unassigned: String::new(),
                temperature: 0.0,
                battery_voltage: 0.0,
                device_status: String::new(),
                measurement_frequency: 30.0,
            }
        }

        pub fn set_field(&mut self, field: &str, value: &str) {
            match field {
                id::record::SERIAL => self.device_unique_serial_code = value.to_string(),
                id::record::SEQUENCE => self.sequence_number = value.parse::<i32>().unwrap(),
                id::record::PAGE_TIME => {
                    self.page_time = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S:%3f")
                        .unwrap()
                        .to_utc();
                }
                id::record::UNASSIGNED => self.unassigned = value.to_string(),
                id::record::TEMPERATURE => self.temperature = value.parse::<f32>().unwrap(),
                id::record::BATTERY_VOLTAGE => self.battery_voltage = value.parse::<f32>().unwrap(),
                id::record::DEVICE_STATUS => self.device_status = value.to_string(),
                id::record::MEASUREMENT_FREQUENCY => {
                    self.measurement_frequency = value.parse::<f32>().unwrap()
                }
                _ => {}
            }
        }
    }
}
