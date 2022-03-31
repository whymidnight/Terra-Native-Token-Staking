import asyncio
from terra_sdk.client.lcd import AsyncLCDClient

"""
contract_address	
terra1cam4mn8srq2n32f950acpx36v8wq4yhm5tz4v8
aterra	
terra1u6jyrgfvtjdl2kq8l7ktz8a6fgmzjluv34dz0y
"""
contract_address = "terra1cam4mn8srq2n32f950acpx36v8wq4yhm5tz4v8"
atoken_address = "terra1u6jyrgfvtjdl2kq8l7ktz8a6fgmzjluv34dz0y"


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


async def get_depositor(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={"state": {}},
    )

    print(resp)


async def get_state(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={"ident": {"address": "terra1xxxs5jjt666elnezqwyqft0j6ptvaldl6c73dn"}},
    )

    print(resp)


async def main():
    terra = AsyncLCDClient("https://bombay-lcd.terra.dev", "bombay-12")
    await get_config(terra)
    await get_state(terra)
    await get_depositor(terra)
    await get_atokens(terra)
    await terra.session.close()  # you must close the session


if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(main())

