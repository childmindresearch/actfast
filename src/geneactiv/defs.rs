use crate::error::{ActfastError, FileLocation};

const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S:%3f";

#[allow(dead_code)]
pub mod id {
    pub mod identity {
        pub const HEADER: &str = "Device Identity";
        pub const SERIAL: &str = "Device Unique Serial Code";
        pub const TYPE: &str = "Device Type";
        pub const MODEL: &str = "Device Model";
        pub const FIRMWARE: &str = "Device Firmware Version";
        pub const CALIBRATION_DATE: &str = "Calibration Date";
    }

    pub mod capabilities {
        pub const HEADER: &str = "Device Capabilities";
        pub const ACCELEROMETER_RANGE: &str = "Accelerometer Range";
        pub const ACCELEROMETER_RESOLUTION: &str = "Accelerometer Resolution";
        pub const ACCELEROMETER_UNITS: &str = "Accelerometer Units";
        pub const LIGHT_METER_RANGE: &str = "Light Meter Range";
        pub const LIGHT_METER_RESOLUTION: &str = "Light Meter Resolution";
        pub const LIGHT_METER_UNITS: &str = "Light Meter Units";
        pub const TEMPERATURE_SENSOR_RANGE: &str = "Temperature Sensor Range";
        pub const TEMPERATURE_SENSOR_RESOLUTION: &str = "Temperature Sensor Resolution";
        pub const TEMPERATURE_SENSOR_UNITS: &str = "Temperature Sensor Units";
    }

    pub mod configuration {
        pub const HEADER: &str = "Configuration Info";
        pub const MEASUREMENT_FREQUENCY: &str = "Measurement Frequency";
        pub const MEASUREMENT_PERIOD: &str = "Measurement Period";
        pub const START_TIME: &str = "Start Time";
        pub const TIME_ZONE: &str = "Time Zone";
    }

    pub mod trial {
        pub const HEADER: &str = "Trial Info";
        pub const STUDY_CENTRE: &str = "Study Centre";
        pub const STUDY_CODE: &str = "Study Code";
        pub const INVESTIGATOR_ID: &str = "Investigator ID";
        pub const EXERCISE_TYPE: &str = "Exercise Type";
        pub const CONFIG_OPERATOR_ID: &str = "Config Operator ID";
        pub const CONFIG_TIME: &str = "Config Time";
        pub const CONFIG_NOTES: &str = "Config Notes";
        pub const EXTRACT_OPERATOR_ID: &str = "Extract Operator ID";
        pub const EXTRACT_TIME: &str = "Extract Time";
        pub const EXTRACT_NOTES: &str = "Extract Notes";
    }

    pub mod subject {
        pub const HEADER: &str = "Subject Info";
        pub const LOCATION_CODE: &str = "Device Location Code";
        pub const CODE: &str = "Subject Code";
        pub const DATE_OF_BIRTH: &str = "Date of Birth";
        pub const SEX: &str = "Sex";
        pub const HEIGHT: &str = "Height";
        pub const WEIGHT: &str = "Weight";
        pub const HANDEDNESS_CODE: &str = "Handedness Code";
        pub const NOTES: &str = "Subject Notes";
    }

    pub mod calibration {
        pub const HEADER: &str = "Calibration Data";
        pub const X_GAIN: &str = "x gain";
        pub const X_OFFSET: &str = "x offset";
        pub const Y_GAIN: &str = "y gain";
        pub const Y_OFFSET: &str = "y offset";
        pub const Z_GAIN: &str = "z gain";
        pub const Z_OFFSET: &str = "z offset";
        pub const VOLTS: &str = "Volts";
        pub const LUX: &str = "Lux";
    }

    pub mod memory {
        pub const HEADER: &str = "Memory Status";
        pub const PAGES: &str = "Number of Pages";
    }

    pub mod record {
        pub const HEADER: &str = "Recorded Data";
        pub const SERIAL: &str = "Device Unique Serial Code";
        pub const SEQUENCE: &str = "Sequence Number";
        pub const PAGE_TIME: &str = "Page Time";
        pub const UNASSIGNED: &str = "Unassigned";
        pub const TEMPERATURE: &str = "Temperature";
        pub const BATTERY_VOLTAGE: &str = "Battery voltage";
        pub const DEVICE_STATUS: &str = "Device Status";
        pub const MEASUREMENT_FREQUENCY: &str = "Measurement Frequency";
    }
}

pub fn parse_date_time(
    date_time: &str,
    location: FileLocation,
) -> Result<chrono::DateTime<chrono::Utc>, ActfastError> {
    chrono::NaiveDateTime::parse_from_str(date_time, DATETIME_FORMAT)
        .map(|dt| dt.and_utc())
        .map_err(|_| ActfastError::InvalidDateTime {
            value: date_time.to_string(),
            format: DATETIME_FORMAT,
            location,
        })
}
