use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read},
};

// String constants
// "Page Time:"
// "Measurement Frequency:"
const ID_PAGE_TIME: &str = "Page Time:";
const ID_PAGE_MEASUREMENT_FREQUENCY: &str = "Measurement Frequency:";

const ID_CALIBRATION_X_GAIN: &str = "x gain:";
const ID_CALIBRATION_X_OFFSET: &str = "x offset:";
const ID_CALIBRATION_Y_GAIN: &str = "y gain:";
const ID_CALIBRATION_Y_OFFSET: &str = "y offset:";
const ID_CALIBRATION_Z_GAIN: &str = "z gain:";
const ID_CALIBRATION_Z_OFFSET: &str = "z offset:";
const ID_CALIBRATION_VOLTS: &str = "Volts:";
const ID_CALIBRATION_LUX: &str = "Lux:";

pub struct AccelerometerData;

pub fn load_data(path: String) -> AccelerometerData {
    let file = fs::File::open(path).unwrap();
    let buf_reader = BufReader::new(file);

    let mut line_counter: usize = 0;
    let mut sample_rate: f32 = 0.0;
    let mut date: chrono::NaiveDateTime = chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap();
    
    let mut x_gain: i32 = 0;
    let mut x_offset: i32 = 0;
    let mut y_gain: i32 = 0;
    let mut y_offset: i32 = 0;
    let mut z_gain: i32 = 0;
    let mut z_offset: i32 = 0;
    let mut volts: i32 = 1;
    let mut lux: i32 = 0;

    for line in buf_reader.lines() {
        if line_counter >= 100 {
            break;
        }
        let line = line.unwrap();


        // skip header for now
        if line_counter < 59 {
            if line.starts_with(ID_CALIBRATION_X_GAIN) {
                x_gain = line[ID_CALIBRATION_X_GAIN.len()..].parse::<i32>().unwrap();
            } else if line.starts_with(ID_CALIBRATION_X_OFFSET) {
                x_offset = line[ID_CALIBRATION_X_OFFSET.len()..]
                    .parse::<i32>()
                    .unwrap();
            } else if line.starts_with(ID_CALIBRATION_Y_GAIN) {
                y_gain = line[ID_CALIBRATION_Y_GAIN.len()..].parse::<i32>().unwrap();
            } else if line.starts_with(ID_CALIBRATION_Y_OFFSET) {
                y_offset = line[ID_CALIBRATION_Y_OFFSET.len()..]
                    .parse::<i32>()
                    .unwrap();
            } else if line.starts_with(ID_CALIBRATION_Z_GAIN) {
                z_gain = line[ID_CALIBRATION_Z_GAIN.len()..].parse::<i32>().unwrap();
            } else if line.starts_with(ID_CALIBRATION_Z_OFFSET) {
                z_offset = line[ID_CALIBRATION_Z_OFFSET.len()..]
                    .parse::<i32>()
                    .unwrap();
            } else if line.starts_with(ID_CALIBRATION_VOLTS) {
                volts = line[ID_CALIBRATION_VOLTS.len()..].parse::<i32>().unwrap();
            } else if line.starts_with(ID_CALIBRATION_LUX) {
                lux = line[ID_CALIBRATION_LUX.len()..].parse::<i32>().unwrap();
            }
            println!("{} --- \"{}\"", line_counter, line);

            line_counter += 1;
            continue;
        }
        if line_counter == 59 {
            println!("x: gain: {}, offset: {}", x_gain, x_offset);
            println!("y: gain: {}, offset: {}", y_gain, y_offset);
            println!("z: gain: {}, offset: {}", z_gain, z_offset);
            println!("volts: {}", volts);
            println!("lux: {}", lux);
        }

        //let page_index = (line_counter - 59) / 10;
        let page_offset = (line_counter - 59) % 10;

        //println!("{} {}-{} --- \"{}\"", line_counter, page_index, page_offset, line);

        if page_offset == 9 {
            let buf = line.as_bytes();
            let mut bitreader = bitreader::BitReader::new(buf);

            for i in 0..buf.len()/6 {

                let x = bitreader.read_i16(12).unwrap();
                let y = bitreader.read_i16(12).unwrap();
                let z = bitreader.read_i16(12).unwrap();
                let light = bitreader.read_i16(10).unwrap();
                let button_state = bitreader.read_bool().unwrap();
                bitreader.skip(1).unwrap();

                let x = (x as f32 - x_offset as f32) / x_gain as f32;
                let y = (y as f32 - y_offset as f32) / y_gain as f32;
                let z = (z as f32 - z_offset as f32) / z_gain as f32;
                let light = light as f32 * lux as f32 / volts as f32;

                let sample_time = date + chrono::Duration::nanoseconds((1_000_000_000.0 / sample_rate) as i64 * i as i64);

                println!(
                    "time: {} x: {}, y: {}, z: {}, light: {}, button: {}",
                    sample_time, x, y, z, light, button_state
                );
            }
        } else {
            if line.starts_with(ID_PAGE_TIME) {
                // split everything after 10th char
                let date_str = &line[ID_PAGE_TIME.len()..];
                // parse date (e.g. 2024-01-11 15:35:49:000)
                date = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S:%f")
                    .unwrap();
            } else if line.starts_with(ID_PAGE_MEASUREMENT_FREQUENCY) {
                let sample_rate_str = &line[ID_PAGE_MEASUREMENT_FREQUENCY.len()..];
                sample_rate = sample_rate_str.parse::<f32>().unwrap();
            }
        }

        line_counter += 1;
    }

    AccelerometerData {}
}
