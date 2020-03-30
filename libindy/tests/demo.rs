#[macro_use]
mod utils;

inject_indy_dependencies!();

extern crate indyrs as indy;
extern crate indyrs as api;
extern crate indy_sys;

#[cfg(feature = "local_nodes_pool")]
use crate::utils::callback;
use crate::utils::constants::{WALLET_CREDENTIALS, PROTOCOL_VERSION};
use crate::utils::{pool as pool_utils, timeout};
use crate::utils::domain::anoncreds::credential_definition::CredentialDefinition;
use crate::utils::domain::anoncreds::credential_for_proof_request::CredentialsForProofRequest;
use crate::utils::domain::anoncreds::proof::Proof;
use crate::utils::domain::anoncreds::revocation_registry_definition::RevocationRegistryDefinition;
use crate::utils::domain::anoncreds::revocation_registry::RevocationRegistry;
use crate::utils::domain::anoncreds::revocation_state::RevocationState;
use crate::utils::domain::anoncreds::schema::Schema;

use crate::utils::environment;
use crate::utils::Setup;

use self::indy::ErrorCode;
use self::indy_sys::*;

use std::ptr::null;
use std::ffi::CString;

#[cfg(feature = "local_nodes_pool")]
use std::thread;

#[test]
fn anoncreds_demo_works() {
    Setup::empty();

    let (issuer_create_wallet_receiver, issuer_create_wallet_command_handle, issuer_create_wallet_callback) = callback::_closure_to_cb_ec();
    let (prover_create_wallet_receiver, prover_create_wallet_command_handle, prover_create_wallet_callback) = callback::_closure_to_cb_ec();
    let (issuer_open_wallet_receiver, issuer_open_wallet_command_handle, issuer_open_wallet_callback) = callback::_closure_to_cb_ec_wallethandle();
    let (prover_open_wallet_receiver, prover_open_wallet_command_handle, prover_open_wallet_callback) = callback::_closure_to_cb_ec_wallethandle();
    let (issuer_create_schema_receiver, issuer_create_schema_command_handle, issuer_create_schema_callback) = callback::_closure_to_cb_ec_string_string();
    let (issuer_create_credential_definition_receiver, issuer_create_credential_definition_command_handle, issuer_create_credential_definition_callback) = callback::_closure_to_cb_ec_string_string();
    let (issuer_create_credential_offer_receiver, issuer_create_credential_offer_command_handle, issuer_create_credential_offer_callback) = callback::_closure_to_cb_ec_string();
    let (prover_create_master_secret_receiver, prover_create_master_secret_command_handle, prover_create_master_secret_callback) = callback::_closure_to_cb_ec_string();
    let (prover_create_credential_req_receiver, prover_create_credential_req_command_handle, prover_create_credential_req_callback) = callback::_closure_to_cb_ec_string_string();
    let (issuer_create_credential_receiver, issuer_create_credential_command_handle, issuer_create_credential_callback) = callback::_closure_to_cb_ec_string_opt_string_opt_string();
    let (prover_store_credential_receiver, prover_store_credential_command_handle, prover_store_credential_callback) = callback::_closure_to_cb_ec_string();
    let (prover_get_credentials_for_proof_req_receiver, prover_get_credentials_for_proof_req_command_handle, prover_get_credentials_for_proof_req_callback) = callback::_closure_to_cb_ec_string();
    let (prover_create_proof_receiver, prover_create_proof_command_handle, prover_create_proof_callback) = callback::_closure_to_cb_ec_string();
    let (verifier_verify_proof_receiver, verifier_verify_proof_command_handle, verifier_verify_proof_callback) = callback::_closure_to_cb_ec_bool();
    let (issuer_close_wallet_receiver, issuer_close_wallet_command_handle, issuer_close_wallet_callback) = callback::_closure_to_cb_ec();
    let (prover_close_wallet_receiver, prover_close_wallet_command_handle, prover_close_wallet_callback) = callback::_closure_to_cb_ec();
    let (bs_writer_receiver, bs_writer_command_handle, bs_writer_cb) = callback::_closure_to_cb_ec_i32();
    let (bs_reader_receiver, bs_reader_command_handle, bs_reader_cb) = callback::_closure_to_cb_ec_i32();
    let (cs_rev_reg_receiver, cs_rev_reg_command_handle, cs_rev_reg_cb) = callback::_closure_to_cb_ec_string_string_string();
    let (create_rev_state_receiver, create_rev_state_command_handle, create_rev_state_cb) = callback::_closure_to_cb_ec_string();

    let issuer_wallet_config = json!({"id": "issuer_wallet"}).to_string();
    let issuer_wallet_credentials = json!({"key":"issuerKey1111111111111111111111111111111111", "key_derivation_method":"RAW"}).to_string();

    // Issuer Creates Wallet
    let err =
        unsafe {
            wallet::indy_create_wallet(issuer_create_wallet_command_handle,
                                       CString::new(issuer_wallet_config.as_str()).unwrap().as_ptr(),
                                       CString::new(issuer_wallet_credentials.as_str()).unwrap().as_ptr(),
                                       issuer_create_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let err = issuer_create_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Issuer Opens Wallet
    let err =
        unsafe {
            wallet::indy_open_wallet(issuer_open_wallet_command_handle,
                                     CString::new(issuer_wallet_config.as_str()).unwrap().as_ptr(),
                                     CString::new(issuer_wallet_credentials.as_str()).unwrap().as_ptr(),
                                     issuer_open_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, issuer_wallet_handle) = issuer_open_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Prover Creates Wallet
    let prover_wallet_config = json!({"id": "prover_wallet"}).to_string();
    let prover_wallet_credentials = json!({"key":"ProverKey1111111111111111111111111111111111", "key_derivation_method":"RAW"}).to_string();

    let err =
        unsafe {
            wallet::indy_create_wallet(prover_create_wallet_command_handle,
                                       CString::new(prover_wallet_config.as_str()).unwrap().as_ptr(),
                                       CString::new(prover_wallet_credentials.as_str()).unwrap().as_ptr(),
                                       prover_create_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let err = prover_create_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Prover Opens Wallet
    let err =
        unsafe {
            wallet::indy_open_wallet(prover_open_wallet_command_handle,
                                     CString::new(prover_wallet_config.as_str()).unwrap().as_ptr(),
                                     CString::new(prover_wallet_credentials.as_str()).unwrap().as_ptr(),
                                     prover_open_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, prover_wallet_handle) = prover_open_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));


    let issuer_did = "NcYxiDXkpYi6ov5FcYDi1e";
    let prover_did = "VsKV7grR1BUE29mG2Fm2kX";
    let schema_name = "gvt";
    let version = "1.0";
    let attrs = r#"["name", "age", "sex", "height", "empty_param", "ssn", "zero_param"]"#;

    // Issuer create Schema
    let err =
        unsafe {
            anoncreds::indy_issuer_create_schema(issuer_create_schema_command_handle,
                                                 CString::new(issuer_did).unwrap().as_ptr(),
                                                 CString::new(schema_name).unwrap().as_ptr(),
                                                 CString::new(version).unwrap().as_ptr(),
                                                 CString::new(attrs).unwrap().as_ptr(),
                                                 issuer_create_schema_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, schema_id, schema_json) = issuer_create_schema_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Issuer create Credential Definition for Schema
    let tag = r#"TAG1"#;
    let config = r#"{ "support_revocation": true }"#;

    let err =
        unsafe {
            anoncreds::indy_issuer_create_and_store_credential_def(issuer_create_credential_definition_command_handle,
                                                                   issuer_wallet_handle,
                                                                   CString::new(issuer_did).unwrap().as_ptr(),
                                                                   CString::new(schema_json.as_str()).unwrap().as_ptr(),
                                                                   CString::new(tag).unwrap().as_ptr(),
                                                                   null(),
                                                                   CString::new(config).unwrap().as_ptr(),
                                                                   issuer_create_credential_definition_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, credential_def_id, credential_def_json) = issuer_create_credential_definition_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Issuer configure blob storage for Tails then create and store RevocationRegistry
    let tails_writer_config = json!({
        "base_dir": environment::tmp_file_path("tails").to_str().unwrap(),
        "uri_pattern":"",
    }).to_string();

    let err = unsafe {
        blob_storage::indy_open_blob_storage_writer(bs_writer_command_handle,
                                                    CString::new("default").unwrap().as_ptr(),
                                                    CString::new(tails_writer_config).unwrap().as_ptr(),
                                                    bs_writer_cb)
    };
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, tails_writer_handle) = bs_writer_receiver.recv_timeout(timeout::short_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    let err = unsafe {
        anoncreds::indy_issuer_create_and_store_revoc_reg(cs_rev_reg_command_handle,
                                                          issuer_wallet_handle,
                                                          CString::new(issuer_did).unwrap().as_ptr(),
                                                          null(),
                                                          CString::new("TAG1").unwrap().as_ptr(),
                                                          CString::new(credential_def_id.as_str()).unwrap().as_ptr(),
                                                          CString::new(r#"{"max_cred_num":5, "issuance_type":"ISSUANCE_ON_DEMAND"}"#).unwrap().as_ptr(),
                                                          tails_writer_handle,
                                                          cs_rev_reg_cb)
    };
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, rev_reg_id, revoc_reg_def_json, _) = cs_rev_reg_receiver.recv().unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));


    // Prover create Master Secret
    let master_secret_id = "master_secret";
    let err =
        unsafe {
            anoncreds::indy_prover_create_master_secret(prover_create_master_secret_command_handle,
                                                        prover_wallet_handle,
                                                        CString::new(master_secret_id).unwrap().as_ptr(),
                                                        prover_create_master_secret_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, _) = prover_create_master_secret_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Issuer create Credential Offer
    let err =
        unsafe {
            anoncreds::indy_issuer_create_credential_offer(issuer_create_credential_offer_command_handle,
                                                           issuer_wallet_handle,
                                                           CString::new(credential_def_id.as_str()).unwrap().as_ptr(),
                                                           issuer_create_credential_offer_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, credential_offer_json) = issuer_create_credential_offer_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Prover create Credential Request
    let err =
        unsafe {
            anoncreds::indy_prover_create_credential_req(prover_create_credential_req_command_handle,
                                                         prover_wallet_handle,
                                                         CString::new(prover_did).unwrap().as_ptr(),
                                                         CString::new(credential_offer_json.as_str()).unwrap().as_ptr(),
                                                         CString::new(credential_def_json.as_str()).unwrap().as_ptr(),
                                                         CString::new(master_secret_id).unwrap().as_ptr(),
                                                         prover_create_credential_req_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, credential_req_json, credential_req_metadata_json) = prover_create_credential_req_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Issuer create Credential for Credential Request
    // note that encoding is not standardized by Indy except that 32-bit integers are encoded as themselves. IS-786
    let credential_json = json!({
        "sex": { "raw": "male", "encoded": "5944657099558967239210949258394887428692050081607692519917050011144233115103" },
        "name": { "raw": "Alex", "encoded": "1139481716457488690172217916278103335" },
        "height": { "raw": "175", "encoded": "175" },
        "age": { "raw": "28", "encoded": "28" },
        "empty_param": { "raw": "", "encoded": "111222333" },
        "ssn": { "raw": "00000001", "encoded": "00000001" },
        "zero_param": { "raw": "0", "encoded": "0" },
    }).to_string();

    // Creating credential requires access to Tails: Issuer configure blob storage to read
    let tails_reader_config = json!({
        "base_dir": environment::tmp_file_path("tails").to_str().unwrap(),
    }).to_string();
    let err = unsafe {
        blob_storage::indy_open_blob_storage_reader(bs_reader_command_handle,
                                                    CString::new("default").unwrap().as_ptr(),
                                                    CString::new(tails_reader_config).unwrap().as_ptr(),
                                                    bs_reader_cb)
    };
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, blob_storage_reader_handle) =
        bs_reader_receiver.recv_timeout(timeout::short_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    let err =
        unsafe {
            anoncreds::indy_issuer_create_credential(issuer_create_credential_command_handle,
                                                     issuer_wallet_handle,
                                                     CString::new(credential_offer_json).unwrap().as_ptr(),
                                                     CString::new(credential_req_json.as_str()).unwrap().as_ptr(),
                                                     CString::new(credential_json).unwrap().as_ptr(),
                                                     CString::new(rev_reg_id.as_str()).unwrap().as_ptr(),
                                                     blob_storage_reader_handle,
                                                     issuer_create_credential_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, credential_json, cred_rev_id, rreg_issue_delta_json) =
        issuer_create_credential_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let rreg_issue_delta_json = rreg_issue_delta_json.unwrap();
    let cred_rev_id = cred_rev_id.unwrap();

    // Prover process and store Credential
    let credential_id = "credential_id";
    let err =
        unsafe {
            anoncreds::indy_prover_store_credential(prover_store_credential_command_handle,
                                                    prover_wallet_handle,
                                                    CString::new(credential_id).unwrap().as_ptr(),
                                                    CString::new(credential_req_metadata_json).unwrap().as_ptr(),
                                                    CString::new(credential_json).unwrap().as_ptr(),
                                                    CString::new(credential_def_json.as_str()).unwrap().as_ptr(),
                                                    CString::new(revoc_reg_def_json.as_str()).unwrap().as_ptr(),
                                                    prover_store_credential_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, _) = prover_store_credential_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    let proof_req_json = json!({
        "nonce": "123432421212",
        "name": "proof_req_1",
        "version": "0.1",
        "requested_attributes": {
            "attr1_referent": {
                "names": ["name", "height", "sex", "ssn", "zero_param"],
                "restrictions": {
                    "attr::name::value": "Alex",
                    "attr::sex::value": "male"
                }
            },
            "attr2_referent": {
                "name": "empty_param",
                "restrictions": {
                    "attr::empty_param::value": ""
                }
            }
        },
        "requested_predicates": {
            "predicate1_referent": {
                "name": "age",
                "p_type": ">=",
                "p_value": 18
            }
        },
        "non_revoked": { "from": 80, "to": 120 }
    }).to_string();

    // Prover prepare Credential to prove
    // Prover gets Credentials for Proof Request
    #[allow(deprecated)] //TODO FIXME use new one
    let err =
        unsafe {
            anoncreds::indy_prover_get_credentials_for_proof_req(prover_get_credentials_for_proof_req_command_handle,
                                                                 prover_wallet_handle,
                                                                 CString::new(proof_req_json.as_str()).unwrap().as_ptr(),
                                                                 prover_get_credentials_for_proof_req_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, credentials_json) = prover_get_credentials_for_proof_req_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
    let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
    assert_eq!(1, credentials_for_attr_1.len());

    let credential = credentials_for_attr_1[0].cred_info.clone();

    // Prover select appropriate timestamp for revocation part of each credential and build states
    let issue_ts = 100;

    let err = unsafe {
        anoncreds::indy_create_revocation_state(create_rev_state_command_handle,
                                                blob_storage_reader_handle,
                                                CString::new(revoc_reg_def_json.as_str()).unwrap().as_ptr(),
                                                CString::new(rreg_issue_delta_json.as_str()).unwrap().as_ptr(),
                                                issue_ts,
                                                CString::new(cred_rev_id).unwrap().as_ptr(),
                                                create_rev_state_cb)
    };
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, rev_state_json) = create_rev_state_receiver.recv().unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let rev_state_json: RevocationState = serde_json::from_str(&rev_state_json).unwrap();

    // Prover collect map with revocation states in the next format:
    // rev_reg_id1 -> {
    //  ts1 -> state1_1,
    //  ts2 -> state1_2
    //  ...
    // },
    // rev_reg2 -> { ... }
    let rev_states_json = json!({
        rev_reg_id.as_str(): {
            issue_ts.to_string(): rev_state_json
        }
    }).to_string();

    let requested_credentials_json = json!({
        "self_attested_attributes": {},
        "requested_attributes": {
            "attr1_referent": {
                "cred_id": credential.referent,
                "timestamp": issue_ts,
                "revealed": true
            },
            "attr2_referent": {
                "cred_id": credential.referent,
                "timestamp": issue_ts,
                "revealed": true
            }
        },
        "requested_predicates":{
            "predicate1_referent":{
                "cred_id": credential.referent,
                "timestamp": issue_ts
            }
        }
    }).to_string();

    let schemas_json = json!({
        schema_id.as_str(): serde_json::from_str::<Schema>(&schema_json).unwrap()
    }).to_string();
    let credential_defs_json = json!({
        credential_def_id.as_str(): serde_json::from_str::<CredentialDefinition>(&credential_def_json).unwrap()
    }).to_string();

    // Prover create Proof for Proof Request
    let err =
        unsafe {
            anoncreds::indy_prover_create_proof(prover_create_proof_command_handle,
                                                prover_wallet_handle,
                                                CString::new(proof_req_json.as_str()).unwrap().as_ptr(),
                                                CString::new(requested_credentials_json).unwrap().as_ptr(),
                                                CString::new(master_secret_id).unwrap().as_ptr(),
                                                CString::new(schemas_json.as_str()).unwrap().as_ptr(),
                                                CString::new(credential_defs_json.as_str()).unwrap().as_ptr(),
                                                CString::new(rev_states_json.as_str()).unwrap().as_ptr(),
                                                prover_create_proof_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, proof_json) = prover_create_proof_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // Verifier verify proof
    let proof: Proof = serde_json::from_str(&proof_json).unwrap();

    let revealed_attr_1 = proof.requested_proof.revealed_attr_groups.get("attr1_referent").unwrap();
    assert_eq!("Alex", revealed_attr_1.values["name"].raw);

    let rev_reg_defs_json = json!({
        rev_reg_id.as_str(): serde_json::from_str::<RevocationRegistryDefinition>(&revoc_reg_def_json).unwrap()
    }).to_string();

    let rev_regs_json = json!({
        rev_reg_id: {
            issue_ts.to_string(): serde_json::from_str::<RevocationRegistry>(&rreg_issue_delta_json).unwrap()
        }
    }).to_string();

    let err =
        unsafe {
            anoncreds::indy_verifier_verify_proof(verifier_verify_proof_command_handle,
                                                  CString::new(proof_req_json).unwrap().as_ptr(),
                                                  CString::new(proof_json).unwrap().as_ptr(),
                                                  CString::new(schemas_json).unwrap().as_ptr(),
                                                  CString::new(credential_defs_json).unwrap().as_ptr(),
                                                  CString::new(rev_reg_defs_json).unwrap().as_ptr(),
                                                  CString::new(rev_regs_json).unwrap().as_ptr(),
                                                  verifier_verify_proof_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, valid) = verifier_verify_proof_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    assert!(valid);

    // Issuer Closes Wallet
    let res = unsafe {
        wallet::indy_close_wallet(issuer_close_wallet_command_handle, issuer_wallet_handle, issuer_close_wallet_callback)
    };
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);
    let res = issuer_close_wallet_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);

    // Prover Closes Wallet
    let res = unsafe {
        wallet::indy_close_wallet(prover_close_wallet_command_handle, prover_wallet_handle, prover_close_wallet_callback)
    };
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);
    let res = prover_close_wallet_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);

    utils::test::cleanup_storage("issuer_wallet");
    utils::test::cleanup_storage("prover_wallet");
}

#[test]
#[cfg(feature = "local_nodes_pool")]
fn ledger_demo_works() {
    let setup = Setup::empty();
    let my_wallet_config = json!({"id": "my_wallet"}).to_string();
    let their_wallet_config = json!({"id": "their_wallet"}).to_string();

    let c_pool_name = CString::new(setup.name.clone()).unwrap();

    let (set_protocol_version_receiver, set_protocol_version_command_handle, set_protocol_version_callback) = callback::_closure_to_cb_ec();
    let (open_receiver, open_command_handle, open_callback) = callback::_closure_to_cb_ec_i32();
    let (create_receiver, create_command_handle, create_callback) = callback::_closure_to_cb_ec();
    let (send_receiver, send_command_handle, send_callback) = callback::_closure_to_cb_ec_string();
    let (get_nym_receiver, get_nym_command_handle, get_nym_callback) = callback::_closure_to_cb_ec_string();
    let (create_my_wallet_receiver, create_my_wallet_command_handle, create_my_wallet_callback) = callback::_closure_to_cb_ec();
    let (create_their_wallet_receiver, create_their_wallet_command_handle, create_their_wallet_callback) = callback::_closure_to_cb_ec();
    let (open_my_wallet_receiver, open_my_wallet_command_handle, open_my_wallet_callback) = callback::_closure_to_cb_ec_wallethandle();
    let (open_their_wallet_receiver, open_their_wallet_command_handle, open_their_wallet_callback) = callback::_closure_to_cb_ec_wallethandle();
    let (create_and_store_my_did_receiver, create_and_store_my_did_command_handle, create_and_store_my_did_callback) = callback::_closure_to_cb_ec_string_string();
    let (create_and_store_their_did_receiver, create_and_store_their_did_command_handle, create_and_store_their_did_callback) = callback::_closure_to_cb_ec_string_string();
    let (store_their_did_receiver, store_their_did_command_handle, store_their_did_callback) = callback::_closure_to_cb_ec();
    let (close_pool_receiver, close_pool_command_handle, close_pool_callback) = callback::_closure_to_cb_ec();
    let (close_my_wallet_receiver, close_my_wallet_command_handle, close_my_wallet_callback) = callback::_closure_to_cb_ec();
    let (close_their_wallet_receiver, close_their_wallet_command_handle, close_their_wallet_callback) = callback::_closure_to_cb_ec();
    let (build_nym_request_receiver, build_nym_request_command_handle, build_nym_request_callback) = callback::_closure_to_cb_ec_string();
    let (build_get_nym_request_receiver, build_get_nym_request_command_handle, build_get_nym_request_callback) = callback::_closure_to_cb_ec_string();

    // Set protocol version
    let err =
        unsafe {
            pool::indy_set_protocol_version(set_protocol_version_command_handle,
                                            PROTOCOL_VERSION,
                                            set_protocol_version_callback)
        };
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    let err = set_protocol_version_receiver.recv_timeout(timeout::short_timeout()).unwrap();
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);

    // 1. Create ledger config from genesis txn file
    let txn_file_path = pool_utils::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
    let pool_config = pool_utils::pool_config_json(txn_file_path.as_path());
    let c_pool_config = CString::new(pool_config).unwrap();

    let err = unsafe {
        pool::indy_create_pool_ledger_config(create_command_handle,
                                             c_pool_name.as_ptr(),
                                             c_pool_config.as_ptr(),
                                             create_callback)
    };
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    let err = create_receiver.recv_timeout(timeout::short_timeout()).unwrap();
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);

    // 2. Open pool ledger
    let err = unsafe {
        pool::indy_open_pool_ledger(open_command_handle,
                                    c_pool_name.as_ptr(),
                                    null(),
                                    open_callback)
    };
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    let (err, pool_handle) = open_receiver.recv_timeout(timeout::short_timeout()).unwrap();
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    thread::sleep(timeout::short_timeout());

    // 3. Create My Wallet
    let err =
        unsafe {
            wallet::indy_create_wallet(create_my_wallet_command_handle,
                                       CString::new(my_wallet_config.as_str()).unwrap().as_ptr(),
                                       CString::new(WALLET_CREDENTIALS).unwrap().as_ptr(),
                                       create_my_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let err = create_my_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 4. Open My Wallet. Gets My wallet handle
    let err =
        unsafe {
            wallet::indy_open_wallet(open_my_wallet_command_handle,
                                     CString::new(my_wallet_config.as_str()).unwrap().as_ptr(),
                                     CString::new(WALLET_CREDENTIALS).unwrap().as_ptr(),
                                     open_my_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, my_wallet_handle) = open_my_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 5. Create Their Wallet
    let err =
        unsafe {
            wallet::indy_create_wallet(create_their_wallet_command_handle,
                                       CString::new(their_wallet_config.as_str()).unwrap().as_ptr(),
                                       CString::new(WALLET_CREDENTIALS).unwrap().as_ptr(),
                                       create_their_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let err = create_their_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 6. Open Their Wallet. Gets Their wallet handle
    let err =
        unsafe {
            wallet::indy_open_wallet(open_their_wallet_command_handle,
                                     CString::new(their_wallet_config.as_str()).unwrap().as_ptr(),
                                     CString::new(WALLET_CREDENTIALS).unwrap().as_ptr(),
                                     open_their_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, their_wallet_handle) = open_their_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 7. Create My DID
    let my_did_json = "{}";
    let err =
        unsafe {
            did::indy_create_and_store_my_did(create_and_store_my_did_command_handle,
                                              my_wallet_handle,
                                              CString::new(my_did_json).unwrap().as_ptr(),
                                              create_and_store_my_did_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, my_did, my_verkey) = create_and_store_my_did_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 8. Create Their DID from Trustee1 seed
    let their_did_json = r#"{"seed":"000000000000000000000000Trustee1"}"#;
    let err =
        unsafe {
            did::indy_create_and_store_my_did(create_and_store_their_did_command_handle,
                                              their_wallet_handle,
                                              CString::new(their_did_json).unwrap().as_ptr(),
                                              create_and_store_their_did_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, their_did, their_verkey) = create_and_store_their_did_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 9. Store Their DID
    let their_identity_json = json!({"did": their_did, "verkey": their_verkey}).to_string();
    let err =
        unsafe {
            did::indy_store_their_did(store_their_did_command_handle,
                                      my_wallet_handle,
                                      CString::new(their_identity_json).unwrap().as_ptr(),
                                      store_their_did_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let err = store_their_did_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 10. Prepare NYM transaction
    let err =
        unsafe {
            ledger::indy_build_nym_request(build_nym_request_command_handle,
                                           CString::new(their_did.as_str()).unwrap().as_ptr(),
                                           CString::new(my_did.as_str()).unwrap().as_ptr(),
                                           CString::new(my_verkey).unwrap().as_ptr(),
                                           null(),
                                           null(),
                                           build_nym_request_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, request) = build_nym_request_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 11. Send NYM request with signing
    let err = unsafe {
        ledger::indy_sign_and_submit_request(send_command_handle,
                                             pool_handle,
                                             their_wallet_handle,
                                             CString::new(their_did.as_str()).unwrap().as_ptr(),
                                             CString::new(request).unwrap().as_ptr(),
                                             send_callback)
    };
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    let (err, resp) = send_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    serde_json::from_str::<serde_json::Value>(&resp).unwrap();

    // pause for synchronization of all nodes in the ledger
    ::std::thread::sleep(timeout::short_timeout());

    // 12. Prepare and send GET_NYM request
    let err =
        unsafe {
            ledger::indy_build_get_nym_request(build_get_nym_request_command_handle,
                                               CString::new(my_did.as_str()).unwrap().as_ptr(),
                                               CString::new(my_did.as_str()).unwrap().as_ptr(),
                                               build_get_nym_request_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, request) = build_get_nym_request_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    let err =
        unsafe {
            ledger::indy_submit_request(get_nym_command_handle,
                                        pool_handle,
                                        CString::new(request).unwrap().as_ptr(),
                                        get_nym_callback)
        };
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);
    let (err, resp) = get_nym_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(err), ErrorCode::Success);

    let get_nym_resp: Reply = serde_json::from_str(&resp).unwrap();
    let get_nym_resp_data: ReplyResultData = serde_json::from_str(&get_nym_resp.result.data.as_ref().unwrap()).unwrap();

    assert_eq!(get_nym_resp_data.dest, my_did);

    // 13. Close pool
    let res = unsafe {
        pool::indy_close_pool_ledger(close_pool_command_handle, pool_handle, close_pool_callback)
    };
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);
    let res = close_pool_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);

    // 14. Close my wallet
    let res = unsafe { wallet::indy_close_wallet(close_my_wallet_command_handle, my_wallet_handle, close_my_wallet_callback) };
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);
    let res = close_my_wallet_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);

    // 15. Close their wallet
    let res = unsafe { wallet::indy_close_wallet(close_their_wallet_command_handle, their_wallet_handle, close_their_wallet_callback) };
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);
    let res = close_their_wallet_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);

    utils::test::cleanup_storage("my_wallet");
    utils::test::cleanup_storage("their_wallet");

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    struct Reply {
        op: String,
        result: ReplyResult,
    }

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ReplyResult {
        identifier: String,
        req_id: u64,
        data: Option<String>,
    }

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ReplyResultData {
        dest: String,
        identifier: String,
        role: Option<String>,
    }
}

#[test]
fn crypto_demo_works() {
    Setup::empty();

    let (create_wallet_receiver, create_wallet_command_handle, create_wallet_callback) = callback::_closure_to_cb_ec();
    let (open_wallet_receiver, open_wallet_command_handle, open_wallet_callback) = callback::_closure_to_cb_ec_wallethandle();
    let (create_and_store_did_receiver, create_and_store_did_command_handle, create_and_store_did_callback) = callback::_closure_to_cb_ec_string_string();
    let (sign_receiver, sign_command_handle, sign_callback) = callback::_closure_to_cb_ec_vec_u8();
    let (verify_receiver, verify_command_handle, verify_callback) = callback::_closure_to_cb_ec_bool();
    let (close_wallet_receiver, close_wallet_command_handle, close_wallet_callback) = callback::_closure_to_cb_ec();

    let wallet_name = "wallet_crypto_demo_works";
    let wallet_config = json!({"id": wallet_name}).to_string();

    // 1. Create Wallet
    let err =
        unsafe {
            wallet::indy_create_wallet(create_wallet_command_handle,
                                       CString::new(wallet_config.as_str()).unwrap().as_ptr(),
                                       CString::new(WALLET_CREDENTIALS).unwrap().as_ptr(),
                                       create_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let err = create_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 2. Open Wallet. Gets wallet handle
    let err =
        unsafe {
            wallet::indy_open_wallet(open_wallet_command_handle,
                                     CString::new(wallet_config.as_str()).unwrap().as_ptr(),
                                     CString::new(WALLET_CREDENTIALS).unwrap().as_ptr(),
                                     open_wallet_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, wallet_handle) = open_wallet_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 3. Create DID
    let did_json = "{}";
    let err =
        unsafe {
            did::indy_create_and_store_my_did(create_and_store_did_command_handle,
                                              wallet_handle,
                                              CString::new(did_json).unwrap().as_ptr(),
                                              create_and_store_did_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, _, verkey) = create_and_store_did_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 4. Sign message
    let message = r#"{
        "reqId":1495034346617224651,
        "identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
        "operation":{
            "type":"1",
            "dest":"4efZu2SXufS556yss7W5k6Po37jt4371RM4whbPKBKdB"
        }
    }"#;

    let message_ptr = message.as_ptr() as *const u8;
    let message_len = message.len() as u32;

    let err =
        unsafe {
            crypto::indy_crypto_sign(sign_command_handle,
                                     wallet_handle,
                                     CString::new(verkey.as_str()).unwrap().as_ptr(),
                                     message_ptr,
                                     message_len,
                                     sign_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, signature) = sign_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 5. Verify message
    let err =
        unsafe {
            crypto::indy_crypto_verify(verify_command_handle,
                                       CString::new(verkey).unwrap().as_ptr(),
                                       message_ptr,
                                       message_len,
                                       signature.as_ptr() as *const u8,
                                       signature.len() as u32,
                                       verify_callback)
        };

    assert_eq!(ErrorCode::Success, ErrorCode::from(err));
    let (err, valid) = verify_receiver.recv_timeout(timeout::long_timeout()).unwrap();
    assert!(valid);
    assert_eq!(ErrorCode::Success, ErrorCode::from(err));

    // 6. Close Wallet
    let res = unsafe { wallet::indy_close_wallet(close_wallet_command_handle, wallet_handle, close_wallet_callback) };
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);
    let res = close_wallet_receiver.recv_timeout(timeout::medium_timeout()).unwrap();
    assert_eq!(ErrorCode::from(res), ErrorCode::Success);

    utils::test::cleanup_storage(wallet_name);
}
