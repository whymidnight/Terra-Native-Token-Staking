import time
import sys
from terra_sdk.key.raw import RawKey
from bip32utils import BIP32_HARDEN, BIP32Key
from mnemonic import Mnemonic
import base64
from terra_sdk.client.lcd.api.tx import CreateTxOptions
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgMigrateContract
from terra_sdk.core.fee import Fee
from terra_sdk.client.lcd import LCDClient


class MnemonicKey(RawKey):
    def __init__(
        self,
        mnemonic: str = None,
        account: int = 0,
        index: int = 0,
        coin_type: int = 330,
    ):
        if mnemonic is None:
            mnemonic = Mnemonic("english").generate(256)
        seed = Mnemonic("english").to_seed(mnemonic)
        root = BIP32Key.fromEntropy(seed)
        # derive from hdpath
        child: BIP32Key = (
            root.ChildKey(44 + BIP32_HARDEN)
            .ChildKey(coin_type + BIP32_HARDEN)  # type: ignore
            .ChildKey(account + BIP32_HARDEN)  # type: ignore
            .ChildKey(0)  # type: ignore
            .ChildKey(index)  # type: ignore
        )

        super().__init__(child.PrivateKey())
        self.mnemonic = mnemonic
        self.account = account
        self.index = index


terra = LCDClient(chain_id="bombay-12", url="https://bombay-lcd.terra.dev")
test1 = terra.wallet(
    MnemonicKey(
        mnemonic="crop window emotion sister mesh winner syrup horse foot claim couch test essence toward nominee flight blur connect female mom bread gesture gown extra"
    )
)  # terra1799q25fnkxledqyj8sdgrmhc92apy6yq7wz6j9


def upload() -> str:
    contract_file = open("./contract.wasm", "rb")
    file_bytes = base64.b64encode(contract_file.read()).decode()
    store_code = MsgStoreCode(test1.key.acc_address, file_bytes)
    store_code_tx = test1.create_and_sign_tx(
        CreateTxOptions(msgs=[store_code], fee=Fee(4000000, "4000000uusd"))
    )
    store_code_tx_result = terra.tx.broadcast(store_code_tx)
    code_id = store_code_tx_result.logs[0].events_by_type["store_code"]["code_id"][0]  # type: ignore
    return code_id


payloads = [
    {
        "stable_denom": "uusd",
        "aterra_code_id": 1572,
        "interest": "0.000382982750338989",
    },
    {
        "stable_denom": "uluna",
        "aterra_code_id": 1572,
        "interest": "0.000382982750338989",
    },
]


def instance(code_id: str):
    def instance_payload(payload):
        instantiate = MsgInstantiateContract(
            test1.key.acc_address,
            test1.key.acc_address,
            code_id,
            payload,
            {"uusd": 1000000},
        )
        instantiate_tx = test1.create_and_sign_tx(CreateTxOptions(msgs=[instantiate]))
        instantiate_tx_result = terra.tx.broadcast(instantiate_tx)
        contract_address = instantiate_tx_result.logs[0].events_by_type[  # type: ignore
            "instantiate_contract"
        ]

        return contract_address

    successes = 0
    contract_addresses = []
    while successes < len(payloads):
        time.sleep(2.0)
        try:
            contract_addresses.append(
                {
                    "denom": payloads[successes]["stable_denom"],
                    "contract_addresses": instance_payload(payloads[successes])[
                        "contract_address"
                    ],
                }
            )
            successes += 1
            continue
        except:
            print("retrying...")

    return contract_addresses


def migrate(contracts, migration_code_id):
    def migrate_contract(contract_address):
        migration = MsgMigrateContract(
            test1.key.acc_address,
            contract_address,
            migration_code_id,
            {},
        )
        migration_tx = test1.create_and_sign_tx(CreateTxOptions(msgs=[migration]))
        migration_tx_result = terra.tx.broadcast(migration_tx)
        print()
        print(migration_tx_result)
        print()

    successes = 0
    contract_addresses = []
    while successes < len(contracts):
        time.sleep(2.0)
        try:
            contract_addresses.append(
                migrate_contract(contracts[successes]["contract_address"][0])
            )
            successes += 1
            continue
        except Exception as e:
            print(e)
            print("retrying...")

    return contract_addresses


if __name__ == "__main__":
    if sys.argv[1] == "upload":
        upload()

    if sys.argv[1] == "instance":
        code_id = sys.argv[2]  # type: ignore
        instance(code_id)

    if sys.argv[1] == "deploy":
        code_id = upload()
        print("sleeping")
        time.sleep(5.0)
        print("slept")
        print(instance(code_id))

    if sys.argv[1] == "migrate":
        code_id = upload()
        time.sleep(5.0)
        contracts = instance(code_id)
        print(contracts)
        time.sleep(5.0)
        migration_code_id = upload()
        time.sleep(5.0)
        migrate(contracts, migration_code_id)

