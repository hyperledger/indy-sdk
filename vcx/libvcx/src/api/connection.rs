use libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use utils::threadpool::spawn;
use std::ptr;
use connection::*;
use error::prelude::*;
use messages::get_message::Message;
use indy_sys::CommandHandle;

/*
    Tha API represents a pairwise connection with another identity owner.
    Once the connection, is established communication can happen securely and privately.
    Credentials and Presentations are exchanged using this object.

    # States

    The set of object states, messages and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

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
*/

/// Delete a Connection object from the agency and release its handle.
///
/// NOTE: This eliminates the connection and any ability to use it for any communication.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle of the connection to delete.
///
/// cb: Callback that provides feedback of the api call.
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_delete_connection(command_handle: CommandHandle,
                                               connection_handle: u32,
                                               cb: Option<extern fn(
                                                   xcommand_handle: CommandHandle,
                                                   err: u32)>) -> u32 {
    info!("vcx_delete_connection >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }
    trace!("vcx_connection_delete_connection(command_handle: {}, connection_handle: {})", command_handle, connection_handle);
    spawn(move || {
        match delete_connection(connection_handle) {
            Ok(_) => {
                trace!("vcx_connection_delete_connection_cb(command_handle: {}, rc: {})", command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                trace!("vcx_connection_delete_connection_cb(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Connection object that provides a pairwise connection for an institution's user
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: institution's personal identification for the connection
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_create(command_handle: CommandHandle,
                                    source_id: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create(command_handle: {}, source_id: {})", command_handle, source_id);

    spawn(move || {
        match create_connection(&source_id) {
            Ok(handle) => {
                trace!("vcx_connection_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Connection object from the given invite_details that provides a pairwise connection.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: institution's personal identification for the connection
///
/// invite_details: A string representing a json object which is provided by an entity that wishes to make a connection.
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Examples
/// invite_details -> depends on communication method:
///     proprietary:
///         {"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}
///     aries: https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
///      {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
///         "label": "Alice",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "serviceEndpoint": "https://example.com/endpoint",
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]
///      }
///
/// # Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_create_with_invite(command_handle: CommandHandle,
                                                source_id: *const c_char,
                                                invite_details: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_create_with_invite >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(invite_details, VcxErrorKind::InvalidOption);
    trace!("vcx_connection_create_with_invite(command_handle: {}, source_id: {})", command_handle, source_id);
    spawn(move || {
        match create_connection_with_invite(&source_id, &invite_details) {
            Ok(handle) => {
                trace!("vcx_connection_create_with_invite_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_with_invite_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Establishes connection between institution and its user
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies connection object
///
/// connection_options: Provides details indicating if the connection will be established by text or QR Code
///
/// # Examples connection_options ->
/// "{"connection_type":"SMS","phone":"123","use_public_did":true}"
///     OR:
/// "{"connection_type":"QR","phone":"","use_public_did":false}"
///
/// cb: Callback that provides error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_connect(command_handle: CommandHandle,
                                     connection_handle: u32,
                                     connection_options: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, invite_details: *const c_char)>) -> u32 {
    info!("vcx_connection_connect >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    let options = if !connection_options.is_null() {
        check_useful_opt_c_str!(connection_options, VcxErrorKind::InvalidOption);
        connection_options.to_owned()
    } else {
        None
    };

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_connect(command_handle: {}, connection_handle: {}, connection_options: {:?}), source_id: {:?}",
           command_handle, connection_handle, options, source_id);

    spawn(move || {
        match connect(connection_handle, options) {
            Ok(_) => {
                match get_invite_details(connection_handle, true) {
                    Ok(x) => {
                        trace!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}), source_id: {:?}",
                               command_handle, connection_handle, error::SUCCESS.message, x, source_id);
                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(_) => {
                        warn!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}), source_id: {:?}",
                              command_handle, connection_handle, error::SUCCESS.message, "null", source_id); // TODO: why Success?????
                        cb(command_handle, error::SUCCESS.code_num, ptr::null_mut());
                    }
                }
            }
            Err(x) => {
                warn!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}, source_id: {})",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_redirect(command_handle: CommandHandle,
                                      connection_handle: u32,
                                      redirect_connection_handle: u32,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_connection_redirect >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_redirect - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    if !is_valid_handle(redirect_connection_handle) {
        error!("vcx_connection_redirect - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_redirect(command_handle: {}, connection_handle: {}, redirect_connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, redirect_connection_handle, source_id);

    spawn(move|| {
        match redirect(connection_handle, redirect_connection_handle) {
            Ok(_) => {
                trace!("vcx_connection_redirect_cb(command_handle: {}, rc: {})", command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            },
            Err(e) => {
                trace!("vcx_connection_redirect_cb(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_get_redirect_details(command_handle: CommandHandle,
                                                  connection_handle: u32,
                                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, details: *const c_char)>) -> u32 {
    info!("vcx_connection_get_redirect_details >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_redirect_details(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_redirect_details - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    spawn(move|| {
        match get_redirect_details(connection_handle){
            Ok(str) => {
                trace!("vcx_connection_get_redirect_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, str, source_id);
                let msg = CStringUtils::string_to_cstring(str);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_connection_get_redirect_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}, source_id: {:?})",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the Connection object and returns a json string of all its attributes
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides json string of the connection's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_serialize(command_handle: CommandHandle,
                                       connection_handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_serialize(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    spawn(move || {
        match to_string(connection_handle) {
            Ok(json) => {
                trace!("vcx_connection_serialize_cb(command_handle: {}, connection_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, json, source_id);
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_serialize_cb(command_handle: {}, connection_handle: {}, rc: {}, state: {}), source_id: {:?}",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing a connection object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_data: json string representing a connection object. Is an output of `vcx_connection_serialize` function.
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_deserialize(command_handle: CommandHandle,
                                         connection_data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(connection_data, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_deserialize(command_handle: {}, connection_data: {})", command_handle, connection_data);

    spawn(move || {
        let (rc, handle) = match from_string(&connection_data) {
            Ok(x) => {
                let source_id = get_source_id(x).unwrap_or_default();
                trace!("vcx_connection_deserialize_cb(command_handle: {}, rc: {}, handle: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, x, source_id);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_connection_deserialize_cb(command_handle: {}, rc: {}, handle: {} )",
                      command_handle, x, 0);
                (x.into(), 0)
            }
        };

        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received messages.
/// Checks for any messages changing state in the connection and updates the state attribute.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides most current state of the credential and error status of request
///     Connection states:
///         1 - Initialized
///         2 - Request Sent
///         3 - Offer Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_update_state(command_handle: CommandHandle,
                                          connection_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_update_state(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    spawn(move || {
        let rc = match update_state(connection_handle, None) {
            Ok(x) => {
                trace!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, connection_handle, get_state(connection_handle), source_id);
                x
            }
            Err(x) => {
                warn!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                      command_handle, x, connection_handle, get_state(connection_handle), source_id);
                x.into()
            }
        };
        let state = get_state(connection_handle);
        cb(command_handle, rc, state);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the state of the connection based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// message: message to process.
///
/// cb: Callback that provides most current state of the connection and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_update_state_with_message(command_handle: CommandHandle,
                                                       connection_handle: u32,
                                                       message: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_update_state(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    let message: Message = match serde_json::from_str(&message) {
        Ok(x) => x,
        Err(_) => return VcxError::from(VcxErrorKind::InvalidJson).into(),
    };

    spawn(move|| {
        let result = update_state_with_message(connection_handle, message);

        let rc = match result {
            Ok(x) => {
                trace!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, connection_handle, get_state(connection_handle), source_id);
                x
            }
            Err(x) => {
                warn!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
                      command_handle, x, connection_handle, get_state(connection_handle), source_id);
                x.into()
            }
        };

        let state = get_state(connection_handle);
        cb(command_handle, rc, state);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Returns the current internal state of the connection. Does NOT query agency for state updates.
///     Possible states:
///         1 - Initialized
///         2 - Offer Sent
///         3 - Request Received
///         4 - Accepted
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that was provided during creation. Used to access connection object
///
/// cb: Callback that provides most current state of the connection and error status of request
///
/// #Returns
#[no_mangle]
pub extern fn vcx_connection_get_state(command_handle: CommandHandle,
                                       connection_handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_state(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    spawn(move || {
        trace!("vcx_connection_get_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {}), source_id: {:?}",
               command_handle, error::SUCCESS.message, connection_handle, get_state(connection_handle), source_id);
        cb(command_handle, error::SUCCESS.code_num, get_state(connection_handle));

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the invite details that were sent or can be sent to the remote side.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// abbreviated: abbreviated connection details for QR codes or not (applicable for `proprietary` communication method only)
///
/// cb: Callback that provides the json string of details
///
/// # Example
/// details -> depends on communication method:
///     proprietary:
///       {"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}
///     aries:
///      {
///         "label": "Alice",
///         "serviceEndpoint": "https://example.com/endpoint",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///    }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_invite_details(command_handle: CommandHandle,
                                            connection_handle: u32,
                                            abbreviated: bool,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, details: *const c_char)>) -> u32 {
    info!("vcx_connection_invite_details >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_invite_details(command_handle: {}, connection_handle: {}, abbreviated: {}), source_id: {:?}",
           command_handle, connection_handle, abbreviated, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    spawn(move || {
        match get_invite_details(connection_handle, abbreviated) {
            Ok(str) => {
                trace!("vcx_connection_invite_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, str, source_id);
                let msg = CStringUtils::string_to_cstring(str);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_invite_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {}, source_id: {:?})",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a message to the specified connection
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send the message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// msg: actual message to send
///
/// send_msg_options: (applicable for `proprietary` communication method only)
///     {
///         msg_type: String, // type of message to send. can be any string.
///         msg_title: String, // message title (user notification)
///         ref_msg_id: Option<String>, // If responding to a message, id of the message
///     }
///
/// # Example:
/// msg ->
///     "HI"
///   OR
///     {"key": "value"}
///   OR
///     {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/trust_ping/1.0/ping",
///         "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
///         "comment": "Hi. Are you listening?",
///         "response_requested": true
///     }
///
/// send_msg_options ->
///     {
///         "msg_type":"Greeting",
///         "msg_title": "Hi There"
///     }
///   OR
///     {
///         "msg_type":"Greeting",
///         "msg_title": "Hi There",
///         "ref_msg_id" "as2d343sag"
///     }
///
/// cb: Callback that provides id of retrieved response message
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_message(command_handle: CommandHandle,
                                          connection_handle: u32,
                                          msg: *const c_char,
                                          send_msg_options: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg_id: *const c_char)>) -> u32 {
    info!("vcx_message_send >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg, VcxErrorKind::InvalidOption);
    check_useful_c_str!(send_msg_options, VcxErrorKind::InvalidOption);

    trace!("vcx_message_send(command_handle: {}, connection_handle: {}, msg: {}, send_msg_options: {})",
           command_handle, connection_handle, msg, send_msg_options);

    spawn(move || {
        match send_generic_message(connection_handle, &msg, &send_msg_options) {
            Ok(x) => {
                trace!("vcx_connection_send_message_cb(command_handle: {}, rc: {}, msg_id: {})",
                       command_handle, error::SUCCESS.message, x);

                let msg_id = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg_id.as_ptr());
            }
            Err(e) => {
                warn!("vcx_connection_send_message_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
///
/// Note that this function is useful in case `aries` communication method is used.
/// In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send ping message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// comment: (Optional) human-friendly description of the ping.
///
/// cb: Callback that provides success or failure of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_ping(command_handle: u32,
                                       connection_handle: u32,
                                       comment: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_ping >>>");

    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_ping(command_handle: {}, connection_handle: {}, comment: {:?})",
           command_handle, connection_handle, comment);

    spawn(move || {
        match send_ping(connection_handle, comment) {
            Ok(()) => {
                trace!("vcx_connection_send_ping(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_send_ping(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Generate a signature for the specified data using connection pairwise keys
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to sign the message.
///                    Was provided during creation. Used to identify connection object.
///
/// data_raw: raw data buffer for signature
///
/// data_len: length of data buffer
///
/// cb: Callback that provides the generated signature
///
/// # Example
/// data_raw -> [1, 2, 3, 4, 5, 6]
/// data_len -> 6
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_sign_data(command_handle: CommandHandle,
                                       connection_handle: u32,
                                       data_raw: *const u8,
                                       data_len: u32,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: u32,
                                                            signature_raw: *const u8,
                                                            signature_len: u32)>) -> u32 {
    trace!("vcx_connection_sign_data: >>> connection_handle: {}, data_raw: {:?}, data_len: {}",
           connection_handle, data_raw, data_len);

    check_useful_c_byte_array!(data_raw, data_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_sign_data: entities >>> connection_handle: {}, data_raw: {:?}, data_len: {}",
           connection_handle, data_raw, data_len);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_sign - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    let vk = match ::connection::get_pw_verkey(connection_handle) {
        Ok(x) => x,
        Err(e) => return e.into(),
    };

    spawn(move || {
        match ::utils::libindy::crypto::sign(&vk, &data_raw) {
            Ok(x) => {
                trace!("vcx_connection_sign_data_cb(command_handle: {}, connection_handle: {}, rc: {}, signature: {:?})",
                       command_handle, connection_handle, error::SUCCESS.message, x);

                let (signature_raw, signature_len) = ::utils::cstring::vec_to_pointer(&x);
                cb(command_handle, error::SUCCESS.code_num, signature_raw, signature_len);
            }
            Err(e) => {
                warn!("vcx_messages_sign_data_cb(command_handle: {}, rc: {}, signature: null)",
                      command_handle, e);

                cb(command_handle, e.into(), ptr::null_mut(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Verify the signature is valid for the specified data using connection pairwise keys
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to verify signature.
///                    Was provided during creation. Used to identify connection object.
///
/// data_raw: raw data buffer for signature
///
/// data_len: length of data buffer
///
/// signature_raw: raw data buffer for signature
///
/// signature_len: length of data buffer
///
/// cb: Callback that specifies whether the signature was valid or not
///
/// # Example
/// data_raw -> [1, 2, 3, 4, 5, 6]
/// data_len -> 6
/// signature_raw -> [2, 3, 4, 5, 6, 7]
/// signature_len -> 6
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_verify_signature(command_handle: CommandHandle,
                                              connection_handle: u32,
                                              data_raw: *const u8,
                                              data_len: u32,
                                              signature_raw: *const u8,
                                              signature_len: u32,
                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                   err: u32,
                                                                   valid: bool)>) -> u32 {
    trace!("vcx_connection_verify_signature: >>> connection_handle: {}, data_raw: {:?}, data_len: {}, signature_raw: {:?}, signature_len: {}",
           connection_handle, data_raw, data_len, signature_raw, signature_len);

    check_useful_c_byte_array!(data_raw, data_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_byte_array!(signature_raw, signature_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_verify_signature: entities >>> connection_handle: {}, data_raw: {:?}, data_len: {}, signature_raw: {:?}, signature_len: {}",
           connection_handle, data_raw, data_len, signature_raw, signature_len);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_verify_signature - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    let vk = match ::connection::get_their_pw_verkey(connection_handle) {
        Ok(x) => x,
        Err(e) => return e.into(),
    };

    spawn(move || {
        match ::utils::libindy::crypto::verify(&vk, &data_raw, &signature_raw) {
            Ok(x) => {
                trace!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                       command_handle, error::SUCCESS.message, x);

                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(e) => {
                warn!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                      command_handle, e, false);

                cb(command_handle, e.into(), false);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the connection object by de-allocating memory
///
/// #Params
/// connection_handle: was provided during creation. Used to identify connection object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_connection_release(connection_handle: u32) -> u32 {
    info!("vcx_connection_release >>>");

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    match release(connection_handle) {
        Ok(()) => {
            trace!("vcx_connection_release(connection_handle: {}, rc: {}), source_id: {:?}",
                   connection_handle, error::SUCCESS.message, source_id);
            error::SUCCESS.code_num
        }
        Err(e) => {
            warn!("vcx_connection_release(connection_handle: {}), rc: {}), source_id: {:?}",
                  connection_handle, e, source_id);
            e.into()
        }
    }
}

/// Send discovery features message to the specified connection to discover which features it supports, and to what extent.
///
/// Note that this function is useful in case `aries` communication method is used.
/// In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// query: (Optional) query string to match against supported message types.
///
/// comment: (Optional) human-friendly description of the query.
///
/// cb: Callback that provides success or failure of request
///
/// # Example
/// query -> `did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/`
///
/// comment -> `share please`
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_discovery_features(command_handle: u32,
                                                     connection_handle: u32,
                                                     query: *const c_char,
                                                     comment: *const c_char,
                                                     cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_discovery_features >>>");

    check_useful_opt_c_str!(query, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_discovery_features(command_handle: {}, connection_handle: {}, query: {:?}, comment: {:?})",
           command_handle, connection_handle, query, comment);

    spawn(move || {
        match send_discovery_features(connection_handle, query, comment) {
            Ok(()) => {
                trace!("vcx_connection_send_discovery_features(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_send_discovery_features(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the information about the connection state.
///
/// Note: This method can be used for `aries` communication method only.
///     For other communication method it returns ActionNotSupported error.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides the json string of connection information
///
/// # Example
/// info ->
///      {
///         "current": {
///             "did": <str>
///             "recipientKeys": array<str>
///             "routingKeys": array<str>
///             "serviceEndpoint": <str>,
///             "protocols": array<str> -  The set of protocol supported by current side.
///         },
///         "remote: { <Option> - details about remote connection side
///             "did": <str> - DID of remote side
///             "recipientKeys": array<str> - Recipient keys
///             "routingKeys": array<str> - Routing keys
///             "serviceEndpoint": <str> - Endpoint
///             "protocols": array<str> - The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
///          }
///    }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_info(command_handle: CommandHandle,
                                  connection_handle: u32,
                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, info: *const c_char)>) -> u32 {
    info!("vcx_connection_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_info(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_info - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    spawn(move || {
        match get_connection_info(connection_handle) {
            Ok(info) => {
                trace!("vcx_connection_info(command_handle: {}, connection_handle: {}, rc: {}, info: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, info, source_id);
                let info = CStringUtils::string_to_cstring(info);
                cb(command_handle, error::SUCCESS.code_num, info.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_info(command_handle: {}, connection_handle: {}, rc: {}, info: {}, source_id: {:?})",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves pw_did from Connection object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides your pw_did for this connection
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_pw_did(command_handle: u32,
                                        connection_handle: u32,
                                        cb: Option<extern fn(xcommand_handle: u32, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_get_pw_did >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_pw_did(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    spawn(move|| {
        match get_pw_did(connection_handle) {
            Ok(json) => {
                trace!("vcx_connection_get_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, pw_did: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, json, source_id);
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_connection_get_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, pw_did: {}), source_id: {:?}",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves their_pw_did from Connection object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides your pw_did for this connection
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_their_pw_did(command_handle: u32,
                                              connection_handle: u32,
                                              cb: Option<extern fn(xcommand_handle: u32, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_get_pw_did >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = get_source_id(connection_handle).unwrap_or_default();
    trace!("vcx_connection_get_their_pw_did(command_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, connection_handle, source_id);

    if !is_valid_handle(connection_handle) {
        error!("vcx_connection_get_state - invalid handle");
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    spawn(move|| {
        match get_their_pw_did(connection_handle) {
            Ok(json) => {
                trace!("vcx_connection_get_their_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, their_pw_did: {}), source_id: {:?}",
                       command_handle, connection_handle, error::SUCCESS.message, json, source_id);
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_connection_get_their_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, their_pw_did: {}), source_id: {:?}",
                      command_handle, connection_handle, x, "null", source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a connection object in completed state based on provided remote details.
/// Once object is created it's already ready for message exchange and no addition connection steps are required.
///
/// This method can be used for testing purposes to skip connection protocol steps.
///
/// Note: It's STRONGLY not recommended to use this method for real connection establishment.
/// Note: This method can be used for `aries` communication method only.
///     For other communication method it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// label: Institution's unique ID for the connection
///
/// did: DID of remote side
///
/// verkey: Recipient Verkey recipient
///
/// endpoint: Endpoint of remote side
///
/// cb: Callback that provides success or failure of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_create_test_connection(command_handle: CommandHandle,
                                                    label: *const c_char,
                                                    did: *const c_char,
                                                    verkey: *const c_char,
                                                    endpoint: *const c_char,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: u32)>) -> u32 {
    info!("vcx_connection_create_test_connection >>>");

    check_useful_c_str!(did, VcxErrorKind::InvalidOption);
    check_useful_c_str!(verkey, VcxErrorKind::InvalidOption);
    check_useful_c_str!(label, VcxErrorKind::InvalidOption);
    check_useful_c_str!(endpoint, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create_test_connection(command_handle: {}, did: {}, verkey: {}, endpoint: {}), label: {:?}",
           command_handle, did, verkey, endpoint, label);

    spawn(move || {
        match create_test_connection(did, verkey, label, endpoint) {
            Ok(handle) => {
                trace!("vcx_connection_create_test_connection(command_handle: {}, connection_handle: {}, rc: {})",
                       command_handle, handle, error::SUCCESS.message);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_test_connection(command_handle: {}, connection_handle: {}, rc: {})",
                      command_handle, 0, x);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;
    use connection::tests::build_test_connection;
    use utils::error;
    use api::{return_types_u32, VcxStateType};
    use utils::constants::{GET_MESSAGES_RESPONSE, INVITE_ACCEPTED_RESPONSE};
    use utils::error::SUCCESS;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;
    use utils::timeout::TimeoutUtils;

    #[test]
    fn test_vcx_connection_create() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let _rc = vcx_connection_create(cb.command_handle,
                                       CString::new("test_create").unwrap().into_raw(),
                                       Some(cb.get_callback()));

        assert!(cb.receive(TimeoutUtils::some_medium()).unwrap() > 0);
    }

    #[test]
    fn test_vcx_connection_create_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_connection_create(0,
                                       CString::new("test_create_fails").unwrap().into_raw(),
                                       None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_create(cb.command_handle,
                                       ptr::null(),
                                       Some(cb.get_callback()));
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_vcx_connection_connect() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_connect(cb.command_handle, 0, CString::new("{}").unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);
        let handle = build_test_connection();
        assert!(handle > 0);
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_connect(cb.command_handle, handle, CString::new("{}").unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        let invite_details = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert!(invite_details.is_some());
    }

    #[test]
    fn test_vcx_connection_redirect() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_connection_redirect(cb.command_handle, 0, 0,Some(cb.get_callback()));
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);

        let handle = build_test_connection();
        assert!(handle > 0);

        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_connection_redirect(cb.command_handle,handle, 0,Some(cb.get_callback()));
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);

        let handle2 = create_connection("alice2").unwrap();
        connect(handle2, Some("{}".to_string())).unwrap();
        assert!(handle2 > 0);

        let cb = return_types_u32::Return_U32::new().unwrap();
        let rc = vcx_connection_redirect(cb.command_handle,handle, handle2,Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
    }

    #[test]
    fn test_vcx_connection_update_state() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();
        assert!(handle > 0);
        connect(handle, None).unwrap();
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        AgencyMock::set_next_response(GET_MESSAGES_RESPONSE.to_vec());
        let rc = vcx_connection_update_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    fn test_vcx_connection_update_state_with_message() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();
        assert!(handle > 0);
        connect(handle, None).unwrap();
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_update_state_with_message(cb.command_handle, handle, CString::new(INVITE_ACCEPTED_RESPONSE).unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    fn test_vcx_connection_update_state_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_connection_update_state(0, 0, None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_vcx_connection_serialize() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();
        assert!(handle > 0);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_serialize(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);

        // unwraps on the option, if none, then serializing failed and panic! ensues.
        cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();
    }

    #[test]
    fn test_vcx_connection_release() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();

        let rc = vcx_connection_release(handle);
        assert_eq!(rc, error::SUCCESS.code_num);

        let unknown_handle = handle + 1;
        assert_eq!(vcx_connection_release(unknown_handle), error::INVALID_CONNECTION_HANDLE.code_num);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        let rc = vcx_connection_connect(0, handle, CString::new("{}").unwrap().into_raw(), Some(cb.get_callback()));
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);
    }

    #[test]
    fn test_vcx_connection_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let string = ::utils::constants::DEFAULT_CONNECTION;
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let err = vcx_connection_deserialize(cb.command_handle,
                                             CString::new(string).unwrap().into_raw(),
                                             Some(cb.get_callback()));
        assert_eq!(err, SUCCESS.code_num);
        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_connection_get_state() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();

        AgencyMock::set_next_response(GET_MESSAGES_RESPONSE.to_vec());

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let _rc = vcx_connection_update_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateAccepted as u32);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_connection_get_state(cb.command_handle, handle, Some(cb.get_callback()));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateAccepted as u32)
    }

    #[test]
    fn test_vcx_connection_delete_connection() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_connection_delete_connection(cb.command_handle, connection_handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();

        assert_eq!(::connection::get_source_id(connection_handle).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_send_message() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();
        ::connection::set_state(connection_handle, VcxStateType::VcxStateAccepted).unwrap();

        let msg = CString::new("MESSAGE").unwrap().into_raw();
        let send_msg_options = CString::new(json!({"msg_type":"type", "msg_title": "title", "ref_msg_id":null}).to_string()).unwrap().into_raw();
        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_connection_send_message(cb.command_handle, connection_handle, msg, send_msg_options, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_sign() {
        let _setup = SetupMocks::init();

        let connection_handle = ::connection::tests::build_test_connection();

        let msg = format!("My message");
        let msg_len = msg.len();

        let cb = return_types_u32::Return_U32_BIN::new().unwrap();
        assert_eq!(vcx_connection_sign_data(cb.command_handle,
                                            connection_handle,
                                            CString::new(msg).unwrap().as_ptr() as *const u8,
                                            msg_len as u32,
                                            Some(cb.get_callback())), error::SUCCESS.code_num);
        let _sig = cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_verify_signature() {
        let _setup = SetupMocks::init();

        let connection_handle = ::connection::tests::build_test_connection();

        let msg = format!("My message");
        let msg_len = msg.len();

        let signature = format!("signature");
        let signature_length = signature.len();

        let cb = return_types_u32::Return_U32_BOOL::new().unwrap();
        assert_eq!(vcx_connection_verify_signature(cb.command_handle,
                                                   connection_handle,
                                                   CString::new(msg).unwrap().as_ptr() as *const u8,
                                                   msg_len as u32,
                                                   CString::new(signature).unwrap().as_ptr() as *const u8,
                                                   signature_length as u32,
                                                   Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
