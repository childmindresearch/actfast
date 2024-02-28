# `actfast` Fast actigraphy data loader

Install development version from GitHub with pip:

```bash
pip install git+https://github.com/childmindresearch/actfast.git
```


Usage

```python
import actfast

data = actfast.read_gt3x("data/NDARAA948VFH.gt3x")
```
    
If you want a similar pandas dataframe as gt3xpy has:

```python
import pandas as pd

df = pd.DataFrame.from_dict({
    "Timestamp": data["datetime"],
    "X": data["data"][:, 0],
    "Y": data["data"][:, 1],
    "Z": data["data"][:, 2]
})

df["Timestamp"] = pd.to_datetime(df["Timestamp"], unit='ns')
# df = df.set_index("Timestamp")
```