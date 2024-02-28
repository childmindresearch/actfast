# `actfast` Fast actigraphy data loader

Install development version from GitHub with pip:

```bash
pip install git+https://github.com/childmindresearch/actfast.git
```


Usage

```python
import actfast
import pandas as pd

data = actfast.read_gt3x(r"..\..\testdata\NDARAA948VFH.gt3x")

df = pd.DataFrame.from_dict({
    "Timestamp": data["datetime"],
    "X": data["data"][:, 0],
    "Y": data["data"][:, 1],
    "Z": data["data"][:, 2]
})

df["Timestamp"] = pd.to_datetime(df["Timestamp"], unit='ns')
# df = df.set_index("Timestamp")
```