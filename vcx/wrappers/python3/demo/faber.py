import asyncio
import json
import random
import os
import time
from ctypes import cdll
from time import sleep

from demo_utils import file_ext
from vcx.api.connection import Connection
from vcx.api.credential_def import CredentialDef
from vcx.api.issuer_credential import IssuerCredential
from vcx.api.proof import Proof
from vcx.api.schema import Schema
from vcx.api.utils import vcx_agent_provision, vcx_get_ledger_author_agreement, vcx_set_active_txn_author_agreement_meta
from vcx.api.vcx_init import vcx_init_with_config
from vcx.state import State, ProofState

TAA_ACCEPT = bool(os.getenv("TAA_ACCEPT", "0") == "1")

# logging.basicConfig(level=logging.DEBUG) uncomment to get logs

# 'agency_url': URL of the agency
# 'agency_did':  public DID of the agency
# 'agency_verkey': public verkey of the agency
# 'wallet_name': name for newly created encrypted wallet
# 'wallet_key': encryption key for encoding wallet
# 'payment_method': method that will be used for payments
provisionConfig = {
    'agency_url': 'http://localhost:8080',
    'agency_did': 'VsKV7grR1BUE29mG2Fm2kX',
    'agency_verkey': 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR',
    'wallet_name': 'faber_wallet',
    'wallet_key': '123',
    'payment_method': 'null',
    'enterprise_seed': '000000000000000000000000Trustee1',
    'protocol_type': '3.0',
}


async def main():
    payment_plugin = cdll.LoadLibrary('libnullpay' + file_ext())
    payment_plugin.nullpay_init()

    print("#1 Provision an agent and wallet, get back configuration details")
    config = await vcx_agent_provision(json.dumps(provisionConfig))
    config = json.loads(config)
    # Set some additional configuration options specific to faber
    config['institution_name'] = 'Faber'
    config['institution_logo_url'] = 'http://robohash.org/234'
    config['genesis_path'] = 'docker.txn'
    config['payment_method'] = 'null'

    print("#2 Initialize libvcx with new configuration")
    await vcx_init_with_config(json.dumps(config))

    if TAA_ACCEPT:
        # To support ledger which transaction author agreement accept needed
        print("#2.1 Accept transaction author agreement")
        txn_author_agreement = await vcx_get_ledger_author_agreement()
        txn_author_agreement_json = json.loads(txn_author_agreement)
        first_acc_mech_type = list(txn_author_agreement_json['aml'].keys())[0]
        vcx_set_active_txn_author_agreement_meta(text=txn_author_agreement_json['text'], version=txn_author_agreement_json['version'],
                                                 hash=None,
                                                 acc_mech_type=first_acc_mech_type, time_of_acceptance=int(time.time()))

    print("#3 Create a new schema on the ledger")
    version = format("%d.%d.%d" % (random.randint(1, 101), random.randint(1, 101), random.randint(1, 101)))
    schema = await Schema.create('schema_uuid', 'degree schema', version, ['email', 'first_name', 'last_name'], 0)
    schema_id = await schema.get_schema_id()

    print("#4 Create a new credential definition on the ledger")
    cred_def = await CredentialDef.create('credef_uuid', 'degree', schema_id, 0)
    cred_def_handle = cred_def.handle

    print("#5 Create a connection to alice and print out the invite details")
    connection_to_alice = await Connection.create('alice')
    await connection_to_alice.connect('{"use_public_did": true}')
    await connection_to_alice.update_state()
    details = await connection_to_alice.invite_details(False)
    print("**invite details**")
    print(json.dumps(details))
    print("******************")

    print("#6 Poll agency and wait for alice to accept the invitation (start alice.py now)")
    connection_state = await connection_to_alice.get_state()
    while connection_state != State.Accepted:
        sleep(2)
        await connection_to_alice.update_state()
        connection_state = await connection_to_alice.get_state()

    print("Connection is established")

    while True:
        answer = input(
            "Would you like to do? \n "
            "1 - issue credential \n "
            "2 - ask for proof request \n "
            "3 - send ping \n "
            "4 - update connection state \n "
            "else finish \n") \
            .lower().strip()
        if answer == '1':
            await issue_credential(connection_to_alice, cred_def_handle)
        elif answer == '2':
            await ask_for_proof(connection_to_alice, config['institution_did'])
        elif answer == '3':
            await connection_to_alice.send_ping(None)
            connection_state = await connection_to_alice.get_state()
            while connection_state != State.Accepted:
                sleep(5)
                await connection_to_alice.update_state()
                connection_state = await connection_to_alice.get_state()
                print("State: " + str(connection_state))
        elif answer == '4':
            await connection_to_alice.update_state()
        else:
            break

    print("Finished")


async def issue_credential(connection_to_alice, cred_def_handle):
    schema_attrs = {
        'email': 'test',
        'first_name': 'DemoName',
        'last_name': 'DemoLastName',
    }

    print("#12 Create an IssuerCredential object using the schema and credential definition")
    credential = await IssuerCredential.create('alice_degree', schema_attrs, cred_def_handle, 'cred', '0')

    print("#13 Issue credential offer to alice")
    await credential.send_offer(connection_to_alice)
    await credential.update_state()

    print("#14 Poll agency and wait for alice to send a credential request")
    credential_state = await credential.get_state()
    while credential_state != State.RequestReceived:
        sleep(2)
        await credential.update_state()
        credential_state = await credential.get_state()

    print("#17 Issue credential to alice")
    await credential.send_credential(connection_to_alice)

    print("#18 Wait for alice to accept credential")
    await credential.update_state()
    credential_state = await credential.get_state()
    while credential_state != State.Accepted:
        sleep(2)
        await credential.update_state()
        credential_state = await credential.get_state()


async def ask_for_proof(connection_to_alice, institution_did):
    proof_attrs = [
        {'name': 'email', 'restrictions': [{'issuer_did': institution_did}]},
        {'name': 'first_name', 'restrictions': [{'issuer_did': institution_did}]},
        {'name': 'last_name', 'restrictions': [{'issuer_did': institution_did}]}
    ]

    print("#19 Create a Proof object")
    proof = await Proof.create('proof_uuid', 'proof_from_alice', proof_attrs, {})

    print("#20 Request proof of degree from alice")
    await proof.request_proof(connection_to_alice)

    print("#21 Poll agency and wait for alice to provide proof")
    proof_state = await proof.get_state()
    while proof_state != State.Accepted:
        sleep(2)
        await proof.update_state()
        proof_state = await proof.get_state()

    print("#27 Process the proof provided by alice")
    await proof.get_proof(connection_to_alice)

    print("#28 Check if proof is valid")
    if proof.proof_state == ProofState.Verified:
        print("proof is verified!!")
    else:
        print("could not verify proof :(")


if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
    sleep(1)
