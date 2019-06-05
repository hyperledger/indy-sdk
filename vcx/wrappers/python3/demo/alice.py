import asyncio
import json
import os
from ctypes import cdll
from time import sleep
import platform

import logging

from demo_utils import file_ext
from vcx.api.connection import Connection
from vcx.api.credential import Credential
from vcx.api.disclosed_proof import DisclosedProof
from vcx.api.utils import vcx_agent_provision
from vcx.api.vcx_init import vcx_init_with_config
from vcx.state import State

# logging.basicConfig(level=logging.DEBUG) uncomment to get logs
agentIp = os.environ.get('AGENT_IP', 'localhost')
provisionConfig = {
    'agency_url': 'http://'+agentIp+':8080',
    'agency_did': 'VsKV7grR1BUE29mG2Fm2kX',
    'agency_verkey': 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR',
    'wallet_name': 'alice_wallet',
    'wallet_key': '123',
    'payment_method': 'null',
    'enterprise_seed': '000000000000000000000000Trustee1'
}

async def main():

    payment_plugin = cdll.LoadLibrary('libnullpay' + file_ext())
    payment_plugin.nullpay_init()

    print("#7 Provision an agent and wallet, get back configuration details")
    config = await vcx_agent_provision(json.dumps(provisionConfig))
    config = json.loads(config)
    # Set some additional configuration options specific to alice
    config['institution_name'] = 'alice'
    config['institution_logo_url'] = 'http://robohash.org/456'
    config['genesis_path'] = os.environ.get('POOL_TXN_FILE','pool-local.txn')

    print("#8 Initialize libvcx with new configuration")
    await vcx_init_with_config(json.dumps(config))

    print("#9 Input faber.py invitation details")
    details = input('invite details: ')

    print("#10 Convert to valid json and string and create a connection to faber")
    jdetails = json.loads(details)
    connection_to_faber = await Connection.create_with_details('faber', json.dumps(jdetails))
    await connection_to_faber.connect('{"use_public_did": true}')
    await connection_to_faber.update_state()

    print("#11 Wait for faber.py to issue a credential offer")
    sleep(10)
    offers = await Credential.get_offers(connection_to_faber)

    # Create a credential object from the credential offer
    credential = await Credential.create('credential', offers[0])

    print("#15 After receiving credential offer, send credential request")
    await credential.send_request(connection_to_faber, 0)

    print("#16 Poll agency and accept credential offer from faber")
    credential_state = await credential.get_state()
    while credential_state != State.Accepted:
        sleep(2)
        await credential.update_state()
        credential_state = await credential.get_state()

    print("#22 Poll agency for a proof request")
    requests = await DisclosedProof.get_requests(connection_to_faber)

    print("#23 Create a Disclosed proof object from proof request")
    proof = await DisclosedProof.create('proof', requests[0])

    print("#24 Query for credentials in the wallet that satisfy the proof request")
    credentials = await proof.get_creds()

    # Use the first available credentials to satisfy the proof request
    for attr in credentials['attrs']:
        credentials['attrs'][attr] = {
            'credential': credentials['attrs'][attr][0]
        }

    print("#25 Generate the proof")
    await proof.generate_proof(credentials, {})

    print("#26 Send the proof to faber")
    await proof.send_proof(connection_to_faber)


if __name__ == '__main__':
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
