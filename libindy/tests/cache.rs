#[macro_use]
mod utils;

inject_indy_dependencies!();

extern crate indyrs as indy;
extern crate indyrs as api;

use crate::utils::cache::*;
use crate::utils::Setup;
use indy_api_types::domain::crypto::did::DidValue;

use self::indy::ErrorCode;

pub const FORBIDDEN_TYPE: &'static str = "Indy::Test";

mod high_cases {
    use super::*;

    mod schema_cache {
        use super::*;
        use crate::utils::domain::anoncreds::schema::{SchemaV1, SchemaId};
        use crate::utils::constants::*;
        use std::thread::sleep;

        #[test]
        fn indy_get_schema_empty_options() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();

            let schema_json = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json).unwrap();

            let _schema: SchemaV1 = serde_json::from_str(&schema_json).unwrap();
        }

        #[test]
        fn indy_get_schema_empty_options_for_unknown_id() {
            let setup = Setup::wallet_and_pool();

            let options_json = json!({}).to_string();

            let res = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                &SchemaId::new(&DidValue(DID.to_string()), "other_schema", "1.0").0,
                &options_json);

            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_schema_only_cache_no_cached_data() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _, _) = utils::ledger::post_entities();

            let options_json = json!({"noUpdate": true}).to_string();

            let res = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json);

            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_schema_cache_works() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();
            let schema_json1 = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            ).unwrap();
            let _schema: SchemaV1 = serde_json::from_str(&schema_json1).unwrap();

            // now retrieve it from cache
            let options_json = json!({"noUpdate": true}).to_string();
            let schema_json2 = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            ).unwrap();
            let _schema: SchemaV1 = serde_json::from_str(&schema_json2).unwrap();

            assert_eq!(schema_json1, schema_json2);
        }

        #[test]
        fn indy_get_schema_no_store_works() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _, _) = utils::ledger::post_entities();

            let options_json = json!({"noStore": true}).to_string();
            let schema_json1 = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            ).unwrap();
            let _schema: SchemaV1 = serde_json::from_str(&schema_json1).unwrap();

            // it should not be present inside of cache, because of noStore option in previous request.
            let options_json = json!({"noUpdate": true}).to_string();
            let res = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            );
            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_schema_no_cache_works() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();
            let schema_json1 = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            ).unwrap();
            let _schema: SchemaV1 = serde_json::from_str(&schema_json1).unwrap();

            // it should not be present inside of cache, because of noStore option in previous request.
            let options_json = json!({"noUpdate": true, "noCache": true}).to_string();
            let res = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            );
            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_schema_fully_qualified_ids() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _) = utils::ledger::post_qualified_entities();

            let options_json = json!({}).to_string();

            let schema_json = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1_V1,
                &schema_id,
                &options_json).unwrap();

            let schema: SchemaV1 = serde_json::from_str(&schema_json).unwrap();
            assert_eq!(schema_id, schema.id.0);
        }

        #[test]
        fn indy_get_schema_min_fresh_works() {
            let setup = Setup::wallet_and_pool();

            let (schema_id, _, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();
            let schema_json1 = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            ).unwrap();
            let _schema: SchemaV1 = serde_json::from_str(&schema_json1).unwrap();

            sleep(std::time::Duration::from_secs(2));

            // it should not be present inside of cache, because of noStore option in previous request.
            let options_json = json!({"noUpdate": true, "minFresh": 1}).to_string();
            let res = get_schema_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                schema_id,
                &options_json
            );
            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_purge_schema_cache_no_options() {
            let setup = Setup::wallet();
            purge_schema_cache(setup.wallet_handle, "{}").unwrap();
        }

        #[test]
        fn indy_purge_schema_cache_all_data() {
            let setup = Setup::wallet();
            purge_schema_cache(setup.wallet_handle, &json!({"minFresh": -1}).to_string()).unwrap();
        }

        #[test]
        fn indy_purge_schema_cache_older_than_1000_seconds() {
            let setup = Setup::wallet();
            purge_schema_cache(setup.wallet_handle, &json!({"minFresh": 1000}).to_string()).unwrap();
        }
    }

    mod cred_def_cache {
        use super::*;
        use crate::utils::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionV1};
        use crate::utils::constants::*;
        use std::thread::sleep;


        #[test]
        fn indy_get_cred_def_empty_options() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();

            let cred_def_json = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json).unwrap();

            let _cred_def: CredentialDefinition = serde_json::from_str(&cred_def_json).unwrap();
        }

        #[test]
        fn indy_get_cred_def_only_cache_no_cached_data() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id, _) = utils::ledger::post_entities();

            let options_json = json!({"noUpdate": true}).to_string();

            let res = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json);

            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_cred_def_cache_works() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();
            let cred_def_json1 = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            ).unwrap();
            let _cred_def: CredentialDefinition = serde_json::from_str(&cred_def_json1).unwrap();

            // now retrieve it from cache
            let options_json = json!({"noUpdate": true}).to_string();
            let cred_def_json2 = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            ).unwrap();
            let _cred_def: CredentialDefinition = serde_json::from_str(&cred_def_json2).unwrap();

            assert_eq!(cred_def_json1, cred_def_json2);
        }

        #[test]
        fn indy_get_cred_def_no_store_works() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id, _) = utils::ledger::post_entities();

            let options_json = json!({"noStore": true}).to_string();
            let cred_def_json1 = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            ).unwrap();
            let _cred_def: CredentialDefinition = serde_json::from_str(&cred_def_json1).unwrap();

            // it should not be present inside of cache, because of noStore option in previous request.
            let options_json = json!({"noUpdate": true}).to_string();
            let res = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            );
            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_cred_def_no_cache_works() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();
            let cred_def_json1 = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            ).unwrap();
            let _cred_def: CredentialDefinition = serde_json::from_str(&cred_def_json1).unwrap();

            // it should not be present inside of cache, because of noStore option in previous request.
            let options_json = json!({"noUpdate": true, "noCache": true}).to_string();
            let res = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            );
            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_cred_def_min_fresh_works() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id, _) = utils::ledger::post_entities();

            let options_json = json!({}).to_string();
            let cred_def_json1 = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            ).unwrap();
            let _cred_def: CredentialDefinition = serde_json::from_str(&cred_def_json1).unwrap();

            sleep(std::time::Duration::from_secs(2));

            // it should not be present inside of cache, because of noStore option in previous request.
            let options_json = json!({"noUpdate": true, "minFresh": 1}).to_string();
            let res = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1,
                cred_def_id,
                &options_json
            );
            assert_code!(ErrorCode::LedgerNotFound, res);
        }

        #[test]
        fn indy_get_cred_def_fully_qualified_ids() {
            let setup = Setup::wallet_and_pool();

            let (_, cred_def_id) = utils::ledger::post_qualified_entities();

            let options_json = json!({}).to_string();

            let cred_def_json = get_cred_def_cache(
                setup.pool_handle,
                setup.wallet_handle,
                DID_MY1_V1,
                &cred_def_id,
                &options_json).unwrap();

            let cred_def: CredentialDefinitionV1 = serde_json::from_str(&cred_def_json).unwrap();
            assert_eq!(cred_def_id, cred_def.id.0);

        }

        #[test]
        fn indy_purge_cred_def_cache_no_options() {
            let setup = Setup::wallet();
            purge_cred_def_cache(setup.wallet_handle, "{}").unwrap();
        }

        #[test]
        fn indy_purge_cred_def_cache_all_data() {
            let setup = Setup::wallet();
            purge_cred_def_cache(setup.wallet_handle, &json!({"minFresh": -1}).to_string()).unwrap();
        }

        #[test]
        fn indy_purge_cred_def_cache_older_than_1000_seconds() {
            let setup = Setup::wallet();
            purge_cred_def_cache(setup.wallet_handle, &json!({"minFresh": 1000}).to_string()).unwrap();
        }
    }
}