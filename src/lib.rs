mod actigraph;
mod geneactiv;

use chrono::{NaiveDateTime, Timelike, NaiveDate};
use numpy::PyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use struct_iterable::Iterable;

#[pyfunction]
fn read_actigraph_gt3x(_py: Python, path: &str) -> PyResult<PyObject> {
    // Attempt to open the file
    /*let file = File::open(path).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;

    // Read the contents of the file into a vector
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;*/

    let data = actigraph::load_data(path.to_string());

    // Convert data to 3*n NumPy array
    let data_arr = PyArray1::from_slice(_py, &data.acceleration)
        .reshape([data.acceleration.len() as usize / 3, 3])
        .unwrap();

    // datetime array
    let datetime_arr = PyArray1::from_slice(_py, &data.time).to_owned();

    let lux_arr = PyArray1::from_slice(_py, &data.lux);

    let dict = PyDict::new(_py);
    dict.set_item("datetime", datetime_arr)?;
    dict.set_item("data", data_arr)?;
    dict.set_item("lux", lux_arr)?;
    // metadata dict
    let metadata_dict = PyDict::new(_py);
    for (key, value) in data.metadata.iter() {
        metadata_dict.set_item(key, value)?;
    }
    dict.set_item("metadata", metadata_dict)?;

    Ok(dict.into())
}

fn dict_set_any(dict: &PyDict, key: &str, value: &dyn std::any::Any) -> PyResult<()> {
    if let Some(v) = value.downcast_ref::<String>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<i8>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<i16>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<i32>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<i64>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<u8>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<u16>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<u32>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<u64>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<usize>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<f32>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<f64>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<bool>() {
        dict.set_item(key, v)?;
    } else if let Some(v) = value.downcast_ref::<NaiveDateTime>() {
        dict.set_item(key, format!("{}", v))?;
    } else if let Some(v) = value.downcast_ref::<NaiveDate>() {
        dict.set_item(key, format!("{}", v))?;
    } else {
        dict.set_item(key, format!("{:?}", value))?;
    }
    Ok(())
}

#[pyfunction]
fn read_geneactiv_bin(_py: Python, path: &str) -> PyResult<PyObject> {
    let data = geneactiv::load_data(path.to_string());
    let dict = PyDict::new(_py);

    let a_np_datetime = PyArray1::from_slice(_py, &data.a_time).to_owned();
    let a_np_acceleration = PyArray1::from_slice(_py, &data.a3_acceleration)
        .reshape([data.a3_acceleration.len() as usize / 3, 3])
        .unwrap();
    let a_np_light = PyArray1::from_slice(_py, &data.a_light).to_owned();
    let a_np_button_state = PyArray1::from_slice(_py, &data.a_button_state).to_owned();
    
    let b_np_datetime = PyArray1::from_slice(_py, &data.b_time).to_owned();
    let b_np_temperature = PyArray1::from_slice(_py, &data.b_temperature).to_owned();
    let b_np_battery_voltage = PyArray1::from_slice(_py, &data.b_battery_voltage).to_owned();

    let dict_timeseries = PyDict::new(_py);
    dict.set_item("timeseries", dict_timeseries)?;

    let dict_hf = PyDict::new(_py);
    dict_timeseries.set_item("hf", dict_hf)?;
    
    dict_hf.set_item("datetime", a_np_datetime)?;
    dict_hf.set_item("acceleration", a_np_acceleration)?;
    dict_hf.set_item("light", a_np_light)?;
    dict_hf.set_item("button_state", a_np_button_state)?;

    let dict_lf = PyDict::new(_py);
    dict_timeseries.set_item("lf", dict_lf)?;

    dict_lf.set_item("datetime", b_np_datetime)?;
    dict_lf.set_item("temperature", b_np_temperature)?;
    dict_lf.set_item("battery_voltage", b_np_battery_voltage)?;

    let dict_metadata = PyDict::new(_py);
    dict.set_item("metadata", dict_metadata)?;

    for (key, value) in data.header.identity.iter() {
        let key_prefixed = format!("identity_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    for (key, value) in data.header.capabilities.iter() {
        let key_prefixed = format!("capabilities_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    for (key, value) in data.header.configuration.iter() {
        let key_prefixed = format!("configuration_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    for (key, value) in data.header.trial.iter() {
        let key_prefixed = format!("trial_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    for (key, value) in data.header.subject.iter() {
        let key_prefixed = format!("subject_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    for (key, value) in data.header.calibration.iter() {
        let key_prefixed = format!("calibration_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    for (key, value) in data.header.memory.iter() {
        let key_prefixed = format!("memory_{}", key);
        dict_set_any(&dict_metadata, &key_prefixed, value)?;
    }

    Ok(dict.into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn actfast(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_actigraph_gt3x, m)?)?;
    m.add_function(wrap_pyfunction!(read_geneactiv_bin, m)?)?;
    Ok(())
}
