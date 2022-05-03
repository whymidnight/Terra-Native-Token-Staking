#!/usr/local/bin/python3
"""
Suppose the following records as records appended from anonymous fns.
"""
import typing
import json


class State:
    epochs: int
    tvl: int

    def __init__(self):
        self.epochs = 0
        self.tvl = 0


class TvlT:
    epochs: int

    def __init__(self):
        self.epoch = 0
        self.tvl_sum = 0


"""
bucket_tvl_t: {
    "state_epochs": {
        "tvl": 0,
        "epoch": 1648857973,
    }
}
"""
bucket_tvl_t: typing.Dict = {}
bucket_tvl_over_t: typing.List = [
    {
        "sum": 50,
        "epoch": 151,
    },
    {
        "sum": 500,
        "epoch": 261,
    },
    {
        "sum": 590,
        "epoch": 301,
    },
    {
        "sum": 690,
        "epoch": 404,
    },
    {
        "sum": 710,
        "epoch": 503,
    },
    {
        "sum": 850,
        "epoch": 690,
    },
    {
        "sum": 950,
        "epoch": 710,
    },
]


if __name__ == "__main__":
    state = State()
    for tvl_epoch in bucket_tvl_over_t:
        state.tvl += tvl_epoch["sum"]
        bucket_tvl_t[f"{state.epochs}"] = {
            "tvl": state.tvl,
            "epoch": tvl_epoch["epoch"],
        }
        print(f"tvl bucket - epoch: {state.epochs}", state.tvl)
        state.epochs += 1

    latest_tvl = json.dumps(
        bucket_tvl_t[f"{state.epochs-1}"], default=lambda x: x.__dict__, indent="  "
    )
    historical_tvl = json.dumps(bucket_tvl_t, default=lambda x: x.__dict__, indent="  ")
    print(historical_tvl, "", f"latest - {latest_tvl}", sep="\n")

    for i in range(0, state.epochs):
        t_tvl = json.dumps(
            bucket_tvl_t[f"{i}"], default=lambda x: x.__dict__, indent="  "
        )
        print(i, t_tvl)

    state_json = json.dumps(state, default=lambda x: x.__dict__, indent="  ")
    print(state_json)

