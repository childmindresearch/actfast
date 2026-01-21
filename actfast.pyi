"""Fast actigraphy data reader for Python, written in Rust."""

from os import PathLike
from typing import Required, TypedDict

import numpy as np
from numpy.typing import NDArray


class TimeseriesData(TypedDict, total=False):
    """Timeseries data from a sensor.
    
    The `datetime` field is always present. Other fields depend on the
    file format and sensor table.
    """

    datetime: Required[NDArray[np.int64]]
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
    warnings: list[str]


def read(path: str | PathLike[str], lenient: bool = False) -> ActfastResult:
    """Read a raw actigraphy file.

    Args:
        path: Path to the actigraphy file (.gt3x, .bin).
        lenient: If True, return partial data on corruption instead of raising.
            Any issues encountered will be reported in the `warnings` field.

    Returns:
        Dictionary containing:
        - `format`: File format name (e.g., "Actigraph GT3X", "GeneActiv BIN")
        - `metadata`: Device-specific metadata as nested dicts
        - `timeseries`: Sensor data with `datetime` (int64 nanoseconds) and sensor arrays
        - `warnings`: List of warnings (only populated when `lenient=True`)

    Raises:
        ValueError: If the file format is unknown, unsupported, or malformed
            (when `lenient=False`).
        OSError: If the file cannot be read.

    Example:
        >>> data = actfast.read("subject1.gt3x")
        >>> data["timeseries"]["acceleration"]["datetime"]  # int64 timestamps
        >>> data["timeseries"]["acceleration"]["acceleration"]  # float32 (n, 3)

        >>> # For corrupted files, use lenient mode:
        >>> data = actfast.read("corrupted.gt3x", lenient=True)
        >>> if data["warnings"]:
        ...     print(f"Recovered partial data with {len(data['warnings'])} warnings")
    """
    ...