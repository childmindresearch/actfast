use crate::error::Result;

pub struct MetadataEntry<'a> {
    pub category: &'a str,
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorKind {
    Accelerometer,
    Light,
    ButtonState,
    Capacitive,
    Temperature,
    BatteryVoltage,
}

impl SensorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SensorKind::Accelerometer => "acceleration",
            SensorKind::Light => "light",
            SensorKind::ButtonState => "button_state",
            SensorKind::Capacitive => "capsense",
            SensorKind::Temperature => "temperature",
            SensorKind::BatteryVoltage => "battery_voltage",
        }
    }
}

#[allow(dead_code)]
pub enum SensorDataDyn<'a> {
    F32(&'a [f32]),
    F64(&'a [f64]),
    U8(&'a [u8]),
    U16(&'a [u16]),
    U32(&'a [u32]),
    U64(&'a [u64]),
    I8(&'a [i8]),
    I16(&'a [i16]),
    I32(&'a [i32]),
    I64(&'a [i64]),
    Bool(&'a [bool]),
}

pub struct SensorData<'a> {
    pub kind: SensorKind,
    pub data: SensorDataDyn<'a>,
}

pub struct SensorTable<'a> {
    pub name: &'a str,
    pub datetime: &'a [i64],
    pub data: Vec<SensorData<'a>>,
}

pub trait SensorsFormatReader<'a> {
    fn read<R: std::io::Read + std::io::Seek, M, S>(
        &'a mut self,
        reader: R,
        metadata_callback: M,
        sensor_table_callback: S,
    ) -> Result<()>
    where
        M: FnMut(MetadataEntry),
        S: FnMut(SensorTable<'a>);
}
