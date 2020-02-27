use indy_api_types::errors::prelude::*;

use crate::domain::anoncreds::credential::AttributeValues;
use crate::domain::anoncreds::proof_request::{AttributeInfo, PredicateInfo, NonRevocedInterval};
use ursa::cl::{issuer, verifier, CredentialSchema, NonCredentialSchema, MasterSecret, CredentialValues, SubProofRequest};

use indy_api_types::domain::crypto::did::DidValue;
use crate::domain::anoncreds::schema::SchemaId;
use crate::domain::anoncreds::credential_definition::CredentialDefinitionId;
use crate::domain::anoncreds::revocation_registry_definition::RevocationRegistryId;
use crate::domain::anoncreds::schema::Schema;
use crate::domain::anoncreds::credential_definition::CredentialDefinition;
use crate::domain::anoncreds::revocation_registry_definition::RevocationRegistryDefinition;
use crate::domain::anoncreds::credential_offer::CredentialOffer;
use crate::domain::anoncreds::credential_request::CredentialRequest;
use crate::domain::anoncreds::proof_request::ProofRequest;

use std::collections::{HashSet, HashMap};

pub fn attr_common_view(attr: &str) -> String {
    attr.replace(" ", "").to_lowercase()
}

pub fn build_credential_schema(attrs: &HashSet<String>) -> IndyResult<CredentialSchema> {
    trace!("build_credential_schema >>> attrs: {:?}", attrs);

    let mut credential_schema_builder = issuer::Issuer::new_credential_schema_builder()?;
    for attr in attrs {
        credential_schema_builder.add_attr(&attr_common_view(attr))?;
    }

    let res = credential_schema_builder.finalize()?;

    trace!("build_credential_schema <<< res: {:?}", res);

    Ok(res)
}

pub fn build_non_credential_schema() -> IndyResult<NonCredentialSchema> {
    trace!("build_non_credential_schema");

    let mut non_credential_schema_builder = issuer::Issuer::new_non_credential_schema_builder()?;
    non_credential_schema_builder.add_attr("master_secret")?;
    let res = non_credential_schema_builder.finalize()?;

    trace!("build_non_credential_schema <<< res: {:?}", res);
    Ok(res)
}

pub fn build_credential_values(credential_values: &HashMap<String, AttributeValues>, master_secret: Option<&MasterSecret>) -> IndyResult<CredentialValues> {
    trace!("build_credential_values >>> credential_values: {:?}", credential_values);

    let mut credential_values_builder = issuer::Issuer::new_credential_values_builder()?;
    for (attr, values) in credential_values {
        credential_values_builder.add_dec_known(&attr_common_view(attr), &values.encoded)?;
    }
    if let Some(ms) = master_secret {
        credential_values_builder.add_value_hidden("master_secret", &ms.value()?)?;
    }

    let res = credential_values_builder.finalize()?;

    trace!("build_credential_values <<< res: {:?}", res);

    Ok(res)
}

pub fn build_sub_proof_request(attrs_for_credential: &[AttributeInfo],
                               predicates_for_credential: &[PredicateInfo]) -> IndyResult<SubProofRequest> {
    trace!("build_sub_proof_request >>> attrs_for_credential: {:?}, predicates_for_credential: {:?}", attrs_for_credential, predicates_for_credential);

    let mut sub_proof_request_builder = verifier::Verifier::new_sub_proof_request_builder()?;

    for attr in attrs_for_credential {
        let names = if let Some(name) = &attr.name {
            vec![name.clone()]
        } else if let Some(names) = &attr.names {
            names.to_owned()
        } else {
            error!(r#"Attr for credential restriction should contain "name" or "names" param. Current attr: {:?}"#, attr);
            return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, r#"Attr for credential restriction should contain "name" or "names" param."#));
        };

        for name in names {
            sub_proof_request_builder.add_revealed_attr(&attr_common_view(&name))?
        }
    }

    for predicate in predicates_for_credential {
        let p_type = format!("{}", predicate.p_type);

        sub_proof_request_builder.add_predicate(&attr_common_view(&predicate.name), &p_type, predicate.p_value)?;
    }

    let res = sub_proof_request_builder.finalize()?;

    trace!("build_sub_proof_request <<< res: {:?}", res);

    Ok(res)
}

pub fn parse_cred_rev_id(cred_rev_id: &str) -> IndyResult<u32> {
    trace!("parse_cred_rev_id >>> cred_rev_id: {:?}", cred_rev_id);

    let res = cred_rev_id.parse::<u32>()
        .to_indy(IndyErrorKind::InvalidStructure, "Cannot parse CredentialRevocationId")?;

    trace!("parse_cred_rev_id <<< res: {:?}", res);

    Ok(res)
}

pub fn get_non_revoc_interval(global_interval: &Option<NonRevocedInterval>, local_interval: &Option<NonRevocedInterval>) -> Option<NonRevocedInterval> {
    trace!("get_non_revoc_interval >>> global_interval: {:?}, local_interval: {:?}", global_interval, local_interval);

    let interval = local_interval.clone().or_else(|| global_interval.clone().or(None));

    trace!("get_non_revoc_interval <<< interval: {:?}", interval);

    interval
}

macro_rules! _id_to_unqualified {
    ($entity:expr, $type_:ident) => ({
        if $entity.starts_with($type_::PREFIX) {
            return Ok($type_($entity.to_string()).to_unqualified().0);
        }
    })
}

macro_rules! _object_to_unqualified {
    ($entity:expr, $type_:ident) => ({
        if let Ok(object) = ::serde_json::from_str::<$type_>(&$entity) {
            return Ok(json!(object.to_unqualified()).to_string())
        }
    })
}

pub fn to_unqualified(entity: &str) -> IndyResult<String> {
    info!("to_unqualified >>> entity: {:?}", entity);

    _id_to_unqualified!(entity, DidValue);
    _id_to_unqualified!(entity, SchemaId);
    _id_to_unqualified!(entity, CredentialDefinitionId);
    _id_to_unqualified!(entity, RevocationRegistryId);

    _object_to_unqualified!(entity, Schema);
    _object_to_unqualified!(entity, CredentialDefinition);
    _object_to_unqualified!(entity, RevocationRegistryDefinition);
    _object_to_unqualified!(entity, CredentialOffer);
    _object_to_unqualified!(entity, CredentialRequest);
    _object_to_unqualified!(entity, ProofRequest);

    Ok(entity.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _interval() -> NonRevocedInterval { NonRevocedInterval { from: None, to: Some(123) } }

    #[test]
    fn get_non_revoc_interval_for_global() {
        let res = get_non_revoc_interval(&Some(_interval()), &None).unwrap();
        assert_eq!(_interval(), res);
    }

    #[test]
    fn get_non_revoc_interval_for_local() {
        let res = get_non_revoc_interval(&None, &Some(_interval())).unwrap();
        assert_eq!(_interval(), res);
    }

    #[test]
    fn get_non_revoc_interval_for_none() {
        let res = get_non_revoc_interval(&None, &None);
        assert_eq!(None, res);
    }

    mod to_unqualified {
        use super::*;

        const DID_QUALIFIED: &str = "did:sov:NcYxiDXkpYi6ov5FcYDi1e";
        const DID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e";
        const SCHEMA_ID_QUALIFIED: &str = "schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
        const SCHEMA_ID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
        const CRED_DEF_ID_QUALIFIED: &str = "creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag";
        const CRED_DEF_ID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag";
        const REV_REG_ID_QUALIFIED: &str = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
        const REV_REG_ID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
        const SCHEMA_ID_WITH_SPACES_QUALIFIED: &str = "schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:Passport Schema:1.0";
        const SCHEMA_ID_WITH_SPACES_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:2:Passport Schema:1.0";

        #[test]
        fn test_to_unqualified() {
            // DID
            assert_eq!(DID_UNQUALIFIED, to_unqualified(DID_QUALIFIED).unwrap());
            assert_eq!(DID_UNQUALIFIED, to_unqualified(DID_UNQUALIFIED).unwrap());

            // SchemaId
            assert_eq!(SCHEMA_ID_UNQUALIFIED, to_unqualified(SCHEMA_ID_QUALIFIED).unwrap());
            assert_eq!(SCHEMA_ID_UNQUALIFIED, to_unqualified(SCHEMA_ID_UNQUALIFIED).unwrap());

            // SchemaId
            assert_eq!(SCHEMA_ID_WITH_SPACES_UNQUALIFIED, to_unqualified(SCHEMA_ID_WITH_SPACES_QUALIFIED).unwrap());
            assert_eq!(SCHEMA_ID_WITH_SPACES_UNQUALIFIED, to_unqualified(SCHEMA_ID_WITH_SPACES_UNQUALIFIED).unwrap());

            // Credential Definition Id
            assert_eq!(CRED_DEF_ID_UNQUALIFIED, to_unqualified(CRED_DEF_ID_QUALIFIED).unwrap());
            assert_eq!(CRED_DEF_ID_UNQUALIFIED, to_unqualified(CRED_DEF_ID_UNQUALIFIED).unwrap());

            // Revocation Registry Id
            assert_eq!(REV_REG_ID_UNQUALIFIED, to_unqualified(REV_REG_ID_QUALIFIED).unwrap());
            assert_eq!(REV_REG_ID_UNQUALIFIED, to_unqualified(REV_REG_ID_UNQUALIFIED).unwrap());
        }
    }
}