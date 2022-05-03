import asyncio
import time
from terra_sdk.client.lcd import AsyncLCDClient

"""
uusd
```
contract_address	
terra1mshd2fy5g9c5zxcg5f7d42pgmwf8rjp3x8cev9
aterra	
terra1jwtg2p0p9mreu2qlrc6vgzk723gp6n76n4lu3m
```
uluna
```
contract_address	
terra18z45ccdqnk5zr72rpqzwszazaglrj2dxymmjmc
aterra	
terra1a47lhpxpuxsdtjtpk4p6jul5fr0wh6aa8r246z
```
"""
contract_address = "terra1gvaqwxtpptuuxuhvxkn5n05e9zvjrgd74serta"
atoken_address = "terra1jmwart643ta5z64mrrj47tmveud0ar82vd0k7u"


async def get_config(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={"config": {}},
    )

    print(resp)


async def get_atokens(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=atoken_address,
        query={"balance": {"address": "terra1xxxs5jjt666elnezqwyqft0j6ptvaldl6c73dn"}},
    )

    print(resp)


async def get_state(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={"state": {}},
    )

    print(resp)


async def get_depositor(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={
            "ident": {
                "address": "terra1xxxs5jjt666elnezqwyqft0j6ptvaldl6c73dn",
                "epoch": int(time.time()),
            }
        },
    )

    print(resp)


async def get_tvl(terra: AsyncLCDClient):
    print("---------")
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={"tvl": {"indice": -1}},
    )
    print(resp)
    for i in range(0, 6):
        print("---------")
        resp = await terra.wasm.contract_query(
            contract_address=contract_address,
            query={"tvl": {"indice": i}},
        )

        print(i, resp)
    print("---------")


async def main():
    terra = AsyncLCDClient("https://bombay-lcd.terra.dev", "bombay-12")
    # await get_config(terra)
    await get_state(terra)
    await get_tvl(terra)
    await get_depositor(terra)
    # await get_atokens(terra)
    await terra.session.close()  # you must close the session
    print(int(time.time()))


if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(main())

