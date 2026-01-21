"""Fast actigraphy data reader for Python, written in Rust."""

from os import PathLike
from typing import TypedDict

import numpy as np
from numpy.typing import NDArray


class TimeseriesData(TypedDict, total=False):
    """Timeseries data from a sensor."""

    datetime: NDArray[np.int64]
    acceleration: NDArray[np.float32]
    light: NDArray[np.float32] | NDArray[np.uint16]
    temperature: NDArray[np.float32]
    battery_voltage: NDArray[np.float32] | NDArray[np.uint16]
    button_state: NDArray[np.bool_]
    capsense: NDArray[np.bool_]


class ActfastResult(TypedDict):
    """Result from reading an actigraphy file."""

    format: str
    metadata: dict[str, dict[str, str]]
    timeseries: dict[str, TimeseriesData]


def read(path: str | PathLike[str]) -> ActfastResult:
    """Read a raw actigraphy file.

    Args:
        path: Path to the actigraphy file (.gt3x, .bin).

    Returns:
        Dictionary containing:
        - `format`: File format name (e.g., "Actigraph GT3X", "GeneActiv BIN")
        - `metadata`: Device-specific metadata as nested dicts
        - `timeseries`: Sensor data with `datetime` (int64 nanoseconds) and sensor arrays

    Raises:
        ValueError: If the file format is unknown, unsupported, or malformed.
        OSError: If the file cannot be read.

    Example:
        >>> data = actfast.read("subject1.gt3x")
        >>> data["timeseries"]["acceleration"]["datetime"]  # int64 timestamps
        >>> data["timeseries"]["acceleration"]["acceleration"]  # float32 (n, 3)
    """
    ...