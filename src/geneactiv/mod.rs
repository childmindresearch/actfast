mod defs;

use crate::geneactiv::defs::*;
use crate::geneactiv::defs::data::*;

use std::{
    fs,
    io::{BufRead, BufReader},
};

pub struct AccelerometerData {
    pub header: MainHeader,

    pub a_time: Vec<i64>,
    pub a3_acceleration: Vec<f32>,
    pub a_light: Vec<f32>,
    pub a_button_state: Vec<bool>,

    pub b_time: Vec<i64>,
    pub b_temperature: Vec<f32>,
    pub b_battery_voltage: Vec<f32>,
}

#[allow(dead_code)]
impl AccelerometerData {
    pub fn new(header: MainHeader) -> AccelerometerData {
        AccelerometerData {
            header,

            a_time: Vec::new(),
            a3_acceleration: Vec::new(),
            a_light: Vec::new(),
            a_button_state: Vec::new(),

            b_time: Vec::new(),
            b_temperature: Vec::new(),
            b_battery_voltage: Vec::new(),
        }
    }

    pub fn reserve(&mut self, num_records: usize, measurements_per_record: usize) {
        let num_measurements = num_records * measurements_per_record;
        self.a_time.reserve(num_measurements);
        self.a3_acceleration.reserve(num_measurements * 3);
        self.a_light.reserve(num_measurements);
        self.a_button_state.reserve(num_measurements);

        self.b_time.reserve(num_records);
        self.b_temperature.reserve(num_records);
        self.b_battery_voltage.reserve(num_records);
    }
}

pub struct SampleDataUncalibrated {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub light: i16,
    pub button_state: bool,
}

impl SampleDataUncalibrated {
    pub fn read(bitreader: &mut bitreader::BitReader) -> SampleDataUncalibrated {
        let x = bitreader.read_i16(12).unwrap();
        let y = bitreader.read_i16(12).unwrap();
        let z = bitreader.read_i16(12).unwrap();
        let light = bitreader.read_i16(10).unwrap();
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
            x: (self.x as f32 - cal.x_offset as f32) / cal.x_gain as f32,
            y: (self.y as f32 - cal.y_offset as f32) / cal.y_gain as f32,
            z: (self.z as f32 - cal.z_offset as f32) / cal.z_gain as f32,
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


fn read_prefixed<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.starts_with(prefix) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

fn parse_value<'a, T>(s: &str, prefix: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    read_prefixed(s, prefix).and_then(|v| v.trim().parse::<T>().ok())
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

pub fn load_data(path: String) -> AccelerometerData {
    let file = fs::File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);

    let mut data = AccelerometerData::new(MainHeader::new());
    let mut data_reserved = false;

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
        data.header.set_field(&last_category, &line[..colon.unwrap()+1], &line[colon.unwrap() + 1..]);
    }


    let mut lines_record = vec![String::new(); 10];
    let mut record_header = RecordedData::new();

    while Some(()) == read_n_lines(&mut buf_reader, &mut lines_record) {
        if !data_reserved {
            data.reserve(data.header.memory.number_of_pages, lines_record[9].as_bytes().len() / 6);
            data_reserved = true;
        }

        // read record header

        for i in 0..9 {
            let line = lines_record[i].trim();
            if let Some(measurement_frequency) = parse_value(line, id::record::MEASUREMENT_FREQUENCY) {
                record_header.measurement_frequency = measurement_frequency;
            } else if let Some(page_time_str) = parse_value(line, id::record::PAGE_TIME) {
                let _: String = page_time_str;
                record_header.page_time = defs::parse_date_time(&page_time_str);
            } else if let Some(temperature) = parse_value(line, id::record::TEMPERATURE) {
                record_header.temperature = temperature;
            } else if let Some(battery_voltage) = parse_value(line, id::record::BATTERY_VOLTAGE) {
                record_header.battery_voltage = battery_voltage;
            }
        }

        data.b_time.push(record_header.page_time.timestamp_nanos_opt().unwrap());
        data.b_temperature.push(record_header.temperature);
        data.b_battery_voltage.push(record_header.battery_voltage);

        // read record data 

        let buf = lines_record[9].as_bytes();
        let mut bitreader = bitreader::BitReader::new(buf);

        for i in 0..buf.len() / 6 {
            let sample = SampleDataUncalibrated::read(&mut bitreader).calibrate(&data.header.calibration);

            let sample_time = record_header.page_time
                + chrono::Duration::nanoseconds((1_000_000_000.0 / record_header.measurement_frequency) as i64 * i as i64);

            data.a_time.push(sample_time.timestamp_nanos_opt().unwrap());
            data.a3_acceleration.push(sample.x);
            data.a3_acceleration.push(sample.y);
            data.a3_acceleration.push(sample.z);
            data.a_light.push(sample.light);
            data.a_button_state.push(sample.button_state);
        }
    }

    data
}
