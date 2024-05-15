# `actfast` Fast actigraphy data reader

`actfast` is a Python package for reading raw actigraphy data of various devices and manufacturers. It is designed to be fast, lightweight, memory efficient, and suitable for reading large datasets.

## Supported formats & devices

The package currently supports the following formats:

| Format | Manufacturer | Device | Implementation status |
| --- | --- | --- | --- |
| GT3X | Actigraph | wGT3X-BT | ✅ |
| BIN | GENEActiv | GENEActiv | ✅ |
| CWA | Axivity | AX3, AX6 | ❌ |
| BIN | Genea | Genea | ❌ |
| BIN | Movisens | Movisens | ❌ |
| WAV | Axivity | Axivity | Use general-purpose WAV audio file reader |
| AGD/SQLite | Actigraph | ActiGraph | Use general-purpose SQLite reader |
| AWD | Philips | Actiwatch | Use general-purpose CSV reader |
| MTN | Philips | Actiwatch | Use general-purpose XML reader |
| CSV | Any | Any | Use general-purpose CSV reader |
| XLS, XLSX, ODS | Any | Any | Use general-purpose Excel reader |

The scope of this package is limited to reading raw sensor
data. It does not read CSV or other standard file formats commonly used by various manufacturers. Use general-purpose libraries to read these files.

The package is designed to be easily extensible to support new formats and devices. If you have a non-standard device format that is not supported yet, please open an issue and attach a sample file. We will do our best to add support for it.

## Installation

Install from PyPI via:

```bash
pip install actfast
```

Or, install the latest development version from GitHub via:

```bash
pip install git+https://github.com/childmindresearch/actfast.git
```

## Tested devices

This package has been extensively tested with data captured by the following devices:

| Device | Firmware |
| --- | --- |
| ActiGraph wGT3X-BT | `1.9.2` |
| GENEActiv 1.2 | `Ver06.17 15June23` |

Similar devices might work, but have not been tested. Please open an issue and attach a sample file if you encounter any issues.

## Usage

The package provides a single function, `read`, which reads an actigraphy file and returns a dictionary:

```python
import actfast

subject1 = actfast.read("data/subject1.gt3x")
```

The returned dictionary has the following structure:

```python
{
    "format": "Actigraph GT3X",  # file format, any of "Actigraph GT3X", "Axivity CWA", "GeneActiv BIN", "Genea BIN", "Unknown WAV", "Unknown SQLite"
    "metadata": {
        # device specific key value pairs of metadata (e.g., device model, firmware version)
    },
    "timeseries": {
        # device specific key value pairs of "timeseries name" -> {timeseries data}, e.g.:
        "high_frequency": {
            "datetime": # 1D int64 numpy array of timestamps in nanoseconds (Unix epoch time)
            # other data fields are various device specific sensor data, e.g.:
            "acceleration": # 2D numpy array (n_samples x 3) of acceleration data (x, y, z)
            "light": # 1D numpy array of light data
            "temperature": # temperature data
            # ...
        },
        "low_frequency": {
            # similar structure as high_frequency
        }
    },
```

## Architecture & usage considerations

All supported formats seem to be constructed as streams of variable-length, variable-content records. While this stream of records is easy to write for the manufacturers, it is not ideal for vectorized operations. `actfast` collects data in contiguous arrays.

Consider reading large datasets once and storing them in a more efficient format (e.g., Parquet, HDF5) for subsequent analysis. This will always speed up data reading and enable streaming data processing.

## License

This package is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
