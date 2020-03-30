import pytest
from vcx.error import ErrorCode, VcxError
from vcx.state import State
from vcx.api.credential import Credential
from vcx.api.connection import Connection

connection_options = '{"connection_type":"SMS","phone":"8019119191","use_public_did":true}'

source_id = '1'
msg_id = '1'
offer = [{
   "msg_type": "CLAIM_OFFER",
   "version": "0.1",
   "to_did": "8XFh8yBzrpJQmNyZzgoTqB",
   "from_did": "8XFh8yBzrpJQmNyZzgoTqB",
   "libindy_offer": '{}',
   "credential_attrs": {
      "address1": [
         "101 Tela Lane"
      ],
      "address2": [
         "101 Wilson Lane"
      ],
      "city": [
         "SLC"
      ],
      "state": [
         "UT"
      ],
      "zip": [
         "87121"
      ]
   },
   "schema_seq_no": 1487,
   "cred_def_id": "id1",
   "claim_name": "Credential",
   "claim_id": "defaultCredentialId",
   "msg_ref_id": None,
}]

credential_json = {
    'source_id': 'wrapper_tests',
    'state': 3,
    'credential_name': None,
    'credential_request': {
        "libindy_cred_req": "",
        "libindy_cred_req_meta": "",
        "cred_def_id": "id",
        "tid": "",
        "to_did": "",
        "from_did": "",
        "mid": "",
        "version": "",
    },
    'credential_offer': {
       "msg_type": "CLAIM_OFFER",
       "version": "0.1",
       "to_did": "8XFh8yBzrpJQmNyZzgoTqB",
       "from_did": "8XFh8yBzrpJQmNyZzgoTqB",
       "libindy_offer": '{}',
       "credential_attrs": {
          "address1": [
             "101 Tela Lane"
          ],
          "address2": [
             "101 Wilson Lane"
          ],
          "city": [
             "SLC"
          ],
          "state": [
             "UT"
          ],
          "zip": [
             "87121"
          ]
       },
       "schema_seq_no": 1487,
       "cred_def_id": "id1",
       "claim_name": "Credential",
       "claim_id": "defaultCredentialId",
       "msg_ref_id": "id"
    },
    'link_secret_alias': 'main',
    'msg_uid': None,
    'agent_did': None,
    'agent_vk': None,
    'my_did': None,
    'my_vk': None,
    'their_did': None,
    'cred_id': None,
    'credential': None,
    'their_vk': None,
    "payment_info":{
      "payment_required":"one-time",
      "payment_addr":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j",
      "price":1
   }
  }

credential_json_versioned = {
    'version': "1.0",
    'data': credential_json
}

@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_create_credential():
    credential = await Credential.create(source_id, offer)
    assert credential.source_id == source_id
    assert credential.handle > 0
    assert await credential.get_state() == State.RequestReceived


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_create_credential_with_msgid():
    connection = await Connection.create(source_id)
    await connection.connect(connection_options)

    credential = await Credential.create_with_msgid(source_id, connection, msg_id)
    assert credential.source_id == source_id
    assert credential.handle > 0
    assert credential.cred_offer
    assert await credential.get_state() == State.RequestReceived


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_serialize():
    credential = await Credential.create(source_id, offer)
    data = await credential.serialize()
    assert data.get('data').get('source_id') == source_id


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_serialize_with_bad_handle():
    with pytest.raises(VcxError) as e:
        credential = Credential(source_id)
        credential.handle = 0
        await credential.serialize()
    assert ErrorCode.InvalidCredentialHandle == e.value.error_code


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_deserialize():
    credential = await Credential.create(source_id, offer)
    data = await credential.serialize()
    data['data']['state'] = State.Expired
    credential2 = await Credential.deserialize(data)
    assert credential2.source_id == data.get('data').get('source_id')
    assert await credential2.get_state() == State.Expired


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_deserialize_with_invalid_data():
    with pytest.raises(VcxError) as e:
        data = {'data':{'invalid': -99}}
        await Credential.deserialize(data)
    assert ErrorCode.InvalidJson == e.value.error_code


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_serialize_deserialize_and_then_serialize():
    credential = await Credential.create(source_id, offer)
    data1 = await credential.serialize()
    credential2 = await Credential.deserialize(data1)
    data2 = await credential2.serialize()
    assert data1 == data2


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_update_state():
    credential = await Credential.create(source_id, offer)
    assert await credential.update_state() == State.RequestReceived


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_update_state_with_invalid_handle():
    with pytest.raises(VcxError) as e:
        credential = Credential(source_id)
        credential.handle = 0
        await credential.update_state()
    assert ErrorCode.InvalidCredentialHandle == e.value.error_code


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_get_state():
    credential = await Credential.create(source_id, offer)
    assert await credential.get_state() == State.RequestReceived


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_credential_release():
    with pytest.raises(VcxError) as e:
        credential = await Credential.create(source_id, offer)
        assert credential.handle > 0
        credential.release()
        await credential.serialize()
    assert ErrorCode.InvalidCredentialHandle == e.value.error_code


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_send_request():
    connection = await Connection.create(source_id)
    await connection.connect(connection_options)
    cred_with_msg_id = credential_json
    credential = await Credential.deserialize(credential_json_versioned)
    await credential.send_request(connection, 0)
    assert await credential.update_state() == State.OfferSent


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_get_request_msg():
    connection = await Connection.create(source_id)
    await connection.connect(connection_options)
    my_pw_did = await connection.get_my_pw_did()
    their_pw_did = await connection.get_their_pw_did()
    credential = await Credential.deserialize(credential_json_versioned)
    msg = await credential.get_request_msg(my_pw_did, their_pw_did, 0)
    assert(msg)


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_send_request_with_invalid_state():
    with pytest.raises(VcxError) as e:
        connection = await Connection.create(source_id)
        await connection.connect(connection_options)
        credential = await Credential.create(source_id, offer)
        await credential.send_request(connection, 0)
    assert ErrorCode.CreateCredentialFailed == e.value.error_code


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_send_request_with_bad_connection():
    with pytest.raises(VcxError) as e:
        connection = Connection(source_id)
        credential = await Credential.create(source_id, offer)
        await credential.send_request(connection, 0)
    assert ErrorCode.InvalidConnectionHandle == e.value.error_code

@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_credential_get_payment_txn():
    with pytest.raises(VcxError) as e:
        credential = await Credential.create(source_id, offer)
        await credential.get_payment_txn()
    assert ErrorCode.NoPaymentInformation == e.value.error_code
