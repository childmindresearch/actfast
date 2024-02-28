import actfast

import numpy as np 
import pandas as pd

acti = actfast.read_gt3x(r"..\..\testdata\NDARAA948VFH.gt3x")

df_me = pd.DataFrame.from_dict({
    "Timestamp": acti["datetime"],
    "X": acti["data"][:, 0],
    "Y": acti["data"][:, 1],
    "Z": acti["data"][:, 2]
})

df_me["Timestamp"] = pd.to_datetime(df_me["Timestamp"], unit='ns')



df_them = pd.read_parquet(r"C:\Users\floru\Projects\cmi\actigrapy\NDARAA948VFH.parquet")
df_them.reset_index(inplace=True)
df_them["Timestamp"] = pd.to_datetime(df_them["Timestamp"], unit='s')

# plot first 100 values for comparison
import matplotlib.pyplot as plt

# 2 subplots
fig, axs = plt.subplots(2)

# me 
axs[0].plot(df_me["Timestamp"].head(100), df_me["X"].head(100), label="X me")
axs[0].plot(df_me["Timestamp"].head(100), df_me["Y"].head(100), label="Y me")
axs[0].plot(df_me["Timestamp"].head(100), df_me["Z"].head(100), label="Z me")

# them
axs[1].plot(df_them["Timestamp"].head(100), df_them["X"].head(100), label="X them")
axs[1].plot(df_them["Timestamp"].head(100), df_them["Y"].head(100), label="Y them")
axs[1].plot(df_them["Timestamp"].head(100), df_them["Z"].head(100), label="Z them")

fig.show()


# compare shapes
print(df_me.shape)
print(df_them.shape)

# compare first 5 rows
print(df_me.head())

print(df_them.head())

# compare last 5 rows

print(df_me.tail())

print(df_them.tail())