"""
The basic object of the VCX API.  Represents a pairwise relationship with another identity owner.  Once the
relationship, or connection, is established communication can happen securely and privately.  Credentials and
proofs are exchanged using this object.

TODO: document attributes
"""
from typing import Optional
from ctypes import *
from vcx.common import do_call, create_cb
from vcx.api.vcx_stateful import VcxStateful

import json


class Connection(VcxStateful):
    """
    The object of the VCX API representing a pairwise relationship with another identity owner.
    Once the relationship, or connection, is established communication can happen securely and privately.
    Credentials and Proofs are exchanged using this object.

    # States

    The set of object states and transitions depends on communication method is used.
    The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.

    proprietary:
        Inviter:
            VcxStateType::VcxStateInitialized - once `vcx_connection_create` (create Connection object) is called.

            VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (send Connection invite) is called.

            VcxStateType::VcxStateAccepted - once `connReqAnswer` messages is received.
                                             use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called.

        Invitee:
            VcxStateType::VcxStateRequestReceived - once `vcx_connection_create_with_invite` (create Connection object with invite) is called.

            VcxStateType::VcxStateAccepted - once `vcx_connection_connect` (accept Connection invite) is called.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called.

    aries:
        Inviter:
            VcxStateType::VcxStateInitialized - once `vcx_connection_create` (create Connection object) is called.

            VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (prepared Connection invite) is called.

            VcxStateType::VcxStateRequestReceived - once `ConnectionRequest` messages is received.
                                                    accept `ConnectionRequest` and send `ConnectionResponse` message.
                                                    use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateAccepted - once `Ack` messages is received.
                                             use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
                                            OR
                                        `ConnectionProblemReport` messages is received on state updates.

        Invitee:
            VcxStateType::VcxStateOfferSent - once `vcx_connection_create_with_invite` (create Connection object with invite) is called.

            VcxStateType::VcxStateRequestReceived - once `vcx_connection_connect` (accept `ConnectionInvite` and send `ConnectionRequest` message) is called.

            VcxStateType::VcxStateAccepted - once `ConnectionResponse` messages is received.
                                             send `Ack` message if requested.
                                             use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
                                            OR
                                        `ConnectionProblemReport` messages is received on state updates.

    # Transitions

    proprietary:
        Inviter:
            VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized
            VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent
            VcxStateType::VcxStateOfferSent - received `connReqAnswer` - VcxStateType::VcxStateAccepted
            any state - `vcx_connection_delete_connection` - `VcxStateType::VcxStateNone`

        Invitee:
            VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateRequestReceived - `vcx_connection_connect` - VcxStateType::VcxStateAccepted
            any state - `vcx_connection_delete_connection` - `VcxStateType::VcxStateNone`

    aries - RFC: https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        Inviter:
            VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized

            VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent

            VcxStateType::VcxStateOfferSent - received `ConnectionRequest` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateRequestReceived - received `Ack` - VcxStateType::VcxStateAccepted
            VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted

            any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone


        Invitee:
            VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateOfferSent

            VcxStateType::VcxStateOfferSent - `vcx_connection_connect` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateRequestReceived - received `ConnectionResponse` - VcxStateType::VcxStateAccepted
            VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted

            any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone

    # Messages

    proprietary:
        ConnectionRequest (`connReq`)
        ConnectionRequestAnswer (`connReqAnswer`)

    aries:
        Invitation - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
        ConnectionRequest - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#1-connection-request
        ConnectionResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#2-connection-response
        ConnectionProblemReport - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#error-message-example
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
        Ping - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
        PingResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
        Query - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#query-message-type
        Disclose - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#disclose-message-type

    TODO: document attributes
    """

    def __init__(self, source_id: str):
        VcxStateful.__init__(self, source_id)

    def __del__(self):
        self.release()
        self.logger.debug("Deleted {} obj: {}".format(Connection, self.handle))

    @staticmethod
    async def create(source_id: str):
        """
        Create a connection object, represents a single endpoint and can be used for sending and receiving
        credentials and proofs

        :param source_id: Institution's unique ID for the connection
        :return: connection object
        Example:
        connection = await Connection.create(source_id)
        """
        constructor_params = (source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_params = (c_source_id,)

        return await Connection._create( "vcx_connection_create",
                                        constructor_params,
                                        c_params)

    @staticmethod
    async def create_with_details(source_id: str, invite_details: str):
        """
        Create a connection object with a provided invite, represents a single endpoint and can be used for sending and receiving
        credentials and proofs

        Invite details are provided by the entity offering a connection and generally pulled from a provided QRCode.
        :param source_id: Institution's unique ID for the connection
        :param invite_details: A string representing a json object which is provided by an entity that wishes to make a connection.
            Invite format depends on communication method:
                proprietary:
                    {"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}
                aries: https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
                 {
                    "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
                    "label": "Alice",
                    "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
                    "serviceEndpoint": "https://example.com/endpoint",
                    "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]
                 }

        Example:
        connection2 = await Connection.create_with_details('MyBank', invite_details)
        :return: connection object
        """
        constructor_params = (source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_invite_details = c_char_p(invite_details.encode('utf-8'))

        c_params = (c_source_id, c_invite_details, )

        return await Connection._create( "vcx_connection_create_with_invite",
                                        constructor_params,
                                        c_params)

    @staticmethod
    async def create_test_connection(label: str, did: str, verkey: str, endpoint: str):
        """
        Create a connection object in completed state based on provided remote details.
        Once object is created it's already ready for message exchange and no addition connection steps are required.
        This method can be used for testing purposes to skip connection protocol steps.
        Note: It's not recommended to use this method for real connection establishment.
        Note: This method can be used for `aries` communication method only.
            For other communication method it returns ActionNotSupported error.
        :param label: Institution's unique ID for the connection
        :param did: DID of remote side
        :param verkey: Recipient Verkey of remote side
        :param endpoint: Endpoint of remote side
        :return: no value
        """
        constructor_params = ("test",)

        c_did = c_char_p(did.encode('utf-8'))
        c_verkey = c_char_p(verkey.encode('utf-8'))
        c_label = c_char_p(label.encode('utf-8'))
        c_endpoint = c_char_p(endpoint.encode('utf-8'))

        c_params = (c_label, c_did, c_verkey, c_endpoint,)

        return await Connection._create("vcx_connection_create_test_connection",
                                        constructor_params,
                                        c_params)

    @staticmethod
    async def deserialize(data: dict):
        """
        Takes a json string representing a connection object and recreates an object matching the json.

        :param data: json string representing a connection object. Is an output of `serialize` function.
        Example:
        data = await connection1.serialize()
        connection2 = await Connection.deserialize(data)
        :return: A re-instantiated object
        """
        return await Connection._deserialize("vcx_connection_deserialize",
                                             json.dumps(data),
                                             data.get('source_id'))

    async def connect(self, options: str) -> str:
        """
        Connect securely and privately to the endpoint represented by the object.

        :param options: detailed connection options
        Example options:
        {"connection_type":"SMS","phone":"5555555555","use_public_did":true}
        or:
        {"connection_type":"QR"}
        Example code:
        connection = await Connection.create('Sally')
        invite_details = await connection.connect('{"connection_type":"QR"}')
        :return: the invite details sent via SMS or ready to be sent via some other mechanism (QR for example)
        """
        if not hasattr(Connection.connect, "cb"):
            self.logger.debug("vcx_connection_connect: Creating callback")
            Connection.connect.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)
        c_connection_data = c_char_p(options.encode('utf-8'))
        invite_details = await do_call('vcx_connection_connect',
                                       c_connection_handle,
                                       c_connection_data,
                                       Connection.connect.cb)
        return invite_details

    async def redirect(self, redirect_to) -> None:
        """
        Connect securely and privately to the endpoint represented by the object.

        :param redirect_to: Existing connection to redirect to

        Example code:
        connection = await Connection.create_with_details('Sally', invite_details)
        await connection.redirect(old_connection)
        """
        if not hasattr(Connection.redirect, "cb"):
            self.logger.debug("vcx_connection_redirect: Creating callback")
            Connection.redirect.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_connection_handle = c_uint32(self.handle)
        c_redirect_handle = c_uint32(redirect_to.handle)
        await do_call('vcx_connection_redirect',
                      c_connection_handle,
                      c_redirect_handle,
                      Connection.redirect.cb)

    async def get_redirect_details(self) -> str:
        if not hasattr(Connection.get_redirect_details, "cb"):
            self.logger.debug("vcx_connection_get_redirect_details: Creating callback")
            Connection.get_redirect_details.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)
        result = await do_call('vcx_connection_get_redirect_details',
                               c_connection_handle,
                               Connection.get_redirect_details.cb)

        self.logger.debug("vcx_connection_get_redirect_details completed")
        return json.loads(result.decode())

    async def send_message(self, msg: str, msg_type: str, msg_title: str, ref_msg_id: str = None) -> str:
        """
            Send a generic message to the connection
            :param msg: actual message to send
            :param msg_type: type of message to send. can be any string.
            :param msg_title: message title (user notification)
            :param ref_msg_id: if responding to a message, provide msg id

            Example options:
            msg: "HI" or "{"key": "value"}" or "{ "@type": "type", "@id": "518be002-de8e-456e-b3d5-8fe472477a86", "comment": "Hi. Are you listening?"}"
            msg_type: "Greeting"
            msg_title: "Hi There"

            Example code:
            connection = await Connection.create_with_details('MyBank', invite_details)
            await connection.send_message("HI", "Greeting", "Hi There")

            :return: response message
            """
        if not hasattr(Connection.send_message, "cb"):
            self.logger.debug("vcx_connection_send_message: Creating callback")
            Connection.send_message.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        send_msg_options = {
            "msg_type": msg_type,
            "msg_title": msg_title,
            "ref_msg_id": ref_msg_id
        }
        c_connection_handle = c_uint32(self.handle)
        c_msg = c_char_p(msg.encode('utf-8'))
        c_send_msg_options = c_char_p(json.dumps(send_msg_options).encode('utf-8'))

        result = await do_call('vcx_connection_send_message',
                               c_connection_handle,
                               c_msg,
                               c_send_msg_options,
                               Connection.send_message.cb)

        self.logger.debug("vcx_connection_send_message completed")
        return result

    async def sign_data(self, msg: bytes) -> bytes:
        """
        Sign data using connection's pairwise key
        :param msg: message to sign represented as bytes
        :return: signature
        """

        def transform_cb(arr_ptr: POINTER(c_uint8), arr_len: c_uint32):
            return bytes(arr_ptr[:arr_len]),

        if not hasattr(Connection.sign_data, "cb"):
            self.logger.debug("vcx_connection_sign_data: Creating callback")
            Connection.sign_data.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, POINTER(c_uint8), c_uint32), transform_cb)

        c_connection_handle = c_uint32(self.handle)
        c_msg_len = c_uint32(len(msg))

        result = await do_call('vcx_connection_sign_data',
                               c_connection_handle,
                               msg,
                               c_msg_len,
                               Connection.sign_data.cb)

        self.logger.debug("vcx_connection_sign_data completed")
        return result

    async def verify_signature(self, msg: bytes, signature: bytes) -> bool:
        """
        Verify the signature is valid for the specified data using connection pairwise keys
        :param msg: message was signed
        :param signature: generated signature
        :return: bool - whether the signature was valid or not
        """
        if not hasattr(Connection.verify_signature, "cb"):
            self.logger.debug("vcx_connection_verify_signature: Creating callback")
            Connection.verify_signature.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_bool))

        c_connection_handle = c_uint32(self.handle)
        c_msg_len = c_uint32(len(msg))
        c_signature_len = c_uint32(len(signature))

        result = await do_call('vcx_connection_verify_signature',
                               c_connection_handle,
                               msg,
                               c_msg_len,
                               signature,
                               c_signature_len,
                               Connection.verify_signature.cb)

        self.logger.debug("vcx_connection_verify_signature completed")
        return result

    async def _delete(self):
        if not hasattr(Connection._delete, "cb"):
            Connection._delete.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_connection_handle = c_uint32(self.handle)
        return await do_call('vcx_connection_delete_connection', c_connection_handle, Connection._delete.cb)

    async def serialize(self) -> dict:
        """
        Takes the Connection object and returns a json string of all its attributes
        Example:
        data = await connection1.serialize()
        :return: serialized object
        """
        return await self._serialize(Connection, 'vcx_connection_serialize')

    async def update_state(self) -> int:
        """
        Query the agency for the received messages.
        Checks for any messages changing state in the connection object and updates the state attribute.
        Example:
        connection = await Connection.create(source_id)
        assert await connection.update_state() == State.Initialized
        :return: Current state of the connection. Possible states:
                                                     1 - Initialized
                                                     2 - Request Sent
                                                     3 - Offer Received
                                                     4 - Accepted
        """
        return await self._update_state(Connection, 'vcx_connection_update_state')

    async def update_state_with_message(self, message: str) -> int:
        """
        Update the state of the connection based on the given message.
        :param msg: message to process
        Example:
        connection = await Connection.create(source_id)
        assert await connection.update_state_with_message(message) == State.Accepted
        :param message: message to process for state changes
        :return Current state of the connection. Possible states:
                                                    1 - Initialized
                                                    2 - Request Sent
                                                    3 - Offer Received
                                                    4 - Accepted
        """
        return await self._update_state_with_message(Connection, message, 'vcx_connection_update_state_with_message')

    async def get_state(self) -> int:
        """
        Returns the current internal state of the connection. Does NOT query agency for state updates.
        Possible states:
            1 - Initialized
            2 - Offer Sent
            3 - Request Received
            4 - Accepted
        Example:
        connection = await Connection.create(source_id)
        assert await connection.get_state() == State.Initialized
        :return:  Current internal state of the connection
        """
        return await self._get_state(Connection, 'vcx_connection_get_state')

    def release(self) -> None:
        """
        destroy the object and release any memory associated with it

        :return: None
        """
        self._release(Connection, 'vcx_connection_release')

    async def delete(self):
        """
        Delete the object from the agency and release any memory associated with it.

        NOTE: This eliminates the connection and any ability to use it for any communication.

        Example:
        connection = await Connection.create(source_id)
        await connection.delete()
        :return: None
        """
        await self._delete()

    async def invite_details(self, abbreviated: bool) -> dict:
        """
        Get the invite details that were sent or can be sent to the remote side.

        :param abbreviated: abbreviate invite details or not (applicable for `proprietary` communication method only)
        Example:
        phone_number = '8019119191'
        connection = await Connection.create('foobar123')
        invite_details = await connection.connect(phone_number)
        inivte_details_again = await connection.invite_details()
        :return: JSON of invite_details sent to connection
            Invite format depends on communication method:
                proprietary:
                    {"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}
                aries: https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
                 {
                    "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
                    "label": "Alice",
                    "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
                    "serviceEndpoint": "https://example.com/endpoint",
                    "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]
                 }

        """
        if not hasattr(Connection.invite_details, "cb"):
            self.logger.debug("vcx_connection_invite_details: Creating callback")
            Connection.invite_details.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)
        c_abbreviated = c_bool(abbreviated)

        details = await do_call('vcx_connection_invite_details',
                                c_connection_handle,
                                c_abbreviated,
                                Connection.invite_details.cb)

        return json.loads(details.decode())


    async def send_ping(self, comment: Optional[str] = None):
        """
        Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.

        Note that this function is useful in case `aries` communication method is used.
        In other cases it returns ActionNotSupported error.
        :param comment: (Optional) human-friendly description of the ping.

        :return: no value
        """
        if not hasattr(Connection.send_ping, "cb"):
            self.logger.debug("vcx_connection_send_ping: Creating callback")
            Connection.send_ping.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_connection_handle = c_uint32(self.handle)
        c_comment = c_char_p(comment.encode('utf-8')) if comment is not None else None

        await do_call('vcx_connection_send_ping',
                      c_connection_handle,
                      c_comment,
                      Connection.send_ping.cb)


    async def send_discovery_features(self, query: Optional[str] = None, comment: Optional[str] = None):
        """
        Send discovery features message to the specified connection to discover which features it supports, and to what extent.

        Note that this function is useful in case `aries` communication method is used.
        In other cases it returns ActionNotSupported error.

        :param query: (Optional) query string to match against supported message types.
        :param comment: (Optional) human-friendly description of the ping.
        :return: no value
        """
        if not hasattr(Connection.send_discovery_features, "cb"):
            self.logger.debug("vcx_connection_send_discovery_features: Creating callback")
            Connection.send_discovery_features.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_connection_handle = c_uint32(self.handle)
        c_query = c_char_p(query.encode('utf-8')) if query is not None else None
        c_comment = c_char_p(comment.encode('utf-8')) if comment is not None else None

        await do_call('vcx_connection_send_discovery_features',
                      c_connection_handle,
                      c_query,
                      c_comment,
                      Connection.send_discovery_features.cb)

    async def get_my_pw_did(self) -> str:
        if not hasattr(Connection.get_my_pw_did, "cb"):
            self.logger.debug("get_my_pw_did: Creating callback")
            Connection.get_my_pw_did.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)

        my_pw_did = await do_call('vcx_connection_get_pw_did', c_connection_handle, Connection.get_my_pw_did.cb)
        return my_pw_did.decode()

    async def get_their_pw_did(self) -> str:
        if not hasattr(Connection.get_their_pw_did, "cb"):
            self.logger.debug("get_their_pw_did: Creating callback")
            Connection.get_their_pw_did.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)

        their_pw_did =\
            await do_call('vcx_connection_get_their_pw_did', c_connection_handle, Connection.get_their_pw_did.cb)
        return their_pw_did.decode()

    async def info(self) -> dict:
        """
        Get the information regarding the connection object.
        Note: This method can be used for `aries` communication method only.
            For other communication method it returns ActionNotSupported error.
        :return: JSON of connection information
        {
            "current": {
                "did": <str>
                "recipientKeys": array<str>
                "routingKeys": array<str>
                "serviceEndpoint": <str>,
                "protocols": array<str> -  The set of protocol supported by current side.
            },
            "remote: { <Option> - details about remote connection side
                "did": <str> - DID of remote side
                "recipientKeys": array<str> - Recipient keys
                "routingKeys": array<str> - Routing keys
                "serviceEndpoint": <str> - Endpoint
                "protocols": array<str> -
                    The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
             }
        }
        """
        if not hasattr(Connection.info, "cb"):
            self.logger.debug("vcx_connection_info: Creating callback")
            Connection.info.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)

        details = await do_call('vcx_connection_info',
                                c_connection_handle,
                                Connection.info.cb)

        return json.loads(details.decode())
