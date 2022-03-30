import asyncio
from terra_sdk.client.lcd import AsyncLCDClient

"""

contract_address	
terra1hl0gx6530w0mp8eqsdav6nuvf0fjnxqyvgfveh
aterra	
terra1lpv5dwuzz3xhznnpwjhcy4ptaevlyuq5kz8a9l
"""
contract_address = "terra1hl0gx6530w0mp8eqsdav6nuvf0fjnxqyvgfveh"


async def get_config(terra: AsyncLCDClient):
    resp = await terra.wasm.contract_query(
        contract_address=contract_address,
        query={"config": {}},
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
    terra = AsyncLCDClient("https://bombay-fcd.terra.dev", "bombay-12")
    await get_config(terra)
    await get_state(terra)
    await get_depositor(terra)
    await terra.session.close()  # you must close the session


if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(main())

