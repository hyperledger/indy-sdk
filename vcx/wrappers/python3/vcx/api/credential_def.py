from typing import Optional
from ctypes import *
from vcx.error import VcxError, ErrorCode
from vcx.api.vcx_base import VcxBase
from vcx.common import do_call, create_cb
from vcx.api.vcx_stateful import VcxStateful

import json


class CredentialDef(VcxStateful):
    """
    Object that represents a credential definition written on the ledger.

    Attributes:
        source_id: user generated unique identifier
        schema_id: the ledger ID of the schema
        name: name of the credential definition
        transaction: credential def transaction that must be published to the ledger by Endorser
    """


    def __init__(self, source_id: str, name: str, schema_id: str, transaction: Optional[str] = None):
        VcxStateful.__init__(self, source_id)
        self._source_id = source_id
        self._schema_id = schema_id
        self._name = name
        self._transaction = transaction


    def __del__(self):
        self.release()
        self.logger.debug("Deleted {} obj: {}".format(CredentialDef, self.handle))

    @property
    def name(self):
        return self._name

    @name.setter
    def name(self, x):
        self._name = x

    @property
    def schema_id(self):
        return self._schema_id

    @schema_id.setter
    def schema_id(self, x):
        self._schema_id = x


    @property
    def transaction(self):
        return self._transaction


    @transaction.setter
    def transaction(self, x):
        self._transaction = x


    @staticmethod
    async def create(source_id: str, name: str, schema_id: str, payment_handle: int, revocation_details: dict):
        """
        Creates a new CredentialDef object that is written to the ledger

        :param source_id: Institution's unique ID for the credential definition
        :param name: Name of credential definition
        :param schema_id: The schema ID given during the creation of the schema
        :param payment_handle: NYI - payment of ledger fee is taken from wallet automatically
        :param revocation_details: setting up revocation registry
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        endorser = 'V4SGRU86Z58d6TV7PBUe6f'
        revocation_details = {"support_revocation": True, "tails_file": "/tmp/tails_py", "max_creds": 5} or {"support_revocation": False}
        credential_def1 = await CredentialDef.prepare_for_endorser(source_id, name, schema_id, endorser)
        :return: credential_def object,
        credential_def transaction that should be passed to Endorser for witting to ledger
        """

        constructor_params = (source_id, name, schema_id)
        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_schema_id = c_char_p(schema_id.encode('utf-8'))
        c_name = c_char_p(name.encode('utf-8'))
        # default institution_did in config is used as issuer_did
        c_issuer_did = None
        c_payment = c_uint32(payment_handle)
        # Todo: add params for tag and config
        c_tag = c_char_p('tag1'.encode('utf-8'))
        c_config = c_char_p(json.dumps(revocation_details).encode('utf-8'))
        c_params = (c_source_id, c_name, c_schema_id, c_issuer_did, c_tag, c_config, c_payment)

        return await CredentialDef._create("vcx_credentialdef_create",
                                           constructor_params,
                                           c_params)


    @staticmethod
    async def prepare_for_endorser(source_id: str, name: str, schema_id: str, endorser: str, revocation_details: dict):
        """
        Create a new CredentialDef object that will be published on the ledger by Endorser later.
        
        Note that CredentialDef can't be used for credential issuing until it will be published on the ledger.

        :param source_id: Institution's unique ID for the credential definition
        :param name: Name of credential definition
        :param schema_id: The schema ID given during the creation of the schema
        :param endorser: DID of the Endorser that will submit the transaction.
        :param revocation_details: setting up revocation registry
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        revocation_details = {"support_revocation": True, "tails_file": "/tmp/tails_py", "max_creds": 5} or {"support_revocation": False}
        credential_def1 = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        :return: credential_def object, written to ledger
        """
        try:
            credentialdef = CredentialDef(source_id, '', '')

            if not hasattr(CredentialDef.prepare_for_endorser, "cb"):
                credentialdef.logger.debug("vcx_prepare_for_endorser: Creating callback")
                CredentialDef.prepare_for_endorser.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint32,
                                                                            c_char_p, c_char_p, c_char_p))
            
            c_source_id = c_char_p(source_id.encode('utf-8'))
            c_name = c_char_p(name.encode('utf-8'))
            c_schema_id = c_char_p(schema_id.encode('utf-8'))
            c_endorser = c_char_p(endorser.encode('utf-8'))
            c_issuer_did = None
            c_tag = c_char_p('tag1'.encode('utf-8'))
            c_config = c_char_p(json.dumps(revocation_details).encode('utf-8'))

            handle, transaction, _, _ = await do_call('vcx_credentialdef_prepare_for_endorser',
                                                      c_source_id,
                                                      c_name,
                                                      c_schema_id,
                                                      c_issuer_did,
                                                      c_tag,
                                                      c_config,
                                                      c_endorser,
                                                      CredentialDef.prepare_for_endorser.cb)
            credentialdef.logger.debug("created credential def object")

            credentialdef.name = name
            credentialdef.handle = handle
            credentialdef.schema_id = schema_id
            credentialdef.transaction = transaction
            return credentialdef
        except KeyError:
            raise VcxError(ErrorCode.InvalidCredentialDef)


    async def serialize(self) -> dict:
        """
        Serialize the object for storage

        :return: serialized object
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def1 = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        data1 = await credential_def1.serialize()
        :return: A re-instantiated object
        """
        return await self._serialize(CredentialDef, 'vcx_credentialdef_serialize')

    @staticmethod
    async def deserialize(data: dict):
        """
        Create the object from a previously serialized object.

        :param data: The output of the "serialize" call
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def1 = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        data1 = await credential_def1.serialize()
        credential_def2 = await CredentialDef.deserialize(data1)
        :return: A re-instantiated object
        """
        try:
            credential_def = await CredentialDef._deserialize("vcx_credentialdef_deserialize",
                                                              json.dumps(data),
                                                              data['data']['source_id'],
                                                              data['data']['name'],
                                                              data['data']['id'])
            return credential_def
        except KeyError:
            raise VcxError(ErrorCode.InvalidCredentialDef)


    async def get_cred_def_id(self):
        """
        Get the ledger ID of the object

        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def1 = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        assert await credential_def.get_cred_def_id() == '2hoqvcwupRTUNkXn6ArYzs:3:CL:2471'
        :return: ID string
        """
        cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))
        c_handle = c_uint32(self.handle)
        cred_def_id = await do_call('vcx_credentialdef_get_cred_def_id', c_handle, cb)
        return cred_def_id.decode()

    def release(self) -> None:
        """
        destroy the object and release any memory associated with it
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def1 = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        credential_def1.release()
        :return: None
        """
        self._release(CredentialDef, 'vcx_credentialdef_release')

    async def get_payment_txn(self):
        """
        Get the payment transaction information generated when paying the ledger fee

        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def1 = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        txn = await credential_def1.get_payment_txn()
        :return: JSON object with input address and output UTXOs
         {
             "amount":25,
             "inputs":[
                 "pay:null:1_3FvPC7dzFbQKzfG"
             ],
             "outputs":[
                 {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null}
             ]
         }
        """
        if not hasattr(CredentialDef.get_payment_txn, "cb"):
            self.logger.debug("vcx_credentialdef_get_payment_txn: Creating callback")
            CredentialDef.get_payment_txn.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_credential_handle = c_uint32(self.handle)

        payment_txn = await do_call('vcx_credentialdef_get_payment_txn',
                                    c_credential_handle,
                                    CredentialDef.get_payment_txn.cb)

        return json.loads(payment_txn.decode())


    async def update_state(self) -> int:
        """
        Checks if credential definition is published on the Ledger and updates the state.
        Possible states:
             0 = Built
             1 = Published
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        assert await credential_def.update_state() == PublicEntityState.Published
        :return: Current state of the credential definition
        """
        return await self._update_state(CredentialDef, 'vcx_credentialdef_update_state')


    async def get_state(self) -> int:
        """
        Get the current state of the credential definition object
        Possible states:
             0 = Built
             1 = Published
        Example:
        source_id = 'foobar123'
        schema_name = 'Schema Name'
        payment_handle = 0
        credential_def = await CredentialDef.create(source_id, name, schema_id, payment_handle)
        assert await credential_def.get_state() == PublicEntityState.Published
        :return:  Current state of the credential definition
        """
        return await self._get_state(CredentialDef, 'vcx_credentialdef_get_state')
