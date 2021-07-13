extern crate log_derive;
extern crate rmp_serde;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use indy_utils::crypto::hash::{Hash};
use rust_base58::ToBase58;

use rlp::UntrustedRlp;
use serde_json;
use serde_json::Value as SJsonValue;

use indy_api_types::ErrorCode;
use crate::domain::ledger::{constants, request::ProtocolVersion};
use indy_api_types::errors::prelude::*;
use crate::services::pool::events::{REQUESTS_FOR_STATE_PROOFS, REQUESTS_FOR_MULTI_STATE_PROOFS};
use indy_utils::crypto::hash::hash as openssl_hash;

use super::PoolService;
use super::types::*;

use self::log_derive::logfn;
use ursa::bls::{Bls, Generator, MultiSignature, VerKey};
use self::node::{Node, TrieDB};
use rust_base58::FromBase58;
use crate::services::pool::Nodes;

mod node;

pub fn parse_generic_reply_for_proof_checking(json_msg: &SJsonValue, raw_msg: &str, sp_key: Option<&[u8]>) -> Option<Vec<ParsedSP>> {
    let type_ = if let Some(type_) = json_msg["type"].as_str() {
        trace!("TransactionHandler::parse_generic_reply_for_proof_checking: type_: {:?}", type_);
        type_
    } else {
        debug!("TransactionHandler::parse_generic_reply_for_proof_checking: <<< No type field");
        return None;
    };

    if REQUESTS_FOR_STATE_PROOFS.contains(&type_) {
        trace!("TransactionHandler::parse_generic_reply_for_proof_checking: built-in");
        if let Some(sp_key) = sp_key {
            _parse_reply_for_builtin_sp(json_msg, type_, sp_key)
        } else {
            warn!("parse_generic_reply_for_proof_checking: can't get key in sp for built-in type");
            None
        }
    } else if let Some((parser, free)) = PoolService::get_sp_parser(type_) {
        trace!("TransactionHandler::parse_generic_reply_for_proof_checking: plugged: parser {:?}, free {:?}",
               parser, free);

        let msg = CString::new(raw_msg).ok()?;
        let mut parsed_c_str = ::std::ptr::null();
        let err = parser(msg.as_ptr(), &mut parsed_c_str);
        if err != ErrorCode::Success {
            debug!("TransactionHandler::parse_generic_reply_for_proof_checking: <<< plugin return err {:?}", err);
            return None;
        }
        let c_str = if parsed_c_str.is_null() { None } else { Some(unsafe { CStr::from_ptr(parsed_c_str) }) };
        let parsed_sps = c_str
            .and_then(|c_str| c_str.to_str().map_err(map_err_trace!()).ok())
            .and_then(|c_str|
                serde_json::from_str::<Vec<ParsedSP>>(c_str)
                    .map_err(|err|
                        debug!("TransactionHandler::parse_generic_reply_for_proof_checking: <<< can't parse plugin response {}", err))
                    .ok());

        let err = free(parsed_c_str);
        trace!("TransactionHandler::parse_generic_reply_for_proof_checking: plugin free res {:?}", err);

        parsed_sps
    } else {
        trace!("TransactionHandler::parse_generic_reply_for_proof_checking: <<< type not supported");
        None
    }
}

pub fn verify_parsed_sp(parsed_sps: Vec<ParsedSP>,
                        nodes: &Nodes,
                        f: usize,
                        gen: &Generator) -> bool {
    for parsed_sp in parsed_sps {
        if parsed_sp.multi_signature["value"]["state_root_hash"].as_str().ne(
            &Some(&parsed_sp.root_hash)) && parsed_sp.multi_signature["value"]["txn_root_hash"].as_str().ne(
            &Some(&parsed_sp.root_hash)) {
            error!("Given signature is not for current root hash, aborting");
            return false;
        }

        let data_to_check_proof_signature =
            _parse_reply_for_proof_signature_checking(&parsed_sp.multi_signature);
        let (signature, participants, value) = unwrap_opt_or_return!(data_to_check_proof_signature, false);
        if !_verify_proof_signature(signature,
                                    participants.as_slice(),
                                    &value,
                                    nodes, f, gen)
            .map_err(|err| warn!("{:?}", err)).unwrap_or(false) {
            return false;
        }

        let proof_nodes = unwrap_or_return!(base64::decode(&parsed_sp.proof_nodes), false);
        let root_hash = unwrap_or_return!(parsed_sp.root_hash.from_base58(), false);
        match parsed_sp.kvs_to_verify {
            KeyValuesInSP::Simple(kvs) => {
                match kvs.verification_type {
                    KeyValueSimpleDataVerificationType::Simple => {
                        for (k, v) in kvs.kvs {
                            let key = unwrap_or_return!(base64::decode(&k), false);
                            if !_verify_proof(proof_nodes.as_slice(),
                                              root_hash.as_slice(),
                                              &key,
                                              v.as_ref().map(String::as_str)) {
                                return false;
                            }
                        }
                    }
                    KeyValueSimpleDataVerificationType::NumericalSuffixAscendingNoGaps(data) => {
                        if !_verify_proof_range(proof_nodes.as_slice(),
                                                root_hash.as_slice(),
                                                data.prefix.as_str(),
                                                data.from,
                                                data.next,
                                                &kvs.kvs) {
                            return false;
                        }
                    }
                    KeyValueSimpleDataVerificationType::MerkleTree(length) => {
                        if !_verify_merkle_tree(proof_nodes.as_slice(),
                                                root_hash.as_slice(),
                                                &kvs.kvs,
                                                length){
                            return false;
                        }
                    }
                }
            }
            //TODO IS-713 support KeyValuesInSP::SubTrie
            kvs => {
                warn!("Unsupported parsed state proof format for key-values {:?} ", kvs);
                return false;
            }
        }
    }

    true
}

#[logfn(Trace)]
pub fn parse_key_from_request_for_builtin_sp(json_msg: &SJsonValue) -> Option<Vec<u8>> {
    let type_ = json_msg["operation"]["type"].as_str()?;
    let json_msg = &json_msg["operation"];
    let key_suffix: String = match type_ {
        constants::GET_ATTR => {
            if let Some(attr_name) = json_msg["raw"].as_str()
                .or_else(|| json_msg["enc"].as_str())
                .or_else(|| json_msg["hash"].as_str()) {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_ATTR attr_name {:?}", attr_name);

                let marker = if ProtocolVersion::is_node_1_3() { '\x01' } else { '1' };
                let hash = openssl_hash(attr_name.as_bytes()).ok()?;
                format!(":{}:{}", marker, hex::encode(hash))
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_ATTR No key suffix");
                return None;
            }
        }
        constants::GET_CRED_DEF => {
            if let (Some(sign_type), Some(sch_seq_no)) = (json_msg["signature_type"].as_str(),
                                                          json_msg["ref"].as_u64()) {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_CRED_DEF sign_type {:?}, sch_seq_no: {:?}", sign_type, sch_seq_no);
                let marker = if ProtocolVersion::is_node_1_3() { '\x03' } else { '3' };
                let tag = if ProtocolVersion::is_node_1_3() { None } else { json_msg["tag"].as_str() };
                let tag = tag.map(|t| format!(":{}", t)).unwrap_or_else(|| "".to_owned());
                format!(":{}:{}:{}{}", marker, sign_type, sch_seq_no, tag)
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_CRED_DEF No key suffix");
                return None;
            }
        }
        constants::GET_NYM | constants::GET_REVOC_REG_DEF => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_NYM");
            "".to_string()
        }
        constants::GET_SCHEMA => {
            if let (Some(name), Some(ver)) = (json_msg["data"]["name"].as_str(),
                                              json_msg["data"]["version"].as_str()) {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_SCHEMA name {:?}, ver: {:?}", name, ver);
                let marker = if ProtocolVersion::is_node_1_3() { '\x02' } else { '2' };
                format!(":{}:{}:{}", marker, name, ver)
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_SCHEMA No key suffix");
                return None;
            }
        }
        constants::GET_REVOC_REG => {
            //{MARKER}:{REVOC_REG_DEF_ID} MARKER = 6
            if let Some(revoc_reg_def_id) = json_msg["revocRegDefId"].as_str() {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_REVOC_REG revoc_reg_def_id {:?}", revoc_reg_def_id);
                let marker = if ProtocolVersion::is_node_1_3() { '\x06' } else { '6' };
                format!("{}:{}", marker, revoc_reg_def_id)
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_REVOC_REG No key suffix");
                return None;
            }
        }
        constants::GET_AUTH_RULE => {
            if let (Some(auth_type), Some(auth_action), Some(field),
                new_value, old_value) = (json_msg["auth_type"].as_str(),
                                         json_msg["auth_action"].as_str(),
                                         json_msg["field"].as_str(),
                                         json_msg["new_value"].as_str(),
                                         json_msg["old_value"].as_str()) {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_AUTH_RULE auth_type {:?}", auth_type);
                let default_old_value = if auth_action == "ADD" { "*" } else { "" };
                format!("1:{}--{}--{}--{}--{}", auth_type, auth_action, field, old_value.unwrap_or(default_old_value), new_value.unwrap_or(""))
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_AUTH_RULE No key suffix");
                return None;
            }
        }
        constants::GET_REVOC_REG_DELTA if json_msg["from"].is_null() => {
            //{MARKER}:{REVOC_REG_DEF_ID} MARKER = 5
            if let Some(revoc_reg_def_id) = json_msg["revocRegDefId"].as_str() {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_REVOC_REG_DELTA revoc_reg_def_id {:?}", revoc_reg_def_id);
                let marker = if ProtocolVersion::is_node_1_3() { '\x05' } else { '5' };
                format!("{}:{}", marker, revoc_reg_def_id)
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_REVOC_REG_DELTA No key suffix");
                return None;
            }
        }
        // TODO add external verification of indexes
        constants::GET_REVOC_REG_DELTA if !json_msg["from"].is_null() => {
            //{MARKER}:{REVOC_REG_DEF_ID} MARKER = 6 for both
            if let Some(revoc_reg_def_id) = json_msg["revocRegDefId"].as_str() {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: GET_REVOC_REG_DELTA revoc_reg_def_id {:?}", revoc_reg_def_id);
                let marker = if ProtocolVersion::is_node_1_3() { '\x06' } else { '6' };
                format!("{}:{}", marker, revoc_reg_def_id)
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< GET_REVOC_REG_DELTA No key suffix");
                return None;
            }
        }
        constants::GET_TXN_AUTHR_AGRMT => {
            match (json_msg["version"].as_str(), json_msg["digest"].as_str(), json_msg["timestamp"].as_u64()) {
                (None, None, _ts) => "2:latest".to_owned(),
                (None, Some(digest), None) => format!("2:d:{}", digest),
                (Some(version), None, None) => format!("2:v:{}", version),
                _ => {
                    error!("parse_key_from_request_for_builtin_sp: <<< GET_TXN_AUTHR_AGRMT Unexpected combination of request parameters, skip StateProof logic");
                    return None;
                }
            }
        }
        constants::GET_TXN_AUTHR_AGRMT_AML => {
            if let Some(version) = json_msg["version"].as_str() {
                format!("3:v:{}", version)
            } else {
                "3:latest".to_owned()
            }
        }
        constants::GET_TXN => {
            if let Some(seq_no) = json_msg["data"].as_u64() {
                format!("{}", seq_no)
            } else {
                error!("parse_key_from_request_for_builtin_sp: <<< GET_TXN has no seq_no, skip AuditProof logic");
                return None;
            }
        }
        _ => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< Unsupported transaction");
            return None;
        }
    };

    let dest = json_msg["dest"].as_str().or_else(|| json_msg["origin"].as_str());
    let key_prefix = match type_ {
        constants::GET_NYM => {
            if let Some(dest) = dest {
                openssl_hash(dest.as_bytes()).ok()?
            } else {
                debug!("TransactionHandler::parse_reply_for_builtin_sp: <<< No dest");
                return None;
            }
        }
        constants::GET_REVOC_REG | constants::GET_REVOC_REG_DELTA | constants::GET_TXN_AUTHR_AGRMT | constants::GET_TXN_AUTHR_AGRMT_AML | constants::GET_AUTH_RULE => {
            Vec::new()
        }
        constants::GET_REVOC_REG_DEF => {
            if let Some(id) = json_msg["id"].as_str() {
                //FIXME
                id.as_bytes().to_vec()
            } else {
                debug!("TransactionHandler::parse_reply_for_builtin_sp: <<< No dest");
                return None;
            }
        }
        constants::GET_TXN => vec![],
        _ => {
            if let Some(dest) = dest {
                dest.as_bytes().to_vec()
            } else {
                debug!("TransactionHandler::parse_reply_for_builtin_sp: <<< No dest");
                return None;
            }
        }
    };

    let mut key = key_prefix;
    key.extend_from_slice(key_suffix.as_bytes());

    Some(key)
}

fn _parse_reply_for_builtin_sp(json_msg: &SJsonValue, type_: &str, key: &[u8]) -> Option<Vec<ParsedSP>> {
    trace!("TransactionHandler::parse_reply_for_builtin_sp: >>> json_msg: {:?}", json_msg);

    assert!(REQUESTS_FOR_STATE_PROOFS.contains(&type_));

    // TODO: FIXME: It is a workaround for Node's problem. Node returns some transactions as strings and some as objects.
    // If node returns marshaled json it can contain spaces and it can cause invalid hash.
    // So we have to save the original string too.
    // See https://jira.hyperledger.org/browse/INDY-699
    let (data, parsed_data): (Option<String>, SJsonValue) = match json_msg["data"] {
        SJsonValue::Null => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: Data is null");
            (None, SJsonValue::Null)
        }
        SJsonValue::String(ref str) => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: Data is string");
            if let Ok(parsed_data) = serde_json::from_str(str) {
                (Some(str.to_owned()), parsed_data)
            } else {
                trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< Data field is invalid json");
                return None;
            }
        }
        SJsonValue::Object(ref map) => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: Data is object");
            (Some(json_msg["data"].to_string()), SJsonValue::from(map.clone()))
        }
        SJsonValue::Array(ref array) => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: Data is array");
            (Some(json_msg["data"].to_string()), SJsonValue::from(array.clone()))
        }
        _ => {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: <<< Data field is invalid type");
            return None;
        }
    };

    trace!("TransactionHandler::parse_reply_for_builtin_sp: data: {:?}, parsed_data: {:?}", data, parsed_data);

    let mut state_proofs = vec![];

    match _parse_reply_for_sp(json_msg, data.as_ref().map(String::as_str), &parsed_data, type_, key) {
        Ok(state_proof) => {
            trace!("TransactionHandler::_parse_reply_for_sp: proof: {:?}", state_proof);
            state_proofs.push(state_proof)
        }
        Err(err) => {
            trace!("TransactionHandler::_parse_reply_for_sp: <<<  {:?}", err);
            return None;
        }
    }

    if REQUESTS_FOR_MULTI_STATE_PROOFS.contains(&type_) {
        match _parse_reply_for_multi_sp(json_msg, data.as_ref().map(String::as_str), &parsed_data, type_, key) {
            Ok(Some(state_proof)) => {
                trace!("TransactionHandler::_parse_reply_for_multi_sp: proof: {:?}", state_proof);
                state_proofs.push(state_proof);
            }
            Ok(None) => {
                trace!("TransactionHandler::_parse_reply_for_multi_sp: <<<  No proof");
            }
            Err(err) => {
                trace!("TransactionHandler::_parse_reply_for_multi_sp: <<<  {:?}", err);
                return None;
            }
        }
    }

    Some(state_proofs)
}

fn _parse_reply_for_sp(json_msg: &SJsonValue, data: Option<&str>, parsed_data: &SJsonValue, xtype: &str, sp_key: &[u8]) -> Result<ParsedSP, String> {
    trace!("TransactionHandler::_parse_reply_for_sp: data: {:?}, parsed_data: {:?}", data, parsed_data);

    let (proof, root_hash, ver_type, multi_sig) = if xtype != constants::GET_TXN {
        let proof = if let Some(proof) = json_msg["state_proof"]["proof_nodes"].as_str() {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: proof: {:?}", proof);
            proof.to_string()
        } else {
            return Err("No proof".to_string());
        };

        let root_hash = if let Some(root_hash) = json_msg["state_proof"]["root_hash"].as_str() {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: root_hash: {:?}", root_hash);
            root_hash
        } else {
            return Err("No root hash".to_string());
        };

        (proof, root_hash, KeyValueSimpleDataVerificationType::Simple, json_msg["state_proof"]["multi_signature"].clone())
    } else {
        let proof = if let Some(path) = parsed_data["auditPath"].as_array() {
            let path_str = json!(path).to_string();
            trace!("TransactionHandler::parse_reply_for_builtin_sp: proof: {:?}", path);
            base64::encode(path_str.as_bytes())
        } else {
            return Err("No proof".to_string());
        };

        let root_hash = if let Some(root_hash) = parsed_data["rootHash"].as_str() {
            trace!("TransactionHandler::parse_reply_for_builtin_sp: root_hash: {:?}", root_hash);
            root_hash
        } else {
            return Err("No root hash".to_string());
        };

        let len = if let Some(len) = parsed_data["ledgerSize"].as_u64() {
            trace!("Ledger length: {}", len);
            len
        } else {
            return Err("No ledger length for this proof".to_string())
        };

        (proof, root_hash, KeyValueSimpleDataVerificationType::MerkleTree(len), json_msg["state_proof"]["multi_signature"].clone())
    };

    let value: Option<String> = match _parse_reply_for_proof_value(json_msg, data, parsed_data, xtype, sp_key) {
        Ok(value) => value,
        Err(err_str) => {
            return Err(err_str);
        }
    };

    trace!("parse_reply_for_builtin_sp: <<< proof {:?}, root_hash: {:?}, dest: {:?}, value: {:?}", proof, root_hash, sp_key, value);

    Ok(ParsedSP {
        root_hash: root_hash.to_owned(),
        proof_nodes: proof.to_owned(),
        multi_signature: multi_sig,
        kvs_to_verify: KeyValuesInSP::Simple(KeyValueSimpleData {
            kvs: vec![(base64::encode(sp_key), value)],
            verification_type: ver_type,
        }),
    })
}

fn _parse_reply_for_multi_sp(_json_msg: &SJsonValue, data: Option<&str>, parsed_data: &SJsonValue, xtype: &str, sp_key: &[u8]) -> Result<Option<ParsedSP>, String> {
    trace!("TransactionHandler::_parse_reply_for_multi_sp: data: {:?}, parsed_data: {:?}", data, parsed_data);

    let (proof_nodes, root_hash, multi_signature, value) = match xtype {
        constants::GET_REVOC_REG_DELTA if _if_rev_delta_multi_state_proof_expected(sp_key) => {
            let proof = if let Some(proof) = parsed_data["stateProofFrom"]["proof_nodes"].as_str() {
                trace!("TransactionHandler::_parse_reply_for_multi_sp: proof: {:?}", proof);
                proof
            } else {
                return Err("No proof".to_string());
            };

            let root_hash = if let Some(root_hash) = parsed_data["stateProofFrom"]["root_hash"].as_str() {
                trace!("TransactionHandler::_parse_reply_for_multi_sp: root_hash: {:?}", root_hash);
                root_hash
            } else {
                return Err("No root hash".to_string());
            };

            let multi_signature = parsed_data["stateProofFrom"]["multi_signature"].clone();

            let value_str = if !parsed_data["value"]["accum_from"].is_null() {
                Some(json!({
                    "lsn": parsed_data["value"]["accum_from"]["seqNo"],
                    "lut": parsed_data["value"]["accum_from"]["txnTime"],
                    "val": parsed_data["value"]["accum_from"],
                }).to_string())
            } else {
                None
            };

            (proof.to_owned(), root_hash.to_owned(), multi_signature, value_str)
        }
        constants::GET_REVOC_REG_DELTA => return Ok(None),
        _ => {
            return Err("Unsupported transaction".to_string());
        }
    };

    trace!("_parse_reply_for_multi_sp: <<< proof {:?}, root_hash: {:?}, dest: {:?}, value: {:?}", proof_nodes, root_hash, sp_key, value);

    Ok(Some(ParsedSP {
        root_hash,
        proof_nodes,
        multi_signature,
        kvs_to_verify: KeyValuesInSP::Simple(KeyValueSimpleData {
            kvs: vec![(base64::encode(sp_key), value)],
            verification_type: KeyValueSimpleDataVerificationType::Simple,
        }),
    }))
}

fn _parse_reply_for_proof_signature_checking(json_msg: &SJsonValue) -> Option<(&str, Vec<&str>, Vec<u8>)> {
    match (json_msg["signature"].as_str(),
           json_msg["participants"].as_array(),
           rmp_serde::to_vec_named(&json_msg["value"])
               .map_err(map_err_trace!())) {
        (Some(signature), Some(participants), Ok(value)) => {
            let participants_unwrap: Vec<&str> = participants
                .iter()
                .flat_map(SJsonValue::as_str)
                .collect();

            if participants.len() == participants_unwrap.len() {
                Some((signature, participants_unwrap, value))
            } else {
                None
            }
        }
        _ => None
    }
}

fn _verify_merkle_tree(proof_nodes: &[u8], root_hash: &[u8], kvs: &[(String, Option<String>)], length: u64) -> bool {
    let nodes = match std::str::from_utf8(proof_nodes) {
        Ok(res) => res,
        Err(err) => {
            error!("Wrong state during mapping bytes to string: {:?}", err);
            return false;
        }
    };
    trace!("_verify_merkle_tree >> nodes: {:?}", nodes);
    let hashes: Vec<String> = match serde_json::from_str(nodes) {
        Ok(vec) => vec,
        Err(err) => {
            error!("Errors during deserialization: {:?}", err);
            return false;
        }
    };

    trace!("_verify_merkle_tree >> hashes: {:?}", hashes);

    let (key, value) = &kvs[0];
    let key = unwrap_or_return!(base64::decode(&key), false);
    let key = unwrap_or_return!(std::str::from_utf8(&key), false);
    let seq_no = match key.parse::<u64>() {
        Ok(num) => num,
        Err(err) => {
            error!("Error while parsing seq_no: {:?}", err);
            return false;
        }
    };

    let turns = _calculate_turns(length, seq_no - 1);
    trace!("_verify_merkle_tree >> turns: {:?}", turns);

    if hashes.len() != turns.len() {
        error!("Different count of hashes and turns, unable to verify");
        return false;
    }

    let hashes_with_turns = hashes.iter().zip(turns).collect::<Vec<(&String, bool)>>();

    let value = match value{
        Some(val) => val,
        None => {return false;}
    };

    trace!("Value to hash: {}", value);

    let value = unwrap_or_return!(serde_json::from_str::<serde_json::Value>(&value), false);
    trace!("serde json success: {:?}", value);
    let value = unwrap_or_return!(rmp_serde::to_vec(&value), false);
    trace!("rmp serde success: {:?}", value);
    let mut hash = match Hash::hash_leaf(&value) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Error while hashing: {:?}", err);
            return false;
        }
    };

    trace!("Hashed leaf in b58: {}", hash.to_base58());

    for (next_hash, turn_right) in hashes_with_turns {
        let _next_hash = unwrap_or_return!(next_hash.from_base58(), false);
        let turned_hash = if turn_right {
            Hash::hash_nodes(&hash, &_next_hash)
        } else {
            Hash::hash_nodes(&_next_hash, &hash)
        };
        hash = match turned_hash {
            Ok(hash) => hash,
            Err(err) => {
                error!("Error while hashing: {:?}", err);
                return false;
            }
        }
    }

    let result = hash.as_slice() == root_hash;
    trace!("_verify_merkle_tree << res: {}, hash: {:?}, root_hash: {:?}", result, hash, root_hash);

    result
}

// true is right
// false is left
fn _calculate_turns(length: u64, idx: u64) -> Vec<bool> {
    let mut idx = idx;
    let mut length = length;
    let mut result: Vec<bool> = vec![];
    while length != 1 {
        let middle = length.next_power_of_two()/2;
        let right = idx < middle;
        result.push(right);
        idx = if right {idx} else {idx - middle};
        length = if right {middle} else {length - middle};
    }
    result.reverse();
    result
}

fn _verify_proof(proofs_rlp: &[u8], root_hash: &[u8], key: &[u8], expected_value: Option<&str>) -> bool {
    debug!("verify_proof >> key {:?}, expected_value {:?}", key, expected_value);
    let nodes: Vec<Node> = UntrustedRlp::new(proofs_rlp).as_list().unwrap_or_default(); //default will cause error below
    let mut map: TrieDB = HashMap::with_capacity(nodes.len());
    for node in &nodes {
        map.insert(node.get_hash(), node);
    }
    map.get(root_hash).map(|root| {
        root
            .get_str_value(&map, key)
            .map_err(map_err_trace!())
            .map(|value| value.as_ref().map(String::as_str).eq(&expected_value))
            .unwrap_or(false)
    }).unwrap_or(false)
}

fn _verify_proof_range(proofs_rlp: &[u8],
                       root_hash: &[u8],
                       prefix: &str,
                       from: Option<u64>,
                       next: Option<u64>,
                       kvs: &[(String, Option<String>)]) -> bool {
    debug!("verify_proof_range >> from {:?}, prefix {:?}, kvs {:?}", from, prefix, kvs);
    let nodes: Vec<Node> = UntrustedRlp::new(proofs_rlp).as_list().unwrap_or_default(); //default will cause error below
    let mut map: TrieDB = HashMap::with_capacity(nodes.len());
    for node in &nodes {
        map.insert(node.get_hash(), node);
    }
    map.get(root_hash).map(|root| {
        let res = root.get_all_values(&map, Some(prefix.as_bytes())).map_err(map_err_err!());
        trace!("All values from trie: {:?}", res);
        let vals = if let Ok(vals) = res {
            vals
        } else {
            error!("Some errors happened while collecting values from state proof");
            return false;
        };
        // Preparation of data for verification
        // Fetch numerical suffixes
        let vals_for_sort_check: Vec<Option<(u64, (String, Option<String>))>> = vals.into_iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, value)| {
                let no = key.replacen(prefix, "", 1).parse::<u64>();
                no.ok().map(|a| (a, (key, Some(value))))
            }).collect();
        if !vals_for_sort_check.iter().all(|a| a.is_some()) {
            error!("Some values in state proof are not correlating with state proof rule, aborting.");
            return false;
        }
        let mut vals_for_sort: Vec<(u64, (String, Option<String>))> = vals_for_sort_check.into_iter().flat_map(|a| a).collect();
        // Sort by numerical suffixes in ascending order
        vals_for_sort.sort_by_key(|&(a, _)| a);
        trace!("Sorted trie values: {:?}", vals_for_sort);
        // Shift on the left side by from
        let vals_with_from = if let Some(from_seqno) = from {
            match vals_for_sort.binary_search_by_key(&from_seqno, |&(a, _)| a) {
                Ok(idx) | Err(idx) => vals_for_sort[idx..].to_vec()
            }
        } else {
            vals_for_sort
        };
        // Verification
        // Check that all values from response match the trie
        trace!("Got values from trie: {:?}", vals_with_from);
        let vals_slice = if let Some(next_seqno) = next {
            match vals_with_from.binary_search_by_key(&next_seqno, |&(a, _)| a) {
                Ok(idx) => &vals_with_from[..idx],
                Err(_) => {
                    error!("Next seqno is incorrect");
                    return false;
                }
            }
        } else {
            vals_with_from.as_slice()
        };
        let vals_prepared: Vec<(String, Option<String>)> = vals_slice.iter().map(|&(_, ref pair)| pair.clone()).collect();
        vals_prepared[..] == kvs[..]
    }).unwrap_or(false)
}

fn _verify_proof_signature(signature: &str,
                           participants: &[&str],
                           value: &[u8],
                           nodes: &Nodes,
                           f: usize,
                           gen: &Generator) -> IndyResult<bool> {
    trace!("verify_proof_signature: >>> signature: {:?}, participants: {:?}, pool_state_root: {:?}", signature, participants, value);

    let mut ver_keys: Vec<&VerKey> = Vec::with_capacity(nodes.len());

    for (name, verkey) in nodes {
        if participants.contains(&name.as_str()) {
            match *verkey {
                Some(ref blskey) => ver_keys.push(blskey),
                _ => return Err(err_msg(IndyErrorKind::InvalidState, format!("Blskey not found for node: {:?}", name)))
            };
        }
    }

    debug!("verify_proof_signature: ver_keys.len(): {:?}", ver_keys.len());

    if ver_keys.len() < (nodes.len() - f) {
        return Ok(false);
    }

    let signature =
        if let Ok(signature) = signature.from_base58() {
            signature
        } else {
            return Ok(false);
        };

    let signature =
        if let Ok(signature) = MultiSignature::from_bytes(signature.as_slice()) {
            signature
        } else {
            return Ok(false);
        };

    debug!("verify_proof_signature: signature: {:?}", signature);

    let res = Bls::verify_multi_sig(&signature, value, ver_keys.as_slice(), gen).unwrap_or(false);

    debug!("verify_proof_signature: <<< res: {:?}", res);
    Ok(res)
}

fn _parse_reply_for_proof_value(json_msg: &SJsonValue, data: Option<&str>, parsed_data: &SJsonValue, xtype: &str, sp_key: &[u8]) -> Result<Option<String>, String> {
    if let Some(data) = data {
        let mut value = json!({});

        let (seq_no, time) = (json_msg["seqNo"].clone(), json_msg["txnTime"].clone());

        match xtype {
            constants::GET_NYM => {
                value["seqNo"] = seq_no;
                value["txnTime"] = time;
            }
            constants::GET_AUTH_RULE => {}
            xtype if xtype.ne(constants::GET_TXN_AUTHR_AGRMT) || _is_full_taa_state_value_expected(sp_key) => {
                value["lsn"] = seq_no;
                value["lut"] = time;
            }
            _ => {}
        }

        match xtype {
            //TODO constants::GET_DDO => support DDO
            constants::GET_TXN => {
                value = json!({});
                if parsed_data["txn"].is_null() && parsed_data["txnMetadata"].is_null() &&
                    parsed_data["ver"].is_null() && parsed_data["reqSignature"].is_null() {
                    return Ok(None);
                }
                if !parsed_data["txn"].is_null() {
                    value["txn"] = parsed_data["txn"].clone();
                }
                if !parsed_data["txnMetadata"].is_null() {
                    value["txnMetadata"] = parsed_data["txnMetadata"].clone();
                }
                if !parsed_data["ver"].is_null() {
                    value["ver"] = parsed_data["ver"].clone();
                }
                if !parsed_data["reqSignature"].is_null() {
                    value["reqSignature"] = parsed_data["reqSignature"].clone();
                }

                // Adjust attrib transaction to match stored state
                if value["txn"]["type"].as_str() == Some(constants::ATTRIB) {
                    if let Some(raw) = value["txn"]["data"]["raw"].as_str() {
                        if raw.is_empty() {
                            value["txn"]["data"]["raw"] = SJsonValue::from("");
                        } else {

                            value["txn"]["data"]["raw"] =
                                SJsonValue::from(hex::encode(openssl_hash(raw.as_bytes()).map_err(|err| err.to_string())?));
                        }
                    } else if let Some(enc) = value["txn"]["data"]["enc"].as_str() {
                        if enc.is_empty() {
                            value["txn"]["data"]["enc"] = SJsonValue::from("");
                        } else {
                            value["txn"]["data"]["enc"] =
                                SJsonValue::from(hex::encode(openssl_hash(enc.as_bytes()).map_err(|err| err.to_string())?));
                        }
                    }
                }
            }
            constants::GET_NYM => {
                value["identifier"] = parsed_data["identifier"].clone();
                value["role"] = parsed_data["role"].clone();
                value["verkey"] = parsed_data["verkey"].clone();
            }
            constants::GET_ATTR => {
                value["val"] = SJsonValue::String(hex::encode(openssl_hash(data.as_bytes()).map_err(|err| err.to_string())?));
            }
            constants::GET_CRED_DEF | constants::GET_REVOC_REG_DEF | constants::GET_REVOC_REG | constants::GET_TXN_AUTHR_AGRMT_AML => {
                value["val"] = parsed_data.clone();
            }
            constants::GET_AUTH_RULE => {
                let constraint = parsed_data
                    .as_array()
                    .and_then(|data| data.first())
                    .map(|auth_rule| auth_rule["constraint"].clone());
                match constraint {
                    Some(ref x) => value = x.clone(),
                    None => return Ok(None)
                };
            }
            constants::GET_SCHEMA => {
                if let Some(map) = parsed_data.as_object() {
                    let mut map = map.clone();
                    map.remove("name");
                    map.remove("version");
                    if map.is_empty() {
                        return Ok(None); // TODO FIXME remove after INDY-699 will be fixed
                    } else {
                        value["val"] = SJsonValue::from(map)
                    }
                } else {
                    return Err("Invalid data for GET_SCHEMA".to_string());
                };
            }
            constants::GET_REVOC_REG_DELTA => {
                if !parsed_data["value"]["accum_to"].is_null() {
                    value["val"] = parsed_data["value"]["accum_to"].clone()
                } else {
                    return Ok(None);
                }
            }
            constants::GET_TXN_AUTHR_AGRMT => {
                if _is_full_taa_state_value_expected(sp_key) {
                    value["val"] = parsed_data.clone();
                } else {
                    value = SJsonValue::String(hex::encode(_calculate_taa_digest(parsed_data["text"].as_str().unwrap_or(""),
                                                                                 parsed_data["version"].as_str().unwrap_or(""))
                        .map_err(|err| format!("Can't calculate expected TAA digest to verify StateProof on the request ({})", err))?));
                }
            }
            _ => {
                return Err("Unknown transaction".to_string());
            }
        }

        let value_str = if let Some(value) = value.as_str() {
            value.to_owned()
        } else {
            value.to_string()
        };

        Ok(Some(value_str))
    } else {
        Ok(None)
    }
}

fn _calculate_taa_digest(text: &str, version: &str) -> IndyResult<Vec<u8>> {
    let content: String = version.to_string() + text;
    openssl_hash(content.as_bytes())
}

fn _is_full_taa_state_value_expected(expected_state_key: &[u8]) -> bool {
    expected_state_key.starts_with(b"2:d:")
}

fn _if_rev_delta_multi_state_proof_expected(sp_key: &[u8]) -> bool {
    sp_key.starts_with(b"\x06:") || sp_key.starts_with(b"6:")
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex::FromHex;
    use libc::c_char;

    /// For audit proofs tree looks like this
    ///         12345
    ///         /  \
    ///      1234  5
    ///     /    \
    ///   12     34
    ///  /  \   /  \
    /// 1   2  3   4

    #[test]
    fn audit_proof_verify_works() {
        let nodes = json!(
            [
                "Gf9aBhHCtBpTYbJXQWnt1DU8q33hwi6nN4f3NhnsBgMZ",
                "68TGAdRjeQ29eNcuFYhsX5uLakGQLgKMKp5wSyPzt9Nq",
                "25KLEkkyCEPSBj4qMFE3AcH87mFocyJEuPJ5xzPGwDgz"
            ]
        ).to_string();
        let kvs = vec![(base64::encode("3".as_bytes()), Some(r#"{"3":"3"}"#.to_string()))];
        let node_bytes = &nodes;
        let root_hash = "CrA5sqYe3ruf2uY7d8re7ePmyHqptHqANtMZcfZd4BvK".from_base58().unwrap();
        assert!(_verify_merkle_tree(node_bytes.as_bytes(), root_hash.as_slice(), kvs.as_slice(), 5));
    }

    #[test]
    fn audit_proof_verify_works_for_invalid_proof() {
        let nodes = json!(
            [
                "Gf9aBhHCtBpTYbJXQWnt1DU8q33hwi6nN4f3NhnsBgM3", //wrong hash here
                "68TGAdRjeQ29eNcuFYhsX5uLakGQLgKMKp5wSyPzt9Nq",
                "25KLEkkyCEPSBj4qMFE3AcH87mFocyJEuPJ5xzPGwDgz"
            ]
        ).to_string();
        let kvs = vec![(base64::encode("3".as_bytes()), Some(r#"{"3":"3"}"#.to_string()))];
        let node_bytes = &nodes;
        let root_hash = "CrA5sqYe3ruf2uY7d8re7ePmyHqptHqANtMZcfZd4BvK".from_base58().unwrap();
        assert!(!_verify_merkle_tree(node_bytes.as_bytes(), root_hash.as_slice(), kvs.as_slice(), 5));
    }

    #[test]
    fn audit_proof_verify_works_for_invalid_root_hash() {
        let nodes = json!(
            [
                "Gf9aBhHCtBpTYbJXQWnt1DU8q33hwi6nN4f3NhnsBgMZ",
                "68TGAdRjeQ29eNcuFYhsX5uLakGQLgKMKp5wSyPzt9Nq",
                "25KLEkkyCEPSBj4qMFE3AcH87mFocyJEuPJ5xzPGwDgz"
            ]
        ).to_string();
        let kvs = vec![(base64::encode("3".as_bytes()), Some(r#"{"3":"3"}"#.to_string()))];
        let node_bytes = &nodes;
        let root_hash = "G9QooEDKSmEtLGNyTwafQiPfGHMqw3A3Fjcj2eLRG4G1".from_base58().unwrap();
        assert!(!_verify_merkle_tree(node_bytes.as_bytes(), root_hash.as_slice(), kvs.as_slice(), 5));
    }

    #[test]
    fn audit_proof_verify_works_for_invalid_ledger_length() {
        let nodes = json!(
            [
                "Gf9aBhHCtBpTYbJXQWnt1DU8q33hwi6nN4f3NhnsBgMZ",
                "68TGAdRjeQ29eNcuFYhsX5uLakGQLgKMKp5wSyPzt9Nq",
                "25KLEkkyCEPSBj4qMFE3AcH87mFocyJEuPJ5xzPGwDgz"
            ]
        ).to_string();
        let kvs = vec![(base64::encode("3".as_bytes()), Some(r#"{"3":"3"}"#.to_string()))];
        let node_bytes = &nodes;
        let root_hash = "CrA5sqYe3ruf2uY7d8re7ePmyHqptHqANtMZcfZd4BvK".from_base58().unwrap();
        assert!(!_verify_merkle_tree(node_bytes.as_bytes(), root_hash.as_slice(), kvs.as_slice(), 9));
    }

    #[test]
    fn audit_proof_verify_works_for_invalid_value() {
        let nodes = json!(
            [
                "Gf9aBhHCtBpTYbJXQWnt1DU8q33hwi6nN4f3NhnsBgMZ",
                "68TGAdRjeQ29eNcuFYhsX5uLakGQLgKMKp5wSyPzt9Nq",
                "25KLEkkyCEPSBj4qMFE3AcH87mFocyJEuPJ5xzPGwDgz"
            ]
        ).to_string();
        let kvs = vec![(base64::encode("3".as_bytes()), Some(r#"{"4":"4"}"#.to_string()))];
        let node_bytes = &nodes;
        let root_hash = "CrA5sqYe3ruf2uY7d8re7ePmyHqptHqANtMZcfZd4BvK".from_base58().unwrap();
        assert!(!_verify_merkle_tree(node_bytes.as_bytes(), root_hash.as_slice(), kvs.as_slice(), 5));
    }

    #[test]
    fn audit_proof_verify_works_for_invalid_seqno() {
        let nodes = json!(
            [
                "Gf9aBhHCtBpTYbJXQWnt1DU8q33hwi6nN4f3NhnsBgMZ",
                "68TGAdRjeQ29eNcuFYhsX5uLakGQLgKMKp5wSyPzt9Nq",
                "25KLEkkyCEPSBj4qMFE3AcH87mFocyJEuPJ5xzPGwDgz"
            ]
        ).to_string();
        let kvs = vec![(base64::encode("4".as_bytes()), Some(r#"{"3":"3"}"#.to_string()))];
        let node_bytes = &nodes;
        let root_hash = "CrA5sqYe3ruf2uY7d8re7ePmyHqptHqANtMZcfZd4BvK".from_base58().unwrap();
        assert!(!_verify_merkle_tree(node_bytes.as_bytes(), root_hash.as_slice(), kvs.as_slice(), 5));
    }

    #[test]
    fn state_proof_nodes_parse_and_get_works() {
        /*
            '33' -> 'v1'
            '34' -> 'v2'
            '3C' -> 'v3'
            '4'  -> 'v4'
            'D'  -> 'v5asdfasdf'
            'E'  -> 'v6fdsfdfs'
        */
        let str = "f8c0f7808080a0762fc4967c792ef3d22fefd3f43209e2185b25e9a97640f09bb4b61657f67cf3c62084c3827634808080808080808080808080f4808080dd808080c62084c3827631c62084c3827632808080808080808080808080c63384c3827633808080808080808080808080f851808080a0099d752f1d5a4b9f9f0034540153d2d2a7c14c11290f27e5d877b57c801848caa06267640081beb8c77f14f30c68f30688afc3e5d5a388194c6a42f699fe361b2f808080808080808080808080";
        let vec = Vec::from_hex(str).unwrap();
        let rlp = UntrustedRlp::new(vec.as_slice());
        let proofs: Vec<Node> = rlp.as_list().unwrap();
        info!("Input");
        for rlp in rlp.iter() {
            info!("{:?}", rlp.as_raw());
        }
        info!("parsed");
        let mut map: TrieDB = HashMap::with_capacity(proofs.len());
        for node in &proofs {
            info!("{:?}", node);
            let out = node.get_hash();
            info!("{:?}", out);
            map.insert(out, node);
        }
        for k in 33..35 {
            info!("Try get {}", k);
            let x = proofs[2].get_str_value(&map, k.to_string().as_bytes()).unwrap().unwrap();
            info!("{:?}", x);
            assert_eq!(x, format!("v{}", k - 32));
        }
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf() {
        /*
            '33' -> 'v1'
            '34' -> 'v2'
            '3C' -> 'v3'
            '4'  -> 'v4'
            'D'  -> 'v5asdfasdf'
            'E'  -> 'v6fdsfdfs'
        */
        let proofs = Vec::from_hex("f8c0f7808080a0762fc4967c792ef3d22fefd3f43209e2185b25e9a97640f09bb4b61657f67cf3c62084c3827634808080808080808080808080f4808080dd808080c62084c3827631c62084c3827632808080808080808080808080c63384c3827633808080808080808080808080f851808080a0099d752f1d5a4b9f9f0034540153d2d2a7c14c11290f27e5d877b57c801848caa06267640081beb8c77f14f30c68f30688afc3e5d5a388194c6a42f699fe361b2f808080808080808080808080").unwrap();
        let root_hash = Vec::from_hex("badc906111df306c6afac17b62f29792f0e523b67ba831651d6056529b6bf690").unwrap();
        assert!(_verify_proof(proofs.as_slice(), root_hash.as_slice(), "33".as_bytes(), Some("v1")));
        assert!(_verify_proof(proofs.as_slice(), root_hash.as_slice(), "34".as_bytes(), Some("v2")));
        assert!(_verify_proof(proofs.as_slice(), root_hash.as_slice(), "3C".as_bytes(), Some("v3")));
        assert!(_verify_proof(proofs.as_slice(), root_hash.as_slice(), "4".as_bytes(), Some("v4")));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            Some(99),
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_empty_from() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(101),
            None,
            &vec![]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_fails_missing_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        // no "abcdefgh11" value in kvs
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            Some(99),
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_fails_extra_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        // no "abcdefgh11" value in kvs
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            Some(99),
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh13".to_string(), Some("4234".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_fails_changed_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            Some(99),
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh12".to_string(), Some("4373".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_fails_wrong_next() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            Some(100),
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_next() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            None,
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
                ("abcdefgh99".to_string(), Some("4522".to_string())),
                ("abcdefgh100".to_string(), Some("3833".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_next_fails_missing_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            None,
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
//                ("abcdefgh24".to_string(), Some("4905".to_string())),
                ("abcdefgh99".to_string(), Some("4522".to_string())),
                ("abcdefgh100".to_string(), Some("3833".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_next_fails_extra_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            None,
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh24".to_string(), Some("4905".to_string())),
                ("abcdefgh25".to_string(), Some("4905".to_string())),
                ("abcdefgh99".to_string(), Some("4522".to_string())),
                ("abcdefgh100".to_string(), Some("3833".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_next_fails_changed_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            Some(10),
            None,
            &vec![
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh25".to_string(), Some("4905".to_string())),
                ("abcdefgh99".to_string(), Some("4522".to_string())),
                ("abcdefgh100".to_string(), Some("3833".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_from() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            None,
            Some(24),
            &vec![
                ("abcdefgh1".to_string(), Some("3630".to_string())),
                ("abcdefgh4".to_string(), Some("3037".to_string())),
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_from_fails_missing_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            None,
            Some(24),
            &vec![
                ("abcdefgh1".to_string(), Some("3630".to_string())),
                ("abcdefgh4".to_string(), Some("3037".to_string())),
//                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_from_fails_extra_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            None,
            Some(24),
            &vec![
                ("abcdefgh1".to_string(), Some("3630".to_string())),
                ("abcdefgh4".to_string(), Some("3037".to_string())),
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
                ("abcdefgh12".to_string(), Some("4373".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_from_fails_changed_values() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            None,
            Some(24),
            &vec![
                ("abcdefgh1".to_string(), Some("3630".to_string())),
                ("abcdefgh4".to_string(), Some("3037".to_string())),
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh12".to_string(), Some("4373".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_in_range_no_from_fails_wrong_next() {
        /*
            'abcdefgh1'     -> '3630'
            'abcdefgh4'     -> '3037'
            'abcdefgh10'    -> '4970'
            'abcdefgh11'    -> '4373'
            'abcdefgh24'    -> '4905'
            'abcdefgh99'    -> '4522'
            'abcdefgh100'   -> '3833'
        */
        let proofs = base64::decode("+QEO34CAgMgwhsWEMzgzM4CAgICAgICAgICAgIbFhDQ5NzD4TYCgWvV3JP22NK5fmfA2xp0DgkFi9rkBdw4ADHTeyez/RtzKgiA0hsWENDkwNYDIIIbFhDMwMzeAgICAyoIgOYbFhDQ1MjKAgICAgICA94CAgKCwvJK5hgh1xdoCVjFsZLAr2Ct5ADxnseuJtF+m80+y64CAgICAgICAgICAgIbFhDM2MzD4OaAfBo1nqEW9/DhdOYucHjHAgqpZsF3f96awYBKZkmR2i8gghsWENDM3M4CAgICAgICAgICAgICAgOuJFhYmNkZWZnaDoNDKeVFnNI85QpRhrd2t8hS4By3wpD4R5ZyUegAPUtga").unwrap();
        let root_hash = "EA9zTfmf5Ex4ZUTPpMwpsQxQzTkevtwg9PADTqJczhSF".from_base58().unwrap();
        assert!(!_verify_proof_range(
            proofs.as_slice(),
            root_hash.as_slice(),
            "abcdefgh",
            None,
            Some(99),
            &vec![
                ("abcdefgh1".to_string(), Some("3630".to_string())),
                ("abcdefgh4".to_string(), Some("3037".to_string())),
                ("abcdefgh10".to_string(), Some("4970".to_string())),
                ("abcdefgh11".to_string(), Some("4373".to_string())),
            ]));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_leaf_through_extension() {
        /*
            '33'  -> 'v1'
            'D'   -> 'v2'
            'E'   -> 'v3'
            '333' -> 'v4'
            '334' -> 'v5'
        */
        let proofs = Vec::from_hex("f8a8e4821333a05fff9765fa0c56a26b361c81b7883478da90259d0c469896e8da7edd6ad7c756f2808080dd808080c62084c3827634c62084c382763580808080808080808080808080808080808080808080808084c3827631f84e808080a06a4096e59e980d2f2745d0ed2d1779eb135a1831fd3763f010316d99fd2adbb3dd80808080c62084c3827632c62084c38276338080808080808080808080808080808080808080808080").unwrap();
        let root_hash = Vec::from_hex("d01bd87a6105a945c5eb83e328489390e2843a9b588f03d222ab1a51db7b9fab").unwrap();
        assert!(_verify_proof(proofs.as_slice(), root_hash.as_slice(), "333".as_bytes(), Some("v4")));
    }

    #[test]
    fn state_proof_verify_proof_works_for_get_value_from_full_node() {
        /*
            '33'  -> 'v1'
            'D'   -> 'v2'
            'E'   -> 'v3'
            '333' -> 'v4'
            '334' -> 'v5'
        */
        let proofs = Vec::from_hex("f8a8e4821333a05fff9765fa0c56a26b361c81b7883478da90259d0c469896e8da7edd6ad7c756f2808080dd808080c62084c3827634c62084c382763580808080808080808080808080808080808080808080808084c3827631f84e808080a06a4096e59e980d2f2745d0ed2d1779eb135a1831fd3763f010316d99fd2adbb3dd80808080c62084c3827632c62084c38276338080808080808080808080808080808080808080808080").unwrap();
        let root_hash = Vec::from_hex("d01bd87a6105a945c5eb83e328489390e2843a9b588f03d222ab1a51db7b9fab").unwrap();
        assert!(_verify_proof(proofs.as_slice(), root_hash.as_slice(), "33".as_bytes(), Some("v1")));
    }

    #[test]
    fn state_proof_verify_proof_works_for_corrupted_rlp_bytes_for_proofs() {
        let proofs = Vec::from_hex("f8c0f7798080a0792fc4967c792ef3d22fefd3f43209e2185b25e9a97640f09bb4b61657f67cf3c62084c3827634808080808080808080808080f4808080dd808080c62084c3827631c62084c3827632808080808080808080808080c63384c3827633808080808080808080808080f851808080a0099d752f1d5a4b9f9f0034540153d2d2a7c14c11290f27e5d877b57c801848caa06267640081beb8c77f14f30c68f30688afc3e5d5a388194c6a42f699fe361b2f808080808080808080808080").unwrap();
        assert_eq!(_verify_proof(proofs.as_slice(), &[0x00], "".as_bytes(), None), false);
    }

    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_get_txn() {
        let json_msg = &json!({
            "type": constants::GET_TXN,
            "data": {
                "auditPath": ["1", "2"],
                "ledgerSize": 2,
                "rootHash": "123",
                "txn": {"test1": "test2", "seqNo": 2},
            },
            "state_proof": {
                "multi_signature": "ms"
            }
        });

        let nodes_str = base64::encode(json!(["1", "2"]).to_string().as_bytes());

        let mut parsed_sps = super::parse_generic_reply_for_proof_checking(json_msg,
                                                                           "",
                                                                           Some("2".as_bytes()))
            .unwrap();

        assert_eq!(parsed_sps.len(), 1);
        let parsed_sp = parsed_sps.remove(0);
        assert_eq!(parsed_sp.root_hash, "123");
        assert_eq!(parsed_sp.multi_signature, "ms");
        assert_eq!(parsed_sp.proof_nodes, nodes_str);
        assert_eq!(parsed_sp.kvs_to_verify,
                   KeyValuesInSP::Simple(KeyValueSimpleData {
                       kvs: vec![(base64::encode("2".as_bytes()), Some(json!({"txn":{"test1": "test2", "seqNo": 2}}).to_string()))],
                       verification_type: KeyValueSimpleDataVerificationType::MerkleTree(2),
                   }));
    }


    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_get_txn_no_multi_signature() {
        let json_msg = &json!({
            "type": constants::GET_TXN,
            "data": {
                "auditPath": ["1", "2"],
                "ledgerSize": 2,
                "rootHash": "123",
                "txn": {"test1": "test2", "seqNo": 2},
//              "multi_signature": "ms"
            }
        });

        let nodes_str = base64::encode(json!(["1", "2"]).to_string().as_bytes());

        let mut parsed_sps = super::parse_generic_reply_for_proof_checking(json_msg,
                                                                           "",
                                                                           Some("2".as_bytes()))
            .unwrap();

        assert_eq!(parsed_sps.len(), 1);
        let parsed_sp = parsed_sps.remove(0);
        assert_eq!(parsed_sp.root_hash, "123");
        assert!(parsed_sp.multi_signature.is_null());
        assert_eq!(parsed_sp.proof_nodes, nodes_str);
        assert_eq!(parsed_sp.kvs_to_verify,
                   KeyValuesInSP::Simple(KeyValueSimpleData {
                       kvs: vec![(base64::encode("2".as_bytes()), Some(json!({"txn":{"test1": "test2", "seqNo": 2}}).to_string()))],
                       verification_type: KeyValueSimpleDataVerificationType::MerkleTree(2),
                   }));
    }

    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_get_txn_no_ledger_length() {
        let json_msg = &json!({
            "type": constants::GET_TXN,
            "data": {
                "auditPath": ["1", "2"],
//                "ledgerSize": 2,
                "rootHash": "123",
                "txn": {"test1": "test2", "seqNo": 2},
                "state_proof": {
                    "multi_signature": "ms"
                }
            }
        });

        assert!(super::parse_generic_reply_for_proof_checking(json_msg,
                                                              "",
                                                              Some("2".as_bytes())).is_none());
    }

    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_get_txn_no_txn() {
        let json_msg = &json!({
            "type": constants::GET_TXN,
            "data": {
                "auditPath": ["1", "2"],
                "ledgerSize": 2,
                "rootHash": "123",
//                "txn": {"test1": "test2", "seqNo": 2},
            },
            "state_proof": {
                "multi_signature": "ms"
            }
        });

        let nodes_str = base64::encode(json!(["1", "2"]).to_string().as_bytes());

        let mut parsed_sps = super::parse_generic_reply_for_proof_checking(json_msg,
                                                                           "",
                                                                           Some("2".as_bytes()))
            .unwrap();

        assert_eq!(parsed_sps.len(), 1);
        let parsed_sp = parsed_sps.remove(0);
        assert_eq!(parsed_sp.root_hash, "123");
        assert_eq!(parsed_sp.multi_signature, "ms");
        assert_eq!(parsed_sp.proof_nodes, nodes_str);
        assert_eq!(parsed_sp.kvs_to_verify,
                   KeyValuesInSP::Simple(KeyValueSimpleData {
                       kvs: vec![(base64::encode("2".as_bytes()), None)],
                       verification_type: KeyValueSimpleDataVerificationType::MerkleTree(2),
                   }));
    }

    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_plugged() {
        extern fn parse(msg: *const c_char, parsed: *mut *const c_char) -> ErrorCode {
            unsafe { *parsed = msg; }
            ErrorCode::Success
        }
        extern fn free(_data: *const c_char) -> ErrorCode { ErrorCode::Success }

        let parsed_sp = json!([{
            "root_hash": "rh",
            "proof_nodes": "pns",
            "multi_signature": "ms",
            "kvs_to_verify": {
                "type": "Simple",
                "kvs": [],
            },
        }]);

        PoolService::register_sp_parser("test", parse, free).unwrap();
        let mut parsed_sps = super::parse_generic_reply_for_proof_checking(&json!({"type".to_owned(): "test"}),
                                                                           parsed_sp.to_string().as_str(),
                                                                           None)
            .unwrap();

        assert_eq!(parsed_sps.len(), 1);
        let parsed_sp = parsed_sps.remove(0);
        assert_eq!(parsed_sp.root_hash, "rh");
        assert_eq!(parsed_sp.multi_signature, "ms");
        assert_eq!(parsed_sp.proof_nodes, "pns");
        assert_eq!(parsed_sp.kvs_to_verify,
                   KeyValuesInSP::Simple(KeyValueSimpleData {
                       kvs: Vec::new(),
                       verification_type: KeyValueSimpleDataVerificationType::Simple,
                   }));
    }

    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_plugged_range() {
        extern fn parse(msg: *const c_char, parsed: *mut *const c_char) -> ErrorCode {
            unsafe { *parsed = msg; }
            ErrorCode::Success
        }
        extern fn free(_data: *const c_char) -> ErrorCode { ErrorCode::Success }

        let parsed_sp = json!([{
            "root_hash": "rh",
            "proof_nodes": "pns",
            "multi_signature": "ms",
            "kvs_to_verify": {
                "type": "Simple",
                "kvs": [],
                "verification_type": {
                    "type": "NumericalSuffixAscendingNoGaps",
                    "from": 1,
                    "next": 2,
                    "prefix": "abc"
                }
            },
        }]);

        PoolService::register_sp_parser("test", parse, free).unwrap();
        let mut parsed_sps = super::parse_generic_reply_for_proof_checking(&json!({"type".to_owned(): "test"}),
                                                                           parsed_sp.to_string().as_str(),
                                                                           None)
            .unwrap();

        assert_eq!(parsed_sps.len(), 1);
        let parsed_sp = parsed_sps.remove(0);
        assert_eq!(parsed_sp.root_hash, "rh");
        assert_eq!(parsed_sp.multi_signature, "ms");
        assert_eq!(parsed_sp.proof_nodes, "pns");
        assert_eq!(parsed_sp.kvs_to_verify,
                   KeyValuesInSP::Simple(KeyValueSimpleData {
                       kvs: Vec::new(),
                       verification_type: KeyValueSimpleDataVerificationType::NumericalSuffixAscendingNoGaps(
                           NumericalSuffixAscendingNoGapsData {
                               from: Some(1),
                               next: Some(2),
                               prefix: "abc".to_string(),
                           }),
                   }));
    }

    #[test]
    fn transaction_handler_parse_generic_reply_for_proof_checking_works_for_plugged_range_nones() {
        extern fn parse(msg: *const c_char, parsed: *mut *const c_char) -> ErrorCode {
            unsafe { *parsed = msg; }
            ErrorCode::Success
        }
        extern fn free(_data: *const c_char) -> ErrorCode { ErrorCode::Success }

        let parsed_sp = json!([{
            "root_hash": "rh",
            "proof_nodes": "pns",
            "multi_signature": "ms",
            "kvs_to_verify": {
                "type": "Simple",
                "kvs": [],
                "verification_type": {
                    "type": "NumericalSuffixAscendingNoGaps",
                    "from": serde_json::Value::Null,
                    "next": serde_json::Value::Null,
                    "prefix": "abc"
                }
            },
        }]);

        PoolService::register_sp_parser("test", parse, free).unwrap();
        let mut parsed_sps = super::parse_generic_reply_for_proof_checking(&json!({"type".to_owned(): "test"}),
                                                                           parsed_sp.to_string().as_str(),
                                                                           None)
            .unwrap();

        assert_eq!(parsed_sps.len(), 1);
        let parsed_sp = parsed_sps.remove(0);
        assert_eq!(parsed_sp.root_hash, "rh");
        assert_eq!(parsed_sp.multi_signature, "ms");
        assert_eq!(parsed_sp.proof_nodes, "pns");
        assert_eq!(parsed_sp.kvs_to_verify,
                   KeyValuesInSP::Simple(KeyValueSimpleData {
                       kvs: Vec::new(),
                       verification_type: KeyValueSimpleDataVerificationType::NumericalSuffixAscendingNoGaps(
                           NumericalSuffixAscendingNoGapsData {
                               from: None,
                               next: None,
                               prefix: "abc".to_string(),
                           }),
                   }));
    }
}
