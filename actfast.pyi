"""`actfast` is a Python package for reading raw actigraphy data of various devices and manufacturers. 

It is designed to be fast, lightweight, memory efficient, and suitable for reading large datasets.
"""

from os import PathLike
from pathlib import Path
from typing import Any, Dict, Union

def read(path: Union[str, Path, PathLike]) -> Dict[str, Any]:
    """Read a raw actigraphy file and return a dictionary with metadata and data.

    The returned dictionary will contain the following:

    - "format": str: File format, any of "Actigraph GT3X", "Axivity CWA", "GeneActiv BIN", "Genea BIN", "Unknown WAV", "Unknown SQLite".
    - "metadata": Dict[str, Any]: Device specific key value pairs of metadata (e.g., device model, firmware version).
    - "timeseries": Dict[str, Dict[str, Any]]: Device specific key value pairs of "timeseries name" -> {timeseries data}, e.g.:
        - "high_frequency": Dict[str, Any]: High frequency timeseries data.
            - "datetime": 1D int64 numpy array of timestamps in nanoseconds (Unix epoch time).
            - Other data fields are various device specific sensor data, e.g.:
                - "acceleration": 2D numpy array (n_samples x 3) of acceleration data (x, y, z).
                - "light": 1D numpy array of light data.
                - "temperature": Temperature data.
                - ...
        - "low_frequency": Dict[str, Any]: Low frequency timeseries data.
            - Similar structure as high_frequency.

    Args:
        path (Union[str, Path, PathLike]): Path to the file.

    Returns:
        Dict[str, Any]: Dictionary with metadata and data.

    Raises:
        IOError: If the file is not found or corrupted.
    """
    ...
