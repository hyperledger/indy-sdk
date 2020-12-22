use std::env;
use std::fs;
use std::path::PathBuf;

use actix::prelude::*;
use base64;
use dirs_next;
use env_logger;
use failure::{err_msg, Error, Fail};
use futures::*;
use tokio_core::reactor::Core;

use crate::actors::admin::Admin;
use crate::actors::agent::Agent;
use crate::actors::forward_agent::ForwardAgent;
use crate::actors::ForwardA2AMsg;
use crate::domain::a2a::*;
use crate::domain::a2connection::*;
use crate::domain::admin_message::AdminQuery;
use crate::domain::config::*;
use crate::domain::invite::*;
use crate::domain::key_deligation_proof::*;
use crate::domain::status::*;
use crate::indy::{crypto, did, wallet, WalletHandle};
use crate::utils::futures::*;
use crate::domain::key_derivation::KeyDerivationFunction;
use std::sync::{RwLock, Arc};

pub const EDGE_AGENT_WALLET_ID: &'static str = "edge_agent_wallet_id";
pub const EDGE_AGENT_WALLET_CONFIG: &'static str = "{\"id\": \"edge_agent_wallet_id\"}";
pub const EDGE_AGENT_WALLET_PASSPHRASE: &'static str = "edge_agent_wallet_passphrase";
pub const EDGE_AGENT_WALLET_CREDENTIALS: &'static str = "{\"key\": \"edge_agent_wallet_passphrase\"}";
pub const EDGE_AGENT_DID: &'static str = "NcYxiDXkpYi6ov5FcYDi1e";
pub const EDGE_AGENT_DID_INFO: &'static str = "{\"did\": \"NcYxiDXkpYi6ov5FcYDi1e\", \"seed\": \"0000000000000000000000000000Edge\"}";
pub const EDGE_AGENT_DID_VERKEY: &'static str = "B4aUxMQdPFkwBtcNUgs4fAbJhLSbXjQmrXByzL6gfDEq";
pub const EDGE_PAIRWISE_DID: &'static str = "BJ8T5EQm8QoVopUR2sd5L2";
pub const EDGE_PAIRWISE_DID_INFO: &'static str = "{\"did\": \"BJ8T5EQm8QoVopUR2sd5L2\", \"seed\": \"00000000000000000000EdgePairwise\"}";
pub const EDGE_PAIRWISE_DID_VERKEY: &'static str = "6cTQci8sG8CEr3pNz71yqxbEch8CiNwoNhoUE7unpWkS";
pub const FORWARD_AGENT_ENDPOINT: &'static str = "http://localhost:8080";

pub const EDGE_PAIRWISE_DID_2: &'static str = "WNnf2uJPZNmvMmA6LkdVAp";
pub const EDGE_PAIRWISE_DID_INFO_2: &'static str = "{\"did\": \"WNnf2uJPZNmvMmA6LkdVAp\", \"seed\": \"0000000000000000000EdgePairwise2\"}";
pub const EDGE_PAIRWISE_DID_VERKEY_2: &'static str = "H1d58X25s91rTXdd46hTfn7mhtPmohQFYRHD379UtytR";

pub static mut FORWARD_AGENT_WALLET_HANDLE: WalletHandle = WalletHandle(0);
pub const FORWARD_AGENT_WALLET_ID: &'static str = "forward_agent_wallet_id";
pub const FORWARD_AGENT_WALLET_CONFIG: &'static str = "{\"id\": \"forward_agent_wallet_id\"}";
pub const FORWARD_AGENT_WALLET_PASSPHRASE: &'static str = "forward_agent_wallet_passphrase";
pub const FORWARD_AGENT_WALLET_CREDENTIALS: &'static str = "{\"key\": \"forward_agent_wallet_passphrase\"}";
pub const FORWARD_AGENT_DID: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
pub const FORWARD_AGENT_DID_SEED: &'static str = "0000000000000000000000000Forward";
pub const FORWARD_AGENT_DID_INFO: &'static str = "{\"did\": \"VsKV7grR1BUE29mG2Fm2kX\", \"seed\": \"0000000000000000000000000Forward\"}";
pub const FORWARD_AGENT_DID_VERKEY: &'static str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

pub const PHONE_NO: &'static str = "80000000000";
pub const PAYLOAD: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

pub fn indy_home_path() -> PathBuf {
    // TODO: FIXME: Provide better handling for the unknown home path case!!!
    let mut path = dirs_next::home_dir().unwrap_or(PathBuf::from("/home/indy"));
    path.push(if cfg!(target_os = "ios") { "Documents/.indy_client" } else { ".indy_client" });
    path
}

pub fn wallet_home_path() -> PathBuf {
    let mut path = indy_home_path();
    path.push("wallet");
    path
}

pub fn wallet_path(wallet_name: &str) -> PathBuf {
    let mut path = wallet_home_path();
    path.push(wallet_name);
    path
}

pub fn tmp_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("indy_client");
    path
}

pub fn tmp_file_path(file_name: &str) -> PathBuf {
    let mut path = tmp_path();
    path.push(file_name);
    path
}

pub fn cleanup_indy_home() {
    let path = indy_home_path();
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}

pub fn cleanup_temp() {
    let path = tmp_path();
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}

pub fn cleanup_storage() {
    cleanup_indy_home();
    cleanup_temp();
}

pub fn run_test<F, B>(f: F)
    where
        F: FnOnce(Addr<ForwardAgent>, Arc<RwLock<Admin>>) -> B + 'static,
        B: IntoFuture<Item=(), Error=Error> + 'static {
    crate::indy::logger::set_default_logger(None).ok();
    env_logger::try_init().ok();
    cleanup_storage();

    System::run(|| {
        Arbiter::spawn_fn(move || {
            let admin = Admin::create();
            let admin_for_test = admin.clone();
            future::ok(())
                .and_then(move |_| {
                    let admin = Some(admin);
                    ForwardAgent::create_or_restore(forward_agent_config(), wallet_storage_config(), admin)
                })
                .and_then(move |fw_agent| {
                    f(fw_agent, admin_for_test)
                })
                .and_then(|wallet_handle|
                    unsafe {
                        wallet::close_wallet(FORWARD_AGENT_WALLET_HANDLE)
                            .map_err(|err| err.context("Can't close Forward Agent wallet.`").into())
                    }
                )
                .map(move |_| {
                    System::current().stop()
                })
                .map_err(|err| panic!("Test error: {}!", err))
        })
    });
}

pub fn run_agent_test<F, B>(f: F)
    where
        F: FnOnce((WalletHandle, String, String, String, String, Addr<ForwardAgent>)) -> B + 'static,
        B: IntoFuture<Item=WalletHandle, Error=Error> + 'static {
    run_test(|forward_agent, admin| {
        future::ok(())
            .and_then(|()| {
                setup_agent(forward_agent)
            })
            .and_then(f)
            .map(|wallet_handle| wallet::close_wallet(wallet_handle).wait().unwrap())
    })
}

pub fn run_admin_test<F, B>(f: F)
    where
        F: FnOnce((WalletHandle, String, String, String, String, Addr<ForwardAgent>, Arc<RwLock<Admin>>)) -> B + 'static,
        B: IntoFuture<Item=WalletHandle, Error=Error> + 'static {
    run_test(|forward_agent, admin| {
        future::ok(())
            .and_then(|()| {
                setup_agent(forward_agent)
            })
            .and_then(move |(agent_wallet_handle, agent_did, agent_verkey, with_pairwise_did, with_pairwise_did_verkey, forward_agent)| {
                f((agent_wallet_handle, agent_did, agent_verkey, with_pairwise_did, with_pairwise_did_verkey, forward_agent, admin))
            })
            .map(|wallet_handle| wallet::close_wallet(wallet_handle).wait().unwrap())
    })
}

pub fn setup_agent(forward_agent: Addr<ForwardAgent>) -> ResponseFuture<(WalletHandle, String, String, String, String, Addr<ForwardAgent>), Error> {
    future::ok(())
        .and_then(|()| {
            let e_wallet_handle = edge_wallet_setup().wait().unwrap();
            let connect_msg = compose_connect(e_wallet_handle).wait().unwrap();
            forward_agent
                .send(ForwardA2AMsg(connect_msg))
                .from_err()
                .and_then(|res| res)
                .map(move |connected_msg| (forward_agent, e_wallet_handle, connected_msg))
        })
        .and_then(|(forward_agent, e_wallet_handle, connected_msg)| {
            let (sender_verkey, pairwise_did, pairwise_verkey) = decompose_connected(e_wallet_handle, &connected_msg).wait().unwrap();
            let signup_msg = compose_signup(e_wallet_handle, &pairwise_did, &pairwise_verkey).wait().unwrap();
            forward_agent
                .send(ForwardA2AMsg(signup_msg))
                .from_err()
                .and_then(|res| res)
                .map(move |signedup_msg| (forward_agent, e_wallet_handle, signedup_msg, pairwise_did, pairwise_verkey))
        })
        .and_then(move |(forward_agent, e_wallet_handle, signedup_msg, pairwise_did, pairwise_verkey)| {
            let sender_verkey = decompose_signedup(e_wallet_handle, &signedup_msg).wait().unwrap();
            let create_agent_msg = compose_create_agent(e_wallet_handle, &pairwise_did, &pairwise_verkey).wait().unwrap();
            forward_agent
                .send(ForwardA2AMsg(create_agent_msg))
                .from_err()
                .and_then(|res| res)
                .map(move |agent_created_msg| (e_wallet_handle, agent_created_msg, pairwise_verkey, forward_agent))
        })
        .and_then(|(e_wallet_handle, agent_created_msg, pairwise_verkey, forward_agent)| {
            let (_, agent_did, agent_verkey) = decompose_agent_created(e_wallet_handle, &agent_created_msg).wait().unwrap();

            let create_key_msg = compose_create_key(e_wallet_handle, &agent_did, &agent_verkey, EDGE_PAIRWISE_DID, EDGE_PAIRWISE_DID_VERKEY).wait().unwrap();
            forward_agent
                .send(ForwardA2AMsg(create_key_msg))
                .from_err()
                .and_then(|res| res)
                .map(move |key_created_msg| (e_wallet_handle, key_created_msg, agent_did, agent_verkey, forward_agent))
        })
        .map(|(e_wallet_handle, key_created_msg, agent_did, agent_verkey, forward_agent)| {
            let (_, key) = decompose_key_created(e_wallet_handle, &key_created_msg).wait().unwrap();
            (e_wallet_handle, agent_did, agent_verkey, key.with_pairwise_did, key.with_pairwise_did_verkey, forward_agent)
        })
        .into_box()
}

pub fn forward_agent_config() -> ForwardAgentConfig {
    ForwardAgentConfig {
        wallet_id: FORWARD_AGENT_WALLET_ID.into(),
        wallet_passphrase: FORWARD_AGENT_WALLET_PASSPHRASE.into(),
        did: FORWARD_AGENT_DID.into(),
        did_seed: Some(FORWARD_AGENT_DID_SEED.into()),
        endpoint: FORWARD_AGENT_ENDPOINT.into(),
    }
}

pub fn wallet_storage_config() -> WalletStorageConfig {
    WalletStorageConfig {
        xtype: None,
        config: None,
        credentials: None,
        plugin_library_path: None,
        plugin_init_function: None
    }
}

pub fn edge_wallet_setup() -> BoxedFuture<WalletHandle, Error> {
    future::ok(())
        .and_then(|_| {
            wallet::create_wallet(EDGE_AGENT_WALLET_CONFIG, EDGE_AGENT_WALLET_CREDENTIALS)
                .map_err(|err| err.context("Can't create edge agent wallet.").into())
        })
        .and_then(|_| {
            wallet::open_wallet(EDGE_AGENT_WALLET_CONFIG, EDGE_AGENT_WALLET_CREDENTIALS)
                .map_err(|err| err.context("Can't open edge agent wallet.").into())
        })
        .and_then(|wallet_handle| {
            did::create_and_store_my_did(wallet_handle, EDGE_AGENT_DID_INFO)
                .map(move |_| wallet_handle)
                .map_err(|err| err.context("Can't create edge agent did.").into())
        })
        .and_then(|wallet_handle| {
            did::create_and_store_my_did(wallet_handle, EDGE_PAIRWISE_DID_INFO)
                .map(move |_| wallet_handle)
                .map_err(|err| err.context("Can't create edge agent did.").into())
        })
        .into_box()
}

pub fn compose_connect(wallet_handle: WalletHandle) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::Connect(
        Connect {
            from_did: EDGE_AGENT_DID.into(),
            from_did_verkey: EDGE_AGENT_DID_VERKEY.into(),
        }))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              FORWARD_AGENT_DID_VERKEY,
                                              &msgs).wait().unwrap();
    compose_forward(wallet_handle, FORWARD_AGENT_DID, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn decompose_connected(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, String, String), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            if let Some(A2AMessage::Version1(A2AMessageV1::Connected(msg))) = msgs.pop() {
                let Connected { with_pairwise_did: pairwise_did, with_pairwise_did_verkey: pairwise_verkey } = msg;
                Ok((sender_verkey, pairwise_did, pairwise_verkey))
            } else {
                Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_signup(wallet_handle: WalletHandle, pairwise_did: &str, pairwise_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::SignUp(SignUp {}))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              pairwise_verkey,
                                              &msgs).wait().unwrap();
    compose_forward(wallet_handle, &pairwise_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn decompose_signedup(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<String, Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            if let Some(A2AMessage::Version1(A2AMessageV1::SignedUp(_))) = msgs.pop() {
                Ok(sender_verkey)
            } else {
                Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_create_agent(wallet_handle: WalletHandle, pairwise_did: &str, pairwise_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = vec![A2AMessage::Version1(A2AMessageV1::CreateAgent(CreateAgent {}))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              pairwise_verkey,
                                              &msgs).wait().unwrap();
    compose_forward(wallet_handle, pairwise_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn decompose_agent_created(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, String, String), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            if let Some(A2AMessage::Version1(A2AMessageV1::AgentCreated(agent_created))) = msgs.pop() {
                let AgentCreated { with_pairwise_did: pw_did, with_pairwise_did_verkey: pw_vk } = agent_created;
                Ok((sender_verkey, pw_did, pw_vk))
            } else {
                Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_create_key(wallet_handle: WalletHandle, agent_did: &str, agent_verkey: &str, for_did: &str, for_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::CreateKey(
        CreateKey {
            for_did: for_did.into(),
            for_did_verkey: for_verkey.into(),
        }))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              agent_verkey,
                                              &msgs).wait().unwrap();
    compose_forward(wallet_handle, agent_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn decompose_key_created(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, KeyCreated), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            if let Some(A2AMessage::Version1(A2AMessageV1::KeyCreated(key_created))) = msgs.pop() {
                Ok((sender_verkey, key_created))
            } else {
                Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_create_connection_request(wallet_handle: WalletHandle,
                                         agent_did: &str,
                                         agent_verkey: &str,
                                         agent_pairwise_did: &str,
                                         agent_pairwise_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let create_msg = build_create_message_request(RemoteMessageType::ConnReq, None);

    let msg_details = A2AMessage::Version1(A2AMessageV1::MessageDetail(
        MessageDetail::ConnectionRequest(
            ConnectionRequestMessageDetail {
                key_dlg_proof: gen_key_delegated_proof(wallet_handle, EDGE_PAIRWISE_DID_VERKEY, &agent_pairwise_did, &agent_pairwise_verkey),
                target_name: None,
                phone_no: Some(PHONE_NO.to_string()),
                use_public_did: Some(true),
                thread_id: None,
            })));

    let msgs = [create_msg, msg_details];

    compose_message(wallet_handle, &msgs, agent_pairwise_did, agent_pairwise_verkey, agent_did, agent_verkey)
}

pub fn decompose_connection_request_created(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, String, InviteDetail), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(2, msgs.len());
            match (msgs.remove(0), msgs.remove(0)) {
                (A2AMessage::Version1(A2AMessageV1::MessageCreated(msg_created)),
                    A2AMessage::Version1(A2AMessageV1::MessageDetail(MessageDetail::ConnectionRequestResp(msg_details)))) =>
                    Ok((sender_verkey, msg_created.uid, msg_details.invite_detail)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_create_connection_request_answer(wallet_handle: WalletHandle,
                                                agent_did: &str,
                                                agent_verkey: &str,
                                                agent_pairwise_did: &str,
                                                agent_pairwise_verkey: &str,
                                                reply_to_msg_id: &str) -> BoxedFuture<Vec<u8>, Error> {
    let (remote_user_pw_did, remote_user_pw_verkey) = did::create_and_store_my_did(wallet_handle, "{}").wait().unwrap();
    let (remote_agent_pw_did, remote_agent_pw_verkey) = did::create_and_store_my_did(wallet_handle, "{}").wait().unwrap();

    let create_msg = build_create_message_request(RemoteMessageType::ConnReqAnswer, Some(reply_to_msg_id.to_string()));

    let msg_details: A2AMessage = A2AMessage::Version1(A2AMessageV1::MessageDetail(
        MessageDetail::ConnectionRequestAnswer(
            ConnectionRequestAnswerMessageDetail {
                key_dlg_proof: Some(gen_key_delegated_proof(wallet_handle, EDGE_PAIRWISE_DID_VERKEY, &agent_pairwise_did, &agent_pairwise_verkey)),
                sender_detail: SenderDetail {
                    did: remote_user_pw_did,
                    verkey: remote_user_pw_verkey.clone(),
                    agent_key_dlg_proof: gen_key_delegated_proof(wallet_handle, &remote_user_pw_verkey, &remote_agent_pw_did, &remote_agent_pw_verkey),
                    name: None,
                    logo_url: None,
                    public_did: None,
                },
                sender_agency_detail: ForwardAgentDetail {
                    did: FORWARD_AGENT_DID.to_string(),
                    verkey: FORWARD_AGENT_DID_VERKEY.to_string(),
                    endpoint: FORWARD_AGENT_ENDPOINT.to_string(),
                },
                answer_status_code: MessageStatusCode::Accepted,
                thread: None,
            }
        )));

    let msgs = [create_msg, msg_details];

    compose_message(wallet_handle, &msgs, agent_pairwise_did, agent_pairwise_verkey, agent_did, agent_verkey)
}

pub fn decompose_connection_request_answer_created(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, String), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(1, msgs.len());
            match msgs.remove(0) {
                A2AMessage::Version1(A2AMessageV1::MessageCreated(msg_created)) => Ok((sender_verkey, msg_created.uid)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_create_general_message(wallet_handle: WalletHandle,
                                      agent_did: &str,
                                      agent_verkey: &str,
                                      agent_pairwise_did: &str,
                                      agent_pairwise_verkey: &str,
                                      mtype: RemoteMessageType) -> BoxedFuture<Vec<u8>, Error> {
    let create_msg = build_create_message_request(mtype, None);

    let msg_details: A2AMessage = A2AMessage::Version1(A2AMessageV1::MessageDetail(
        MessageDetail::General(
            GeneralMessageDetail {
                msg: PAYLOAD.to_vec(),
                title: None,
                detail: None,
            }
        )
    ));

    let msgs = [create_msg, msg_details];

    compose_message(wallet_handle, &msgs, agent_pairwise_did, agent_pairwise_verkey, agent_did, agent_verkey)
}

fn build_create_message_request(mtype: RemoteMessageType, reply_to_msg_id: Option<String>) -> A2AMessage {
    A2AMessage::Version1(A2AMessageV1::CreateMessage(CreateMessage {
        mtype,
        send_msg: false,
        uid: None,
        reply_to_msg_id,
    }))
}

pub fn decompose_general_message_created(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, String), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(1, msgs.len());
            match msgs.remove(0) {
                A2AMessage::Version1(A2AMessageV1::MessageCreated(msg_created)) => Ok((sender_verkey, msg_created.uid)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_get_messages(wallet_handle: WalletHandle,
                            agent_did: &str,
                            agent_verkey: &str,
                            agent_pairwise_did: &str,
                            agent_pairwise_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::GetMessages(GetMessages {
        exclude_payload: None,
        uids: Vec::new(),
        status_codes: Vec::new(),
    }))];

    compose_message(wallet_handle, &msgs, agent_pairwise_did, agent_pairwise_verkey, agent_did, agent_verkey)
}

pub fn decompose_get_messages(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, Vec<GetMessagesDetailResponse>), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(1, msgs.len());
            match msgs.remove(0) {
                A2AMessage::Version1(A2AMessageV1::Messages(messages)) => Ok((sender_verkey, messages.msgs)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_update_message_status_message(wallet_handle: WalletHandle,
                                             agent_did: &str,
                                             agent_verkey: &str,
                                             agent_pairwise_did: &str,
                                             agent_pairwise_verkey: &str,
                                             uid: &str,
                                             status_code: MessageStatusCode) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::UpdateMessageStatus(UpdateMessageStatus {
        uids: vec![uid.to_string()],
        status_code,
    }))];

    compose_message(wallet_handle, &msgs, agent_pairwise_did, agent_pairwise_verkey, agent_did, agent_verkey)
}

pub fn decompose_message_status_updated(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, MessageStatusUpdated), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(1, msgs.len());
            match msgs.remove(0) {
                A2AMessage::Version1(A2AMessageV1::MessageStatusUpdated(msg)) => Ok((sender_verkey, msg)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_update_connection_status_message(wallet_handle: WalletHandle,
                                                agent_did: &str,
                                                agent_verkey: &str,
                                                agent_pairwise_did: &str,
                                                agent_pairwise_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::UpdateConnectionStatus(UpdateConnectionStatus {
        status_code: ConnectionStatus::Deleted
    }))];

    compose_message(wallet_handle, &msgs, agent_pairwise_did, agent_pairwise_verkey, agent_did, agent_verkey)
}

pub fn decompose_connection_status_updated(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, ConnectionStatusUpdated), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(1, msgs.len());
            match msgs.remove(0) {
                A2AMessage::Version1(A2AMessageV1::ConnectionStatusUpdated(msg)) => Ok((sender_verkey, msg)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_get_messages_by_connection(wallet_handle: WalletHandle,
                                          agent_did: &str,
                                          agent_verkey: &str,
                                          agent_pairwise_did: &str,
                                          agent_pairwise_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::GetMessagesByConnections(GetMessagesByConnections {
        exclude_payload: None,
        uids: Vec::new(),
        status_codes: Vec::new(),
        pairwise_dids: Vec::new(),
    }))];
    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              agent_verkey,
                                              &msgs).wait().unwrap();
    compose_forward(wallet_handle, agent_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn decompose_get_messages_by_connection(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<(String, Vec<MessagesByConnection>), Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            assert_eq!(1, msgs.len());
            match msgs.remove(0) {
                A2AMessage::Version1(A2AMessageV1::MessagesByConnections(messages)) => Ok((sender_verkey, messages.msgs)),
                _ => Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_update_configs(wallet_handle: WalletHandle, agent_did: &str, agent_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::UpdateConfigs(
        UpdateConfigs {
            configs: vec![
                ConfigOption { name: "zoom_zoom".to_string(), value: "value".to_string() },
                ConfigOption { name: "name".to_string(), value: "super agent".to_string() },
                ConfigOption { name: "logoUrl".to_string(), value: "http://logo.url".to_string() }
            ]
        }))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              agent_verkey,
                                              &msgs).wait().unwrap();

    compose_forward(wallet_handle, agent_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn compose_get_configs(wallet_handle: WalletHandle, agent_did: &str, agent_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::GetConfigs(
        GetConfigs {
            configs: vec![String::from("name"), String::from("logoUrl")]
        }))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              agent_verkey,
                                              &msgs).wait().unwrap();

    compose_forward(wallet_handle, agent_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn decompose_configs(wallet_handle: WalletHandle, msg: &[u8]) -> BoxedFuture<Vec<ConfigOption>, Error> {
    A2AMessage::unbundle_authcrypted(wallet_handle, EDGE_AGENT_DID_VERKEY, &msg)
        .and_then(|(sender_verkey, mut msgs)| {
            if let Some(A2AMessage::Version1(A2AMessageV1::Configs(configs))) = msgs.pop() {
                Ok((configs.configs))
            } else {
                Err(err_msg("Invalid message"))
            }
        })
        .into_box()
}

pub fn compose_remove_configs(wallet_handle: WalletHandle, agent_did: &str, agent_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::RemoveConfigs(
        RemoveConfigs {
            configs: vec![String::from("name")]
        }))];

    let msg = A2AMessage::prepare_authcrypted(wallet_handle,
                                              EDGE_AGENT_DID_VERKEY,
                                              agent_verkey,
                                              &msgs).wait().unwrap();

    compose_forward(wallet_handle, agent_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn compose_forward(wallet_handle: WalletHandle, recipient_did: &str, recipient_vk: &str, msg: Vec<u8>) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::Forward(
        ForwardV1 {
            fwd: recipient_did.into(),
            msg,
        }))];

    A2AMessage::prepare_anoncrypted(wallet_handle, recipient_vk, &msgs)
}

pub fn compose_authcrypted_forward(wallet_handle: WalletHandle, sender_vk: &str, recipient_did: &str, recipient_vk: &str, msg: Vec<u8>) -> BoxedFuture<Vec<u8>, Error> {
    let msgs = [A2AMessage::Version1(A2AMessageV1::Forward(
        ForwardV1 {
            fwd: recipient_did.into(),
            msg,
        }))];

    A2AMessage::prepare_authcrypted(wallet_handle, sender_vk, recipient_vk, &msgs)
}

pub fn compose_message(wallet_handle: WalletHandle,
                       msgs: &[A2AMessage],
                       agent_pairwise_did: &str,
                       agent_pairwise_verkey: &str,
                       agent_did: &str,
                       agent_verkey: &str) -> BoxedFuture<Vec<u8>, Error> {
    let msg = A2AMessage::prepare_authcrypted(wallet_handle, EDGE_PAIRWISE_DID_VERKEY, agent_pairwise_verkey, &msgs).wait().unwrap();

    let msg = compose_authcrypted_forward(wallet_handle, EDGE_AGENT_DID_VERKEY, agent_pairwise_did, agent_verkey, msg).wait().unwrap();

    compose_forward(wallet_handle, agent_did, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn gen_key_delegated_proof(wallet_handle: WalletHandle, signer_vk: &str, did: &str, verkey: &str) -> KeyDlgProof {
    let signature = format!("{}{}", did, verkey);
    let signature = crypto::sign(wallet_handle, signer_vk, signature.as_bytes()).wait().unwrap();
    let signature = base64::encode(&signature);
    KeyDlgProof {
        agent_did: did.into(),
        agent_delegated_key: verkey.into(),
        signature,
    }
}
