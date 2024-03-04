mod actigraph;
mod geneactiv;

use numpy::PyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;

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

#[pyfunction]
fn read_geneactiv_bin(_py: Python, path: &str) -> PyResult<PyObject> {
    let _data = geneactiv::load_data(path.to_string());
    let dict = PyDict::new(_py);
    Ok(dict.into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn actfast(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_actigraph_gt3x, m)?)?;
    m.add_function(wrap_pyfunction!(read_geneactiv_bin, m)?)?;
    Ok(())
}
