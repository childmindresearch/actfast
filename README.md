> [!NOTE]  
> `actfast` is currently in early development and is not yet ready for production use. Please open an issue if you encounter any problems or have any feature requests.

# `actfast` Fast actigraphy data reader

`actfast` is a minimal Python package for reading raw actigraphy data from various devices. It is written in Rust and Python, and is designed with performance and memory safety in mind.

In preliminary benchmarks, `actfast` showed around 40x speedup compared to `pygt3x` for reading ActiGraph GT3X files.

## Installation

Install from PyPI via:

```bash
pip install actfast
```

Or, install the latest development version from GitHub via:

```bash
pip install git+https://github.com/childmindresearch/actfast.git
```

## Hardware support

This package has been tested with data captured by the following devices:

| Device | Firmware | API |
| --- | --- | --- |
| ActiGraph wGT3X-BT | `1.9.2` | `actfast.read_actigraph_gt3x(file)` |
| GENEActiv 1.2 | `Ver06.17 15June23` | `actfast.read_geneactiv_bin(file)` |

Similar devices might work, but have not been tested. Please open an issue and attach a sample file if you have a device that is not supported yet. We will do our best to add support for it.

## Usage

```python
import actfast

subject1 = actfast.read_actigraph_gt3x("data/subject1.gt3x")
```
    
If you are using Pandas and want a similar dataframe to what `pygt3x` offers, you can convert the data to a dataframe using the following code snippet:

```python
import pandas as pd

accel = subject1["timeseries"]["acceleration"]

df = pd.DataFrame.from_dict({
    "Timestamp": accel["datetime"],
    "X": accel["acceleration"][:, 0],
    "Y": accel["acceleration"][:, 1],
    "Z": accel["acceleration"][:, 2]
})

df["Timestamp"] = pd.to_datetime(df["Timestamp"], unit='ns')
df = df.set_index("Timestamp")
```
