import pytest
from vcx.error import VcxError, ErrorCode
from vcx.api.wallet import *
import json

TYPE = "record type"
EMPTY_TYPE = ""
ID = "123"
EMPTY_ID = ""
VALUE = "record value"
VALUE_NEW = "RecordValueNew"
EMPTY_VALUE = ""
TAGS = "{\"tagName1\":\"str1\",\"tagName2\":\"5\",\"tagName3\":\"12\"}"
OPTIONS = json.dumps({"retrieveType": True, "retrieveValue": True, "retrieveTags": True})
TAGS_EMPTY = ""
TAGS_EMPTY_JSON = "{}"
TAGS_MALFORMED_JSON = "{\"e\":}"
QUERY_JSON = {"tagName1": "str1"}
SEARCHED_RECORD = {
  "id": "RecordId",
  "type": None,
  "value": "RecordValue",
  "tags": TAGS
}


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_get_token_info():
    info = await Wallet.get_token_info(0)
    assert info


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_send_tokens():
    receipt = await Wallet.send_tokens(0,1,"address")
    assert receipt


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_create_payment_address():
    address = await Wallet.create_payment_address()
    assert address


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_create_payment_address_with_seed():
    address = await Wallet.create_payment_address("0000000000000000000000WHATEVER00")
    assert address


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_sign_with_address():
    sig = await Wallet.sign_with_address("pay:null:test", "test_data")
    assert sig

@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_verify_with_address():
    valid = await Wallet.verify_with_address("pay:null:test", "test_data", "test_sig")
    assert valid

@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_validate_payment_address():
    await Wallet.validate_payment_address('sov:1:1234')


@pytest.mark.asyncio
@pytest.mark.usefixtures('vcx_init_test_mode')
async def test_wallet_storage():
    await Wallet.add_record(TYPE, ID, VALUE, TAGS)

    await Wallet.update_record_value(TYPE, ID, VALUE_NEW)
    await Wallet.update_record_tags(TYPE, ID, TAGS_EMPTY_JSON)
    await Wallet.add_record_tags(TYPE, ID, TAGS)
    await Wallet.delete_record_tags(TYPE, ID, ['one', 'two'])
    await Wallet.delete_record(TYPE, ID)
    record = {
        "id": ID,
        "type": TYPE,
        "value": VALUE,
        "tags": None,
    }
    assert (json.loads(await Wallet.get_record(TYPE, ID, OPTIONS)) == record)


@pytest.mark.asyncio
async def test_wallet_search():
    search_handle = await Wallet.open_search(TYPE, QUERY_JSON, None)
    assert (search_handle == 1)
    searched_record = await Wallet.search_next_records(search_handle, 1)
    assert (json.loads(searched_record) == SEARCHED_RECORD)
    await Wallet.close_search(search_handle)

    with pytest.raises(VcxError) as e:
        await Wallet.export("/tmp/output.wallet", "backupKey")


@pytest.mark.asyncio
async def test_import_wallet_failures(vcx_init_test_mode, cleanup):
    with pytest.raises(VcxError) as e:
        await Wallet.import_wallet('Invalid Json')
    assert ErrorCode.InvalidJson == e.value.error_code
    cleanup(True)

    config = {'wallet_name': 'IO_ERROR', 'wallet_key': '', 'exported_wallet_path': '', 'backup_key': ''}
    with pytest.raises(VcxError) as e:
        await Wallet.import_wallet(json.dumps(config))
    assert ErrorCode.IOError == e.value.error_code
    cleanup(True)

    config = {'wallet_key': '', 'exported_wallet_path': '', 'backup_key': ''}
    with pytest.raises(VcxError) as e:
        await Wallet.import_wallet(json.dumps(config))
    assert ErrorCode.InvalidJson == e.value.error_code
    cleanup(True)

    config = {'wallet_name': '', 'exported_wallet_path': '', 'backup_key': ''}
    with pytest.raises(VcxError) as e:
        await Wallet.import_wallet(json.dumps(config))
    assert ErrorCode.InvalidJson == e.value.error_code
    cleanup(True)

    config = {'wallet_name': '', 'wallet_key': '', 'backup_key': ''}
    with pytest.raises(VcxError) as e:
        await Wallet.import_wallet(json.dumps(config))
    assert ErrorCode.InvalidJson == e.value.error_code
    cleanup(True)

    config = {'wallet_name': '', 'wallet_key': '', 'exported_wallet_path': ''}
    with pytest.raises(VcxError) as e:
        await Wallet.import_wallet(json.dumps(config))
    assert ErrorCode.InvalidJson == e.value.error_code
    cleanup(True)


