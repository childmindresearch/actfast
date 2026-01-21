# actfast

Fast actigraphy data reader for Python, written in Rust.

## Installation
```bash
pip install actfast
```

## Usage
```python
import actfast

data = actfast.read("subject1.gt3x")

# Returns:
# {
#     "format": "Actigraph GT3X",
#     "metadata": { ... },
#     "timeseries": {
#         "acceleration": {
#             "datetime": np.ndarray,      # int64, nanoseconds since Unix epoch
#             "acceleration": np.ndarray,  # float32, shape (n_samples, 3)
#         },
#         ...
#     },
# }
```

## Supported Formats

| Format | Manufacturer | Status |
|--------|--------------|--------|
| GT3X | ActiGraph | ✅ |
| BIN | GENEActiv | ✅ |
| CWA | Axivity | 🚧 Planned |

For standard formats (CSV, SQLite, WAV, Excel), use the appropriate Python standard library or pandas.

## Working with Timestamps
```python
import pandas as pd

timestamps = data["timeseries"]["acceleration"]["datetime"]
dt_index = pd.to_datetime(timestamps, unit="ns", utc=True)
```

## License

MIT