mod actigraph;
mod error;
mod file_format;
mod geneactiv;
mod sensors;

use std::io::Read;

use numpy::{PyArray1, prelude::*};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use error::{ActfastError, IoResultExt};
use sensors::SensorsFormatReader;

/// Convert a slice to a numpy array, reshaping for multi-axis sensors
fn sensor_data_to_pyarray<'py, T>(
    py: Python<'py>,
    data: &[T],
    reference_len: usize,
) -> PyResult<Bound<'py, PyAny>>
where
    T: numpy::Element,
{
    let arr = PyArray1::from_slice(py, data);
    if reference_len == 0 || data.len() == reference_len {
        return Ok(arr.into_any());
    }

    let num_channels = data.len() / reference_len;
    Ok(arr.reshape([reference_len, num_channels])?.into_any())
}

/// Convert SensorDataDyn to numpy array using a macro to reduce repetition
macro_rules! sensor_data_dyn_to_pyarray {
    ($py:expr, $data:expr, $ref_len:expr) => {
        match $data {
            sensors::SensorDataDyn::F32(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::F64(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::U8(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::U16(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::U32(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::U64(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::I8(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::I16(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::I32(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::I64(d) => sensor_data_to_pyarray($py, d, $ref_len),
            sensors::SensorDataDyn::Bool(d) => sensor_data_to_pyarray($py, d, $ref_len),
        }
    };
}

#[pyfunction]
fn read(py: Python, path: std::path::PathBuf) -> PyResult<Py<PyAny>> {
    let file = std::fs::File::open(&path).with_context(format!("opening '{}'", path.display()))?;

    let mut reader = std::io::BufReader::new(file);
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .with_context("reading file header")?;

    let format_type = file_format::identify(&magic).ok_or(ActfastError::UnknownFormat { magic })?;

    let dict = PyDict::new(py);
    let dict_metadata = PyDict::new(py);
    let dict_timeseries = PyDict::new(py);

    let metadata_callback = |metadata: sensors::MetadataEntry| {
        let category_dict = dict_metadata
            .get_item(metadata.category)
            .ok()
            .flatten()
            .map(|item| item.cast::<PyDict>().unwrap().clone())
            .unwrap_or_else(|| {
                let d = PyDict::new(py);
                dict_metadata.set_item(metadata.category, &d).unwrap();
                d
            });
        category_dict
            .set_item(metadata.key, metadata.value)
            .unwrap();
    };

    let sensor_table_callback = |sensor_table: sensors::SensorTable| {
        let dict_sensor_table = PyDict::new(py);
        let np_datetime = PyArray1::from_slice(py, sensor_table.datetime);
        dict_sensor_table.set_item("datetime", np_datetime).unwrap();

        for sensor_data in sensor_table.data.iter() {
            let key = sensor_data.kind.as_str();
            let np_array =
                sensor_data_dyn_to_pyarray!(py, &sensor_data.data, sensor_table.datetime.len())
                    .unwrap();
            dict_sensor_table.set_item(key, np_array).unwrap();
        }
        dict_timeseries
            .set_item(sensor_table.name, dict_sensor_table)
            .unwrap();
    };

    // Re-open file for the actual reader (they need fresh file handle)
    let file =
        std::fs::File::open(&path).with_context(format!("reopening '{}'", path.display()))?;

    match format_type {
        file_format::FileFormat::ActigraphGt3x => {
            actigraph::ActigraphReader::new().read(
                file,
                metadata_callback,
                sensor_table_callback,
            )?;
        }
        file_format::FileFormat::GeneactivBin => {
            geneactiv::GeneActivReader::new().read(
                file,
                metadata_callback,
                sensor_table_callback,
            )?;
        }
        file_format::FileFormat::UnknownWav => {
            return Err(ActfastError::UnsupportedFormat {
                format: format_type,
                suggestion: "Use a general purpose audio reader (such as Python's 'wave' module)",
            }
            .into());
        }
        file_format::FileFormat::UnknownSqlite => {
            return Err(ActfastError::UnsupportedFormat {
                format: format_type,
                suggestion: "Use a general purpose SQLite reader (such as Python's 'sqlite3' module)",
            }
            .into());
        }
        _ => {
            return Err(ActfastError::UnsupportedFormat {
                format: format_type,
                suggestion: "This format is recognized but not yet implemented",
            }
            .into());
        }
    };

    dict.set_item("format", format_type.to_string())?;
    dict.set_item("timeseries", dict_timeseries)?;
    dict.set_item("metadata", dict_metadata)?;

    Ok(dict.into())
}

#[pymodule]
fn actfast(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read, m)?)?;
    Ok(())
}
