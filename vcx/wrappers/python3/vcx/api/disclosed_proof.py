from typing import Optional
from ctypes import *
from vcx.common import do_call, create_cb
from vcx.api.connection import Connection
from vcx.api.vcx_stateful import VcxStateful

import json


class DisclosedProof(VcxStateful):
    """
    The object of the VCX API representing a Prover side in the credential presentation process.
    Assumes that pairwise connection between Verifier and Prover is already established.

    # State

    The set of object states and transitions depends on communication method is used.
    The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.

    proprietary:
        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.

        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` (send `PROOF_REQ` message) is called.

        VcxStateType::VcxStateAccepted - once `vcx_disclosed_proof_send_proof` (send `PROOF_REQ` message) is called.

    aries:
        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.

        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.

        VcxStateType::VcxStateOfferSent - once `vcx_disclosed_proof_send_proof` (send `Presentation` message) is called.
        VcxStateType::None - once `vcx_disclosed_proof_decline_presentation_request` (send `PresentationReject` or `PresentationProposal` message) is called.

        VcxStateType::VcxStateAccepted - once `Ack` messages is received.
        VcxStateType::None - once `ProblemReport` messages is received.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_decline_presentation_request` - VcxStateType::None

        VcxStateType::VcxStateOfferSent - received `Ack` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None

    # Messages

    proprietary:
        ProofRequest (`PROOF_REQ`)
        Proof (`PROOF`)

    aries:
        PresentationRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#request-presentation
        Presentation - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#presentation
        PresentationProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
    """

    def __init__(self, source_id: str):
        VcxStateful.__init__(self, source_id)
        self._name = source_id
        self._proof_req = None

    def __del__(self):
        self.release()
        self.logger.debug("Deleted {} obj: {}".format(DisclosedProof, self.handle))

    @property
    def proof_request(self):
        return self._proof_req

    @proof_request.setter
    def proof_request(self, x):
        self._proof_req = x

    @staticmethod
    async def create(source_id: str, proof_request: str):
        """
        Create a proof for fulfilling a corresponding proof request
        :param source_id: Tag associated by user of sdk
        :param proof_request: Proof Request data sent by requester.
        Example:
        source_id = 'sourceId'
        request = {
            "@topic": {
            "mid": 9,
            "tid": 1
            },
            "@type": {
                "name": "PROOF_REQUEST",
                "version":"1.0"
            },
            "msg_ref_id": "ymy5nth",
            "proof_request_data": {
                "name": "Account Certificate",
                "nonce": "838186471541979035208225",
                "requested_attributes": {
                    "business_2": {
                        "name": "business"
                    },
                    "email_1": {
                        "name": "email"
                    },
                    "name_0": {
                        "name": "name"
                    }
                },
                "requested_predicates": {},
                "version": "0.1"
            }
        }
        disclosed_proof = await DisclosedProof.create(source_id, request)
        :return: Disclosed Proof Object
        """
        constructor_params = (source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_proof_request = c_char_p(json.dumps(proof_request).encode('utf-8'))
        c_params = (c_source_id, c_proof_request, )

        return await DisclosedProof._create("vcx_disclosed_proof_create_with_request",
                                   constructor_params,
                                   c_params)

    @staticmethod
    async def create_with_msgid(source_id: str, connection: Connection, msg_id: str):
        """
        Create a proof based off of a known message id for a given connection.

        :param source_id: user defined id of object.
        :param connection: connection to receive proof request from
        :param msg_id: id of the message that contains the proof request
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        :return: DisclosedProof
        """
        proof = DisclosedProof(source_id)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_msg_id = c_char_p(msg_id.encode('utf-8'))
        c_connection_handle = c_uint32(connection.handle)

        if not hasattr(DisclosedProof.create_with_msgid, "cb"):
            DisclosedProof.create_with_msgid.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint32, c_char_p))

        proof.handle, proof_req = await do_call('vcx_disclosed_proof_create_with_msgid',
                                                c_source_id,
                                                c_connection_handle,
                                                c_msg_id,
                                                DisclosedProof.create_with_msgid.cb)

        proof.proof_request = json.loads(proof_req.decode())

        return proof

    @staticmethod
    async def deserialize(data: dict):
        """
        Create a DisclosedProof object from a previously serialized proof object
        :param data: JSON data from a serialized object.
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        data = await disclosed_proof.serialize()
        disclosed_proof2 = await DisclosedProof.deserialize(data)
        :return:  DisclosedProof
        """
        disclosed_proof = await DisclosedProof._deserialize("vcx_disclosed_proof_deserialize",
                                                      json.dumps(data),
                                                      data.get('data').get('source_id'))
        return disclosed_proof

    @staticmethod
    async def get_requests(connection: Connection) -> dict:
        """
        Retrieves all pending proof requests for a given connection.

        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        requests = await DisclosedProof.get_requests(connection)
        :param connection: Connection
        :return: requests associated with the connection
            [{'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}]
        """
        if not hasattr(DisclosedProof.get_requests, "cb"):
            DisclosedProof.get_requests.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(connection.handle)

        data = await do_call('vcx_disclosed_proof_get_requests',
                      c_connection_handle,
                      DisclosedProof.get_requests.cb)

        return json.loads(data.decode())

    async def serialize(self) -> dict:
        """
        Serializes the proof object for storage and later deserialization.
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        data = await disclosed_proof.serialize()
        :return:
        """
        return await self._serialize(DisclosedProof, 'vcx_disclosed_proof_serialize')

    async def update_state(self) -> int:
        """
        Query the agency for the received messages.
        Checks for any messages changing state in the proof object and updates the state attribute.

        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        assert await disclosed_proof.update_state() == State.RequestReceived
        :return:
        """
        return await self._update_state(DisclosedProof, 'vcx_disclosed_proof_update_state')

    async def update_state_with_message(self, message: str) -> int:
        """
        Update the state of the proof based on the given message.
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        assert await disclosed_proof.update_state_with_message(msg) == State.RequestReceived
        :param message: - message to process for state changes
        :return:
        """
        return await self._update_state_with_message(DisclosedProof, message, 'vcx_disclosed_proof_update_state_with_message')

    async def get_state(self) -> int:
        """
        Get the current state of the credential object
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        assert await proof.get_state() == State.Initialized
        :return: credential state of the object. Possible states:
                                                     3 - Request Received
                                                     4 - Accepted
        """
        return await self._get_state(DisclosedProof, 'vcx_disclosed_proof_get_state')

    def release(self) -> None:
        """
        Internal method used for memory management
        :return: None
        """
        self._release(DisclosedProof, 'vcx_disclosed_proof_release')

    async def get_creds(self) -> dict:
        """
        Get credentials from wallet matching to the proof request associated with proof object
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        creds = await disclosed_proof.get_creds()
        :return: credentials
            {'attrs': {'attribute_0': [{'cred_info': {'schema_id': 'id', 'cred_def_id': 'id', 'attrs': {'attr_name': 'attr_value', ...}, 'referent': '914c7e11'}}]}}
        """
        if not hasattr(DisclosedProof.get_creds, "cb"):
            self.logger.debug("vcx_disclosed_proof_retrieve_credentials: Creating callback")
            DisclosedProof.get_creds.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_disclosed_proof_handle = c_uint32(self.handle)

        data = await do_call('vcx_disclosed_proof_retrieve_credentials',
                             c_disclosed_proof_handle,
                             DisclosedProof.get_creds.cb)
        return json.loads(data.decode())

    async def send_proof(self, connection: Optional[Connection] = None):
        """
        Sends the proof to the Connection
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        await disclosed_proof.send_proof(connection)
        :param connection: Connection
        :return: None
        """
        if not hasattr(DisclosedProof.send_proof, "cb"):
            self.logger.debug("vcx_disclosed_proof_send_proof: Creating callback")
            DisclosedProof.send_proof.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_disclosed_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle) if connection else 0

        await do_call('vcx_disclosed_proof_send_proof',
                      c_disclosed_proof_handle,
                      c_connection_handle,
                      DisclosedProof.send_proof.cb)

    async def reject_proof(self, connection: Connection):
        """
        Sends the reject of proof to the Connection
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        await disclosed_proof.reject_proof(connection)
        :param connection: Connection
        :return: None
        """
        if not hasattr(DisclosedProof.reject_proof, "cb"):
            self.logger.debug("vcx_disclosed_proof_reject_proof: Creating callback")
            DisclosedProof.reject_proof.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_disclosed_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)

        await do_call('vcx_disclosed_proof_reject_proof',
                      c_disclosed_proof_handle,
                      c_connection_handle,
                      DisclosedProof.reject_proof.cb)

    async def get_send_proof_msg(self):
        """
        Gets the proof message that can be sent to the specified connection
        Example:
        msg = await disclosed_proof.get_send_proof_msg()
        :param
        :return:
        """
        if not hasattr(DisclosedProof.get_send_proof_msg, "cb"):
            self.logger.debug("vcx_proof_send_request: Creating callback")
            DisclosedProof.get_send_proof_msg.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)

        msg = await do_call('vcx_disclosed_proof_get_proof_msg',
                      c_proof_handle,
                      DisclosedProof.get_send_proof_msg.cb)

        return json.loads(msg.decode())

    async def get_reject_proof_msg(self):
        """
        Example:
        msg = await disclosed_proof.get_reject_proof_msg()
        :param
        :return:
        """
        if not hasattr(DisclosedProof.get_reject_proof_msg, "cb"):
            self.logger.debug("vcx_proof_send_request: Creating callback")
            DisclosedProof.get_reject_proof_msg.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)

        msg = await do_call('vcx_disclosed_proof_get_reject_msg',
                      c_proof_handle,
                      DisclosedProof.get_reject_proof_msg.cb)

        return json.loads(msg.decode())

    async def generate_proof(self, selected_creds: dict, self_attested_attrs: dict):
        """
        Accept proof request associated with proof object and generates a proof from the selected credentials and self attested attributes
        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        await disclosed_proof.generate_proof({}, {})
        :param selected_creds: a json with a credential for each proof request attribute.
            List of possible credentials for each attribute is returned from vcx_disclosed_proof_retrieve_credentials,
                (user needs to select specific credential to use from list of credentials)
                {
                    "attrs":{
                        String:{// Attribute key: This may not be the same as the attr name ex. "age_1" where attribute name is "age"
                            "credential": {
                                "cred_info":{
                                    "referent":String,
                                    "attrs":{ String: String }, // ex. {"age": "111", "name": "Bob"}
                                    "schema_id": String,
                                    "cred_def_id": String,
                                    "rev_reg_id":Option<String>,
                                    "cred_rev_id":Option<String>,
                                    },
                                "interval":Option<{to: Option<u64>, from:: Option<u64>}>
                            }, // This is the exact credential information selected from list of
                               // credentials returned from vcx_disclosed_proof_retrieve_credentials
                            "tails_file": Option<"String">, // Path to tails file for this credential
                        },
                   },
                  "predicates":{ TODO: will be implemented as part of IS-1095 ticket. }
               }
           selected_credentials can be empty "{}" if the proof only contains self_attested_attrs
        :param self_attested_attrs: a json with attributes self attested by user
        
        Example:
         self_attested_attrs: {"self_attested_attr_0":"attested_val"} or {}
         selected_credentials -> {'attrs': {'attribute_0': {'credential': {'cred_info': {'cred_def_id': 'od', 'schema_id': 'id', 'referent': '0c212108-9433-4199-a21f-336a44164f38', 'attrs': {'attr_name': 'attr_value', ...}}}}}}

        :return: None
        """
        if not hasattr(DisclosedProof.generate_proof, "cb"):
            self.logger.debug("vcx_disclosed_proof_generate_proof: Creating callback")
            DisclosedProof.generate_proof.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_disclosed_proof_handle = c_uint32(self.handle)
        c_selected_creds = c_char_p(json.dumps(selected_creds).encode('utf-8'))
        c_self_attested_attrs = c_char_p(json.dumps(self_attested_attrs).encode('utf-8'))

        await do_call('vcx_disclosed_proof_generate_proof',
                      c_disclosed_proof_handle,
                      c_selected_creds,
                      c_self_attested_attrs,
                      DisclosedProof.generate_proof.cb)


    async def decline_presentation_request(self, connection: Connection,
                                           reason: Optional[str] = None, proposal: Optional[dict] = None):
        """
        Declines presentation request.
        There are two ways of following interaction:
           - Prover wants to propose using a different presentation - pass `proposal` parameter.
           - Prover doesn't want to continue interaction - pass `reason` parameter.
        Note that only one of these parameters can be passed.

        Note that proposing of different presentation is supported for `aries` protocol only.

        :param connection: Connection
        :param reason: human-readable string that explain the reason of decline
        :param proposal: the proposed format of presentation request
           (see https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#presentation-preview for details)
           {
              "attributes": [
                  {
                      "name": "<attribute_name>",
                      "cred_def_id": Optional("<cred_def_id>"),
                      "mime-type": Optional("<type>"),
                      "value": Optional("<value>")
                  },
                  // more attributes
              ],
              "predicates": [
                  {
                      "name": "<attribute_name>",
                      "cred_def_id": Optional("<cred_def_id>"),
                      "predicate": "<predicate>", - one of "<", "<=", ">=", ">"
                      "threshold": <threshold>
                  },
                  // more predicates
              ]
           }

        Example:
        msg_id = '1'
        phone_number = '8019119191'
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        disclosed_proof = await DisclosedProof.create_with_msgid(source_id, connection, msg_id)
        await disclosed_proof.decline_presentation_request(connection, 'reason', None)
        :return: None
        """
        if not hasattr(DisclosedProof.decline_presentation_request, "cb"):
            self.logger.debug("vcx_disclosed_proof_decline_presentation_request: Creating callback")
            DisclosedProof.decline_presentation_request.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_disclosed_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)
        c_reason = c_char_p(reason.encode('utf-8')) if reason else None
        c_proposal = c_char_p(json.dumps(proposal).encode('utf-8')) if proposal else None

        await do_call('vcx_disclosed_proof_decline_presentation_request',
                      c_disclosed_proof_handle,
                      c_connection_handle,
                      c_reason,
                      c_proposal,
                      DisclosedProof.decline_presentation_request.cb)
