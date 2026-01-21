import pytest
from pathlib import Path


@pytest.fixture
def test_data_dir() -> Path:
    """Path to test data directory."""
    return Path(__file__).parent.parent.parent / "test_data" / "cmi"


@pytest.fixture
def geneactiv_file(test_data_dir) -> Path:
    """Path to test GeneActiv file."""
    path = test_data_dir / "geneactiv.bin"
    if not path.exists():
        pytest.skip(f"Test file not found: {path}")
    return path


@pytest.fixture
def actigraph_file(test_data_dir) -> Path:
    """Path to test Actigraph file."""
    path = test_data_dir / "actigraph.gt3x"
    if not path.exists():
        pytest.skip(f"Test file not found: {path}")
    return path