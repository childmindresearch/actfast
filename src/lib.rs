mod actigraph;
//mod axivity;
mod geneactiv;
mod sensors;

use std::io::Read;

use numpy::{prelude::*, PyArray1};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use sensors::SensorsFormatReader;

enum FileFormat {
    ActigraphGt3x,
    GeneactivBin,
    AxivityCwa,
}

fn guess_file_format(path: &str) -> std::io::Result<Option<FileFormat>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut magic = [0; 4];
    reader.read_exact(&mut magic)?;

    Ok(
        if magic[0] == 0x50 && magic[1] == 0x4b && magic[2] == 0x03 && magic[3] == 0x04 {
            // this is the general zip magic number
            // if we add another file format that uses zip, we need to check the contents
            Some(FileFormat::ActigraphGt3x)
        } else if magic[0] == 0x44 && magic[1] == 0x65 && magic[2] == 0x76 && magic[3] == 0x69 {
            Some(FileFormat::GeneactivBin)
        } else if magic[0] == 0x4d && magic[1] == 0x44 {
            Some(FileFormat::AxivityCwa)
        } else {
            None
        },
    )
}

fn sensor_data_dyn_to_pyarray<'py, T>(
    py: Python<'py>,
    data: &[T],
    reference_len: usize,
) -> PyResult<pyo3::Bound<'py, PyAny>>
where
    T: numpy::Element,
{
    if reference_len == 0 {
        return Ok(PyArray1::from_slice_bound(py, data).as_any().to_owned());
    }
    let multi_sensor = data.len() / reference_len;
    Ok(if multi_sensor == 1 {
        PyArray1::from_slice_bound(py, data).as_any().to_owned()
    } else {
        PyArray1::from_slice_bound(py, data)
            .reshape([reference_len, multi_sensor])?
            .as_any()
            .to_owned()
    })
}

#[pyfunction]
fn read(_py: Python, path: &str) -> PyResult<PyObject> {
    let file_format = guess_file_format(path)?
        .ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Unknown file format",
        ))?;

    let dict = PyDict::new_bound(_py);
    let dict_metadata = PyDict::new_bound(_py);
    let dict_timeseries = PyDict::new_bound(_py);

    let metadata_callback = |metadata: sensors::MetadataEntry| {
        dict_metadata
            .get_item(metadata.category)
            .unwrap()
            .map_or_else(
                || {
                    let category_dict = PyDict::new_bound(_py);
                    category_dict
                        .set_item(metadata.key, metadata.value)
                        .unwrap();
                    dict_metadata
                        .set_item(metadata.category, category_dict)
                        .unwrap();
                },
                |category_dict| {
                    category_dict
                        .downcast::<PyDict>()
                        .unwrap()
                        .set_item(metadata.key, metadata.value)
                        .unwrap();
                },
            );
    };

    let sensor_table_callback = |sensor_table: sensors::SensorTable| {
        let dict_sensor_table = PyDict::new_bound(_py);
        let np_datetime = PyArray1::from_slice_bound(_py, sensor_table.datetime).to_owned();
        dict_sensor_table.set_item("datetime", np_datetime).unwrap();

        for sensor_data in sensor_table.data.iter() {
            let sensor_data_key = sensor_data.kind.to_str();
            let sensor_data_np = match sensor_data.data {
                sensors::SensorDataDyn::F32(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::F64(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::U8(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::U16(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::U32(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::U64(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::I8(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::I16(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::I32(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::I64(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
                sensors::SensorDataDyn::Bool(data) => {
                    sensor_data_dyn_to_pyarray(_py, data, sensor_table.datetime.len()).unwrap()
                }
            };
            // reshape if accelerometer
            dict_sensor_table
                .set_item(sensor_data_key, sensor_data_np)
                .unwrap();
        }
        dict_timeseries
            .set_item(sensor_table.name, dict_sensor_table)
            .unwrap();
    };

    let fname = std::path::Path::new(path);
    let file = std::fs::File::open(fname)?;

    match file_format {
        FileFormat::ActigraphGt3x => {
            actigraph::ActigraphReader::new()
                .read(file, metadata_callback, sensor_table_callback)
                .or(Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Failed to read file",
                )))?;
        }
        FileFormat::GeneactivBin => {
            geneactiv::GeneActivReader::new()
                .read(file, metadata_callback, sensor_table_callback)
                .or(Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Failed to read file",
                )))?;
        }
        FileFormat::AxivityCwa => {}
    };

    let format_str = match file_format {
        FileFormat::ActigraphGt3x => "Actigraph GT3X",
        FileFormat::GeneactivBin => "GeneActiv BIN",
        FileFormat::AxivityCwa => "Axivity CWA",
    };
    dict.set_item("format", format_str)?;

    dict.set_item("timeseries", dict_timeseries)?;
    dict.set_item("metadata", dict_metadata)?;

    Ok(dict.into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn actfast(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read, m)?)?;
    Ok(())
}
