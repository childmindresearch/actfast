import numpy as np
import pytest
from pathlib import Path
from tempfile import NamedTemporaryFile

import actfast


class TestRead:
    """Tests for actfast.read() function."""

    def test_geneactiv_basic(self, geneactiv_file):
        """Test basic GeneActiv file reading."""
        result = actfast.read(geneactiv_file)

        assert isinstance(result, dict)
        assert "format" in result
        assert "metadata" in result
        assert "timeseries" in result
        assert result["format"] == "GeneActiv BIN"

    def test_actigraph_basic(self, actigraph_file):
        """Test basic Actigraph file reading."""
        result = actfast.read(actigraph_file)

        assert isinstance(result, dict)
        assert result["format"] == "Actigraph GT3X"
        assert "metadata" in result
        assert "timeseries" in result

    def test_geneactiv_metadata(self, geneactiv_file):
        """Test GeneActiv metadata extraction."""
        result = actfast.read(geneactiv_file)
        metadata = result["metadata"]

        assert isinstance(metadata, dict)
        assert len(metadata) > 0

        # Check for expected metadata categories
        assert "Device Identity" in metadata or "Calibration Data" in metadata

    def test_actigraph_metadata(self, actigraph_file):
        """Test Actigraph metadata extraction."""
        result = actfast.read(actigraph_file)
        metadata = result["metadata"]

        assert "info" in metadata
        assert "Sample Rate" in metadata["info"]

    def test_geneactiv_timeseries(self, geneactiv_file):
        """Test GeneActiv timeseries data."""
        result = actfast.read(geneactiv_file)
        timeseries = result["timeseries"]

        assert "high_frequency" in timeseries
        assert "low_frequency" in timeseries

        hf = timeseries["high_frequency"]
        assert "datetime" in hf
        assert "acceleration" in hf
        assert "light" in hf
        assert "button_state" in hf

        # Check numpy arrays
        assert isinstance(hf["datetime"], np.ndarray)
        assert isinstance(hf["acceleration"], np.ndarray)
        assert hf["datetime"].dtype == np.int64
        assert hf["acceleration"].dtype == np.float32

    def test_actigraph_timeseries(self, actigraph_file):
        """Test Actigraph timeseries data."""
        result = actfast.read(actigraph_file)
        timeseries = result["timeseries"]

        assert "acceleration" in timeseries
        assert "light" in timeseries
        assert "capsense" in timeseries
        assert "battery_voltage" in timeseries

        acc = timeseries["acceleration"]
        assert "datetime" in acc
        assert "acceleration" in acc
        assert isinstance(acc["acceleration"], np.ndarray)

    def test_geneactiv_acceleration_shape(self, geneactiv_file):
        """Test that acceleration data has correct shape (n_samples, 3)."""
        result = actfast.read(geneactiv_file)
        hf = result["timeseries"]["high_frequency"]

        acc = hf["acceleration"]
        dt = hf["datetime"]

        # Should be reshaped to (n_samples, 3) for x, y, z
        assert acc.ndim == 2
        assert acc.shape[0] == len(dt)
        assert acc.shape[1] == 3

    def test_actigraph_acceleration_shape(self, actigraph_file):
        """Test that acceleration data has correct shape."""
        result = actfast.read(actigraph_file)
        acc_table = result["timeseries"]["acceleration"]

        acc = acc_table["acceleration"]
        dt = acc_table["datetime"]

        assert acc.ndim == 2
        assert acc.shape[0] == len(dt)
        assert acc.shape[1] == 3

    def test_geneactiv_values(self, geneactiv_file):
        """Test specific values from GeneActiv file match expected."""
        result = actfast.read(geneactiv_file)
        hf = result["timeseries"]["high_frequency"]
        lf = result["timeseries"]["low_frequency"]

        # Values from Rust tests
        assert len(hf["datetime"]) == 6000
        assert len(lf["datetime"]) == 20

        # Check first acceleration value
        np.testing.assert_almost_equal(hf["acceleration"][0, 0], 0.943648595, decimal=5)

        # Check temperature
        np.testing.assert_almost_equal(lf["temperature"][0], 35.8, decimal=1)

    def test_actigraph_values(self, actigraph_file):
        """Test specific values from Actigraph file match expected."""
        result = actfast.read(actigraph_file)
        acc_table = result["timeseries"]["acceleration"]

        assert len(acc_table["datetime"]) == 4860

        # Values from Rust tests
        np.testing.assert_almost_equal(acc_table["acceleration"][0, 0], -0.519531, decimal=5)
        np.testing.assert_almost_equal(acc_table["acceleration"][0, 1], -0.519531, decimal=5)
        np.testing.assert_almost_equal(acc_table["acceleration"][0, 2], -0.636719, decimal=5)


class TestReadErrors:
    """Tests for error handling."""

    def test_file_not_found(self):
        """Test error when file doesn't exist."""
        with pytest.raises(Exception) as exc_info:
            actfast.read("/nonexistent/path/file.bin")

        # Should mention the file path in error
        assert "nonexistent" in str(exc_info.value).lower() or "no such file" in str(exc_info.value).lower()

    def test_unknown_format(self, tmp_path):
        """Test error for unknown file format."""
        test_file = tmp_path / "unknown.bin"
        test_file.write_bytes(b"UNKN" + b"\x00" * 100)

        with pytest.raises(ValueError) as exc_info:
            actfast.read(test_file)

        error_msg = str(exc_info.value).lower()
        assert "unknown" in error_msg or "format" in error_msg

    def test_truncated_geneactiv(self, tmp_path):
        """Test error for truncated GeneActiv file."""
        test_file = tmp_path / "truncated.bin"
        # GeneActiv magic bytes but truncated content
        test_file.write_bytes(b"Device Identity\nSerial:123\n")

        with pytest.raises(ValueError) as exc_info:
            actfast.read(test_file)

        # Should indicate unexpected EOF or similar
        error_msg = str(exc_info.value).lower()
        assert "unexpected" in error_msg or "eof" in error_msg or "end" in error_msg

    def test_invalid_zip_for_actigraph(self, tmp_path):
        """Test error when GT3X file is not a valid ZIP."""
        test_file = tmp_path / "invalid.gt3x"
        # ZIP magic bytes but invalid content
        test_file.write_bytes(b"PK\x03\x04" + b"\x00" * 100)

        with pytest.raises(ValueError) as exc_info:
            actfast.read(test_file)

        error_msg = str(exc_info.value).lower()
        assert "zip" in error_msg or "archive" in error_msg or "parse" in error_msg

    def test_unsupported_wav(self, tmp_path):
        """Test helpful error for WAV files."""
        test_file = tmp_path / "audio.wav"
        test_file.write_bytes(b"RIFF" + b"\x00" * 100)

        with pytest.raises(ValueError) as exc_info:
            actfast.read(test_file)

        error_msg = str(exc_info.value).lower()
        assert "wav" in error_msg or "audio" in error_msg
        assert "wave" in error_msg  # Should suggest Python's wave module

    def test_unsupported_sqlite(self, tmp_path):
        """Test helpful error for SQLite files."""
        test_file = tmp_path / "data.agd"
        test_file.write_bytes(b"SQLite format 3\x00" + b"\x00" * 100)

        with pytest.raises(ValueError) as exc_info:
            actfast.read(test_file)

        error_msg = str(exc_info.value).lower()
        assert "sqlite" in error_msg
        assert "sqlite3" in error_msg  # Should suggest Python's sqlite3 module


class TestPathTypes:
    """Test different path input types."""

    def test_string_path(self, geneactiv_file):
        """Test reading with string path."""
        result = actfast.read(str(geneactiv_file))
        assert result["format"] == "GeneActiv BIN"

    def test_pathlib_path(self, geneactiv_file):
        """Test reading with pathlib.Path."""
        result = actfast.read(geneactiv_file)
        assert result["format"] == "GeneActiv BIN"


class TestDataIntegrity:
    """Tests for data integrity and consistency."""

    def test_datetime_monotonic(self, geneactiv_file):
        """Test that datetime values are monotonically increasing."""
        result = actfast.read(geneactiv_file)

        for table_name, table in result["timeseries"].items():
            dt = table["datetime"]
            if len(dt) > 1:
                # Allow equal timestamps (same second) but not decreasing
                assert np.all(np.diff(dt) >= 0), f"datetime not monotonic in {table_name}"

    def test_acceleration_reasonable_range(self, geneactiv_file):
        """Test that acceleration values are in reasonable range."""
        result = actfast.read(geneactiv_file)
        acc = result["timeseries"]["high_frequency"]["acceleration"]

        # Typical accelerometer range is -8g to 8g
        assert np.all(acc >= -16), "Acceleration below -16g"
        assert np.all(acc <= 16), "Acceleration above 16g"

    def test_no_nan_values(self, geneactiv_file):
        """Test that there are no NaN values in numeric data."""
        result = actfast.read(geneactiv_file)

        for table_name, table in result["timeseries"].items():
            for key, value in table.items():
                if isinstance(value, np.ndarray) and np.issubdtype(value.dtype, np.floating):
                    assert not np.any(np.isnan(value)), f"NaN found in {table_name}/{key}"

    def test_no_inf_values(self, geneactiv_file):
        """Test that there are no infinite values."""
        result = actfast.read(geneactiv_file)

        for table_name, table in result["timeseries"].items():
            for key, value in table.items():
                if isinstance(value, np.ndarray) and np.issubdtype(value.dtype, np.floating):
                    assert not np.any(np.isinf(value)), f"Inf found in {table_name}/{key}"