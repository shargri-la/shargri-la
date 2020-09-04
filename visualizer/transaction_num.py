import sys

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns
import tqdm
from common import *

sns.set_style("ticks")
fig, ax = plt.subplots(figsize=(10, 6))

dirpath = sys.argv[1] if len(sys.argv) > 1 else "data"
raw_data = pd.read_csv(f'{dirpath}/function_num.csv', header=None)

SLOT_NUM = len(raw_data)

x = [i for i in range(0, SLOT_NUM)]

ax.cla()
ax.set_title('Number of Intra-shard/Cross-shard Transfers Over Time')
ax.set_xlabel('Slot')
ax.set_ylabel('Number of Transactions')

# labels = [
#     'Transfer (non-switchers)',
#     'CreateCrossTransfer (non-switchers)',
#     'ApplyCrossTransfer (non-switchers)',
#     'CreateCrossTransferAll (non-switchers)',
#     'ApplyCrossTransferAll (non-switchers)',
#     'Transfer (switchers)',
#     'CreateCrossTransfer (switchers)',
#     'ApplyCrossTransfer (switchers)',
#     'CreateCrossTransferAll (switchers)',
#     'ApplyCrossTransferAll (switchers)',
# ]

labels = [
    "Non-switchers' Intra-shard",
    "Non-switchers' Cross-shard",
    "Switchers' Intra-shard",
    "Switchers' Cross-shard",
]

transfer_n = raw_data[0]
cross_transfer_n = raw_data[1] + raw_data[3]
transfer_s = raw_data[5]
cross_transfer_s = raw_data[6] + raw_data[8]
y = [transfer_n, cross_transfer_n, transfer_s, cross_transfer_s]
colors = ['blue', 'skyblue', 'red', 'pink']

for i in range(4):
    ax.plot(x, y[i], label=labels[i], color=colors[i], alpha=0.5)

ax.legend()

fig.savefig(f'{dirpath}/function_num.png')
