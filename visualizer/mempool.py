import sys

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import tqdm
from common import *

sns.set_style("ticks")
fig, ax = plt.subplots(figsize=(10, 6))

dirpath = sys.argv[1] if len(sys.argv) > 1 else "data"
raw_data = pd.read_csv(f'{dirpath}/mempool.csv', header=None)

ROW_NUM = len(raw_data)
SKIP_FRAME_NUM = 0
SHARD_NUM = len(raw_data.loc[0])
FRAME_NUM = ROW_NUM - SKIP_FRAME_NUM

height = []
for slot in tqdm.tqdm(range(SKIP_FRAME_NUM, ROW_NUM)):
    h = [0 for _ in range(SHARD_NUM)]
    for i in range(SHARD_NUM):
        h[i] = raw_data.loc[slot][i]
    height.append(h)

x = [i for i in range(SKIP_FRAME_NUM, ROW_NUM)]

ax.cla()
ax.set_title('Number of Mempool Transactions in Each Shard Over Time')
ax.set_xlabel('Slot')
ax.set_ylabel('Number of Mempool Transactions')

for shard_id in range(SHARD_NUM - 1, -1, -1):
    h = []
    for slot in range(SKIP_FRAME_NUM, len(raw_data)):
        h.append(height[slot - SKIP_FRAME_NUM][shard_id])
    if shard_id < 3:
        ax.plot(x, h, color=color_list[shard_id], label=f"Shard {shard_id}")
    else:
        ax.plot(x, h, color='silver')
ax.legend()

fig.savefig(f'{dirpath}/mempool.png')
