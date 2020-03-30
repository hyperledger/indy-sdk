<!-- markdownlint-disable MD033 -->

# Libindy 1.13 to 1.14 migration Guide

This document is written for developers using Libindy to provide necessary information and
to simplify their transition to Libindy 1.13 from Libindy 1.14. If you are using older Libindy
version you can check migration guides history:

* [Libindy 1.3 to 1.4 migration](https://github.com/hyperledger/indy-sdk/blob/v1.4.0/doc/migration-guide.md)
* [Libindy 1.4 to 1.5 migration](https://github.com/hyperledger/indy-sdk/blob/v1.5.0/doc/migration-guide-1.4.0-1.5.0.md)
* [Libindy 1.5 to 1.6 migration](https://github.com/hyperledger/indy-sdk/blob/v1.6.0/doc/migration-guide-1.5.0-1.6.0.md)
* [Libindy 1.6 to 1.7 migration](https://github.com/hyperledger/indy-sdk/blob/v1.7.0/doc/migration-guide-1.6.0-1.7.0.md)
* [Libindy 1.7 to 1.8 migration](https://github.com/hyperledger/indy-sdk/blob/v1.8.0/doc/migration-guide-1.7.0-1.8.0.md)
* [Libindy 1.8 to 1.9 migration](https://github.com/hyperledger/indy-sdk/blob/v1.9.0/docs/migration-guides/migration-guide-1.8.0-1.9.0.md)
* [Libindy 1.9 to 1.10 migration](https://github.com/hyperledger/indy-sdk/blob/v1.10.0/docs/migration-guides/migration-guide-1.9.0-1.10.0.md)
* [Libindy 1.10 to 1.11 migration](https://github.com/hyperledger/indy-sdk/blob/v1.11.0/docs/migration-guides/migration-guide-1.10.0-1.11.0.md)
* [Libindy 1.11 to 1.12 migration](https://github.com/hyperledger/indy-sdk/blob/v1.12.0/docs/migration-guides/migration-guide-1.11.0-1.12.0.md)
* [Libindy 1.12 to 1.13 migration](https://github.com/hyperledger/indy-sdk/blob/v1.13.0/docs/migration-guides/migration-guide-1.12.0-1.13.0.md)

## Table of contents

* [Notes](#notes)
* [Libindy 1.13 to 1.14 migration](#libindy-113-to-114-migration)
    * [Ledger API](#ledger-api)
* [Libindy 1.14.0 to 1.14.1 migration](#libindy-1140-to-1141-migration-guide)
* [Libindy 1.14.1 to 1.14.2 migration](#libindy-1141-to-1142-migration-guide)
* [Libindy 1.14.2 to 1.14.4 migration](#libindy-1142-to-1144-migration-guide)

## Libindy 1.13 to 1.14 migration

#### Ledger API

The v1.14 release contains some changes related to transaction author agreement functionality. 

This changes allow to user to review and accept the TAA in advance of it being written to the ledger. 
Thus when we submit a transaction we can report the real date of meaningful acceptance, 
instead of an arbitrary date engineered to be newer than when the TAA is added.

The TAA could be legally accepted at any point after the TAA is approved by network governance. 

There are two changes related to Libindy Ledger API:
* extended definition of `indy_build_txn_author_agreement_request` to accept new parameters:
    * `ratification_ts` - the date (timestamp) of TAA ratification by network government.
    * `retirement_ts` - the date (timestamp) of TAA retirement.
    
   Please take a look that this breaks API regarding earlier Libindy versions.
      
* added a new function `indy_build_disable_all_txn_author_agreements_request` to build DISABLE_ALL_TXN_AUTHR_AGRMTS request. 
Request to disable all Transaction Author Agreement on the ledger.

More details regarding updated transaction author agreement workflow you can find in this [file](../how-tos/transaction-author-agreement.md).

## Libindy 1.14.0 to 1.14.1 migration Guide

The Libindy 1.14.1 release contains fixes that don't affect API functions. 

## Libindy 1.14.1 to 1.14.2 migration Guide

The Libindy 1.14.1 release contains fixes that don't affect API functions. 

##### Changes
* Updated behavior of `indy_store_their_did` function to allow updating of existing `theirDID` record. 
It can be used to rotate a pairwise key (IS-1166).

    This example works now but threw `WalletItemAlreadyExists` error in previous versions:
    ```
    let identity_json = json!({"did": DID, "verkey": VERKEY}).to_string();
    did::store_their_did(setup.wallet_handle, &identity_json).unwrap();
    
    let identity_json = json!({"did": DID, "verkey": VERKEY_TRUSTEE}).to_string();
    did::store_their_did(setup.wallet_handle, &identity_json).unwrap();
    
    let verkey = did::key_for_local_did(setup.wallet_handle, DID).unwrap();
    assert_eq!(VERKEY_TRUSTEE, verkey);
    ```


* Enhanced validation of `schema_json`: added check that `id` is consistent with `name` and `version` values (IS-1430).
    ``` 
    valid:
    {
        "id":"schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
        "name":"gvt",
        "version":"1.0",
        "attrNames":["aaa","bbb","ccc"],
        "ver":"1.0"
    }
    invalid:     
    {
        "id":"schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
        "name":"other_name",
        "version":"1111.0",
        "attrNames":["aaa","bbb","ccc"],
        "ver":"1.0"
    }
    ```


* Added support of the additional format of `rev_states_json` which is used for proof creation. 
Both `rev_reg_def_id` and `credential_id` can be used as map keys. 
    ```
    1)
    {
      rev_reg_id: {
        timestamp: rev_reg_state,
        ..
      },
      ...
    }
    2)
    { 
      credential_id: {
        timestamp: rev_reg_state,
        ...
      },
     ...
    }
    ```

`credential_id` must be used in case of proving that two credentials matching the same `rev_reg_id` are not revoked at the same timestamp (IS-1447).

* others minor bugfixes

## Libindy 1.14.2 to 1.14.4 migration Guide

The Libindy 1.14.4 release contains fixes that don't affect API functions. 
