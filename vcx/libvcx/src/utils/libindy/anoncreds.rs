use futures::Future;
use serde_json;
use serde_json::{map::Map, Value};
use indy::{anoncreds, blob_storage, ledger};
use time;

use settings;
use utils::constants::{LIBINDY_CRED_OFFER, REQUESTED_ATTRIBUTES, PROOF_REQUESTED_PREDICATES, ATTRS, REV_STATE_JSON};
use utils::libindy::{wallet::get_wallet_handle, LibindyMock};
use utils::libindy::payments::{send_transaction, PaymentTxn};
use utils::libindy::ledger::*;
use utils::constants::{SCHEMA_ID, SCHEMA_JSON, SCHEMA_TXN, CREATE_SCHEMA_ACTION, CRED_DEF_ID, CRED_DEF_JSON, CRED_DEF_REQ, CREATE_CRED_DEF_ACTION, CREATE_REV_REG_DEF_ACTION, CREATE_REV_REG_DELTA_ACTION, REVOC_REG_TYPE, rev_def_json, REV_REG_ID, REV_REG_DELTA_JSON, REV_REG_JSON};
use error::prelude::*;

const BLOB_STORAGE_TYPE: &str = "default";
const REVOCATION_REGISTRY_TYPE: &str = "ISSUANCE_BY_DEFAULT";

pub fn libindy_verifier_verify_proof(proof_req_json: &str,
                                     proof_json: &str,
                                     schemas_json: &str,
                                     credential_defs_json: &str,
                                     rev_reg_defs_json: &str,
                                     rev_regs_json: &str) -> VcxResult<bool> {
    anoncreds::verifier_verify_proof(proof_req_json,
                                     proof_json,
                                     schemas_json,
                                     credential_defs_json,
                                     rev_reg_defs_json,
                                     rev_regs_json)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_create_and_store_revoc_reg(issuer_did: &str, cred_def_id: &str, tails_path: &str, max_creds: u32, tag: &str) -> VcxResult<(String, String, String)> {
    trace!("creating revocation: {}, {}, {}", cred_def_id, tails_path, max_creds);

    let tails_config = json!({"base_dir": tails_path,"uri_pattern": ""}).to_string();

    let writer = blob_storage::open_writer(BLOB_STORAGE_TYPE, &tails_config)
        .wait()?;

    let revoc_config = json!({"max_cred_num": max_creds, "issuance_type": REVOCATION_REGISTRY_TYPE}).to_string();

    anoncreds::issuer_create_and_store_revoc_reg(get_wallet_handle(), issuer_did, None, tag, cred_def_id, &revoc_config, writer)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_create_and_store_credential_def(issuer_did: &str,
                                               schema_json: &str,
                                               tag: &str,
                                               sig_type: Option<&str>,
                                               config_json: &str) -> VcxResult<(String, String)> {
    anoncreds::issuer_create_and_store_credential_def(get_wallet_handle(),
                                                      issuer_did,
                                                      schema_json,
                                                      tag,
                                                      sig_type,
                                                      config_json)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_issuer_create_credential_offer(cred_def_id: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        let rc = LibindyMock::get_result();
        if rc != 0 { return Err(VcxError::from(VcxErrorKind::InvalidState)); };
        return Ok(LIBINDY_CRED_OFFER.to_string());
    }
    anoncreds::issuer_create_credential_offer(get_wallet_handle(),
                                              cred_def_id)
        .wait()
        .map_err(VcxError::from)
}

fn blob_storage_open_reader(base_dir: &str) -> VcxResult<i32> {
    let tails_config = json!({"base_dir": base_dir,"uri_pattern": ""}).to_string();
    blob_storage::open_reader("default", &tails_config)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_issuer_create_credential(cred_offer_json: &str,
                                        cred_req_json: &str,
                                        cred_values_json: &str,
                                        rev_reg_id: Option<String>,
                                        tails_file: Option<String>) -> VcxResult<(String, Option<String>, Option<String>)> {
    if settings::indy_mocks_enabled() { return Ok((::utils::constants::CREDENTIAL_JSON.to_owned(), None, None)); }

    let revocation = rev_reg_id.as_ref().map(String::as_str);

    let blob_handle = match tails_file {
        Some(x) => blob_storage_open_reader(&x)?,
        None => -1,
    };
    anoncreds::issuer_create_credential(get_wallet_handle(),
                                        cred_offer_json,
                                        cred_req_json,
                                        cred_values_json,
                                        revocation,
                                        blob_handle)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_create_proof(proof_req_json: &str,
                                   requested_credentials_json: &str,
                                   master_secret_id: &str,
                                   schemas_json: &str,
                                   credential_defs_json: &str,
                                   revoc_states_json: Option<&str>) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(::utils::constants::PROOF_JSON.to_owned()); }

    let revoc_states_json = revoc_states_json.unwrap_or("{}");
    anoncreds::prover_create_proof(get_wallet_handle(),
                                   proof_req_json,
                                   requested_credentials_json,
                                   master_secret_id,
                                   schemas_json,
                                   credential_defs_json,
                                   revoc_states_json)
        .wait()
        .map_err(VcxError::from)
}

fn fetch_credentials(search_handle: i32, requested_attributes: Map<String, Value>) -> VcxResult<String> {
    let mut v: Value = json!({});
    for item_referent in requested_attributes.keys().into_iter() {
        v[ATTRS][item_referent] =
            serde_json::from_str(&anoncreds::prover_fetch_credentials_for_proof_req(search_handle, item_referent, 100).wait()?)
                .map_err(|_| {
                    error!("Invalid Json Parsing of Object Returned from Libindy. Did Libindy change its structure?");
                    VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Invalid Json Parsing of Object Returned from Libindy. Did Libindy change its structure?")
                })?
    }

    Ok(v.to_string())
}

fn close_search_handle(search_handle: i32) -> VcxResult<()> {
    anoncreds::prover_close_credentials_search_for_proof_req(search_handle)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_get_credentials_for_proof_req(proof_req: &str) -> VcxResult<String> {
    let wallet_handle = get_wallet_handle();

    // this may be too redundant since Prover::search_credentials will validate the proof reqeuest already.
    let proof_request_json: Map<String, Value> = serde_json::from_str(proof_req)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest, format!("Cannot deserialize ProofRequest: {:?}", err)))?;

    // since the search_credentials_for_proof request validates that the proof_req is properly structured, this get()
    // fn should never fail, unless libindy changes their formats.
    let requested_attributes: Option<Map<String, Value>> = proof_request_json.get(REQUESTED_ATTRIBUTES)
        .and_then(|v| {
            serde_json::from_value(v.clone()).map_err(|_| {
                error!("Invalid Json Parsing of Requested Attributes Retrieved From Libindy. Did Libindy change its structure?");
            }).ok()
        });

    let requested_predicates: Option<Map<String, Value>> = proof_request_json.get(PROOF_REQUESTED_PREDICATES).and_then(|v| {
        serde_json::from_value(v.clone()).map_err(|_| {
            error!("Invalid Json Parsing of Requested Predicates Retrieved From Libindy. Did Libindy change its structure?");
        }).ok()
    });

    // handle special case of "empty because json is bad" vs "empty because no attributes sepected"
    if requested_attributes == None && requested_predicates == None {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, "Invalid Json Parsing of Requested Attributes Retrieved From Libindy"));
    }

    let mut fetch_attrs: Map<String, Value> = match requested_attributes {
        Some(attrs) => attrs.clone(),
        None => Map::new()
    };
    match requested_predicates {
        Some(attrs) => fetch_attrs.extend(attrs),
        None => ()
    }
    if 0 < fetch_attrs.len() {
        let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, proof_req, None)
            .wait()
            .map_err(|ec| {
                error!("Opening Indy Search for Credentials Failed");
                ec
            })?;
        let creds: String = fetch_credentials(search_handle, fetch_attrs)?;

        // should an error on closing a search handle throw an error, or just a warning?
        // for now we're are just outputting to the user that there is an issue, and continuing on.
        let _ = close_search_handle(search_handle);
        Ok(creds)
    } else {
        Ok("{}".to_string())
    }
}

pub fn libindy_prover_create_credential_req(prover_did: &str,
                                            credential_offer_json: &str,
                                            credential_def_json: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() { return Ok((::utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new())); }

    let master_secret_name = settings::DEFAULT_LINK_SECRET_ALIAS;
    anoncreds::prover_create_credential_req(get_wallet_handle(),
                                            prover_did,
                                            credential_offer_json,
                                            credential_def_json,
                                            master_secret_name)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_create_revocation_state(rev_reg_def_json: &str, rev_reg_delta_json: &str, cred_rev_id: &str, tails_file: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(REV_STATE_JSON.to_string()); }

    let blob_handle = blob_storage_open_reader(tails_file)?;

    anoncreds::create_revocation_state(blob_handle, rev_reg_def_json, rev_reg_delta_json, 100, cred_rev_id)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_update_revocation_state(rev_reg_def_json: &str, rev_state_json: &str, rev_reg_delta_json: &str, cred_rev_id: &str, tails_file: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(REV_STATE_JSON.to_string()); }

    let blob_handle = blob_storage_open_reader(tails_file)?;

    anoncreds::update_revocation_state(blob_handle, rev_state_json, rev_reg_def_json, rev_reg_delta_json, 100, cred_rev_id)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_store_credential(cred_id: Option<&str>,
                                       cred_req_meta: &str,
                                       cred_json: &str,
                                       cred_def_json: &str,
                                       rev_reg_def_json: Option<&str>) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok("cred_id".to_string()); }

    anoncreds::prover_store_credential(get_wallet_handle(),
                                       cred_id,
                                       cred_req_meta,
                                       cred_json,
                                       cred_def_json,
                                       rev_reg_def_json)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_delete_credential(cred_id: &str) -> VcxResult<()>{

    anoncreds::prover_delete_credential(get_wallet_handle(),
                                        cred_id)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_prover_create_master_secret(master_secret_id: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string()); }

    anoncreds::prover_create_master_secret(get_wallet_handle(),
                                           Some(master_secret_id))
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_issuer_create_schema(issuer_did: &str,
                                    name: &str,
                                    version: &str,
                                    attrs: &str) -> VcxResult<(String, String)> {
    anoncreds::issuer_create_schema(issuer_did,
                                    name,
                                    version,
                                    attrs)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_issuer_revoke_credential(tails_file: &str, rev_reg_id: &str, cred_rev_id: &str) -> VcxResult<String> {
    let blob_handle = blob_storage_open_reader(tails_file)?;

    anoncreds::issuer_revoke_credential(get_wallet_handle(), blob_handle, rev_reg_id, cred_rev_id)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_build_revoc_reg_def_request(submitter_did: &str,
                                           rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok("".to_string()); }

    ledger::build_revoc_reg_def_request(submitter_did, rev_reg_def_json)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_build_revoc_reg_entry_request(submitter_did: &str,
                                             rev_reg_id: &str,
                                             rev_def_type: &str,
                                             value: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok("".to_string()); }

    ledger::build_revoc_reg_entry_request(submitter_did, rev_reg_id, rev_def_type, value)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_build_get_revoc_reg_def_request(submitter_did: &str, rev_reg_id: &str) -> VcxResult<String> {
    ledger::build_get_revoc_reg_def_request(Some(submitter_did), rev_reg_id)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_parse_get_revoc_reg_def_response(rev_reg_def_json: &str) -> VcxResult<(String, String)> {
    ledger::parse_get_revoc_reg_def_response(rev_reg_def_json)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_build_get_revoc_reg_delta_request(submitter_did: &str,
                                                 rev_reg_id: &str,
                                                 from: i64,
                                                 to: i64) -> VcxResult<String> {
    ledger::build_get_revoc_reg_delta_request(Some(submitter_did),
                                              rev_reg_id,
                                              from,
                                              to)
        .wait()
        .map_err(VcxError::from)
}

fn libindy_build_get_revoc_reg_request(submitter_did: &str, rev_reg_id: &str, timestamp: u64) -> VcxResult<String> {
    ledger::build_get_revoc_reg_request(Some(submitter_did),
                                        rev_reg_id,
                                        timestamp as i64)
        .wait()
        .map_err(VcxError::from)
}

fn libindy_parse_get_revoc_reg_response(get_rev_reg_resp: &str) -> VcxResult<(String, String, u64)> {
    ledger::parse_get_revoc_reg_response(get_rev_reg_resp)
        .wait()
        .map_err(VcxError::from)
}

pub fn libindy_parse_get_revoc_reg_delta_response(get_rev_reg_delta_response: &str)
                                                  -> VcxResult<(String, String, u64)> {
    ledger::parse_get_revoc_reg_delta_response(get_rev_reg_delta_response)
        .wait()
        .map_err(VcxError::from)
}

pub fn create_schema(name: &str, version: &str, data: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
    }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let (id, create_schema) = libindy_issuer_create_schema(&submitter_did, name, version, data)?;

    Ok((id, create_schema))
}

pub fn build_schema_request(schema: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(SCHEMA_TXN.to_string());
    }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let request = libindy_build_schema_request(&submitter_did, schema)?;

    let request = append_txn_author_agreement_to_request(&request)?;

    Ok(request)
}

pub fn publish_schema(schema: &str) -> VcxResult<Option<PaymentTxn>> {
    if settings::indy_mocks_enabled() {
        let inputs = vec!["pay:null:9UFgyjuJxi1i1HD".to_string()];
        let outputs = serde_json::from_str::<Vec<::utils::libindy::payments::Output>>(r#"[{"amount":4,"extra":null,"recipient":"pay:null:xkIsxem0YNtHrRO"}]"#).unwrap();
        return Ok(Some(PaymentTxn::from_parts(inputs, outputs, 1, false)));
    }

    let request = build_schema_request(schema)?;

    let (payment, response) = send_transaction(&request, CREATE_SCHEMA_ACTION)?;

    _check_schema_response(&response)?;

    Ok(payment)
}

pub fn get_schema_json(schema_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() { return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string())); }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let schema_json = libindy_get_schema(&submitter_did, schema_id)?;

    Ok((schema_id.to_string(), schema_json))
}

pub fn generate_cred_def(issuer_did: &str,
                         schema_json: &str,
                         tag: &str,
                         sig_type: Option<&str>,
                         support_revocation: Option<bool>) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let config_json = json!({"support_revocation": support_revocation.unwrap_or(false)}).to_string();

    libindy_create_and_store_credential_def(issuer_did,
                                            schema_json,
                                            tag,
                                            sig_type,
                                            &config_json)
}

pub fn build_cred_def_request(issuer_did: &str, cred_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(CRED_DEF_REQ.to_string());
    }

    let cred_def_req = libindy_build_create_credential_def_txn(issuer_did, &cred_def_json)?;

    let cred_def_req = append_txn_author_agreement_to_request(&cred_def_req)?;

    Ok(cred_def_req)
}

pub fn publish_cred_def(issuer_did: &str, cred_def_json: &str) -> VcxResult<Option<PaymentTxn>> {
    if settings::indy_mocks_enabled() {
        let inputs = vec!["pay:null:9UFgyjuJxi1i1HD".to_string()];
        let outputs = serde_json::from_str::<Vec<::utils::libindy::payments::Output>>(r#"[{"amount":4,"extra":null,"recipient":"pay:null:xkIsxem0YNtHrRO"}]"#).unwrap();
        return Ok(Some(PaymentTxn::from_parts(inputs, outputs, 1, false)));
    }

    let cred_def_req = build_cred_def_request(issuer_did, &cred_def_json)?;

    let (payment, _) = send_transaction(&cred_def_req, CREATE_CRED_DEF_ACTION)?;

    Ok(payment)
}

pub fn get_cred_def_json(cred_def_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() { return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string())); }

    let cred_def_json = libindy_get_cred_def(cred_def_id)?;

    Ok((cred_def_id.to_string(), cred_def_json))
}

pub fn generate_rev_reg(issuer_did: &str, cred_def_id: &str, tails_file: &str, max_creds: u32, tag: &str)
                        -> VcxResult<(String, String, String)> {
    if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), rev_def_json(), "".to_string())); }

    let (rev_reg_id, rev_reg_def_json, rev_reg_entry_json) =
        libindy_create_and_store_revoc_reg(issuer_did,
                                           cred_def_id,
                                           tails_file,
                                           max_creds,
                                           tag)?;

    Ok((rev_reg_id, rev_reg_def_json, rev_reg_entry_json))
}

pub fn build_rev_reg_request(issuer_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok("".to_string()); }

    let rev_reg_def_req = libindy_build_revoc_reg_def_request(issuer_did, &rev_reg_def_json)?;
    let rev_reg_def_req = append_txn_author_agreement_to_request(&rev_reg_def_req)?;
    Ok(rev_reg_def_req)
}

pub fn publish_rev_reg_def(issuer_did: &str, rev_reg_def_json: &str) -> VcxResult<Option<PaymentTxn>> {
    if settings::indy_mocks_enabled() { return Ok(None); }

    let rev_reg_def_req = build_rev_reg_request(issuer_did, &rev_reg_def_json)?;
    let (payment, _) = send_transaction(&rev_reg_def_req, CREATE_REV_REG_DEF_ACTION)?;
    Ok(payment)
}

pub fn get_rev_reg_def_json(rev_reg_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), rev_def_json())); }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    libindy_build_get_revoc_reg_def_request(&submitter_did, rev_reg_id)
        .and_then(|req| libindy_submit_request(&req))
        .and_then(|response| libindy_parse_get_revoc_reg_def_response(&response))
}

pub fn build_rev_reg_delta_request(issuer_did: &str, rev_reg_id: &str, rev_reg_entry_json: &str)
                                   -> VcxResult<String> {
    let request = libindy_build_revoc_reg_entry_request(issuer_did, rev_reg_id, REVOC_REG_TYPE, rev_reg_entry_json)?;
    let request = append_txn_author_agreement_to_request(&request)?;
    Ok(request)
}

pub fn publish_rev_reg_delta(issuer_did: &str, rev_reg_id: &str, rev_reg_entry_json: &str)
                             -> VcxResult<(Option<PaymentTxn>, String)> {
    let request = build_rev_reg_delta_request(issuer_did, rev_reg_id, rev_reg_entry_json)?;
    send_transaction(&request, CREATE_REV_REG_DELTA_ACTION)
}

pub fn get_rev_reg_delta_json(rev_reg_id: &str, from: Option<u64>, to: Option<u64>)
                              -> VcxResult<(String, String, u64)> {
    if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), REV_REG_DELTA_JSON.to_string(), 1)); }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
    let from: i64 = if let Some(_from) = from { _from as i64 } else { -1 };
    let to = if let Some(_to) = to { _to as i64 } else { time::get_time().sec };

    libindy_build_get_revoc_reg_delta_request(&submitter_did, rev_reg_id, from, to)
        .and_then(|req| libindy_submit_request(&req))
        .and_then(|response| libindy_parse_get_revoc_reg_delta_response(&response))
}

pub fn get_rev_reg(rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
    if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1)); }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    libindy_build_get_revoc_reg_request(&submitter_did, rev_reg_id, timestamp)
        .and_then(|req| libindy_submit_request(&req))
        .and_then(|response| libindy_parse_get_revoc_reg_response(&response))
}

pub fn revoke_credential(tails_file: &str, rev_reg_id: &str, cred_rev_id: &str) -> VcxResult<(Option<PaymentTxn>, String)> {
    if settings::indy_mocks_enabled() {
        let inputs = vec!["pay:null:9UFgyjuJxi1i1HD".to_string()];
        let outputs = serde_json::from_str::<Vec<::utils::libindy::payments::Output>>(r#"[{"amount":4,"extra":null,"recipient":"pay:null:xkIsxem0YNtHrRO"}]"#).unwrap();
        return Ok((Some(PaymentTxn::from_parts(inputs, outputs, 1, false)), REV_REG_DELTA_JSON.to_string()));
    }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let delta = libindy_issuer_revoke_credential(tails_file, rev_reg_id, cred_rev_id)?;
    let (payment, _) = publish_rev_reg_delta(&submitter_did, rev_reg_id, &delta)?;

    Ok((payment, delta))
}

pub fn libindy_to_unqualified(entity: &str) -> VcxResult<String> {
    anoncreds::to_unqualified(entity)
        .wait()
        .map_err(VcxError::from)
}

fn _check_schema_response(response: &str) -> VcxResult<()> {
    // TODO: saved backwardcampatibilyty but actually we can better handle response
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(reject) => Err(VcxError::from_msg(VcxErrorKind::DuplicationSchema, format!("{:?}", reject))),
        Response::ReqNACK(reqnack) => Err(VcxError::from_msg(VcxErrorKind::UnknownSchemaRejection, format!("{:?}", reqnack)))
    }
}

pub fn generate_nonce() -> VcxResult<String> {
    anoncreds::generate_nonce()
        .wait()
        .map_err(VcxError::from)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::get_temp_dir_path;

    extern crate serde_json;
    extern crate rand;

    use rand::Rng;
    use settings;
    use utils::constants::*;
    use std::thread;
    use std::time::Duration;
    #[cfg(feature = "pool_tests")]
    use utils::constants::TEST_TAILS_FILE;
    use utils::devsetup::*;


    pub fn create_schema(attr_list: &str) -> (String, String) {
        let data = attr_list.to_string();
        let schema_name: String = rand::thread_rng().gen_ascii_chars().take(25).collect::<String>();
        let schema_version: String = format!("{}.{}", rand::thread_rng().gen::<u32>().to_string(),
                                             rand::thread_rng().gen::<u32>().to_string());
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        libindy_issuer_create_schema(&institution_did, &schema_name, &schema_version, &data).unwrap()
    }

    pub fn create_schema_req(schema_json: &str) -> String {
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let request = ::utils::libindy::ledger::libindy_build_schema_request(&institution_did, schema_json).unwrap();
        append_txn_author_agreement_to_request(&request).unwrap()
    }

    pub fn create_and_write_test_schema(attr_list: &str) -> (String, String) {
        let (schema_id, schema_json) = create_schema(attr_list);
        let req = create_schema_req(&schema_json);
        ::utils::libindy::payments::send_transaction(&req, CREATE_SCHEMA_ACTION).unwrap();
        thread::sleep(Duration::from_millis(1000));
        (schema_id, schema_json)
    }

    pub fn create_and_store_credential_def(attr_list: &str, support_rev: bool) -> (String, String, String, String, u32, Option<String>) {
        /* create schema */
        let (schema_id, schema_json) = create_and_write_test_schema(attr_list);

        let name: String = rand::thread_rng().gen_ascii_chars().take(25).collect::<String>();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        /* create cred-def */
        let mut revocation_details = json!({"support_revocation":support_rev});
        if support_rev {
            revocation_details["tails_file"] = json!(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string());
            revocation_details["max_creds"] = json!(10);
        }
        let handle = ::credential_def::create_and_publish_credentialdef("1".to_string(),
                                                                        name,
                                                                        institution_did.clone(),
                                                                        schema_id.clone(),
                                                                        "tag1".to_string(),
                                                                        revocation_details.to_string()).unwrap();

        thread::sleep(Duration::from_millis(1000));
        let cred_def_id = ::credential_def::get_cred_def_id(handle).unwrap();
        thread::sleep(Duration::from_millis(1000));
        let (_, cred_def_json) = get_cred_def_json(&cred_def_id).unwrap();
        let rev_reg_id = ::credential_def::get_rev_reg_id(handle).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, handle, rev_reg_id)
    }

    pub fn create_credential_offer(attr_list: &str, revocation: bool) -> (String, String, String, String, String, Option<String>) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, _, rev_reg_id) = create_and_store_credential_def(attr_list, revocation);

        let offer = ::utils::libindy::anoncreds::libindy_issuer_create_credential_offer(&cred_def_id).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, offer, rev_reg_id)
    }

    pub fn create_credential_req(attr_list: &str, revocation: bool) -> (String, String, String, String, String, String, String, Option<String>) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, offer, rev_reg_id) = create_credential_offer(attr_list, revocation);
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (req, req_meta) = ::utils::libindy::anoncreds::libindy_prover_create_credential_req(&institution_did, &offer, &cred_def_json).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, offer, req, req_meta, rev_reg_id)
    }

    pub fn create_and_store_credential(attr_list: &str, revocation: bool) -> (String, String, String, String, String, String, String, String, Option<String>, Option<String>) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, offer, req, req_meta, rev_reg_id) = create_credential_req(attr_list, revocation);

        /* create cred */
        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let encoded_attributes = ::issuer_credential::encode_attributes(&credential_data).unwrap();
        let (rev_def_json, tails_file) = if revocation {
            let (_id, json) = get_rev_reg_def_json(&rev_reg_id.clone().unwrap()).unwrap();
            (Some(json), Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string().to_string()))
        } else { (None, None) };

        let (cred, cred_rev_id, _) = ::utils::libindy::anoncreds::libindy_issuer_create_credential(&offer, &req, &encoded_attributes, rev_reg_id.clone(), tails_file).unwrap();
        /* store cred */
        let cred_id = ::utils::libindy::anoncreds::libindy_prover_store_credential(None, &req_meta, &cred, &cred_def_json, rev_def_json.as_ref().map(String::as_str)).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, offer, req, req_meta, cred_id, rev_reg_id, cred_rev_id)
    }

    pub fn create_proof() -> (String, String, String, String) {
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "zip_2": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();

        let requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true},
                 "zip_2": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{}
        }).to_string();

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        }).to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        }).to_string();

        libindy_prover_get_credentials_for_proof_req(&proof_req).unwrap();

        let proof = libindy_prover_create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None).unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    pub fn create_self_attested_proof() -> (String, String) {
        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();

        let requested_credentials_json = json!({
              "self_attested_attributes":{
                 "address1_1": "my_self_attested_address",
                 "zip_2": "my_self_attested_zip"
              },
              "requested_attributes":{},
              "requested_predicates":{}
        }).to_string();

        let schemas = json!({}).to_string();
        let cred_defs = json!({}).to_string();

        libindy_prover_get_credentials_for_proof_req(&proof_req).unwrap();

        let proof = libindy_prover_create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None).unwrap();
        (proof_req, proof)
    }

    pub fn create_proof_with_predicate(include_predicate_cred: bool) -> (String, String, String, String) {
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({
               "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
           }),
        }).to_string();

        let requested_credentials_json;
        if include_predicate_cred {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
                  "zip_3": {"cred_id": cred_id}
              }
            }).to_string();
        } else {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
              }
            }).to_string();
        }

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        }).to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        }).to_string();

        libindy_prover_get_credentials_for_proof_req(&proof_req).unwrap();

        let proof = libindy_prover_create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None).unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_prover_verify_proof() {
        let _setup = SetupLibraryWalletPool::init();

        let (schemas, cred_defs, proof_req, proof) = create_proof();

        let proof_validation = libindy_verifier_verify_proof(
            &proof_req,
            &proof,
            &schemas,
            &cred_defs,
            "{}",
            "{}",
        ).unwrap();

        assert!(proof_validation, true);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_prover_verify_proof_with_predicate_success_case() {
        let _setup = SetupLibraryWalletPool::init();

        let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(true);

        let proof_validation = libindy_verifier_verify_proof(
            &proof_req,
            &proof,
            &schemas,
            &cred_defs,
            "{}",
            "{}",
        ).unwrap();

        assert!(proof_validation, true);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_prover_verify_proof_with_predicate_fail_case() {
        let _setup = SetupLibraryWalletPool::init();

        let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(false);

        libindy_verifier_verify_proof(
            &proof_req,
            &proof,
            &schemas,
            &cred_defs,
            "{}",
            "{}",
        ).unwrap_err();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn tests_libindy_prover_get_credentials() {
        let _setup = SetupLibraryWallet::init();

        let proof_req = "{";
        let result = libindy_prover_get_credentials_for_proof_req(&proof_req);
        assert_eq!(result.unwrap_err().kind(), VcxErrorKind::InvalidProofRequest);

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();
        let _result = libindy_prover_get_credentials_for_proof_req(&proof_req).unwrap();

        let result_malformed_json = libindy_prover_get_credentials_for_proof_req("{}").unwrap_err();
        assert_eq!(result_malformed_json.kind(), VcxErrorKind::InvalidAttributesStructure);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_issuer_revoke_credential() {
        let _setup = SetupLibraryWalletPool::init();

        let rc = libindy_issuer_revoke_credential(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap(), "", "");
        assert!(rc.is_err());

        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id)
            = create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let rc = ::utils::libindy::anoncreds::libindy_issuer_revoke_credential(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap(), &rev_reg_id.unwrap(), &cred_rev_id.unwrap());

        assert!(rc.is_ok());
    }

    #[test]
    fn test_create_cred_def() {
        let _setup = SetupMocks::init();

        let (id, _) = generate_cred_def("did", SCHEMAS_JSON, "tag_1", None, Some(false)).unwrap();
        assert_eq!(id, CRED_DEF_ID);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_cred_def_real() {
        let _setup = SetupLibraryWalletPool::init();

        let (schema_id, _) = ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let (_, schema_json) = get_schema_json(&schema_id).unwrap();
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let (_, cred_def_json) = generate_cred_def(&did, &schema_json, "tag_1", None, Some(true)).unwrap();
        publish_cred_def(&did, &cred_def_json).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_rev_reg_def_fails_for_cred_def_created_without_revocation() {
        let _setup = SetupLibraryWalletPool::init();

        // Cred def is created with support_revocation=false,
        // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
        let (_, _, cred_def_id, _, _, _) = ::utils::libindy::anoncreds::tests::create_and_store_credential_def(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let rc = generate_rev_reg(&did, &cred_def_id, get_temp_dir_path("path.txt").to_str().unwrap(), 2, "tag1");

        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::LibindyInvalidStructure);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_rev_reg_def() {
        let _setup = SetupLibraryWalletPool::init();

        let (schema_id, _) = ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let (_, schema_json) = get_schema_json(&schema_id).unwrap();
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let (cred_def_id, cred_def_json) = generate_cred_def(&did, &schema_json, "tag_1", None, Some(true)).unwrap();
        publish_cred_def(&did, &cred_def_json).unwrap();
        let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) = generate_rev_reg(&did, &cred_def_id, "tails.txt", 2, "tag1").unwrap();
        publish_rev_reg_def(&did, &rev_reg_def_json).unwrap();
        publish_rev_reg_delta(&did, &rev_reg_def_id, &rev_reg_entry_json).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_rev_reg_def_json() {
        let _setup = SetupLibraryWalletPool::init();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, _, rev_reg_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential_def(attrs, true);

        let rev_reg_id = rev_reg_id.unwrap();
        let (id, _json) = get_rev_reg_def_json(&rev_reg_id).unwrap();
        assert_eq!(id, rev_reg_id);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_rev_reg_delta_json() {
        let _setup = SetupLibraryWalletPool::init();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, _, rev_reg_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential_def(attrs, true);
        let rev_reg_id = rev_reg_id.unwrap();

        let (id, _delta, _timestamp) = get_rev_reg_delta_json(&rev_reg_id, None, None).unwrap();
        assert_eq!(id, rev_reg_id);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_rev_reg() {
        let _setup = SetupLibraryWalletPool::init();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, _, rev_reg_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential_def(attrs, true);
        let rev_reg_id = rev_reg_id.unwrap();

        let (id, _rev_reg, _timestamp) = get_rev_reg(&rev_reg_id, time::get_time().sec as u64).unwrap();
        assert_eq!(id, rev_reg_id);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn from_pool_ledger_with_id() {
        let _setup = SetupLibraryWalletPool::init();

        let (schema_id, _schema_json) = ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);

        let rc = get_schema_json(&schema_id);

        let (_id, retrieved_schema) = rc.unwrap();
        assert!(retrieved_schema.contains(&schema_id));
    }

    #[test]
    fn from_ledger_schema_id() {
        let _setup = SetupMocks::init();

        let (id, retrieved_schema) = get_schema_json(SCHEMA_ID).unwrap();
        assert_eq!(&retrieved_schema, SCHEMA_JSON);
        assert_eq!(&id, SCHEMA_ID);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_revoke_credential() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id)
            = ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, true);

        let rev_reg_id = rev_reg_id.unwrap();
        let (_, first_rev_reg_delta, first_timestamp) = get_rev_reg_delta_json(&rev_reg_id, None, None).unwrap();
        let (_, test_same_delta, test_same_timestamp) = get_rev_reg_delta_json(&rev_reg_id, None, None).unwrap();

        assert_eq!(first_rev_reg_delta, test_same_delta);
        assert_eq!(first_timestamp, test_same_timestamp);

        let (payment, _revoked_rev_reg_delta) = revoke_credential(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap(), &rev_reg_id, cred_rev_id.unwrap().as_str()).unwrap();

        // Delta should change after revocation
        let (_, second_rev_reg_delta, _) = get_rev_reg_delta_json(&rev_reg_id, Some(first_timestamp + 1), None).unwrap();

        assert!(payment.is_some());
        assert_ne!(first_rev_reg_delta, second_rev_reg_delta);
    }
}
