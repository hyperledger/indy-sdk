use libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use disclosed_proof;
use std::ptr;
use utils::threadpool::spawn;
use error::prelude::*;
use indy_sys::CommandHandle;

/*
    The API represents an Prover side in credential presentation process.
    Assumes that pairwise connection between Verifier and Prover is already established.

    # State

    The set of object states, messages and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.

        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.

        VcxStateType::VcxStateAccepted - once `vcx_disclosed_proof_send_proof` (send `PROOF` message) is called.

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
*/

/// Create a Proof object for fulfilling a corresponding proof request
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's identification for the proof, should be unique.
///
/// req: proof request received via "vcx_get_proof_requests"
///
/// cb: Callback that provides proof handle or error status
///
/// # Example proof_req -> "{"@topic":{"mid":9,"tid":1},"@type":{"name":"PROOF_REQUEST","version":"1.0"},"msg_ref_id":"ymy5nth","proof_request_data":{"name":"AccountCertificate","nonce":"838186471541979035208225","requested_attributes":{"business_2":{"name":"business"},"email_1":{"name":"email"},"name_0":{"name":"name"}},"requested_predicates":{},"version":"0.1"}}"
///
/// #Returns
/// Error code as u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_create_with_request(command_handle: CommandHandle,
                                                      source_id: *const c_char,
                                                      proof_req: *const c_char,
                                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_disclosed_proof_create_with_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_req, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_create_with_request(command_handle: {}, source_id: {}, proof_req: {})",
           command_handle, source_id, proof_req);

    spawn(move || {
        match disclosed_proof::create_proof(&source_id, &proof_req) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_create_with_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, x, source_id);
                cb(command_handle, 0, x);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_create_with_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, x, 0, source_id);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Create a proof based off of a known message id for a given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the proof, should be unique.
///
/// connection: connection to query for proof request
///
/// msg_id:  id of the message that contains the proof request
///
/// cb: Callback that provides proof handle and proof request or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_create_with_msgid(command_handle: CommandHandle,
                                                    source_id: *const c_char,
                                                    connection_handle: u32,
                                                    msg_id: *const c_char,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_handle: u32, proof_req: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_create_with_msgid >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_id, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_create_with_msgid(command_handle: {}, source_id: {}, connection_handle: {}, msg_id: {})",
           command_handle, source_id, connection_handle, msg_id);

    spawn(move || {
        match disclosed_proof::create_proof_with_msgid(&source_id, connection_handle, &msg_id) {
            Ok((handle, request)) => {
                trace!("vcx_disclosed_proof_create_with_msgid_cb(command_handle: {}, rc: {}, handle: {}, proof_req: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, handle, request, source_id);
                let msg = CStringUtils::string_to_cstring(request);
                cb(command_handle, error::SUCCESS.code_num, handle, msg.as_ptr())
            }
            Err(e) => {
                cb(command_handle, e.into(), 0,  ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a proof to the connection, called after having received a proof request
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_send_proof(command_handle: CommandHandle,
                                             proof_handle: u32,
                                             connection_handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_send_proof >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_send_proof(command_handle: {}, proof_handle: {}, connection_handle: {}) source_id: {}",
           command_handle, proof_handle, connection_handle, source_id);

    spawn(move || {
        match disclosed_proof::send_proof(proof_handle, connection_handle) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_send_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_send_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x, source_id);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a proof rejection to the connection, called after having received a proof request
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_reject_proof(command_handle: CommandHandle,
                                               proof_handle: u32,
                                               connection_handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_reject_proof >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_reject_proof(command_handle: {}, proof_handle: {}, connection_handle: {}) source_id: {}",
           command_handle, proof_handle, connection_handle, source_id);

    spawn(move || {
        match disclosed_proof::reject_proof(proof_handle, connection_handle) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_reject_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_reject_proof_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x, source_id);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the proof message for sending.
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_proof_msg(command_handle: CommandHandle,
                                                proof_handle: u32,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_proof_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_get_proof_msg(command_handle: {}, proof_handle: {}) source_id: {}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::generate_proof_msg(proof_handle) {
            Ok(msg) => {
                let msg = CStringUtils::string_to_cstring(msg);
                trace!("vcx_disclosed_proof_get_proof_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_proof_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x, source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the reject proof message for sending.
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_reject_msg(command_handle: CommandHandle,
                                                 proof_handle: u32,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_reject_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_get_reject_msg(command_handle: {}, proof_handle: {}) source_id: {}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::generate_reject_proof_msg(proof_handle) {
            Ok(msg) => {
                let msg = CStringUtils::string_to_cstring(msg);
                trace!("vcx_disclosed_proof_get_reject_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_reject_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x, source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Queries agency for all pending proof requests from the given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection to query for proof requests.
///
/// cb: Callback that provides any proof requests and error status of query
/// # Example requests -> "[{'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_requests(command_handle: CommandHandle,
                                               connection_handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, requests: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_requests >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    trace!("vcx_disclosed_proof_get_requests(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match disclosed_proof::get_proof_request_messages(connection_handle, None) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_get_requests_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.message, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_requests_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.message, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the disclosed proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access disclosed proof object
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///     States:
///         3 - Request Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_state(command_handle: CommandHandle,
                                            proof_handle: u32,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_disclosed_proof_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_get_state(command_handle: {}, proof_handle: {}), source_id: {:?}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::get_state(proof_handle) {
            Ok(s) => {
                trace!("vcx_disclosed_proof_get_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_disclosed_proof_get_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Checks for any state change in the disclosed proof and updates the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation. Used to identify disclosed proof object
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_update_state(command_handle: CommandHandle,
                                               proof_handle: u32,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_disclosed_proof_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_update_state(command_handle: {}, proof_handle: {}) source_id: {}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::update_state(proof_handle, None) {
            Ok(s) => {
                trace!("vcx_disclosed_proof_update_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_disclosed_proof_update_state_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Checks for any state change from the given message and updates the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation. Used to identify disclosed proof object
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_update_state_with_message(command_handle: CommandHandle,
                                                            proof_handle: u32,
                                                            message: *const c_char,
                                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_disclosed_proof_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_update_state_with_message(command_handle: {}, proof_handle: {}) source_id: {}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::update_state(proof_handle, Some(message)) {
            Ok(s) => {
                trace!("vcx_disclosed_proof_update_state__with_message_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_disclosed_proof_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}) source_id: {}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the disclosed proof object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the disclosed proof's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_serialize(command_handle: CommandHandle,
                                            proof_handle: u32,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_serialize(command_handle: {}, proof_handle: {}) source_id: {}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::to_string(proof_handle) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_serialize_cb(command_handle: {}, rc: {}, data: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, x, source_id);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_serialize_cb(command_handle: {}, rc: {}, data: {}) source_id: {}",
                       command_handle, x, 0, source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an disclosed proof object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// data: json string representing a disclosed proof object
///
///
/// cb: Callback that provides handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_deserialize(command_handle: CommandHandle,
                                              proof_data: *const c_char,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_disclosed_proof_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_data, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_deserialize(command_handle: {}, proof_data: {})",
           command_handle, proof_data);

    spawn(move || {
        match disclosed_proof::from_string(&proof_data) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_deserialize_cb(command_handle: {}, rc: {}, proof_handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, x, disclosed_proof::get_source_id(x).unwrap_or_default());

                cb(command_handle, 0, x);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_deserialize_cb(command_handle: {}, rc: {}, proof_handle: {}) source_id: {}",
                       command_handle, x, 0, "");
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get credentials from wallet matching to the proof request associated with proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the credentials in wallet associated with proof request
///
/// # Example
/// credentials -> "{'attrs': {'attribute_0': [{'cred_info': {'schema_id': 'id', 'cred_def_id': 'id', 'attrs': {'attr_name': 'attr_value', ...}, 'referent': '914c7e11'}}]}}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_retrieve_credentials(command_handle: CommandHandle,
                                                       proof_handle: u32,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_retrieve_credentials >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_retrieve_credentials(command_handle: {}, proof_handle: {}) source_id: {}",
           command_handle, proof_handle, source_id);

    spawn(move || {
        match disclosed_proof::retrieve_credentials(proof_handle) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_retrieve_credentials(command_handle: {}, rc: {}, data: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, x, source_id);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_retrieve_credentials(command_handle: {}, rc: {}, data: {}) source_id: {}",
                       command_handle, x, 0, source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Accept proof request associated with proof object and generates a proof from the selected credentials and self attested attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// selected_credentials: a json string with a credential for each proof request attribute.
///     List of possible credentials for each attribute is returned from vcx_disclosed_proof_retrieve_credentials,
///         (user needs to select specific credential to use from list of credentials)
///         {
///             "attrs":{
///                 String:{// Attribute key: This may not be the same as the attr name ex. "age_1" where attribute name is "age"
///                     "credential": {
///                         "cred_info":{
///                             "referent":String,
///                             "attrs":{ String: String }, // ex. {"age": "111", "name": "Bob"}
///                             "schema_id": String,
///                             "cred_def_id": String,
///                             "rev_reg_id":Option<String>,
///                             "cred_rev_id":Option<String>,
///                             },
///                         "interval":Option<{to: Option<u64>, from:: Option<u64>}>
///                     }, // This is the exact credential information selected from list of
///                        // credentials returned from vcx_disclosed_proof_retrieve_credentials
///                     "tails_file": Option<"String">, // Path to tails file for this credential
///                 },
///            },
///           "predicates":{ TODO: will be implemented as part of IS-1095 ticket. }
///        }
///     // selected_credentials can be empty "{}" if the proof only contains self_attested_attrs
///
/// self_attested_attrs: a json string with attributes self attested by user
/// # Examples
/// self_attested_attrs -> "{"self_attested_attr_0":"attested_val"}" | "{}"
/// selected_credentials -> "{'attrs': {'attribute_0': {'credential': {'cred_info': {'cred_def_id': 'od', 'schema_id': 'id', 'referent': '0c212108-9433-4199-a21f-336a44164f38', 'attrs': {'attr_name': 'attr_value', ...}}}}}}"
/// cb: Callback that returns error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_generate_proof(command_handle: CommandHandle,
                                                 proof_handle: u32,
                                                 selected_credentials: *const c_char,
                                                 self_attested_attrs: *const c_char,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_generate_proof >>>");

    check_useful_c_str!(selected_credentials, VcxErrorKind::InvalidOption);
    check_useful_c_str!(self_attested_attrs, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into()
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_generate_proof(command_handle: {}, proof_handle: {}, selected_credentials: {}, self_attested_attrs: {}) source_id: {}",
           command_handle, proof_handle, selected_credentials, self_attested_attrs, source_id);

    spawn(move || {
        match disclosed_proof::generate_proof(proof_handle, selected_credentials, self_attested_attrs) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_generate_proof(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_generate_proof(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x, source_id);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Declines presentation request.
/// There are two ways of following interaction:
///     - Prover wants to propose using a different presentation - pass `proposal` parameter.
///     - Prover doesn't want to continue interaction - pass `reason` parameter.
/// Note that only one of these parameters can be passed.
///
/// Note that proposing of different presentation is supported for `aries` protocol only.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// reason: (Optional) human-readable string that explain the reason of decline
///
/// proposal: (Optional) the proposed format of presentation request
/// (see https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#presentation-preview for details)
/// {
///    "attributes": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "mime-type": Optional("<type>"),
///            "value": Optional("<value>")
///        },
///        // more attributes
///    ],
///    "predicates": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "predicate": "<predicate>", - one of "<", "<=", ">=", ">"
///            "threshold": <threshold>
///        },
///        // more predicates
///    ]
/// }
///
/// # Example
///  proposal ->
///     {
///          "attributes": [
///              {
///                  "name": "first name"
///              }
///          ],
///          "predicates": [
///              {
///                  "name": "age",
///                  "predicate": ">",
///                  "threshold": 18
///              }
///          ]
///      }
///
/// cb: Callback that returns error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_decline_presentation_request(command_handle: u32,
                                                               proof_handle: u32,
                                                               connection_handle: u32,
                                                               reason: *const c_char,
                                                               proposal: *const c_char,
                                                               cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_decline_presentation_request >>>");

    check_useful_opt_c_str!(reason, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(proposal, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    if !disclosed_proof::is_valid_handle(proof_handle) {
        return VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle).into();
    }

    let source_id = disclosed_proof::get_source_id(proof_handle).unwrap_or_default();
    trace!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, proof_handle: {}, connection_handle: {}, reason: {:?}, proposal: {:?}) source_id: {}",
           command_handle, proof_handle, connection_handle, reason, proposal, source_id);

    spawn(move || {
        match disclosed_proof::decline_presentation_request(proof_handle, connection_handle, reason, proposal) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x, source_id);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Releases the disclosed proof object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_disclosed_proof_release(handle: u32) -> u32 {
    info!("vcx_disclosed_proof_release >>>");

    let source_id = disclosed_proof::get_source_id(handle).unwrap_or_default();
    match disclosed_proof::release(handle) {
        Ok(()) => {
            trace!("vcx_disclosed_proof_release(handle: {}, rc: {}), source_id: {:?}",
                   handle, error::SUCCESS.message, source_id);
            error::SUCCESS.code_num
        }
        Err(e) => {
            error!("vcx_disclosed_proof_release(handle: {}, rc: {}), source_id: {:?}",
                   handle, e, source_id);
            e.into()
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use std::ffi::CString;
    use connection;
    use api::VcxStateType;
    use utils::constants::PENDING_OBJECT_SERIALIZE_VERSION;
    use api::return_types_u32;
    use serde_json::Value;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;
    use utils::timeout::TimeoutUtils;

    pub const BAD_PROOF_REQUEST: &str = r#"{"version": "0.1","to_did": "LtMgSjtFcyPwenK9SHCyb8","from_did": "LtMgSjtFcyPwenK9SHCyb8","claim": {"account_num": ["8BEaoLf8TBmK4BUyX8WWnA"],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "Pd4fnFtRBcMKRVC2go5w3j","claim_name": "Account Certificate","claim_id": "3675417066","msg_ref_id": "ymy5nth"}"#;

    fn _vcx_disclosed_proof_create_with_request_c_closure(proof_request: &str) -> Result<u32, u32> {
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_disclosed_proof_create_with_request(cb.command_handle,
                                                         CString::new("test_create").unwrap().into_raw(),
                                                         CString::new(proof_request).unwrap().into_raw(),
                                                         Some(cb.get_callback()));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    #[test]
    fn test_vcx_proof_create_with_request_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_proof_create_with_request() {
        let _setup = SetupMocks::init();

        let err = _vcx_disclosed_proof_create_with_request_c_closure(BAD_PROOF_REQUEST).unwrap_err();
        assert_eq!(err, error::INVALID_JSON.code_num);
    }

    #[test]
    fn test_create_with_msgid() {
        let _setup = SetupMocks::init();

        let cxn = ::connection::tests::build_test_connection();

        AgencyMock::set_next_response(::utils::constants::NEW_PROOF_REQUEST_RESPONSE.to_vec());

        let cb = return_types_u32::Return_U32_U32_STR::new().unwrap();
        assert_eq!(vcx_disclosed_proof_create_with_msgid(cb.command_handle,
                                                         CString::new("test_create_with_msgid").unwrap().into_raw(),
                                                         cxn,
                                                         CString::new("123").unwrap().into_raw(),
                                                         Some(cb.get_callback())), error::SUCCESS.code_num);
        let (handle, disclosed_proof) = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert!(handle > 0 && disclosed_proof.is_some());
    }

    #[test]
    fn test_vcx_disclosed_proof_release() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(vcx_disclosed_proof_release(handle + 1), error::INVALID_DISCLOSED_PROOF_HANDLE.code_num);
        assert_eq!(vcx_disclosed_proof_release(handle), error::SUCCESS.code_num);
        assert_eq!(vcx_disclosed_proof_release(handle), error::INVALID_DISCLOSED_PROOF_HANDLE.code_num);
    }

    #[test]
    fn test_vcx_disclosed_proof_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_disclosed_proof_serialize(cb.command_handle,
                                                 handle,
                                                 Some(cb.get_callback())), error::SUCCESS.code_num);
        let s = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();

        let j: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(j["version"], PENDING_OBJECT_SERIALIZE_VERSION);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_disclosed_proof_deserialize(cb.command_handle,
                                                   CString::new(s).unwrap().into_raw(),
                                                   Some(cb.get_callback())),
                   error::SUCCESS.code_num);

        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_generate_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_disclosed_proof_get_proof_msg(cb.command_handle,
                                                     handle,
                                                     Some(cb.get_callback())), error::SUCCESS.code_num);
        let _s = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();
    }

    #[test]
    fn test_vcx_send_proof() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(disclosed_proof::get_state(handle).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_disclosed_proof_send_proof(cb.command_handle, handle, connection_handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_reject_proof_request() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(disclosed_proof::get_state(handle).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_disclosed_proof_reject_proof(cb.command_handle, handle, connection_handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_get_reject_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(disclosed_proof::get_state(handle).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let _connection_handle = connection::tests::build_test_connection();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_disclosed_proof_get_reject_msg(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_proof_get_requests() {
        let _setup = SetupMocks::init();

        let cxn = ::connection::tests::build_test_connection();

        AgencyMock::set_next_response(::utils::constants::NEW_PROOF_REQUEST_RESPONSE.to_vec());

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_disclosed_proof_get_requests(cb.command_handle, cxn, Some(cb.get_callback())), error::SUCCESS.code_num as u32);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_proof_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_disclosed_proof_get_state(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        let state = cb.receive(TimeoutUtils::some_medium()).unwrap();
        assert_eq!(state, VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_vcx_disclosed_proof_retrieve_credentials() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_disclosed_proof_retrieve_credentials(cb.command_handle,
                                                            handle,
                                                            Some(cb.get_callback())),
                   error::SUCCESS.code_num);
        let _credentials = cb.receive(None).unwrap().unwrap();
    }

    #[test]
    fn test_vcx_disclosed_proof_generate_proof() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_disclosed_proof_generate_proof(cb.command_handle,
                                                      handle,
                                                      CString::new("{}").unwrap().into_raw(),
                                                      CString::new("{}").unwrap().into_raw(),
                                                      Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }
}
