use serde_json;
use libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use credential;
use std::ptr;
use utils::threadpool::spawn;
use error::prelude::*;
use indy_sys::CommandHandle;

/*
    The API represents a Holder side in credential issuance process.
    Assumes that pairwise connection between Issuer and Holder is already established.

    # State

    The set of object states, messages and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CRED_REQ` message) is called.

        VcxStateType::VcxStateAccepted - once `CRED` messages is received.
                                         use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.

    aries:
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CredentialRequest` message) is called.

        VcxStateType::VcxStateAccepted - once `Credential` messages is received.
        VcxStateType::None - once `ProblemReport` messages is received.
                                                use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `CRED` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `Credential` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None

    # Messages

    proprietary:
        CredentialOffer (`CRED_OFFER`)
        CredentialRequest (`CRED_REQ`)
        Credential (`CRED`)

    aries:
        CredentialProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#propose-credential
        CredentialOffer - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#offer-credential
        CredentialRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#request-credential
        Credential - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#issue-credential
        ProblemReport - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0035-report-problem#the-problem-report-message-type
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
*/

/// Retrieve Payment Transaction Information for this Credential. Typically this will include
/// how much payment is requried by the issuer, which needs to be provided by the prover, before the issuer will
/// issue the credential to the prover. Ideally a prover would want to know how much payment is being asked before
/// submitting the credential request (which triggers the payment to be made).
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides Payment Info of a Credential
///
/// # Example:
/// payment_info ->
///     {
///         "payment_required":"one-time",
///         "payment_addr":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j",
///         "price":1
///     }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_get_payment_info(command_handle: CommandHandle,
                                              credential_handle: u32,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, *const c_char)>) -> u32 {
    info!("vcx_credential_get_payment_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    spawn(move || {
        match credential::get_payment_information(credential_handle) {
            Ok(p) => {
                match p {
                    Some(p) => {
                        let info = p.to_string().unwrap_or("{}".to_string());
                        trace!("vcx_credential_get_payment_info(command_handle: {}, rc: {}, msg: {})", command_handle, error::SUCCESS.code_num, info.clone());
                        let msg = CStringUtils::string_to_cstring(info);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    None => {
                        let msg = CStringUtils::string_to_cstring(format!("{{}}"));
                        trace!("vcx_credential_get_payment_info(command_handle: {}, rc: {}, msg: {})", command_handle, error::SUCCESS.code_num, "{}");
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                }
            }
            Err(e) => {
                warn!("vcx_credential_get_payment_info(command_handle: {}, rc: {}, msg: {})",
                      command_handle, e, "{}");
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Credential object that requests and receives a credential for an institution
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the credential, should be unique.
///
/// offer: credential offer received via "vcx_credential_get_offers"
///
/// # Example
/// offer -> depends on communication method:
///     proprietary:
///         [{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]
///     aries:
///         {"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential", "@id":"<uuid-of-offer-message>", "comment":"somecomment", "credential_preview":<json-ldobject>, "offers~attach":[{"@id":"libindy-cred-offer-0", "mime-type":"application/json", "data":{"base64":"<bytesforbase64>"}}]}
///
/// cb: Callback that provides credential handle or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_create_with_offer(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               offer: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: u32)>) -> u32 {
    info!("vcx_credential_create_with_offer >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(offer, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_create_with_offer(command_handle: {}, source_id: {}, offer: {})",
           command_handle, source_id, secret!(&offer));

    spawn(move || {
        match credential::credential_create_with_offer(&source_id, &offer) {
            Ok(x) => {
                trace!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {})",
                       command_handle, source_id, error::SUCCESS.message, x);
                cb(command_handle, error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {})",
                      command_handle, source_id, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Retrieve information about a stored credential in user's wallet, including credential id and the credential itself.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides error status of api call, or returns the credential in json format of "{uuid:credential}".
///
/// # Example
/// credential -> depends on communication method:
///     proprietary:
///         {"credential_id":"cred_id", "credential": {"libindy_cred":"{....}","rev_reg_def_json":"","cred_def_id":"cred_def_id","msg_type":"CLAIM","claim_offer_id":"1234","version":"0.1","from_did":"did"}}
///     aries:
///         https://github.com/hyperledger/aries-rfcs/tree/master/features/0036-issue-credential#issue-credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_get_credential(command_handle: CommandHandle,
                                 credential_handle: u32,
                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential: *const c_char)>) -> u32 {
    info!("vcx_get_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_get_credential(command_handle: {}, credential_handle: {}) source_id: {})",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::get_credential(credential_handle) {
            Ok(s) => {
                trace!("vcx_get_credential_cb(commmand_handle: {}, rc: {}, msg: {}) source_id: {}",
                       command_handle, error::SUCCESS.code_num, s, source_id);
                let msg = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                error!("vcx_get_credential_cb(commmand_handle: {}, rc: {}, msg: {}) source_id: {}",
                       command_handle, e, "".to_string(), source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Credential object based off of a known message id for a given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the credential, should be unique.
///
/// connection_handle: connection to query for credential offer
///
/// msg_id: msg_id that contains the credential offer
///
/// cb: Callback that provides credential handle or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_create_with_msgid(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               connection_handle: u32,
                                               msg_id: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: u32, offer: *const c_char)>) -> u32 {
    info!("vcx_credential_create_with_msgid >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_id, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_create_with_msgid(command_handle: {}, source_id: {}, connection_handle: {}, msg_id: {})",
           command_handle, source_id, connection_handle, msg_id);

    spawn(move || {
        match credential::credential_create_with_msgid(&source_id, connection_handle, &msg_id) {
            Ok((handle, offer_string)) => {
                let c_offer = CStringUtils::string_to_cstring(offer_string);
                trace!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, source_id, error::SUCCESS.message, handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle, c_offer.as_ptr())
            }
            Err(e) => {
                warn!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, source_id, e, 0, source_id);
                cb(command_handle, e.into(), 0, ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Approves the credential offer and submits a credential request. The result will be a credential stored in the prover's wallet.
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of credential request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_send_request(command_handle: CommandHandle,
                                          credential_handle: u32,
                                          connection_handle: u32,
                                          _payment_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_credential_send_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_send_request(command_handle: {}, credential_handle: {}, connection_handle: {}), source_id: {:?}",
           command_handle, credential_handle, connection_handle, source_id);

    spawn(move || {
        match credential::send_credential_request(credential_handle, connection_handle) {
            Ok(x) => {
                trace!("vcx_credential_send_request_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, x.to_string(), source_id);
                cb(command_handle, x);
            }
            Err(e) => {
                warn!("vcx_credential_send_request_cb(command_handle: {}, rc: {}) source_id: {}",
                      command_handle, e, source_id);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Approves the credential offer and gets the credential request message that can be sent to the specified connection
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// my_pw_did: Use Connection api (vcx_connection_get_pw_did) with specified connection_handle to retrieve your pw_did
///
/// their_pw_did: Use Connection api (vcx_connection_get_their_pw_did) with specified connection_handle to retrieve theri pw_did
///
/// cb: Callback that provides error status of credential request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_request_msg(command_handle: CommandHandle,
                                             credential_handle: u32,
                                             my_pw_did: *const c_char,
                                             their_pw_did: *const c_char,
                                             _payment_handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_credential_get_request_msg >>>");

    check_useful_c_str!(my_pw_did, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(their_pw_did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_get_request_msg(command_handle: {}, credential_handle: {}, my_pw_did: {}, their_pw_did: {:?}), source_id: {:?}",
           command_handle, credential_handle, my_pw_did, their_pw_did, source_id);

    spawn(move || {
        match credential::generate_credential_request_msg(credential_handle, &my_pw_did, &their_pw_did.unwrap_or_default()) {
            Ok(msg) => {
                let msg = CStringUtils::string_to_cstring(msg);
                trace!("vcx_credential_get_request_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, source_id);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_credential_get_request_msg_cb(command_handle: {}, rc: {}) source_id: {}",
                      command_handle, e, source_id);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Queries agency for credential offers from the given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection to query for credential offers.
///
/// cb: Callback that provides any credential offers and error status of query
///
/// # Example offers -> "[[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_offers(command_handle: CommandHandle,
                                        connection_handle: u32,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_offers: *const c_char)>) -> u32 {
    info!("vcx_credential_get_offers >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !connection::is_valid_handle(connection_handle) {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into()
    }

    trace!("vcx_credential_get_offers(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match credential::get_credential_offer_messages(connection_handle) {
            Ok(x) => {
                trace!("vcx_credential_get_offers_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, x.to_string(), x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_get_offers_cb(command_handle: {}, rc: {}, msg: null)",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received messages.
/// Checks for any messages changing state in the credential object and updates the state attribute.
/// If it detects a credential it will store the credential in the wallet.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides most current state of the credential and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_update_state(command_handle: CommandHandle,
                                          credential_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_update_state(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::update_state(credential_handle, None) {
            Ok(_) => (),
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        }

        match credential::get_state(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the state of the credential based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the credential and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_update_state_with_message(command_handle: CommandHandle,
                                                       credential_handle: u32,
                                                       message: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(credential_handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    let source_id = credential::get_source_id(credential_handle).unwrap_or_default();
    trace!("vcx_credential_update_state_with_message(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, credential_handle, source_id);

    spawn(move || {
        match credential::update_state(credential_handle, Some(message)) {
            Ok(_) => (),
            Err(e) => {
                error!("vcx_credential_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        }

        match credential::get_state(credential_handle) {
            Ok(s) => {
                trace!("vcx_credential_update_state_with_message_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the credential object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation.
///
/// cb: Callback that provides most current state of the credential and error status of request
///     Credential statuses:
///         2 - Request Sent
///         3 - Request Received
///         4 - Accepted
///
/// #Returns
#[no_mangle]
pub extern fn vcx_credential_get_state(command_handle: CommandHandle,
                                       handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credential_get_state(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, handle, source_id);

    spawn(move || {
        match credential::get_state(handle) {
            Ok(s) => {
                trace!("vcx_credential_get_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, s, source_id);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_get_state_cb(command_handle: {}, rc: {}, state: {}), source_id: {:?}",
                       command_handle, e, 0, source_id);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Takes the credential object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides json string of the credential's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_serialize(command_handle: CommandHandle,
                                       handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_credential_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !credential::is_valid_handle(handle) {
        return VcxError::from(VcxErrorKind::InvalidCredentialHandle).into()
    }

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credential_serialize(command_handle: {}, credential_handle: {}), source_id: {:?}",
           command_handle, handle, source_id);

    spawn(move || {
        match credential::to_string(handle) {
            Ok(x) => {
                trace!("vcx_credential_serialize_cb(command_handle: {}, rc: {}, data: {}), source_id: {:?}",
                       command_handle, error::SUCCESS.message, x, source_id);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_serialize_cb(command_handle: {}, rc: {}, data: {}), source_id: {:?}",
                       command_handle, x, 0, source_id);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an credential object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_data: json string representing a credential object
///
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_deserialize(command_handle: CommandHandle,
                                         credential_data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: u32)>) -> u32 {
    info!("vcx_credential_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credential_data, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_deserialize(command_handle: {}, credential_data: {})",
           command_handle, credential_data);

    spawn(move || {
        match credential::from_string(&credential_data) {
            Ok(x) => {
                trace!("vcx_credential_deserialize_cb(command_handle: {}, rc: {}, credential_handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.message, x, credential::get_source_id(x).unwrap_or_default());

                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                error!("vcx_credential_deserialize_cb(command_handle: {}, rc: {}, credential_handle: {}) source_id: {}",
                       command_handle, x, 0, "");
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the credential object by de-allocating memory
///
/// #Params
/// handle: Credential handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_credential_release(handle: u32) -> u32 {
    info!("vcx_credential_release >>>");

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    match credential::release(handle) {
        Ok(()) => {
            trace!("vcx_credential_release(handle: {}, rc: {}), source_id: {:?}",
                   handle, error::SUCCESS.message, source_id);
            error::SUCCESS.code_num
        }

        Err(e) => {
            error!("vcx_credential_release(handle: {}, rc: {}), source_id: {:?}",
                   handle, e, source_id);
            e.into()
        }
    }
}

/// Retrieve the payment transaction associated with this credential. This can be used to get the txn that
/// was used to pay the issuer from the prover.  This could be considered a receipt of payment from the payer to
/// the issuer.
///
/// #param
/// handle: credential handle that was provided during creation.  Used to access credential object.
///
/// #Callback returns
/// PaymentTxn json
/// example: {
///         "amount":25,
///         "inputs":[
///             "pay:null:1_3FvPC7dzFbQKzfG"
///         ],
///         "outputs":[
///             {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null}
///         ]
///     }
#[no_mangle]
pub extern fn vcx_credential_get_payment_txn(command_handle: CommandHandle,
                                             handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, txn: *const c_char)>) -> u32 {
    info!("vcx_credential_get_payment_txn >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let source_id = credential::get_source_id(handle).unwrap_or_default();
    trace!("vcx_credential_get_payment_txn(command_handle: {}) source_id: {}", command_handle, source_id);

    spawn(move || {
        match credential::get_payment_txn(handle) {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_credential_get_payment_txn_cb(command_handle: {}, rc: {}, : {}), source_id: {}",
                               command_handle, error::SUCCESS.message, x, credential::get_source_id(handle).unwrap_or_default());

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, 0, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize payment txn: {:?}", e));
                        error!("vcx_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {}), source_id: {}",
                               command_handle, err, "null", credential::get_source_id(handle).unwrap_or_default());
                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                }
            }
            Err(x) => {
                error!("vcx_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {}), source_id: {}",
                       command_handle, x, "null", credential::get_source_id(handle).unwrap_or_default());
                cb(command_handle, x.into(), ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use std::ffi::CString;
    use connection;
    use api::VcxStateType;
    use api::return_types_u32;
    use serde_json::Value;
    use utils::constants::{DEFAULT_SERIALIZED_CREDENTIAL, FULL_CREDENTIAL_SERIALIZED, PENDING_OBJECT_SERIALIZE_VERSION};
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;
    use utils::timeout::TimeoutUtils;

    use ::credential::tests::BAD_CREDENTIAL_OFFER;
    use utils::constants;
    use credential_request::CredentialRequest;

    fn _vcx_credential_create_with_offer_c_closure(offer: &str) -> Result<u32, u32> {
        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        let rc = vcx_credential_create_with_offer(cb.command_handle,
                                                  CString::new("test_create").unwrap().into_raw(),
                                                  CString::new(offer).unwrap().into_raw(),
                                                  Some(cb.get_callback()));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }

        let handle = cb.receive(TimeoutUtils::some_medium());
        handle
    }

    #[test]
    fn test_vcx_credential_create_with_offer_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_credential_create_with_offer_fails() {
        let _setup = SetupMocks::init();

        let err = _vcx_credential_create_with_offer_c_closure(BAD_CREDENTIAL_OFFER).unwrap_err();
        assert_eq!(err, error::INVALID_JSON.code_num);
    }

    #[test]
    fn test_vcx_credential_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_serialize(cb.command_handle,
                                            handle,
                                            Some(cb.get_callback())), error::SUCCESS.code_num);
        let credential_json = cb.receive(TimeoutUtils::some_short()).unwrap().unwrap();

        let object: Value = serde_json::from_str(&credential_json).unwrap();
        assert_eq!(object["version"], PENDING_OBJECT_SERIALIZE_VERSION);

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_deserialize(cb.command_handle,
                                              CString::new(credential_json).unwrap().into_raw(),
                                              Some(cb.get_callback())), error::SUCCESS.code_num);
        let handle = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_credential_send_request() {
        let _setup = SetupMocks::init();

        let handle = credential::credential_create_with_offer("test_send_request", ::utils::constants::CREDENTIAL_OFFER_JSON).unwrap();
        assert_eq!(credential::get_state(handle).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_credential_send_request(cb.command_handle, handle, connection_handle, 0, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_credential_get_new_offers() {
        let _setup = SetupMocks::init();

        let cxn = ::connection::tests::build_test_connection();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_get_offers(cb.command_handle,
                                             cxn,
                                             Some(cb.get_callback())),
                   error::SUCCESS.code_num as u32);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_credential_create() {
        let _setup = SetupMocks::init();

        let cxn = ::connection::tests::build_test_connection();

        let cb = return_types_u32::Return_U32_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_create_with_msgid(cb.command_handle,
                                                    CString::new("test_vcx_credential_create").unwrap().into_raw(),
                                                    cxn,
                                                    CString::new("123").unwrap().into_raw(),
                                                    Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_credential_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_get_state(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_vcx_credential_update_state() {
        let _setup = SetupMocks::init();

        let cxn = ::connection::tests::build_test_connection();

        let handle = credential::from_string(DEFAULT_SERIALIZED_CREDENTIAL).unwrap();

        AgencyMock::set_next_response(::utils::constants::NEW_CREDENTIAL_OFFER_RESPONSE.to_vec());

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_update_state(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_credential_send_request(cb.command_handle, handle, cxn, 0, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_credential_get_request_msg() {
        let _setup = SetupMocks::init();

        let cxn = ::connection::tests::build_test_connection();

        let my_pw_did = CString::new(::connection::get_pw_did(cxn).unwrap()).unwrap().into_raw();
        let their_pw_did = CString::new(::connection::get_their_pw_did(cxn).unwrap()).unwrap().into_raw();

        let handle = credential::from_string(DEFAULT_SERIALIZED_CREDENTIAL).unwrap();

        AgencyMock::set_next_response(::utils::constants::NEW_CREDENTIAL_OFFER_RESPONSE.to_vec());

        let cb = return_types_u32::Return_U32_U32::new().unwrap();
        assert_eq!(vcx_credential_update_state(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_credential_get_request_msg(cb.command_handle, handle, my_pw_did, their_pw_did, 0, Some(cb.get_callback())), error::SUCCESS.code_num);
        let msg = cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();

        ::serde_json::from_str::<CredentialRequest>(&msg).unwrap();
    }

    #[test]
    fn test_get_credential() {
        let _setup = SetupMocks::init();

        let handle = credential::from_string(FULL_CREDENTIAL_SERIALIZED).unwrap();
        let bad_handle = 1123;

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_credential(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        cb.receive(TimeoutUtils::some_medium()).unwrap().unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_credential(cb.command_handle, bad_handle, Some(cb.get_callback())), error::INVALID_CREDENTIAL_HANDLE.code_num);

        let handle = credential::from_string(DEFAULT_SERIALIZED_CREDENTIAL).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_credential(cb.command_handle, handle, Some(cb.get_callback())), error::SUCCESS.code_num);
        assert_eq!(cb.receive(TimeoutUtils::some_medium()).err(), Some(error::INVALID_STATE.code_num));
    }

    #[test]
    fn test_get_payment_txn() {
        let _setup = SetupMocks::init();

        let handle = credential::from_string(::utils::constants::FULL_CREDENTIAL_SERIALIZED).unwrap();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        vcx_credential_get_payment_txn(cb.command_handle, handle, Some(cb.get_callback()));
        cb.receive(TimeoutUtils::some_medium()).unwrap();
    }

    #[test]
    fn test_vcx_credential_release() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();

        assert_eq!(vcx_credential_release(handle + 1), error::INVALID_CREDENTIAL_HANDLE.code_num);

        assert_eq!(vcx_credential_release(handle), error::SUCCESS.code_num);

        assert_eq!(vcx_credential_release(handle), error::INVALID_CREDENTIAL_HANDLE.code_num);
    }
}
