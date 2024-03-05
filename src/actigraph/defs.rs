#![allow(dead_code)]

pub const GT3X_FILE_INFO: &str = "info.txt";
pub const GT3X_FILE_LOG: &str = "log.bin";

#[derive(Debug)]
pub enum LogRecordType {
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
    pub fn from_u8(val: u8) -> LogRecordType {
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
pub enum ParameterType {
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
    pub fn from_u16(address_space: u16, identifier: u16) -> ParameterType {
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
