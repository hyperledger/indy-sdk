#[macro_use]
mod utils;

inject_indy_dependencies!();

extern crate indyrs as indy;
extern crate indyrs as api;
extern crate indy_sys;

use crate::utils::{wallet, anoncreds};
use crate::utils::anoncreds::{COMMON_MASTER_SECRET, CREDENTIAL1_ID, ANONCREDS_WALLET_CONFIG};

use indy::ErrorCode;
use crate::utils::constants::*;
use crate::utils::Setup;

use crate::utils::domain::anoncreds::credential::CredentialInfo;
use crate::utils::domain::anoncreds::credential_for_proof_request::{CredentialsForProofRequest, RequestedCredential};
use crate::utils::domain::anoncreds::proof::Proof;
use indy_api_types::domain::crypto::did::DidValue;

use indy::INVALID_WALLET_HANDLE;

mod high_cases {
    use super::*;

    mod issuer_create_schema {
        use super::*;
        use crate::utils::domain::anoncreds::schema::SchemaId;

        #[test]
        fn issuer_create_schema_works() {
            let (schema_id, _) = anoncreds::issuer_create_schema(ISSUER_DID,
                                                                 GVT_SCHEMA_NAME,
                                                                 SCHEMA_VERSION,
                                                                 GVT_SCHEMA_ATTRIBUTES).unwrap();
            assert_eq!(anoncreds::gvt_schema_id(), schema_id);
        }

        #[test]
        fn issuer_create_schema_works_for_fully_qualified_did() {
            let (schema_id, _) = anoncreds::issuer_create_schema(ISSUER_DID_V1,
                                                                 GVT_SCHEMA_NAME,
                                                                 SCHEMA_VERSION,
                                                                 GVT_SCHEMA_ATTRIBUTES).unwrap();
            assert_eq!(SchemaId(anoncreds::gvt_schema_id_fully_qualified()).0, schema_id);
        }
    }

    mod issuer_create_and_store_credential_def {
        use super::*;

        #[test]
        fn issuer_create_and_store_credential_def_works() {
            anoncreds::init_common_wallet();
        }
    }

    mod issuer_create_credential_offer {
        use super::*;

        #[test]
        fn issuer_create_credential_offer_works() {
            anoncreds::init_common_wallet();
        }
    }

    mod prover_create_master_secret {
        use super::*;

        #[test]
        fn prover_create_master_secret_works() {
            anoncreds::init_common_wallet();
        }
    }

    mod prover_create_credential_req {
        use super::*;

        #[test]
        fn prover_create_credential_req_works() {
            anoncreds::init_common_wallet();
        }
    }

    mod issuer_create_credential {
        use super::*;

        #[test]
        fn issuer_create_credential_works() {
            anoncreds::init_common_wallet();
        }
    }

    mod prover_store_credential {
        use super::*;

        #[test]
        fn prover_store_credential_works() {
            anoncreds::init_common_wallet();
        }
    }

    mod prover_get_credentials {
        use super::*;

        #[test]
        fn prover_get_credentials_works_for_empty_filter() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, r#"{}"#).unwrap();
            println!("!!!!\n{}", credentials);
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 4);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }


        #[test]
        fn prover_get_credentials_works_for_filter_by_issuer_did() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({"issuer_did": ISSUER_DID}).to_string();
            let credentials = anoncreds::prover_get_credentials(wallet_handle, &filter_json).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 2);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_filter_by_schema_id() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter = json!({"schema_id": anoncreds::gvt_schema_id()}).to_string();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, &filter).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 2);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_filter_by_schema_name() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, &format!(r#"{{"schema_name":"{}"}}"#, GVT_SCHEMA_NAME)).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 2);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_filter_by_schema_version() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, &format!(r#"{{"schema_version":"{}"}}"#, SCHEMA_VERSION)).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 3);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_filter_by_schema_issuer_did() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, &format!(r#"{{"schema_issuer_did":"{}"}}"#, ISSUER_DID)).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 3);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_filter_by_cred_def_id() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter = json!({"cred_def_id": anoncreds::issuer_1_gvt_cred_def_id()}).to_string();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, &filter).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 1);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_empty_result() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credentials = anoncreds::prover_get_credentials(wallet_handle, r#"{"cred_def_id":"other_cred_def_id"}"#).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 0);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_works_for_invalid_wallet_handle() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::prover_get_credentials(invalid_wallet_handle, r#"{}"#);
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_get_credential {
        use super::*;

        #[test]
        fn prover_get_credential_works() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credential = anoncreds::prover_get_credential(wallet_handle, CREDENTIAL1_ID).unwrap();
            let credential: CredentialInfo = serde_json::from_str(&credential).unwrap();
            assert_eq!(credential, anoncreds::issuer_1_gvt_credential());

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credential_works_for_not_found() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::prover_get_credential(wallet_handle, "other_cred_id");
            assert_code!(ErrorCode::WalletItemNotFound, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_credentials_search {
        use super::*;

        #[test]
        fn credentials_search_works() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, "{}").unwrap();
            assert_eq!(count, 4);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, count).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 4);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn credentials_search_works_for_filter_by_issuer_did() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({"issuer_did": ISSUER_DID}).to_string();
            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, &filter_json).unwrap();
            assert_eq!(count, 2);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, count).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 2);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn credentials_search_works_for_filter_by_schema_id() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({"schema_id": anoncreds::gvt_schema_id()}).to_string();
            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, &filter_json).unwrap();
            assert_eq!(count, 2);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, count).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 2);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn credentials_search_works_for_filter_by_schema_id_and_cred_def_id() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({
                "schema_id": anoncreds::gvt_schema_id(),
                "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id()
            }).to_string();

            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, &filter_json).unwrap();
            assert_eq!(count, 1);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, count).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 1);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn credentials_search_works_for_filter_by_multiple_schema_ids() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({
                "schema_id": json!({
                    "$in": vec![anoncreds::gvt_schema_id(), anoncreds::xyz_schema_id()]
                })
            }).to_string();

            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, &filter_json).unwrap();
            assert_eq!(count, 3);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, count).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 3);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn credentials_search_works_for_filter_by_schema_id_or_cred_def_id() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({
                "$or": vec![
                    json!({"schema_id": anoncreds::gvt_schema_id()}),
                    json!({"cred_def_id": anoncreds::issuer_1_xyz_cred_def_id()}),
                ]
            }).to_string();

            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, &filter_json).unwrap();
            assert_eq!(count, 3);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, count).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();

            assert_eq!(credentials.len(), 3);
            assert!(credentials.contains(&anoncreds::issuer_1_gvt_credential()));
            assert!(credentials.contains(&anoncreds::issuer_1_xyz_credential()));
            assert!(credentials.contains(&anoncreds::issuer_2_gvt_credential()));

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn credentials_search_works_for_no_results() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let filter_json = json!({"other_field": "other_value"}).to_string();

            let (search_handle, count) = anoncreds::prover_search_credentials(wallet_handle, &filter_json).unwrap();
            assert_eq!(count, 0);

            let credentials = anoncreds::prover_fetch_credentials(search_handle, 100).unwrap();
            let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials).unwrap();
            assert_eq!(credentials.len(), 0);

            anoncreds::prover_close_credentials_search(search_handle).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    //NOTE: There are following credential stored in wallet:
    // {"issuer_did": ISSUER_DID, "schema_id": gvt_schema_id}
    // {"issuer_did": ISSUER_DID, "schema_id": xyz_schema_id}
    // {"issuer_did": DID, "schema_id": gvt_schema_id}
    mod prover_get_credentials_for_proof_req {
        use super::*;
        use crate::utils::domain::anoncreds::schema::SchemaId;
        use crate::utils::domain::anoncreds::credential_definition::CredentialDefinitionId;

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_empty_req() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({}),
            }).to_string();

            let res = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_revealed_attr_only() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"name"
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();
            println!("credentials_json:\n{}", credentials_json);

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.attrs.len(), 1);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_revealed_attr_names_with_restrictions() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "names":["name", "sex"],
                       "restrictions": {
                            "attr::name::value": "Alex"
                       }
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();
            println!("credentials_json:\n{}", credentials_json);

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.attrs.len(), 1);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 1);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_revealed_attr_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "names":["name", "sex"]
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.attrs.len(), 1);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_revealed_attr_in_upper_case() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"NAME"
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.attrs.len(), 1);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_revealed_attr_contains_spaces() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":" name"
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.attrs.len(), 1);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        mod attribute_restrictions_libindy_1_5_format {
            use super::*;

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_issuer() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "issuer_did": ISSUER_DID })]
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_multiple_issuers() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "issuer_did": ISSUER_DID }), json!({ "issuer_did": ISSUER_DID_2 })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_schema() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_id": anoncreds::gvt_schema_id() })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_schema_name() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_name": GVT_SCHEMA_NAME })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_schema_version() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_version": SCHEMA_VERSION })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_schema_issuer_did() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_issuer_did": ISSUER_DID })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_schema_id_or_specific_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_id": anoncreds::gvt_schema_id() }), json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_multiple_schemas() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_id":  anoncreds::gvt_schema_id() }), json!({ "schema_id": anoncreds::xyz_schema_id() })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();

                wallet::close_wallet(wallet_handle).unwrap();

                assert_eq!(credentials.attrs.len(), 1);
                assert_eq!(credentials_for_attr_1.len(), 1);
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_multiple_cred_def_ids() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() }), json!({ "cred_def_id": anoncreds::issuer_2_gvt_cred_def_id() })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_cred_def_id_or_issuer_did() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() }), json!({ "issuer_did": ISSUER_DID_2 })]
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_other_issuer() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "issuer_did": DID_TRUSTEE })]
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_other_schema_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "schema_id": SchemaId::new(&DidValue(DID_TRUSTEE.to_string()), GVT_SCHEMA_NAME, SCHEMA_VERSION) })]
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_other_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": [json!({ "cred_def_id": CredentialDefinitionId::new(&DidValue(DID_TRUSTEE.to_string()), &SchemaId(anoncreds::gvt_schema_id()), "CL", TAG_1) })]
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_null_restrictions() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = r#"{
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes":{
                      "attr1_referent":{
                         "name":"name",
                         "restrictions":[
                            {
                               "schema_id":null,
                               "schema_issuer_did":null,
                               "schema_name":null,
                               "schema_version":null,
                               "issuer_did":"NcYxiDXkpYi6ov5FcYDi1e",
                               "cred_def_id":null
                            }
                         ]
                      }
                   },
                   "requested_predicates":{
                   }
                }"#;

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }
        }

        mod attribute_restrictions_wql_format {
            use super::*;

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_issuer() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "issuer_did": ISSUER_DID })
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_multiple_issuers() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({
                               "issuer_did": json!({
                                    "$in": vec![ISSUER_DID, ISSUER_DID_2]
                               })
                           })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_schema() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "schema_id": anoncreds::gvt_schema_id() })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_schema_name() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "schema_name": GVT_SCHEMA_NAME })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_schema_version() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "schema_version": SCHEMA_VERSION })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_schema_issuer_did() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "schema_issuer_did": ISSUER_DID })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_schema_id_or_specific_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({
                                "$or": vec![
                                    json!({ "schema_id": anoncreds::gvt_schema_id() }),
                                    json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })
                                ]
                           })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_multiple_schemas() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({
                               "schema_id": json!({
                                    "$in": vec![anoncreds::gvt_schema_id(), anoncreds::xyz_schema_id()]
                               })
                           })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_specific_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_multiple_cred_def_ids() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({
                               "cred_def_id": json!({
                                    "$in": vec![anoncreds::issuer_1_gvt_cred_def_id(), anoncreds::issuer_2_gvt_cred_def_id()]
                               })
                           })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_rea_works_for_revealed_attr_for_specific_schema_id_and_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({
                                "schema_id":  anoncreds::gvt_schema_id(),
                                "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id()
                           })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_cred_def_id_or_issuer_did() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({
                                "$or": vec![
                                    json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() }),
                                    json!({ "issuer_did": ISSUER_DID_2 })
                                ]
                           })
                       })
                   }),
                   "requested_predicates": json!({}),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_other_issuer() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "issuer_did": DID_TRUSTEE })
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_other_schema_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "schema_id": SchemaId::new(&DidValue(DID_TRUSTEE.to_string()), GVT_SCHEMA_NAME, SCHEMA_VERSION) })
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_revealed_attr_for_other_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": json!({ "cred_def_id": CredentialDefinitionId::new(&DidValue(DID_TRUSTEE.to_string()), &SchemaId(anoncreds::gvt_schema_id()), "CL", TAG_1) })
                       })
                   }),
                   "requested_predicates": json!({
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.attrs.len(), 1);

                let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
                assert_eq!(credentials_for_attr_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_predicate() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18 })
               }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();
            println!("!!!\n{}", credentials_json);

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.predicates.len(), 1);

            let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
            assert_eq!(credentials_for_predicate_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_predicate_attribute_in_upper_case() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"AGE", "p_type":">=", "p_value":18 })
               }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.predicates.len(), 1);

            let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
            assert_eq!(credentials_for_predicate_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_predicate_attribute_contains_spaces() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"   age", "p_type":">=", "p_value":18 })
               }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.predicates.len(), 1);

            let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
            assert_eq!(credentials_for_predicate_1.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        mod predicate_restrictions_libindy_1_5_format {
            use super::*;

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_issuer_did() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18, "restrictions": [json!({ "issuer_did": ISSUER_DID })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_multiple_issuers() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [json!({ "issuer_did": ISSUER_DID }), json!({ "issuer_did": ISSUER_DID_2 })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_schema_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [json!({ "schema_id": anoncreds::gvt_schema_id() })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_multiple_schema_ids() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [json!({ "schema_id": anoncreds::gvt_schema_id() }), json!({ "schema_id": anoncreds::xyz_schema_id() })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_multiple_cred_def_ids() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() }), json!({ "cred_def_id": anoncreds::issuer_2_gvt_cred_def_id() })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_other_issuer() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [ json!({ "issuer_did":DID_MY2 })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_other_schema_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18,
                       "restrictions": [ json!({ "schema_id": SchemaId::new(&DidValue(DID_TRUSTEE.to_string()), "other_schema_name", SCHEMA_VERSION) })] })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_uppercase() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"AGe", "p_type":">=", "p_value":18 })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }
        }

        mod predicate_restrictions_wql_format {
            use super::*;

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_issuer_did() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "issuer_did": ISSUER_DID })
                         })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_multiple_issuers() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({
                               "issuer_did": json!({
                                    "$in": vec![ISSUER_DID, ISSUER_DID_2]
                               })
                            })
                        })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_schema_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "schema_id": anoncreds::gvt_schema_id() })
                         })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_multiple_schema_ids() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                             "p_type":">=",
                             "p_value":18,
                             "restrictions": json!({
                               "schema_id": json!({
                                    "$in": vec![anoncreds::gvt_schema_id(), anoncreds::xyz_schema_id()]
                               })
                             })
                         })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })
                         })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_multiple_cred_def_ids() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                             "p_value":18,
                            "restrictions": json!({
                               "cred_def_id": json!({
                                    "$in": vec![anoncreds::issuer_1_gvt_cred_def_id(), anoncreds::issuer_2_gvt_cred_def_id()]
                               })
                            })
                        })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 2);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_specific_schema_id_and_cred_def_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({
                                "schema_id": anoncreds::gvt_schema_id(),
                                "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id()
                            })
                        })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 1);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_other_issuer() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "issuer_did":DID_MY2 })
                         })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_predicate_for_other_schema_id() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "schema_id": SchemaId::new(&DidValue(DID_TRUSTEE.to_string()), "other_schema_name", SCHEMA_VERSION) })
                         })
                   }),
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.predicates.len(), 1);

                let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
                assert_eq!(credentials_for_predicate_1.len(), 0);

                wallet::close_wallet(wallet_handle).unwrap();
            }
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_multiple_revealed_attrs_and_predicates() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"name"
                   }),
                   "attr2_referent": json!({
                       "name":"status"
                   })
               }),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18 }),
                   "predicate2_referent": json!({ "name":"height", "p_type":">=", "p_value":160 }),
               }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();

            assert_eq!(credentials.attrs.len(), 2);
            assert_eq!(credentials.predicates.len(), 2);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 2);

            let credentials_for_attr_2 = credentials.attrs.get("attr2_referent").unwrap();
            assert_eq!(credentials_for_attr_2.len(), 1);

            let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
            assert_eq!(credentials_for_predicate_1.len(), 2);

            let credentials_for_predicate_2 = credentials.predicates.get("predicate2_referent").unwrap();
            assert_eq!(credentials_for_predicate_2.len(), 2);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_not_found_attribute() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"some_attr"
                   })
               }),
               "requested_predicates": json!({}),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.attrs.len(), 1);

            let credentials_for_attr_1 = credentials.attrs.get("attr1_referent").unwrap();
            assert_eq!(credentials_for_attr_1.len(), 0);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_not_found_predicate_attribute() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"weight", "p_type":">=", "p_value":58 })
               }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();

            assert_eq!(credentials.attrs.len(), 0);
            assert_eq!(credentials.predicates.len(), 1);

            let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
            assert_eq!(credentials_for_predicate_1.len(), 0);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_not_satisfied_predicate() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":58 })
               }),
            }).to_string();

            let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

            let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();

            assert_eq!(credentials.attrs.len(), 0);
            assert_eq!(credentials.predicates.len(), 1);

            let credentials_for_predicate_1 = credentials.predicates.get("predicate1_referent").unwrap();
            assert_eq!(credentials_for_predicate_1.len(), 0);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_invalid_wallet_handle() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::prover_get_credentials_for_proof_req(invalid_wallet_handle, &anoncreds::proof_request_attr());
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_omitting_attributes_or_predicates_field() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            {
                // specify requested_attributes only
                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name"
                       }
                   }
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();

                credentials.attrs.get("attr1_referent").unwrap();
            }

            {
                // specify requested_predicates only
                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_predicates": {
                       "predicate1_referent": { "name":"age", "p_type":">=", "p_value":18 },
                   }
                }).to_string();

                let credentials_json = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req).unwrap();

                let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();

                credentials.predicates.get("predicate1_referent").unwrap();
            }

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_search_credentials_for_proof_req {
        use super::*;

        #[test]
        fn prover_search_credentials_for_proof_req_works_for_revealed_attr_only() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"name"
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "attr1_referent", 100).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 2);

            anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_search_credentials_for_proof_req_works_for_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "names":["name", "sex"]
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "attr1_referent", 100).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 2);

            anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_search_credentials_for_proof_req_works_for_names_with_restrictions() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "names":["name", "sex"],
                       "restrictions": {
                            "attr::name::value": "Alex"
                       }
                   })
               }),
               "requested_predicates": json!({ }),
            }).to_string();

            let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "attr1_referent", 100).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 1);

            anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_search_credentials_for_proof_req_works_for_non_significant_predicate() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": {},
               "requested_predicates": {
                   "predicate1_referent": { "name":"height", "p_type":">=", "p_value":170 }
               },
            }).to_string();

            let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();
            println!("search_handle:\n{}", search_handle);

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "predicate1_referent", 1).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 1);

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "predicate1_referent", 100).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 1);

            anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_search_credentials_for_proof_req_works_for_predicate() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": {},
               "requested_predicates": {
                   "predicate1_referent": { "name":"height", "p_type":">=", "p_value":171 }
               },
            }).to_string();

            let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();
            println!("search_handle:\n{}", search_handle);

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "predicate1_referent", 1).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 1);

            let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                search_handle, "predicate1_referent", 100).unwrap();

            let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
            assert_eq!(credentials.len(), 0);

            anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        mod extra_query {
            use super::*;

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_attribute_and_extra_query() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                       "nonce":"123432421212",
                       "name":"proof_req_1",
                       "version":"0.1",
                       "requested_attributes": json!({
                           "attr1_referent": json!({
                               "name":"name",
                               "restrictions": json!({ "issuer_did": ISSUER_DID })
                           })
                       }),
                       "requested_predicates": json!({
                       }),
                    }).to_string();

                let extra_query = json!({
                        "attr1_referent": json!({
                            "attr::name::value": "Alex"
                        })
                    }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_attribute_and_multiple_extra_queries() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                       "nonce":"123432421212",
                       "name":"proof_req_1",
                       "version":"0.1",
                       "requested_attributes": json!({
                           "attr1_referent": json!({
                               "name":"name",
                               "restrictions": json!({ "issuer_did": ISSUER_DID })
                           })
                       }),
                       "requested_predicates": json!({
                       }),
                    }).to_string();

                let extra_query = json!({
                        "attr1_referent": json!({
                            "attr::name::value": "Alex",
                            "schema_id": anoncreds::gvt_schema_id(),
                            "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id()
                        })
                    }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_attribute_and_extra_query_no_results() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                       "nonce":"123432421212",
                       "name":"proof_req_1",
                       "version":"0.1",
                       "requested_attributes": json!({
                           "attr1_referent": json!({
                               "name":"name",
                               "restrictions": json!({ "issuer_did": ISSUER_DID })
                           })
                       }),
                       "requested_predicates": json!({
                       }),
                    }).to_string();

                let extra_query = json!({
                        "attr1_referent": json!({
                            "attr::name::value": "AlexOther"
                        })
                    }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 0);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_attribute_and_extra_query_with_other_attribute_referent() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                       "nonce":"123432421212",
                       "name":"proof_req_1",
                       "version":"0.1",
                       "requested_attributes": json!({
                           "attr1_referent": json!({
                               "name":"name",
                               "restrictions": json!({ "issuer_did": ISSUER_DID })
                           })
                       }),
                       "requested_predicates": json!({
                       }),
                    }).to_string();

                let extra_query = json!({
                        "other_attribute_referent": json!({
                            "attr::name::value": "AlexOtherValue"
                        })
                    }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_predicate_and_extra_query() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "issuer_did": ISSUER_DID })
                         })
                   }),
                }).to_string();

                let extra_query = json!({
                    "attr1_referent": json!({
                        "attr::age::value": "28"
                    })
                }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "predicate1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_predicate_and_extra_query_no_results() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({}),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({
                            "name":"age",
                            "p_type":">=",
                            "p_value":18,
                            "restrictions": json!({ "issuer_did": ISSUER_DID })
                         })
                   }),
                }).to_string();

                let extra_query = json!({
                    "predicate1_referent": json!({
                        "attr::age::value": "0"
                    })
                }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "predicate1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 0);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_multiple_revealed_attrs_and_predicates_and_extra_queries() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name"
                       }),
                       "attr2_referent": json!({
                           "name":"status"
                       })
                   }),
                   "requested_predicates": json!({
                       "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18 }),
                       "predicate2_referent": json!({ "name":"height", "p_type":">=", "p_value":160 }),
                   }),
                }).to_string();

                let extra_query = json!({
                    "attr1_referent": json!({
                        "attr::name::value": "Alex"
                    }),
                    "attr2_referent": json!({
                        "attr::status::value": "partial"
                    }),
                    "predicate1_referent": json!({
                        "attr::age::value": "28"
                    }),
                    "predicate2_referent": json!({
                        "attr::height::value": "175"
                    })
                }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();
                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr2_referent", 100).unwrap();
                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "predicate1_referent", 100).unwrap();
                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 2);

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "predicate2_referent", 100).unwrap();
                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_attribute_restriction_old_format_and_extra_query_contains_or_operator() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                       "nonce":"123432421212",
                       "name":"proof_req_1",
                       "version":"0.1",
                       "requested_attributes": json!({
                           "attr1_referent": json!({
                               "name":"name",
                               "restrictions": [json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })]
                           })
                       }),
                       "requested_predicates": json!({
                       }),
                    }).to_string();

                let extra_query = json!({
                        "attr1_referent": json!({
                            "$or": [
                                {
                                     "attr::name::value": "Alex",
                                },
                                {
                                     "attr::name::value": "Alexander",
                                }
                            ]
                        })
                    }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_get_credentials_for_proof_req_works_for_requested_attribute_restriction_and_extra_query_contain_the_same_operator() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                       "nonce":"123432421212",
                       "name":"proof_req_1",
                       "version":"0.1",
                       "requested_attributes": json!({
                           "attr1_referent": json!({
                               "name":"name",
                               "restrictions": json!({
                                    "$or": vec![
                                        json!({ "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() })
                                    ]
                               })
                           })
                       }),
                       "requested_predicates": json!({
                       }),
                    }).to_string();

                let extra_query = json!({
                        "attr1_referent": json!({
                            "$or": [
                                {
                                     "attr::name::value": "Alex",
                                },
                                {
                                     "attr::name::value": "Alexander",
                                }
                            ]
                        })
                    }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, Some(&extra_query)).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<serde_json::Value> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 1);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_search_credentials_for_proof_req_works_for_empty_restrictions() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": {}
                       }),
                       "attr2_referent": json!({
                           "name":"age",
                           "restrictions": []
                       })
                   }),
                   "requested_predicates": json!({ }),
                }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                    search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 2);

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                    search_handle, "attr2_referent", 100).unwrap();

                let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 2);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn prover_search_credentials_for_proof_req_works_for_proof_req_v1_contains_fully_qualified_restrictions() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": {
                                "issuer_did": ISSUER_DID_V1
                           }
                       })
                   }),
                   "requested_predicates": json!({ }),
                }).to_string();

                let res = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None);
                assert_code!(ErrorCode::CommonInvalidStructure, res);

                wallet::close_wallet(wallet_handle).unwrap();
            }

            #[test]
            fn prover_search_credentials_for_proof_req_works_for_fully_qualified_restrictions() {
                anoncreds::init_common_wallet();

                let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

                let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": json!({
                       "attr1_referent": json!({
                           "name":"name",
                           "restrictions": {
                                "issuer_did": ISSUER_DID_V1
                           }
                       })
                   }),
                   "requested_predicates": json!({ }),
                   "ver": "2.0"
                }).to_string();

                let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, &proof_req, None).unwrap();

                let credentials_json = anoncreds::prover_fetch_next_credentials_for_proof_req(
                    search_handle, "attr1_referent", 100).unwrap();

                let credentials: Vec<RequestedCredential> = serde_json::from_str(&credentials_json).unwrap();
                assert_eq!(credentials.len(), 0);

                anoncreds::prover_close_credentials_search_for_proof_req(search_handle).unwrap();

                wallet::close_wallet(wallet_handle).unwrap();
            }
        }
    }

    mod prover_create_proof_works {
        use super::*;

        #[test]
        fn prover_create_proof_works() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({
                    "predicate1_referent": json!({ "cred_id": CREDENTIAL1_ID })
                 })
            }).to_string();

            anoncreds::prover_create_proof(wallet_handle,
                                           &anoncreds::proof_request_attr_and_predicate(),
                                           &requested_credentials_json,
                                           COMMON_MASTER_SECRET,
                                           &anoncreds::schemas_for_proof(),
                                           &anoncreds::cred_defs_for_proof(),
                                           "{}").unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": {},
                 "requested_attributes": {
                    "attr1_referent": { "cred_id": CREDENTIAL1_ID, "revealed":true }
                 },
                 "requested_predicates": {}
            }).to_string();

            anoncreds::prover_create_proof(wallet_handle,
                                           &anoncreds::proof_request_attr_names(),
                                           &requested_credentials_json,
                                           COMMON_MASTER_SECRET,
                                           &anoncreds::schemas_for_proof(),
                                           &anoncreds::cred_defs_for_proof(),
                                           "{}").unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_using_not_satisfy_credential() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"some_attr"
                   })
               }),
               "requested_predicates": json!({}),
            }).to_string();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &proof_req,
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod verifier_verify_proof {
        use super::*;

        #[test]
        fn verifier_verify_proof_works_for_correct_proof() {
            let valid = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr(),
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_works_for_proof_does_not_correspond_to_request() {
            let other_proof_req_json = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"sex"
                   })
               }),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18 })
               }),
            }).to_string();

            let res = anoncreds::verifier_verify_proof(&other_proof_req_json,
                                                       &anoncreds::proof_json(),
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn verifier_verify_proof_works_for_proof_does_not_correspond_to_request_attribute() {
            let other_proof_req_json = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "attr1_referent": json!({
                       "name":"sex"
                   })
               }),
               "requested_predicates": json!({}),
            }).to_string();
            let res = anoncreds::verifier_verify_proof(&other_proof_req_json,
                                                       &anoncreds::proof_json(),
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected, res);
        }

        #[test]
        fn verifier_verify_proof_works_for_wrong_revealed_attr_value() {
            let proof_json = anoncreds::proof_json().replace(r#"name":"1139481716457488690172217916278103335"#, r#"name":"1111111111111111111111111111111111111"#);

            let res = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr(),
                                                       &proof_json,
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected, res);
        }

        #[test]
        fn verifier_verify_proof_works_for_wrong_encoded() {
            let proof_json = anoncreds::proof_json().replace(r#"encoded":"1139481716457488690172217916278103335"#, r#"encoded":"1111111111111111111111111111111111111"#);

            let res = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr(),
                                                       &proof_json,
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected, res);
        }

        #[test]
        #[ignore] // TODO: Libindy doesn't aware about algorithm used for encoding of attribute values. We can do this check only on application level.
        fn verifier_verify_proof_works_for_wrong_raw() {
            let proof_json = anoncreds::proof_json().replace(r#"raw":"Alex"#, r#"raw":"Bob"#);

            let res = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr(),
                                                       &proof_json,
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected, res);
        }

        #[test]
        fn verifier_verify_proof_works_for_revealed_attr_case_insensitive() {
            let proof_req_json = anoncreds::proof_request_attr().replace(r#""name":"name""#, r#""name":"NAME""#);

            let valid = anoncreds::verifier_verify_proof(&proof_req_json,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_works_for_proof_with_names() {
            let other_proof_req_json = json!({
                "nonce":"123432421212",
                "name":"proof_req_1",
                "version":"0.1",
                "requested_attributes": {
                    "attr1_referent": {
                        "names":["name", "age"],
                        "revealed": "true"
                    }
                },
                "requested_predicates": {},
            }).to_string();

            let valid = anoncreds::verifier_verify_proof(&other_proof_req_json,
                                                         &anoncreds::proof_json_names(),
                                                         &anoncreds::schema_names(),
                                                         &anoncreds::cred_defs_names(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }
    }

    #[test]
    fn verifier_verify_proof_works_for_proof_with_names_in_different_credentials() {
        let other_proof_req_json = json!({
                "nonce":"123432421212",
                "name":"proof_req_1",
                "version":"0.1",
                "requested_attributes": {
                    "attr1_referent": {
                        "names":["name", "age"],
                        "revealed": "true",

                    }
                },
                "requested_predicates": {},
            }).to_string();

        let valid = anoncreds::verifier_verify_proof(&other_proof_req_json,
                                                     &anoncreds::proof_json_names_diff_creds(),
                                                     &anoncreds::schema_names(),
                                                     &anoncreds::cred_defs_names(),
                                                     "{}",
                                                     "{}").unwrap();
        assert!(!valid);
    }

    mod verifier_verify_proof_with_proof_req_restrictions {
        use super::*;

        #[test]
        fn verifier_verify_proof_successful() {
            let valid = anoncreds::verifier_verify_proof(&anoncreds::proof_request_restrictions(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_success_for_valid_issuer_did() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "issuer_did": { "$in": [ISSUER_DID] } }
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_issuer_did() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "issuer_did": { "$in": ["NO DID"] } }
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_predicate_restriction() {
            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": {},
               "requested_predicates": {
                    "attr1_referent": {
                    "name":"age", "p_type":">=", "p_value":18, "restrictions": { "schema_id": "Not HERE" }}

               },
            }).to_string();

            let mut proof: Proof = serde_json::from_str(&anoncreds::proof_json()).unwrap();
            proof.requested_proof.revealed_attrs.remove("attr1_referent").unwrap();
            proof.requested_proof.predicates.insert(
                "attr1_referent".to_string(),
                serde_json::from_str(&json!({ "sub_proof_index": 0 }).to_string()).unwrap(),
            );


            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &serde_json::to_string(&proof).unwrap(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_for_valid_schema_id() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_id": anoncreds::gvt_schema_id() },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_schema_id() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_id": "Not HERE" },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_for_valid_schema_issuer_did() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_issuer_did": ISSUER_DID },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_schema_issuer_did() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_issuer_did": "Not HERE" },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_for_valid_schema_name() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_name": GVT_SCHEMA_NAME },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_schema_name() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_name": "Not HERE" },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_for_valid_schema_version() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_version": SCHEMA_VERSION },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_schema_version() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "schema_version": "Not HERE" },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_for_valid_cred_def_id() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id() },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_missing_cred_def_id() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "cred_def_id": "Not HERE" },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let valid = anoncreds::verifier_verify_proof(&proof_req,
                                                         &anoncreds::proof_json(),
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_fails_for_unknown_restriction() {
            let proof_req = json!({
                   "nonce":"123432421212",
                   "name":"proof_req_1",
                   "version":"0.1",
                   "requested_attributes": {
                       "attr1_referent": {
                           "name":"name",
                           "restrictions": { "UNKNOWN": "Not HERE" },
                       }
                   },
                   "requested_predicates": {},
                }).to_string();

            let res = anoncreds::verifier_verify_proof(&proof_req,
                                                       &anoncreds::proof_json(),
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }
    }

    mod issuer_rotate_credential_def {
        use super::*;

        #[test]
        fn issuer_rotate_credential_def_works() {
            let setup = Setup::wallet();

            let (cred_def_id, cred_def_json) = anoncreds::issuer_create_credential_definition(setup.wallet_handle,
                                                                                              ISSUER_DID,
                                                                                              &anoncreds::gvt_schema_json(),
                                                                                              TAG_1,
                                                                                              Some(SIGNATURE_TYPE),
                                                                                              Some(&anoncreds::default_cred_def_config()))
                .unwrap();

            let temp_cred_def_json = anoncreds::issuer_rotate_credential_def_start(setup.wallet_handle, &cred_def_id, None).unwrap();

            assert_ne!(serde_json::from_str::<serde_json::Value>(&cred_def_json).unwrap(),
                       serde_json::from_str::<serde_json::Value>(&temp_cred_def_json).unwrap());

            anoncreds::issuer_rotate_credential_def_apply(setup.wallet_handle, &cred_def_id).unwrap();
        }

        #[test]
        fn issuer_rotate_credential_def_works_no_cred_def() {
            let setup = Setup::wallet();

            let res = anoncreds::issuer_rotate_credential_def_start(setup.wallet_handle, &anoncreds::issuer_1_gvt_cred_def_id(), None);
            assert_code!(ErrorCode::WalletItemNotFound, res);
        }

        #[test]
        fn issuer_rotate_credential_def_apply_works_for_no_temporary_cred_def() {
            let setup = Setup::wallet();

            let res = anoncreds::issuer_rotate_credential_def_apply(setup.wallet_handle, &anoncreds::issuer_1_gvt_cred_def_id());
            assert_code!(ErrorCode::WalletItemNotFound, res);
        }
    }

    mod to_unqualified {
        use super::*;
        use utils::domain::anoncreds::schema::SchemaV1;
        use utils::domain::anoncreds::credential_definition::CredentialDefinitionV1;
        use utils::domain::anoncreds::credential_offer::CredentialOffer;
        use utils::domain::anoncreds::credential_request::CredentialRequest;

        #[test]
        fn to_unqualified_ids() {
            assert_eq!(DID_MY1, anoncreds::to_unqualified(DID_MY1_V1).unwrap());
            assert_eq!(DID_MY1, anoncreds::to_unqualified(DID_MY1).unwrap());

            assert_eq!(anoncreds::gvt_schema_id(), anoncreds::to_unqualified(&anoncreds::gvt_schema_id_fully_qualified()).unwrap());
            assert_eq!(anoncreds::gvt_cred_def_id(), anoncreds::to_unqualified(&anoncreds::gvt_cred_def_id_fully_qualified()).unwrap());
            assert_eq!(anoncreds::local_gvt_cred_def_id(), anoncreds::to_unqualified(&anoncreds::local_gvt_cred_def_id_fully_qualified()).unwrap());
        }

        #[test]
        fn to_unqualified_objects() {
            let setup = Setup::wallet();

            let (schema_id, schema_json) = anoncreds::issuer_create_schema(ISSUER_DID_V1, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).unwrap();

            assert_eq!(anoncreds::gvt_schema_id(), anoncreds::to_unqualified(&schema_id).unwrap());

            let schema_json_un = anoncreds::to_unqualified(&schema_json).unwrap();
            let schema: SchemaV1 = ::serde_json::from_str(&schema_json_un).unwrap();
            assert_eq!(anoncreds::gvt_schema_id(), schema.id.0);

            let (cred_def_id, cred_def_json) = anoncreds::issuer_create_credential_definition(setup.wallet_handle, ISSUER_DID_V1, &schema_json, TAG_1, None, None).unwrap();

            assert_eq!(anoncreds::local_gvt_cred_def_id(), anoncreds::to_unqualified(&cred_def_id).unwrap());

            let cred_def_json_un = anoncreds::to_unqualified(&cred_def_json).unwrap();
            let cred_def: CredentialDefinitionV1 = ::serde_json::from_str(&cred_def_json_un).unwrap();
            assert_eq!(anoncreds::local_gvt_cred_def_id(), cred_def.id.0);
            assert_eq!(anoncreds::gvt_schema_id(), cred_def.schema_id.0);

            let cred_offer_json = anoncreds::issuer_create_credential_offer(setup.wallet_handle, &cred_def_id).unwrap();

            let cred_offer_json_un = anoncreds::to_unqualified(&cred_offer_json).unwrap();
            let cred_offer: CredentialOffer = ::serde_json::from_str(&cred_offer_json_un).unwrap();
            assert_eq!(anoncreds::local_gvt_cred_def_id(), cred_offer.cred_def_id.0);
            assert_eq!(anoncreds::gvt_schema_id(), cred_offer.schema_id.0);

            anoncreds::prover_create_master_secret(setup.wallet_handle, COMMON_MASTER_SECRET).unwrap();

            let (cred_req_json, _) = anoncreds::prover_create_credential_req(setup.wallet_handle, DID_MY1_V1, &cred_offer_json, &cred_def_json_un, COMMON_MASTER_SECRET).unwrap();

            let cred_req_json_un = anoncreds::to_unqualified(&cred_req_json).unwrap();
            let cred_req: CredentialRequest = ::serde_json::from_str(&cred_req_json_un).unwrap();
            assert_eq!(DID_MY1.to_string(), cred_req.prover_did.0);
            assert_eq!(anoncreds::local_gvt_cred_def_id(), cred_req.cred_def_id.0);
        }
    }
}

#[cfg(not(feature = "only_high_cases"))]
mod medium_cases {
    use super::*;
    use std::collections::HashSet;
    use crate::utils::domain::anoncreds::schema::{AttributeNames, MAX_ATTRIBUTES_COUNT};
    use crate::utils::domain::anoncreds::proof_request::{AttributeInfo, ProofRequestPayload};

    mod issuer_create_schema {
        use super::*;

        #[test]
        fn issuer_create_schema_works_for_empty_attrs() {
            let res = anoncreds::issuer_create_schema(ISSUER_DID,
                                                      GVT_SCHEMA_NAME,
                                                      SCHEMA_VERSION,
                                                      "[]");
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn issuer_create_schema_works_for_invalid_issuer_did() {
            let res = anoncreds::issuer_create_schema(INVALID_BASE58_DID,
                                                      GVT_SCHEMA_NAME,
                                                      SCHEMA_VERSION,
                                                      GVT_SCHEMA_ATTRIBUTES);
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn issuer_create_schema_works_for_attrs_count_more_than_acceptable() {
            let attr_names: AttributeNames = (0..MAX_ATTRIBUTES_COUNT + 1).map(|i| i.to_string())
                .collect::<HashSet<String>>().into();

            let res = anoncreds::issuer_create_schema(ISSUER_DID,
                                                      GVT_SCHEMA_NAME,
                                                      SCHEMA_VERSION,
                                                      &serde_json::to_string(&attr_names).unwrap());
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }
    }

    mod issuer_create_and_store_credential_def {
        use super::*;

        #[test]
        fn issuer_create_and_store_credential_def_works_for_invalid_schema() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let schema = r#"{"name":"name","version":"1.0", "attr_names":["name"]}"#;

            let res = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                     ISSUER_DID,
                                                                     &schema,
                                                                     TAG_1,
                                                                     None,
                                                                     Some(&anoncreds::default_cred_def_config()));
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_invalid_did() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                     INVALID_IDENTIFIER,
                                                                     &anoncreds::gvt_schema_json(),
                                                                     TAG_1,
                                                                     None,
                                                                     Some(&anoncreds::default_cred_def_config()));
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_empty_schema_attr_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let mut schema = anoncreds::gvt_schema();
            schema.attr_names = AttributeNames::new();

            let res = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                     ISSUER_DID,
                                                                     &serde_json::to_string(&schema).unwrap(),
                                                                     TAG_1,
                                                                     None,
                                                                     Some(&anoncreds::default_cred_def_config()));
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_correct_signature_type() {
            let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet("issuer_create_and_store_credential_def_works_for_correct_signature_type").unwrap();

            anoncreds::issuer_create_credential_definition(wallet_handle,
                                                           ISSUER_DID,
                                                           &anoncreds::gvt_schema_json(),
                                                           TAG_1,
                                                           Some(SIGNATURE_TYPE),
                                                           Some(&anoncreds::default_cred_def_config())).unwrap();

            wallet::close_wallet(wallet_handle).unwrap();
            wallet::delete_wallet(&wallet_config, WALLET_CREDENTIALS).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_invalid_signature_type() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                     ISSUER_DID,
                                                                     &anoncreds::gvt_schema_json(),
                                                                     TAG_1,
                                                                     Some("some_type"),
                                                                     Some(&anoncreds::default_cred_def_config()));
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_invalid_config() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                     ISSUER_DID,
                                                                     &anoncreds::gvt_schema_json(),
                                                                     TAG_1,
                                                                     None,
                                                                     Some(r#"{"support_revocation":"TRUE"}"#));
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_duplicate() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            anoncreds::issuer_create_credential_definition(wallet_handle,
                                                           ISSUER_DID,
                                                           &anoncreds::gvt_schema_json(),
                                                           TAG_1,
                                                           Some(SIGNATURE_TYPE),
                                                           Some(&anoncreds::default_cred_def_config())).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_null_config() {
            let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet("issuer_create_and_store_credential_def_works_for_null_config").unwrap();

            anoncreds::issuer_create_credential_definition(wallet_handle,
                                                           DID_MY1,
                                                           &anoncreds::gvt_schema_json(),
                                                           TAG_1,
                                                           None,
                                                           None).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
            wallet::delete_wallet(&wallet_config, WALLET_CREDENTIALS).unwrap();
        }

        #[test]
        fn issuer_create_and_store_credential_def_works_for_invalid_wallet() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::issuer_create_credential_definition(INVALID_WALLET_HANDLE,
                                                                     ISSUER_DID,
                                                                     &anoncreds::gvt_schema_json(),
                                                                     TAG_1,
                                                                     None,
                                                                     Some(&anoncreds::default_cred_def_config()));
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod issuer_create_credential_offer {
        use super::*;

        #[test]
        fn issuer_create_credential_offer_works_for_unknown_cred_def_id() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::issuer_create_credential_offer(wallet_handle, "NcYxiDXkpYi6ov5FcYDi1e:3:CL:100");
            assert_code!(ErrorCode::WalletItemNotFound, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_credential_offer_works_for_invalid_wallet_handle() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::issuer_create_credential_offer(invalid_wallet_handle,
                                                                &anoncreds::issuer_1_gvt_cred_def_id());
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_create_master_secret {
        use super::*;

        #[test]
        fn prover_create_master_secret_works_for_duplicate_name() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::prover_create_master_secret(wallet_handle, COMMON_MASTER_SECRET);
            assert_code!(ErrorCode::AnoncredsMasterSecretDuplicateNameError, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_master_secret_works_invalid_wallet_handle() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::prover_create_master_secret(invalid_wallet_handle, COMMON_MASTER_SECRET);
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_create_credential_req {
        use super::*;

        #[test]
        fn prover_create_credential_req_works_for_invalid_credential_offer() {
            let (credential_def, _, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::prover_create_credential_req(wallet_handle,
                                                              DID_MY1,
                                                              &serde_json::to_string(&anoncreds::issuer_1_gvt_cred_offer_info()).unwrap(),
                                                              &credential_def,
                                                              COMMON_MASTER_SECRET);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_credential_req_works_for_invalid_credential_def() {
            let (_, credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credential_def = r#"{
                            "schema_seq_no":1,
                            "signature_type":"CL",
                            "primary":{
                                "n":"121212",
                                "s":"432192"
                            }
                        }"#;
            let res = anoncreds::prover_create_credential_req(wallet_handle,
                                                              DID_MY1,
                                                              &credential_offer,
                                                              credential_def,
                                                              COMMON_MASTER_SECRET);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_credential_req_works_for_invalid_master_secret() {
            let (credential_def, credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::prover_create_credential_req(wallet_handle,
                                                              DID_MY1,
                                                              &credential_offer,
                                                              &credential_def,
                                                              "invalid_master_secret_name");
            assert_code!(ErrorCode::WalletItemNotFound, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_credential_req_works_for_invalid_wallet() {
            let (credential_def, credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::prover_create_credential_req(invalid_wallet_handle,
                                                              DID_MY1,
                                                              &credential_offer,
                                                              &credential_def,
                                                              COMMON_MASTER_SECRET);
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_credential_req_works_for_credential_def_not_correspond_to_credential_offer() {
            let (issuer1_gvt_credential_def, issuer1_gvt_credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let mut issuer_create_credential_offer: serde_json::Value = serde_json::from_str(&issuer1_gvt_credential_offer).unwrap();
            issuer_create_credential_offer["key_correctness_proof"]["c"] = serde_json::Value::String("11111111".to_string());

            let other_credential_offer = serde_json::to_string(&issuer_create_credential_offer).unwrap();

            let res = anoncreds::prover_create_credential_req(wallet_handle,
                                                              DID_MY1,
                                                              &other_credential_offer,
                                                              &issuer1_gvt_credential_def,
                                                              COMMON_MASTER_SECRET);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod issuer_create_credential {
        use super::*;

        #[test]
        fn issuer_create_credential_works_for_credential_does_not_correspond_to_credential_values() {
            let (_, credential_offer, credential_req, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::issuer_create_credential(wallet_handle,
                                                          &credential_offer,
                                                          &credential_req,
                                                          &anoncreds::xyz_credential_values_json(),
                                                          None,
                                                          None);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_credential_works_for_for_invalid_wallet_handle() {
            let (_, credential_offer, credential_req, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::issuer_create_credential(invalid_wallet_handle,
                                                          &credential_offer,
                                                          &credential_req,
                                                          &anoncreds::gvt_credential_values_json(),
                                                          None,
                                                          None);
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_credential_works_for_for_invalid_credential_req_json() {
            let (_, credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credential_req = r#"{
                                        "blinded_ms":{"ur":null},
                                        "prover_did":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW",
                                    }"#;

            let res = anoncreds::issuer_create_credential(wallet_handle,
                                                          &credential_offer,
                                                          &credential_req,
                                                          &anoncreds::gvt_credential_values_json(),
                                                          None,
                                                          None);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn issuer_create_credential_works_for_for_invalid_credential_values_json() {
            let (_, credential_offer, credential_request, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let credential_values_json = r#"{
                                           "sex":"male",
                                           "name":"Alex",
                                           "height":"175",
                                           "age":"28"
                                         }"#;

            let res = anoncreds::issuer_create_credential(wallet_handle,
                                                          &credential_offer,
                                                          &credential_request,
                                                          &credential_values_json,
                                                          None,
                                                          None);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_store_credential {
        use super::*;

        #[test]
        fn prover_store_credential_works_for_invalid_wallet_handle() {
            let (credential_def_json, credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();
            let (prover_wallet_handle, prover_wallet_config) = wallet::create_and_open_default_wallet("prover_store_credential_works_for_invalid_wallet_handle").unwrap();

            anoncreds::prover_create_master_secret(prover_wallet_handle, COMMON_MASTER_SECRET).unwrap();

            let (credential_req, credential_req_meta) = anoncreds::prover_create_credential_req(prover_wallet_handle,
                                                                                                DID_MY1,
                                                                                                &credential_offer,
                                                                                                credential_def_json,
                                                                                                COMMON_MASTER_SECRET).unwrap();

            let (credential_json, _, _) = anoncreds::issuer_create_credential(wallet_handle,
                                                                              &credential_offer,
                                                                              &credential_req,
                                                                              &anoncreds::gvt_credential_values_json(),
                                                                              None,
                                                                              None).unwrap();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::prover_store_credential(invalid_wallet_handle,
                                                         CREDENTIAL1_ID,
                                                         &credential_req_meta,
                                                         &credential_json,
                                                         &credential_def_json,
                                                         None);
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
            wallet::close_wallet(prover_wallet_handle).unwrap();
            wallet::delete_wallet(&prover_wallet_config, WALLET_CREDENTIALS).unwrap();
        }

        #[test]
        fn prover_store_credential_works_for_invalid_credential_json() {
            let (credential_def_json, credential_offer, _, _) = anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let (_, cred_req_metadata) = anoncreds::prover_create_credential_req(wallet_handle,
                                                                                 DID_MY1,
                                                                                 &credential_offer,
                                                                                 &credential_def_json,
                                                                                 COMMON_MASTER_SECRET).unwrap();

            let credential_json = format!(r#"{{
                                                       "values":{},
                                                       "cred_def_id":"{}",
                                                       "revoc_reg_seq_no":null
                                                    }}"#, anoncreds::gvt_credential_values_json(), anoncreds::issuer_1_gvt_cred_def_id());

            let res = anoncreds::prover_store_credential(wallet_handle,
                                                         CREDENTIAL1_ID,
                                                         &cred_req_metadata,
                                                         &credential_json,
                                                         &credential_def_json,
                                                         None);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_get_credentials {
        use super::*;

        #[test]
        fn prover_get_credentials_works_for_invalid_json() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::prover_get_credentials(wallet_handle, r#""issuer_did": 12345"#);
            assert_code!(ErrorCode::WalletQueryError, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_delete_credential {
        use super::*;

        #[test]
        fn prover_delete_credential_works_for_not_found() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let res = anoncreds::prover_delete_credential(wallet_handle, "other_cred_id");
            assert_code!(ErrorCode::WalletItemNotFound, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod prover_get_credentials_for_proof_req {
        use super::*;

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_invalid_proof_req() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_predicates": json!({}),
            }).to_string();

            let res = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req);
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_invalid_predicate() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"age" })
               }),
            }).to_string();

            let res = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req);

            wallet::close_wallet(wallet_handle).unwrap();

            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn prover_get_credentials_for_proof_req_works_for_invalid_predicate_type() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({}),
               "requested_predicates": json!({
                   "predicate1_referent": json!({ "name":"age", "p_type":"!=", "p_value":18 }),
               }),
            }).to_string();

            let res = anoncreds::prover_get_credentials_for_proof_req(wallet_handle, &proof_req);
            wallet::close_wallet(wallet_handle).unwrap();
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }
    }

    mod prover_create_proof_works {
        use super::*;

        #[test]
        fn prover_create_proof_works_for_no_name_or_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_no_name_or_names(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");

            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_both_name_and_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_both_name_and_names(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");

            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_empty_names() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_empty_names(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");

            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_invalid_wallet_handle() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let invalid_wallet_handle = INVALID_WALLET_HANDLE;
            let res = anoncreds::prover_create_proof(invalid_wallet_handle,
                                                     &anoncreds::proof_request_attr(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");
            assert_code!(ErrorCode::WalletInvalidHandle, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_invalid_master_secret() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_and_predicate(),
                                                     &requested_credentials_json,
                                                     "invalid_master_secret_name",
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");
            assert_code!(ErrorCode::WalletItemNotFound, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_invalid_schemas_json() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_and_predicate(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &"{}",
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_invalid_credential_defs_json() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_attributes": json!({
                    "attr1_referent": json!({ "cred_id": CREDENTIAL1_ID, "revealed":true })
                 }),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_and_predicate(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     "{}",
                                                     "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }

        #[test]
        fn prover_create_proof_works_for_invalid_requested_credentials_json() {
            anoncreds::init_common_wallet();

            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            let requested_credentials_json = json!({
                 "self_attested_attributes": json!({}),
                 "requested_predicates": json!({})
            }).to_string();

            let res = anoncreds::prover_create_proof(wallet_handle,
                                                     &anoncreds::proof_request_attr_and_predicate(),
                                                     &requested_credentials_json,
                                                     COMMON_MASTER_SECRET,
                                                     &anoncreds::schemas_for_proof(),
                                                     &anoncreds::cred_defs_for_proof(),
                                                     "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);

            wallet::close_wallet(wallet_handle).unwrap();
        }
    }

    mod verifier_verify_proof {
        use super::*;

        #[test]
        fn verifier_verify_proof_works_for_wrong_proof() {
            let proof_json = anoncreds::proof_json().replace("1139481716457488690172217916278103335", "1111111111111111111111111111111111111");

            let valid = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr(),
                                                         &proof_json,
                                                         &anoncreds::schemas_for_proof(),
                                                         &anoncreds::cred_defs_for_proof(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(!valid);
        }

        #[test]
        fn verifier_verify_proof_works_for_invalid_proof_json_format() {
            let proof_json = r#"{"proof":{"proofs":{"credential::58479554-187f-40d9-b0a5-a95cfb0338c3":{"primary_proof":{"eq_proof":{"revealed_attrs":{"name":"1139481716457488690172217916278103335"},"a_prime":"80401564260558483983794628158664845806393125691167675024527906210615204776868092566789307767601325086260531777605457298059939671624755239928848057947875953445797869574854365751051663611984607735255307096920094357120779812375573500489773454634756645206823074153240319316758529163584251907107473703779754778699279153037094140428648169418133281187947677937472972061954089873405836249023133445286756991574802740614183730141450546881449500189789970102133738133443822618072337620343825908790734460412932921199267304555521397418007577171242880211812703320270140386219809818196744216958369397014610013338422295772654405475023","e":"31151798717381512709903464053695613005379725796031086912986270617392167764097422442809244590980303622977555221812111085160553241592792901","v":"524407431684833626723631303096063196973911986967748096669183384949467719053669910411426601230736351335262754473490498825342793551112426427823428399937548938048089615644972537564428344526295733169691240937176356626523864731701111189536269488496019586818879697981955044502664124964896796783428945944075084807859935155837238670987272778459356531608865162828109489758902085206073584532002909678902616210042778963974064479140826712481297584040209095459963718975102750913306565864485279810056629704077428898739021040190774575868853629858297299392839284660771662690107106553362040805152261505268111067408422298806905178826507224233050991301274817252924123120887017757639206512015559321675322509820081151404696713509158685022511201565062671933414307463988209696457343022378430051265752251403461414881325357657438328740471164157220698425309006894962942640219890219594168419276308074677144722217081026358892787770650248878952483621","m":{"age":"10477979077744818183854012231360633424177093192344587159214818537659704987539982653663361680650769087122324965941845552897155693994859927792964720675888893623940580527766661802170","sex":"15368219775809326116045200104269422566086585069798988383076685221700842794654771075432385446820819836777771517356551059931242867733879324915651894894695726945279462946826404864068","height":"268172143999991481637372321419290603042446269013750825098514042757459298040087626745653681785038933035820421862976371452111736537699176931068992453946771945552540798204580069806"},"m1":"119095745403940293668103184388411799541118279558928018597628509118163496000813590825371995586347826189221837428823000332905316924389185590810015031744029496470545254805993327676570037596326743185389101389800942263689809725968264069601565478411709555274081560719927118853299543998608664701485475703881376151770","m2":"3166313665375815600922385342096456465402430622944571045536207479553790085339726549928012930073803465171492637049498407367742103524723152099973753540483894420905314750248333232361"},"ge_proofs":[{"u":{"2":"6494171529848192644197417834173236605253723188808961394289041396341136802965710957759175642924978223517091081898946519122412445399638640485278379079647638538597635045303985779767","0":"7739508859260491061487569748588091139318989278758566530899756574128579312557203413565436003310787878172471425996601979342157451689172171025305431595131816910273398879776841751855","3":"9424758820140378077609053635383940574362083113571024891496206162696034958494400871955445981458978146571146602763357500412840538526390475379772903513687358736287298159312524034159","1":"9011979414559555265454106061917684716953356440811838475257096756618761731111646531136628099710567973381801256908067529269805992222342928842825929421929485785888403149296320711642"},"r":{"DELTA":"2119857977629302693157808821351328058251440215802746362450951329352726877165815663955490999790457576333458830301801261754696823614762123890412904169206391143688952648566814660498520188221060505840151491403269696751525874990487604723445355651918681212361562384420233903265612599812725766212744963540390806334870022328290970137051148373040320927100063898502086531019924715927190306801273252711777648467224661735618842887006436195147540705753550974655689586750013569294343535843195025962867299786380033532422131203367401906988124836294104501525520053613392691214421562815044433237816093079784307397782961917892254668290115653012265908717124278607660504580036193346698672079435538219972121355893074219968755049500875222141","2":"879097501989202140886939888802566536179834329508897124489020677433754766947767937608431979796722207676629625451150104784909666168153917345813160237337412296010679353735699663083287427507870244565918756969618964144516025526404618052053542009438548457492400344119561349471929199757453154204191407620539220514897529346602664135146454509169680801061111878075145734123580343470361019624175036825631373890661124315134340427076598351080893567995392248394683875116715114577054906406649006122102488431184007790011073389768061904597267545895265921673106871142463561948479668876241841045522543174660428236658891636170119227855493059358614089146415798861053408542832475696099851160385105386001523305465829676723036394820593263477","0":"1724016272047416140958096373304304971004826284109046259544344355102178044512441391364907122486655755929044720001281832600729467778103556397960700809066582436321515744527550472324028227472294258045699756170293405547851344921626775854114063087070898499913846456795761213291925373770081490280103876827479351849800210782799381740073719081199000612284788683993320623339686128531187019125095700122135094060470612862911102824801065698176788174959069186600426519872015152034176356923049531650418553748519941342115963599848111324793380438600664408464987023646615003553912544410140730587797458882329021327455905737414352355326238028222782957735440607899424838572541602600159016542488644761584240884783618700311735467659132540546","3":"2317535203964314926167241523636020444600002667629517624482931328850422196008281300859516069440995466415138723103558631951648519232327284208990029010060986032518946759289078833125920310350676484457972303378558158127406345804560689086460633931717939234025886786468170219981598030245042011840614339386724945679531091642132820284896626191109974537171662283750959028046143650291367908660204201563611944187723824430780626387525165408619587771059635528553832034409311888615502905143628507219523591091412192645348525327725381323865648645828460581593542176351568614465903523790649219812666979685223535464526901006270478687017672202058914176692964406859722580270696925877498058525086810338471380117323227744481903228027847825795","1":"1119193929864813751243160041764170298897380522230946444206167281178657213260394833843687899872857393015947283159245092452814155776571829885921814072299525859857844030379558685168895306445277750249341844789101670896570226707650318347992386244538723699686941887792682779028216548922683313576597384354842537728667739985216662699631842296096507821667149950956179957306177525178260912379909156360834120816956949271530622510333943914411903103069247646327625753995178999023427645468623522280255892736633780185163496867644317005801241786702434621502492159672660131289312665511793827552317714835658019088880972220344126692027952749318018900669839090109361161616086319604439015851316798257015063653414161203599184730094765941653"},"mj":"10477979077744818183854012231360633424177093192344587159214818537659704987539982653663361680650769087122324965941845552897155693994859927792964720675888893623940580527766661802170","alpha":"46280660038407959140964701167450659223532556136388451390393713283900546119670373626221864441898929302821705811144923685080534692512705456699843367809872982836890616398604933641265111106644805368974824737276965928297120628041257593166650593538539384316563258781595629888673792430276007730792093088812056156937735120078929629310611907731935101448992312370312134173482115524436767558802102266208152808607480693236511858269018733175523724309089010048330044458187371675333889670055578652283806685440133357512406700879353713629795062705271430695988191782837658895477702634883214188598350625843489120361660836956958750828038278027538830855628653513539929730230905015331221220017847248793929813230252015802389329428995718799619565984669228143200627972926117282688854152516298117476837960100343260648687249027349308513966440386556698667484082658689","t":{"DELTA":"46814992964714978733007076702016837564951956529003697497847838781899848384824991374342901164708655443686022921583406187082133141084994843502230809550055933825660668160300304112671478218513259983054489597176651737200716259733573469298437873515151377206364940530308167934399245072298875358347931404742292788785586833114480704138718996633638362933821933388459210678374952072108333767698704767907612549860590824123780096225591372365712106060039646448181221691765233478768574198237963457485496438076793333937013217675591500849193742006533651525421426481898699626618796271544860105422331629265388419155909716261466161258430","2":"59423006413504086085782234600502410213751379553855471973440165009200961757474676407242673622935614782362911290590560535490636029324125251850583605745046201217673654522625983661578962623803698461459190578519097656221453474955879823750445359506290522280566225253310030053812918275525607874059407284653434046369835156477189219911810464401689041140506062300317020407969423270374033482533711564673658146930272487464489365713112043565257807490520178903336328210031106311280471651300486164966423437275272281777742004535722142265580037959473078313965482591454009972765788975683031385823798895914265841131145707278751512534120","0":"56510878078818710798555570103060159621941668074271797077206591818472978018558098567975838757566260370093327989369045722406190165972775356924844244889146946158949660988214890388299203816110339909687790860564719380865809705044646711632599987968183128514431910561478715003212633874423067294596323864121737000450543142072142652163818450299889830999149821558252183477517484127000480272695698860647674027831262149565273068850774090998356019534296579838685977022988536930596918054160990243868372150609770079720240227817149126735182138479851227052696211125454858584118346950878092387488482897777914362341820607560926173967363","3":"63511079416489489495396586813126304469185174450150717746314545118902972011091412254834718868134635251731510764117528579641756327883640004345178347120290107941107152421856942264968771810665927914509411385404403747487862696526824127219640807008235054362138760656969613951620938020257273816713908815343872804442748694361381399025862438391456307852482826748664499083370705834755863016895566228300904018909174673301643617543662527772400085378252706897979609427451977654028887889811453690146157824251379525221390697200211891556653698308665831075787991412401737090471273439878635073797691350863566834141222438011402987450926","1":"30348838247529448929141877305241172943867610065951047292188826263950046630912426030349276970628525991007036685038199133783991618544554063310358191845473212966131475853690378885426974792306638181168558731807811629973716711132134244797541560013139884391800841941607502149630914097258613821336239993125960064136287579351403225717114920758719152701696123905042695943045383536065833292374624566478931465135875411483860059753175449604448434619593495399051968638830805689355610877075130302742512428461286121237297212174164897833936610857614962734658136750299346971377383141235020438750748045568800723867413392427848651081274"},"predicate":{"name":"age","p_type":"GE","p_value":18}}]},"non_revoc_proof":null}},"aggregated_proof":{"c_hash":"81135772044295974649282368084258333955993271555081206390568996949836231116301","c_list":[[2,124,231,47,189,36,247,160,61,220,165,35,97,165,203,185,133,253,81,239,67,127,156,49,189,16,140,30,177,161,221,54,154,0,127,143,98,212,114,193,188,85,206,171,198,140,9,192,10,254,218,120,201,182,40,141,80,35,81,148,204,192,41,5,186,33,50,77,211,163,124,130,32,219,193,167,79,43,181,76,19,249,53,79,70,221,205,36,180,50,120,255,161,227,196,204,71,106,221,131,220,7,73,86,128,208,48,58,123,63,82,24,170,141,143,56,221,96,151,108,105,38,185,243,224,112,177,101,195,87,208,201,39,123,165,125,92,104,234,188,54,92,31,158,178,152,52,205,26,156,237,241,23,15,76,220,168,32,175,230,157,197,225,70,57,237,8,81,13,17,95,70,143,56,162,223,203,8,48,153,51,51,118,116,32,139,187,222,146,86,165,111,125,107,203,18,212,28,168,22,62,69,204,207,122,148,25,30,92,120,83,214,116,221,204,120,230,70,128,139,181,110,69,93,253,240,69,16,113,224,246,41,142,0,83,237,186,4,50,156,206,199,89,74,96,168,249,240,101,16,103,234,162,219,52,218,207],[1,191,167,2,151,36,61,136,184,172,120,86,127,88,109,119,56,21,167,171,217,221,24,64,246,237,255,152,81,183,201,191,59,234,213,101,254,91,33,205,120,71,215,144,160,243,145,109,19,151,241,46,135,132,50,143,219,207,197,35,89,103,83,212,96,83,222,101,55,57,220,161,252,115,39,62,46,160,30,138,221,89,125,66,114,150,5,95,63,10,55,107,102,73,40,69,41,6,57,0,64,226,152,66,181,149,251,50,28,53,18,26,221,5,188,67,125,184,190,200,56,92,132,201,242,211,37,2,43,6,146,88,228,120,204,190,4,118,134,106,118,110,249,145,175,165,116,197,200,183,207,215,197,79,207,203,29,182,231,151,248,233,107,41,79,234,250,27,33,33,107,102,240,47,37,230,243,185,93,192,52,31,73,211,11,173,150,92,194,154,172,247,221,206,129,85,193,105,172,140,201,40,240,200,28,94,1,96,204,175,113,170,46,134,229,111,215,208,237,252,84,50,249,41,214,79,38,194,23,212,7,164,153,217,23,252,32,114,145,58,189,118,104,131,84,184,115,175,199,227,219,117,23,113,113,180,3],[240,104,187,71,84,144,129,123,12,181,215,233,27,55,56,54,94,57,17,42,111,42,112,234,192,23,226,103,118,198,189,175,175,1,102,64,128,100,221,201,134,106,83,239,69,43,150,172,95,206,145,224,207,239,39,193,30,200,90,125,175,125,59,47,250,224,193,21,64,112,101,131,128,249,96,165,73,33,174,64,69,252,209,158,130,53,23,158,217,173,69,51,12,145,70,174,15,206,13,181,50,246,50,110,223,65,250,44,39,33,8,47,169,242,147,3,190,164,110,20,68,5,142,133,38,198,151,161,167,0,219,128,126,120,190,23,153,22,250,78,114,241,252,181,74,142,65,123,225,153,75,159,78,84,28,110,203,105,231,238,75,138,121,233,75,163,221,69,106,143,1,217,251,43,147,252,189,122,19,124,189,180,206,91,165,199,41,172,233,102,14,91,162,254,16,142,60,230,39,200,208,236,101,69,101,152,233,217,100,206,31,120,211,191,90,56,205,40,180,120,47,210,224,86,153,34,86,237,204,11,183,227,0,224,15,201,32,228,4,210,43,156,68,246,137,150,103,197,191,150,155,181,78,5,134,58],[1,214,184,139,205,251,132,131,8,186,140,58,211,242,134,120,121,253,128,192,10,252,172,101,44,26,119,56,212,8,248,71,19,96,59,12,233,191,63,187,217,35,191,160,127,247,189,247,229,111,252,101,126,10,142,252,238,215,211,137,137,164,114,186,255,199,183,50,103,9,158,63,134,140,162,154,188,109,52,31,92,78,38,228,0,60,225,100,239,88,114,95,48,71,7,117,168,45,45,177,178,62,87,197,98,174,123,249,26,237,179,12,63,182,46,218,183,148,163,222,179,159,146,56,142,190,122,100,211,6,86,237,10,7,111,186,27,66,95,252,108,247,203,1,111,60,13,218,104,63,128,125,197,11,201,138,33,122,37,31,163,123,120,132,65,122,208,60,80,87,113,183,28,31,74,106,18,79,52,245,113,184,94,202,72,223,8,128,209,43,77,237,119,208,255,144,26,76,223,77,177,131,237,49,150,251,53,150,115,33,254,237,185,15,140,234,205,99,248,252,171,245,192,104,151,194,190,186,249,180,246,9,169,165,0,221,7,107,39,67,58,178,176,99,212,40,247,49,127,7,94,5,170,65,154,28,104],[1,247,26,202,244,120,131,95,151,52,56,38,141,232,178,50,61,45,235,61,12,68,11,180,174,222,110,211,141,253,198,204,248,192,40,99,237,1,45,170,79,208,3,13,135,89,195,65,3,228,224,146,181,198,14,79,78,237,168,81,108,151,68,12,88,242,120,200,120,193,253,51,167,140,43,175,59,18,160,190,233,21,213,135,162,76,38,48,163,110,155,197,97,93,211,183,95,42,172,249,98,59,161,136,70,39,142,48,242,44,154,103,186,161,214,215,0,254,166,150,111,71,242,102,209,125,25,65,144,223,211,137,223,239,50,96,185,171,120,155,171,98,204,23,102,253,68,141,91,240,127,170,199,249,217,165,164,37,174,212,159,232,140,196,216,140,205,102,84,104,220,223,9,249,75,245,78,157,245,203,235,154,73,34,77,12,227,138,93,105,178,114,255,210,88,216,202,64,69,128,220,211,113,51,15,185,103,236,52,187,49,29,162,20,35,21,65,188,33,46,11,172,59,15,221,36,33,213,14,121,36,218,76,80,97,197,83,64,145,73,194,43,233,144,251,86,112,209,230,67,234,116,172,219,123,50,46],[1,114,216,159,37,214,198,117,230,153,15,176,95,20,29,134,179,207,209,35,101,193,47,54,130,141,78,213,54,167,31,73,105,177,129,135,6,135,45,107,103,16,133,187,74,217,42,40,1,214,60,70,78,245,86,82,150,75,91,235,181,249,129,147,202,15,86,250,222,240,203,236,102,39,53,147,79,178,124,184,97,73,65,136,74,29,219,182,83,167,221,203,32,200,243,130,65,234,133,181,203,35,86,21,123,170,74,174,5,132,1,149,77,141,158,193,249,130,37,53,253,234,228,144,66,152,232,246,26,193,6,53,139,45,231,173,115,87,89,61,197,9,96,73,229,189,49,44,203,214,156,139,58,153,77,13,90,35,157,130,184,150,161,69,145,157,4,206,52,216,227,233,113,202,54,154,153,100,83,97,135,88,197,227,42,52,28,221,91,117,56,183,198,102,231,37,232,226,136,142,115,218,175,45,221,143,130,215,184,39,102,172,126,253,152,108,254,241,17,98,70,223,191,138,251,227,243,32,180,190,223,69,135,0,97,105,115,189,221,134,26,159,32,210,172,233,7,65,238,77,203,159,181,188,203,159,190]]}},"requested_proof":{"revealed_attrs":{"attr1_referent":["credential::58479554-187f-40d9-b0a5-a95cfb0338c3","Alex","1139481716457488690172217916278103335"]},"unrevealed_attrs":{},"self_attested_attrs":{},"predicates":{"predicate1_referent":"credential::58479554-187f-40d9-b0a5-a95cfb0338c3"}}}"#;

            let res = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr_and_predicate(),
                                                       &proof_json,
                                                       &anoncreds::schemas_for_proof(),
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn verifier_verify_proof_works_for_invalid_schemas() {
            let schemas_json = "{}";

            let res = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr_and_predicate(),
                                                       &anoncreds::proof_json(),
                                                       schemas_json,
                                                       &anoncreds::cred_defs_for_proof(),
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn verifier_verify_proof_works_for_invalid_credential_defs() {
            let res = anoncreds::verifier_verify_proof(&anoncreds::proof_request_attr_and_predicate(),
                                                       &anoncreds::proof_json(),
                                                       &anoncreds::schemas_for_proof(),
                                                       "{}",
                                                       "{}",
                                                       "{}");
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }
    }

    mod verifier_verify_proof_with_proof_req_restrictions {
        use super::*;

        #[test]
        fn verifier_verify_proof_success_with_in_wql() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!({
                        "cred_def_id":{
                                "$in":[
                                    anoncreds::issuer_1_gvt_cred_def_id(),
                                    "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0:TAG_1",
                                    "not here 3",
                            ] }
                    })).unwrap(),
                    non_revoked: None,
                },
            );
            let res = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                       &anoncreds::proof_json_restrictions(),
                                                       &anoncreds::schemas_for_proof_restrictions(),
                                                       &anoncreds::cred_defs_for_proof_restrictions(),
                                                       "{}",
                                                       "{}");
            assert!(res.is_ok());
        }

        #[test]
        fn verifier_verify_proof_fails_with_in_wql() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!({
                        "cred_def_id":{
                                "$in":[
                                    "not here 1",
                                    "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0:TAG_1",
                                    "not here 3",
                            ] }
                    })).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_with_vector_of_filters() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!([
                        {
                            "cred_def_id":anoncreds::issuer_1_gvt_cred_def_id(),
                            "issuer_did":ISSUER_DID
                        },
                        {
                            "schema_id":"Not Here 2",
                            "schema_name":"Not Here 2"
                        }
                    ])).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_with_vector_of_filters() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!([
                        {
                            "cred_def_id":"Not Here",
                            "issuer_did":"Not Here"
                        },
                        {
                            "cred_def_id":"Not Here 2",
                            "issuer_did":"Not Here 2"
                        }
                    ])).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_with_or_wql() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!({
                            "$or":[
                                { "schema_id":"not here" },
                                { "cred_def_id":anoncreds::issuer_1_gvt_cred_def_id() }
                            ]
                        })).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_with_or_wql() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!({
                            "$or":[
                                { "schema_id":"not here" },
                                { "cred_def_id":"not here" }
                            ]
                        })).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }

        #[test]
        fn verifier_verify_proof_success_with_and_wql() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!({
                            "$and":[
                                { "cred_def_id": anoncreds::issuer_1_gvt_cred_def_id()},
                                { "schema_name":GVT_SCHEMA_NAME }
                            ]
                        })).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}").unwrap();
            assert!(valid);
        }

        #[test]
        fn verifier_verify_proof_fails_with_and_wql() {
            let mut proof_req: ProofRequestPayload = serde_json::from_str(&anoncreds::proof_request_restrictions()).unwrap();
            proof_req.requested_attributes.insert(
                "attr1_referent".to_string(),
                AttributeInfo {
                    name: Some("name".to_string()),
                    names: None,
                    restrictions: serde_json::from_value(json!({
                            "$and":[
                                { "cred_def_id":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:3:CL:CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0:TAG_1" },
                                { "cred_def_id":"Not Here" }
                            ]
                        })).unwrap(),
                    non_revoked: None,
                },
            );
            let valid = anoncreds::verifier_verify_proof(&serde_json::to_string(&proof_req).unwrap(),
                                                         &anoncreds::proof_json_restrictions(),
                                                         &anoncreds::schemas_for_proof_restrictions(),
                                                         &anoncreds::cred_defs_for_proof_restrictions(),
                                                         "{}",
                                                         "{}");
            assert_code!(ErrorCode::AnoncredsProofRejected , valid);
        }
    }
}
