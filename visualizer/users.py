import sys

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import tqdm
from common import *

sns.set_style("ticks")
fig, ax = plt.subplots(figsize=(10, 6))

dirpath = sys.argv[1] if len(sys.argv) > 1 else "data"
raw_data = pd.read_csv(f'{dirpath}/users.csv', header=None)

switchers = raw_data[raw_data[1] == True]
non_switchers = raw_data[raw_data[1] == False]
switchers = switchers.sort_values(2).reset_index()
non_switchers = non_switchers.sort_values(2).reset_index()

USER_NUM = len(raw_data)

ax.cla()
ax.set_title('User Distribution by Total Transaction Fee')
ax.set_xlabel('Total Transaction Fee (Gwei)')
ax.set_ylabel('Number of Users')

cmap = plt.get_cmap("tab10")
heights = [[] for _ in range(len(strategy_labels))]

PLOT_CONDITION_MIN_USER_NUM = 2
G = 1000000000
for i in tqdm.tqdm(range(USER_NUM)):
    strategy_i = raw_data.loc[i][1]
    total_fee = raw_data.loc[i][2] // G
    heights[strategy_i].append(total_fee)
for i in range(len(strategy_labels)):
    if len(heights[i]) >= PLOT_CONDITION_MIN_USER_NUM:
        ax.hist(heights[i], bins=100, label=strategy_labels[i],
                alpha=0.5, color=cmap(i))
        ax.axvline(sorted(heights[i])[len(heights[i])//2], color=cmap(i),
                   linestyle='dashed', linewidth=1)
ax.legend()

fig.savefig(f'{dirpath}/users.png')
