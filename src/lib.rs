mod actigraph;
mod geneactiv;

use chrono::{NaiveDateTime, NaiveDate};
use numpy::{PyArray1, prelude::*};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use struct_iterable::Iterable;

fn serde_json_to_python(py: Python, json: serde_json::Value) -> PyResult<PyObject> {
    match json {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok(b.to_object(py)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_object(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.to_object(py))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Invalid number type"))
            }
        }
        serde_json::Value::String(s) => Ok(s.to_object(py)),
        serde_json::Value::Array(a) => {
            let list = PyList::empty_bound(py);
            for item in a {
                list.append(serde_json_to_python(py, item)?)?;
            }
            Ok(list.to_object(py))
        }
        serde_json::Value::Object(o) => {
            let dict = PyDict::new_bound(py);
            for (key, value) in o {
                dict.set_item(key, serde_json_to_python(py, value)?)?;
            }
            Ok(dict.to_object(py))
        }
    }
}

#[pyfunction]
fn read_actigraph_gt3x(_py: Python, path: &str) -> PyResult<PyObject> {
    let data = actigraph::load_data(path.to_string());

    let np_datetime = PyArray1::from_slice_bound(_py, &data.acceleration_time).to_owned();
    let np_acceleration = PyArray1::from_slice_bound(_py, &data.acceleration).reshape([data.acceleration.len() as usize / 3, 3]).unwrap();
    let np_lux_time = PyArray1::from_slice_bound(_py, &data.lux_time).to_owned();
    let np_lux = PyArray1::from_slice_bound(_py, &data.lux).to_owned();
    let np_capsense_time = PyArray1::from_slice_bound(_py, &data.capsense_time).to_owned();
    let np_capsense = PyArray1::from_slice_bound(_py, &data.capsense).to_owned();
    let np_battery_voltage_time = PyArray1::from_slice_bound(_py, &data.battery_voltage_time).to_owned();
    let np_battery_voltage = PyArray1::from_slice_bound(_py, &data.battery_voltage).to_owned();

    let dict = PyDict::new_bound(_py);

    let dict_timeseries = PyDict::new_bound(_py);

    let dict_acceleration = PyDict::new_bound(_py);
    
    dict_acceleration.set_item("datetime", np_datetime)?;
    dict_acceleration.set_item("acceleration", np_acceleration)?;

    dict_timeseries.set_item("acceleration", dict_acceleration)?;

    let dict_lux = PyDict::new_bound(_py);
    
    dict_lux.set_item("datetime", np_lux_time)?;
    dict_lux.set_item("lux", np_lux)?;

    dict_timeseries.set_item("lux", dict_lux)?;

    let dict_capsense = PyDict::new_bound(_py);

    dict_capsense.set_item("datetime", np_capsense_time)?;
    dict_capsense.set_item("capsense", np_capsense)?;

    dict_timeseries.set_item("capsense", dict_capsense)?;

    let dict_battery_voltage = PyDict::new_bound(_py);

    dict_battery_voltage.set_item("datetime", np_battery_voltage_time)?;
    dict_battery_voltage.set_item("battery_voltage", np_battery_voltage)?;

    dict_timeseries.set_item("battery_voltage", dict_battery_voltage)?;

    dict.set_item("timeseries", dict_timeseries)?;

    // metadata dict
    let dict_metadata = PyDict::new_bound(_py);

    let metadata_list = PyList::empty_bound(_py);
    
    for entry in data.metadata_json.iter() {
        let entry_json = serde_json::from_str::<serde_json::Value>(entry).unwrap();
        let entry_py = serde_json_to_python(_py, entry_json)?;
        metadata_list.append(entry_py)?;
    }

    dict_metadata.set_item("metadata", metadata_list)?;

    let dict_metadata_info = PyDict::new_bound(_py);

    for (key, value) in data.metadata.iter() {
        dict_metadata_info.set_item(key, value)?;
    }

    dict_metadata.set_item("info", dict_metadata_info)?;

    let dict_metdata_features = PyDict::new_bound(_py);

    dict_metdata_features.set_item("heart_rate_monitor", data.device_features.heart_rate_monitor)?;
    dict_metdata_features.set_item("data_summary", data.device_features.data_summary)?;
    dict_metdata_features.set_item("sleep_mode", data.device_features.sleep_mode)?;
    dict_metdata_features.set_item("proximity_tagging", data.device_features.proximity_tagging)?;
    dict_metdata_features.set_item("epoch_data", data.device_features.epoch_data)?;
    dict_metdata_features.set_item("no_raw_data", data.device_features.no_raw_data)?;

    dict_metadata.set_item("features", dict_metdata_features)?;

    dict.set_item("metadata", dict_metadata)?;

    Ok(dict.into())
}

fn dict_set_any(dict: &pyo3::Bound<'_, PyDict, >, key: &str, value: &dyn std::any::Any) -> PyResult<()> {
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
    let dict = PyDict::new_bound(_py);

    let a_np_datetime = PyArray1::from_slice_bound(_py, &data.a_time).to_owned();
    let a_np_acceleration = PyArray1::from_slice_bound(_py, &data.a3_acceleration)
        .reshape([data.a3_acceleration.len() as usize / 3, 3])
        .unwrap();
    let a_np_light = PyArray1::from_slice_bound(_py, &data.a_light).to_owned();
    let a_np_button_state = PyArray1::from_slice_bound(_py, &data.a_button_state).to_owned();
    
    let b_np_datetime = PyArray1::from_slice_bound(_py, &data.b_time).to_owned();
    let b_np_temperature = PyArray1::from_slice_bound(_py, &data.b_temperature).to_owned();
    let b_np_battery_voltage = PyArray1::from_slice_bound(_py, &data.b_battery_voltage).to_owned();

    let dict_timeseries = PyDict::new_bound(_py);

    let dict_hf = PyDict::new_bound(_py);
    
    dict_hf.set_item("datetime", a_np_datetime)?;
    dict_hf.set_item("acceleration", a_np_acceleration)?;
    dict_hf.set_item("light", a_np_light)?;
    dict_hf.set_item("button_state", a_np_button_state)?;

    dict_timeseries.set_item("hf", dict_hf)?;

    let dict_lf = PyDict::new_bound(_py);
    

    dict_lf.set_item("datetime", b_np_datetime)?;
    dict_lf.set_item("temperature", b_np_temperature)?;
    dict_lf.set_item("battery_voltage", b_np_battery_voltage)?;

    dict_timeseries.set_item("lf", dict_lf)?;

    dict.set_item("timeseries", dict_timeseries)?;

    let dict_metadata = PyDict::new_bound(_py);
    

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

    dict.set_item("metadata", dict_metadata)?;

    Ok(dict.into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn actfast(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_actigraph_gt3x, m)?)?;
    m.add_function(wrap_pyfunction!(read_geneactiv_bin, m)?)?;
    Ok(())
}
