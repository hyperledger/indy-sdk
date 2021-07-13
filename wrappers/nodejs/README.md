# Indy SDK for Node.js

[![stability - experimental](https://img.shields.io/badge/stability-experimental-orange.svg)](https://nodejs.org/api/documentation.html#documentation_stability_index)
![Node version](https://img.shields.io/node/v/indy-sdk.svg)

Native bindings for [Hyperledger Indy](https://www.hyperledger.org/projects/hyperledger-indy).

- [Installing](#installing)
- [Usage](#usage)
- [API](#api)
  * [IndyError](#indyerror)
  * [anoncreds](#anoncreds)
  * [blob_storage](#blob_storage)
  * [crypto](#crypto)
  * [did](#did)
  * [ledger](#ledger)
  * [non_secrets](#non_secrets)
  * [pairwise](#pairwise)
  * [payment](#payment)
  * [pool](#pool)
  * [wallet](#wallet)
  * [logger](#logger)
  * [cache](#cache)
  * [mod](#mod)
- [Advanced](#advanced)
- [Contributing](#contributing)

## Installing

This module has a native compile step. It compiles C++ code and dynamically links to `libindy`.

You will need:

* C++ build tools and Python 3.6+. See [this](https://github.com/nodejs/node-gyp#installation) for platform recommendations.
* `libindy` v1.6+ in your system library path. (i.e. `/usr/lib/libindy.so` for linux)

Then you can install via npm:

```sh
npm install --save indy-sdk
```

#### Troubleshooting
Use environment variable `RUST_LOG={info|debug|trace}` to output logs of Libindy.

##### Linking errors

i.e. `ld: library not found for -llibindy`

First, make sure you have the latest libindy for your platform. Also make sure you have any other libraries it depends on. See [indy-sdk/doc](https://github.com/hyperledger/indy-sdk/tree/master/doc)

Second, make sure it's in the linker search path. The easiest way is to use the system library path.
* ubuntu `/usr/lib/libindy.so`
* osx `/usr/local/lib/libindy.dylib`
* windows use LD_LIBRARY_PATH to indicate the location of dll as specified below

If you want to put the library in a custom folder i.e. `/foo/bar/libindy.so` then you can do this:
```sh
LD_LIBRARY_PATH=/foo/bar npm i --save indy-sdk
```
Then when you run your code, you'll still need the `LD_LIBRARY_PATH` set.
```sh
LD_LIBRARY_PATH=/foo/bar node index.js
```

##### Other build errors

We use [node-gyp](https://github.com/nodejs/node-gyp#installation) to manage the cross-platform build. Their readme is quite helpful.

## Usage

```js
var indy = require('indy-sdk')

var did = '...'
var fullVerkey = '...'

indy.abbreviateVerkey(did, fullVerkey, function(err, verkey){
  ..
})

// if you do not provide a callback, a Promise is returned

var verkey = await indy.abbreviateVerkey(did, fullVerkey)
```

# API

### IndyError

All the functions may yield an IndyError. The errors are based on libindy error codes defined [here](https://github.com/hyperledger/indy-sdk/blob/master/libindy/include/indy_mod.h).

* `err.indyCode`: Int - code number from libindy
* `err.indyName`: String - name for the error code
* `err.indyMessage`: String - human-readable error description
* `err.indyBacktrace`: String? - if enabled, this is the libindy backtrace string

Collecting of backtrace can be enabled by:
1. Setting environment variable `RUST_BACKTRACE=1`
2. Calling [setRuntimeConfig](#setruntimeconfig--config-)(`{collect_backtrace: true}`)

### anoncreds

These functions wrap the Ursa algorithm as documented in this [paper](https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf):

And is documented in this [HIPE](https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md):

#### issuerCreateSchema \( issuerDid, name, version, attrNames \) -&gt; \[ id, schema \]

Create credential schema entity that describes credential attributes list and allows credentials
interoperability.

Schema is public and intended to be shared with all anoncreds workflow actors usually by publishing SCHEMA transaction
to Indy distributed ledger.

It is IMPORTANT for current version POST Schema in Ledger and after that GET it from Ledger
with correct seq\_no to save compatibility with Ledger.
After that can call issuerCreateAndStoreCredentialDef to build corresponding Credential Definition.

* `issuerDid`: String - DID of schema issuer
* `name`: String - a name the schema
* `version`: String - a version of the schema
* `attrNames`: Json - a list of schema attributes descriptions (the number of attributes should be less or equal than 125)
* __->__ [ `id`: String, `schema`: Json ] - schema\_id: identifier of created schema
schema\_json: schema as json

Errors: `Common*`, `Anoncreds*`

#### issuerCreateAndStoreCredentialDef \( wh, issuerDid, schema, tag, signatureType, config \) -&gt; \[ credDefId, credDef \]

Create credential definition entity that encapsulates credentials issuer DID, credential schema, secrets used for signing credentials
and secrets used for credentials revocation.

Credential definition entity contains private and public parts. Private part will be stored in the wallet. Public part
will be returned as json intended to be shared with all anoncreds workflow actors usually by publishing CRED\_DEF transaction
to Indy distributed ledger.

It is IMPORTANT for current version GET Schema from Ledger with correct seq\_no to save compatibility with Ledger.

Note: Use combination of `issuerRotateCredentialDefStart` and `indy_issuer_rotate_credential_def_apply` functions
to generate new keys for an existing credential definition.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `issuerDid`: String - a DID of the issuer signing cred\_def transaction to the Ledger
* `schema`: Json - credential schema as a json
* `tag`: String - allows to distinct between credential definitions for the same issuer and schema
* `signatureType`: String - credential definition type \(optional, 'CL' by default\) that defines credentials signature and revocation math. Supported types are:
  *  'CL': Camenisch-Lysyanskaya credential signature type that is implemented according to the algorithm in this paper:
                https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
           And is documented in this HIPE:
               https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md

* `config`: Json - \(optional\) type-specific configuration of credential definition as json:
  *  'CL':
    *  support\_revocation: whether to request non-revocation credential \(optional, default false\)
* __->__ [ `credDefId`: String, `credDef`: Json ] - cred\_def\_id: identifier of created credential definition
cred\_def\_json: public part of created credential definition

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### issuerRotateCredentialDefStart \( wh, credDefId, config \) -&gt; credDef

 Generate temporary credential definitional keys for an existing one (owned by the caller of the library).
 
 Use `issuerRotateCredentialDefApply` function to set temporary keys as the main.
 
 **WARNING**: Rotating the credential definitional keys will result in making all credentials issued under the previous keys unverifiable.
 
* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `credDefId`: String - an identifier of created credential definition stored in the wallet
* `config`: Json - \(optional\) type-specific configuration of credential definition as json:
  *  'CL':
    *  support\_revocation: whether to request non-revocation credential \(optional, default false\)
* __->__  `credDef`: Json - public part of temporary created credential definition

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### issuerRotateCredentialDefApply \( wh, credDefId \) -&gt; void

 Apply temporary keys as main for an existing Credential Definition (owned by the caller of the library).
 
 **WARNING**: Rotating the credential definitional keys will result in making all credentials issued under the previous keys unverifiable.
 
* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `credDefId`: String - an identifier of created credential definition stored in the wallet
* __->__  void

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### issuerCreateAndStoreRevocReg \( wh, issuerDid, revocDefType, tag, credDefId, config, tailsWriterHandle \) -&gt; \[ revocRegId, revocRegDef, revocRegEntry \]

Create a new revocation registry for the given credential definition as tuple of entities
- Revocation registry definition that encapsulates credentials definition reference, revocation type specific configuration and
secrets used for credentials revocation
- Revocation registry state that stores the information about revoked entities in a non-disclosing way. The state can be
represented as ordered list of revocation registry entries were each entry represents the list of revocation or issuance operations.

Revocation registry definition entity contains private and public parts. Private part will be stored in the wallet. Public part
will be returned as json intended to be shared with all anoncreds workflow actors usually by publishing REVOC\_REG\_DEF transaction
to Indy distributed ledger.

Revocation registry state is stored on the wallet and also intended to be shared as the ordered list of REVOC\_REG\_ENTRY transactions.
This call initializes the state in the wallet and returns the initial entry.

Some revocation registry types \(for example, 'CL\_ACCUM'\) can require generation of binary blob called tails used to hide information about revoked credentials in public
revocation registry and intended to be distributed out of leger \(REVOC\_REG\_DEF transaction will still contain uri and hash of tails\).
This call requires access to pre-configured blob storage writer instance handle that will allow to write generated tails.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `issuerDid`: String - a DID of the issuer signing transaction to the Ledger
* `revocDefType`: String - revocation registry type \(optional, default value depends on credential definition type\). Supported types are:
  *  'CL\_ACCUM': Type-3 pairing based accumulator implemented according to the algorithm in this paper:
                    https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
                  This type is default for 'CL' credential definition type.
* `tag`: String - allows to distinct between revocation registries for the same issuer and credential definition
* `credDefId`: String - id of stored in ledger credential definition
* `config`: Json - type-specific configuration of revocation registry as json:
  *  'CL\_ACCUM':
```
{
    "issuance_type": (optional) type of issuance. Currently supported:
        1) ISSUANCE_BY_DEFAULT: all indices are assumed to be issued and initial accumulator is calculated over all indices;
           Revocation Registry is updated only during revocation.
        2) ISSUANCE_ON_DEMAND: nothing is issued initially accumulator is 1 (used by default);
    "max_cred_num": maximum number of credentials the new registry can process (optional, default 100000)
}
````
* `tailsWriterHandle`: Handle (Number) - handle of blob storage to store tails 

NOTE:
Recursive creation of folder for Default Tails Writer (correspondent to `tailsWriterHandle`)
in the system-wide temporary directory may fail in some setup due to permissions: `IO error: Permission denied`.
In this case use `TMPDIR` environment variable to define temporary directory specific for an application.

* __->__ [ `revocRegId`: String, `revocRegDef`: Json, `revocRegEntry`: Json ] - revoc\_reg\_id: identifier of created revocation registry definition
revoc\_reg\_def\_json: public part of revocation registry definition
revoc\_reg\_entry\_json: revocation registry entry that defines initial state of revocation registry

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### issuerCreateCredentialOffer \( wh, credDefId \) -&gt; credOffer

Create credential offer that will be used by Prover for
credential request creation. Offer includes nonce and key correctness proof
for authentication between protocol steps and integrity checking.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `credDefId`: String - id of credential definition stored in the wallet
* __->__ `credOffer`: Json - credential offer json:
```
    {
        "schema_id": string,
        "cred_def_id": string,
        // Fields below can depend on Cred Def type
        "nonce": string,
        "key_correctness_proof" : key correctness proof for credential definition correspondent to cred_def_id
                                  (opaque type that contains data structures internal to Ursa.
                                  It should not be parsed and are likely to change in future versions).
    }
````

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### issuerCreateCredential \( wh, credOffer, credReq, credValues, revRegId, blobStorageReaderHandle \) -&gt; \[ cred, credRevocId, revocRegDelta \]

Check Cred Request for the given Cred Offer and issue Credential for the given Cred Request.

Cred Request must match Cred Offer. The credential definition and revocation registry definition
referenced in Cred Offer and Cred Request must be already created and stored into the wallet.

Information for this credential revocation will be store in the wallet as part of revocation registry under
generated cred\_revoc\_id local for this wallet.

This call returns revoc registry delta as json file intended to be shared as REVOC\_REG\_ENTRY transaction.
Note that it is possible to accumulate deltas to reduce ledger load.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `credOffer`: Json - a cred offer created by issuerCreateCredentialOffer
* `credReq`: Json - a credential request created by proverCreateCredentialReq
* `credValues`: Json - a credential containing attribute values for each of requested attribute names.
Example:
```
    {
     "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
     "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
    }
````
  If you want to use empty value for some credential field, you should set "raw" to "" and "encoded" should not be empty
* `revRegId`: String - id of revocation registry stored in the wallet
* `blobStorageReaderHandle`: Handle (Number) - configuration of blob storage reader handle that will allow to read revocation tails
* __->__ [ `cred`: Json, `credRevocId`: String, `revocRegDelta`: Json ] - cred\_json: Credential json containing signed credential values
```
    {
        "schema_id": string,
        "cred_def_id": string,
        "rev_reg_def_id", Optional<string>,
        "values": <see cred_values_json above>,
        // Fields below can depend on Cred Def type
        "signature": <credential signature>,
                     (opaque type that contains data structures internal to Ursa.
                     It should not be parsed and are likely to change in future versions).
        "signature_correctness_proof": <signature_correctness_proof>
                                       (opaque type that contains data structures internal to Ursa.
                                        It should not be parsed and are likely to change in future versions).
    }
cred_revoc_id: local id for revocation info (Can be used for revocation of this credential)
revoc_reg_delta_json: Revocation registry delta json with a newly issued credential
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### issuerRevokeCredential \( wh, blobStorageReaderHandle, revRegId, credRevocId \) -&gt; revocRegDelta

Revoke a credential identified by a cred\_revoc\_id \(returned by issuerCreateCredential\).

The corresponding credential definition and revocation registry must be already
created an stored into the wallet.

This call returns revoc registry delta as json file intended to be shared as REVOC\_REG\_ENTRY transaction.
Note that it is possible to accumulate deltas to reduce ledger load.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `blobStorageReaderHandle`: Handle (Number)
* `revRegId`: String - id of revocation registry stored in wallet
* `credRevocId`: String - local id for revocation info
* __->__ `revocRegDelta`: Json - revoc\_reg\_delta\_json: Revocation registry delta json with a revoked credential

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### issuerMergeRevocationRegistryDeltas \( revRegDelta, otherRevRegDelta \) -&gt; mergedRevRegDelta

Merge two revocation registry deltas \(returned by issuerCreateCredential or issuerRevokeCredential\) to accumulate common delta.
Send common delta to ledger to reduce the load.

* `revRegDelta`: Json - revocation registry delta.
* `otherRevRegDelta`: Json - revocation registry delta for which PrevAccum value  is equal to current accum value of rev\_reg\_delta\_json.
* __->__ `mergedRevRegDelta`: Json - merged\_rev\_reg\_delta: Merged revocation registry delta

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverCreateMasterSecret \( wh, masterSecretId \) -&gt; outMasterSecretId

Creates a master secret with a given id and stores it in the wallet.
The id must be unique.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `masterSecretId`: String - \(optional, if not present random one will be generated\) new master id
* __->__ `outMasterSecretId`: String - out\_master\_secret\_id: Id of generated master secret

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverCreateCredentialReq \( wh, proverDid, credOffer, credDef, masterSecretId \) -&gt; \[ credReq, credReqMetadata \]

Creates a credential request for the given credential offer.

The method creates a blinded master secret for a master secret identified by a provided name.
The master secret identified by the name must be already stored in the secure wallet \(see prover\_create\_master\_secret\)
The blinded master secret is a part of the credential request.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `proverDid`: String - a DID of the prover
* `credOffer`: Json - credential offer as a json containing information about the issuer and a credential
* `credDef`: Json - credential definition json related to &lt;cred\_def\_id&gt; in &lt;cred\_offer\_json&gt;
* `masterSecretId`: String - the id of the master secret stored in the wallet
* __->__ [ `credReq`: Json, `credReqMetadata`: Json ] - cred\_req\_json: Credential request json for creation of credential by Issuer
```
    {
     "prover_did" : string,
     "cred_def_id" : string,
        // Fields below can depend on Cred Def type
     "blinded_ms" : <blinded_master_secret>,
                   (opaque type that contains data structures internal to Ursa.
                    It should not be parsed and are likely to change in future versions).
     "blinded_ms_correctness_proof" : <blinded_ms_correctness_proof>,
                                       (opaque type that contains data structures internal to Ursa.
                                        It should not be parsed and are likely to change in future versions).
     "nonce": string
   }
cred_req_metadata_json: Credential request metadata json for further processing of received form Issuer credential.
    Note: cred_req_metadata_json mustn't be shared with Issuer.
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverStoreCredential \( wh, credId, credReqMetadata, cred, credDef, revRegDef \) -&gt; outCredId

Check credential provided by Issuer for the given credential request,
updates the credential by a master secret and stores in a secure wallet.

To support efficient and flexible search the following tags will be created for stored credential:
```
    {
        "schema_id": <credential schema id>,
        "schema_issuer_did": <credential schema issuer did>,
        "schema_name": <credential schema name>,
        "schema_version": <credential schema version>,
        "issuer_did": <credential issuer did>,
        "cred_def_id": <credential definition id>,
        "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
        // for every attribute in <credential values>
        "attr::<attribute name>::marker": "1",
        "attr::<attribute name>::value": <attribute raw value>,
    }
````

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `credId`: String - \(optional, default is a random one\) identifier by which credential will be stored in the wallet
* `credReqMetadata`: Json - a credential request metadata created by proverCreateCredentialReq
* `cred`: Json - credential json received from issuer
* `credDef`: Json - credential definition json related to &lt;cred\_def\_id&gt; in &lt;cred\_json&gt;
* `revRegDef`: Json - revocation registry definition json related to &lt;rev\_reg\_def\_id&gt; in &lt;cred\_json&gt;
* __->__ `outCredId`: String - out\_cred\_id: identifier by which credential is stored in the wallet

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverGetCredentials \( wh, filter \) -&gt; credentials

Gets human readable credentials according to the filter.
If filter is NULL, then all credentials are returned.
Credentials can be filtered by Issuer, credential\_def and\/or Schema.

NOTE: This method is deprecated because immediately returns all fetched credentials.
Use &lt;proverSearchCredentials&gt; to fetch records by small batches.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `filter`: Json - filter for credentials
```
       {
           "schema_id": string, (Optional)
           "schema_issuer_did": string, (Optional)
           "schema_name": string, (Optional)
           "schema_version": string, (Optional)
           "issuer_did": string, (Optional)
           "cred_def_id": string, (Optional)
       }
````
* __->__ `credentials`: Json - credentials json
```
    [{
        "referent": string, // cred_id in the wallet
        "attrs": {"key1":"raw_value1", "key2":"raw_value2"},
        "schema_id": string,
        "cred_def_id": string,
        "rev_reg_id": Optional<string>,
        "cred_rev_id": Optional<string>
    }]
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverGetCredential \( wh, credId \) -&gt; credential

Gets human readable credential by the given id.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `credId`: String - Identifier by which requested credential is stored in the wallet
* __->__ `credential`: Json - credential json:
```
    {
        "referent": string, // cred_id in the wallet
        "attrs": {"key1":"raw_value1", "key2":"raw_value2"},
        "schema_id": string,
        "cred_def_id": string,
        "rev_reg_id": Optional<string>,
        "cred_rev_id": Optional<string>
    }
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverSearchCredentials \( wh, query \) -&gt; \[ sh, totalCount \]

Search for credentials stored in wallet.
Credentials can be filtered by tags created during saving of credential.

Instead of immediately returning of fetched credentials
this call returns search\_handle that can be used later
to fetch records by small batches \(with proverFetchCredentials\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `query`: Json - Wql query filter for credentials searching based on tags.
where query: indy-sdk\/doc\/design\/011-wallet-query-language\/README.md
* __->__ [ `sh`: Handle (Number), `totalCount`: Number ] - search\_handle: Search handle that can be used later to fetch records by small batches \(with proverFetchCredentials\)
total\_count: Total count of records

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverFetchCredentials \( sh, count \) -&gt; credentials

Fetch next credentials for search.

* `sh`: Handle (Number) - Search handle \(created by proverSearchCredentials\)
* `count`: Number - Count of credentials to fetch
* __->__ `credentials`: Json - credentials\_json: List of human readable credentials:
```
    [{
        "referent": string, // cred_id in the wallet
        "attrs": {"key1":"raw_value1", "key2":"raw_value2"},
        "schema_id": string,
        "cred_def_id": string,
        "rev_reg_id": Optional<string>,
        "cred_rev_id": Optional<string>
    }]
NOTE: The list of length less than the requested count means credentials search iterator is completed.
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverCloseCredentialsSearch \( sh \) -&gt; void

Close credentials search \(make search handle invalid\)

* `sh`: Handle (Number) - Search handle \(created by proverSearchCredentials\)
* __->__ void

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverGetCredentialsForProofReq \( wh, proofRequest \) -&gt; credentials

Gets human readable credentials matching the given proof request.

NOTE: This method is deprecated because immediately returns all fetched credentials.
Use &lt;proverSearchCredentialsForProofReq&gt; to fetch records by small batches.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `proofRequest`: Json - proof request json
```
    {
        "name": string,
        "version": string,
        "nonce": string, - a decimal number represented as a string (use `generateNonce` function to generate 80-bit number)
        "requested_attributes": { // set of requested attributes
             "<attr_referent>": <attr_info>, // see below
             ...,
        },
        "requested_predicates": { // set of requested predicates
             "<predicate_referent>": <predicate_info>, // see below
             ...,
         },
        "non_revoked": Optional<<non_revoc_interval>>, // see below,
                       // If specified prover must proof non-revocation
                       // for date in this interval for each attribute
                       // (can be overridden on attribute level)
        "ver": Optional<str>  - proof request version:
            - omit to use unqualified identifiers for restrictions
            - "1.0" to use unqualified identifiers for restrictions
            - "2.0" to use fully qualified identifiers for restrictions
    }
where:
 attr_info: Describes requested attribute
     {
         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
                                              // NOTE: should either be "name" or "names", not both and not none of them.
                                              // Use "names" to specify several attributes that have to match a single credential.
         "restrictions": Optional<wql query>, // see below
         "non_revoked": Optional<<non_revoc_interval>>, // see below,
                        // If specified prover must proof non-revocation
                        // for date in this interval this attribute
                        // (overrides proof level interval)
     }
 predicate_referent: Proof-request local identifier of requested attribute predicate
 predicate_info: Describes requested attribute predicate
     {
         "name": attribute name, (case insensitive and ignore spaces)
         "p_type": predicate type (">=", ">", "<=", "<")
         "p_value": predicate value
         "restrictions": Optional<wql query>, // see below
         "non_revoked": Optional<<non_revoc_interval>>, // see below,
                        // If specified prover must proof non-revocation
                        // for date in this interval this attribute
                        // (overrides proof level interval)
     }
 non_revoc_interval: Defines non-revocation interval
     {
         "from": Optional<int>, // timestamp of interval beginning
         "to": Optional<int>, // timestamp of interval ending
     }
````
* __->__ `credentials`: Json - credentials\_json: json with credentials for the given proof request.
```
    {
        "requested_attrs": {
            "<attr_referent>": [{ cred_info: <credential_info>, interval: Optional<non_revoc_interval> }],
            ...,
        },
        "requested_predicates": {
            "requested_predicates": [{ cred_info: <credential_info>, timestamp: Optional<integer> }, { cred_info: <credential_2_info>, timestamp: Optional<integer> }],
            "requested_predicate_2_referent": [{ cred_info: <credential_2_info>, timestamp: Optional<integer> }]
        }
    }, where credential is
    {
        "referent": <string>,
        "attrs": {"attr_name" : "attr_raw_value"},
        "schema_id": string,
        "cred_def_id": string,
        "rev_reg_id": Optional<int>,
        "cred_rev_id": Optional<int>,
    }
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverSearchCredentialsForProofReq \( wh, proofRequest, extraQuery \) -&gt; sh

Search for credentials matching the given proof request.

Instead of immediately returning of fetched credentials
this call returns search\_handle that can be used later
to fetch records by small batches \(with proverFetchCredentialsForProofReq\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `proofRequest`: Json - proof request json
```
    {
        "name": string,
        "version": string,
        "nonce": string, - a decimal number represented as a string (use `generateNonce` function to generate 80-bit number)
        "requested_attributes": { // set of requested attributes
             "<attr_referent>": <attr_info>, // see below
             ...,
        },
        "requested_predicates": { // set of requested predicates
             "<predicate_referent>": <predicate_info>, // see below
             ...,
         },
        "non_revoked": Optional<<non_revoc_interval>>, // see below,
                       // If specified prover must proof non-revocation
                       // for date in this interval for each attribute
                       // (can be overridden on attribute level)
        "ver": Optional<str>  - proof request version:
            - omit to use unqualified identifiers for restrictions
            - "1.0" to use unqualified identifiers for restrictions
            - "2.0" to use fully qualified identifiers for restrictions
    }
where:
 attr_info: Describes requested attribute
     {
         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
                                              // NOTE: should either be "name" or "names", not both and not none of them.
                                              // Use "names" to specify several attributes that have to match a single credential.
         "restrictions": Optional<wql query>, // see below
         "non_revoked": Optional<<non_revoc_interval>>, // see below,
                        // If specified prover must proof non-revocation
                        // for date in this interval this attribute
                        // (overrides proof level interval)
     }
 predicate_referent: Proof-request local identifier of requested attribute predicate
 predicate_info: Describes requested attribute predicate
     {
         "name": attribute name, (case insensitive and ignore spaces)
         "p_type": predicate type (">=", ">", "<=", "<")
         "p_value": predicate value
         "restrictions": Optional<wql query>, // see below
         "non_revoked": Optional<<non_revoc_interval>>, // see below,
                        // If specified prover must proof non-revocation
                        // for date in this interval this attribute
                        // (overrides proof level interval)
     }
 non_revoc_interval: Defines non-revocation interval
     {
         "from": Optional<int>, // timestamp of interval beginning
         "to": Optional<int>, // timestamp of interval ending
     }
````
* `extraQuery`: Json - \(Optional\) List of extra queries that will be applied to correspondent attribute\/predicate:
```
    {
        "<attr_referent>": <wql query>,
        "<predicate_referent>": <wql query>,
    }
where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
````
* __->__ `sh`: Handle (Number) - search\_handle: Search handle that can be used later to fetch records by small batches \(with proverFetchCredentialsForProofReq\)

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverFetchCredentialsForProofReq \( sh, itemReferent, count \) -&gt; credentials

Fetch next credentials for the requested item using proof request search
handle \(created by proverSearchCredentialsForProofReq\).

* `sh`: Handle (Number) - Search handle \(created by proverSearchCredentialsForProofReq\)
* `itemReferent`: String - Referent of attribute\/predicate in the proof request
* `count`: Number - Count of credentials to fetch
* __->__ `credentials`: Json - credentials\_json: List of credentials for the given proof request.
```
    [{
        cred_info: <credential_info>,
        interval: Optional<non_revoc_interval>
    }]
where
credential_info:
    {
        "referent": <string>,
        "attrs": {"attr_name" : "attr_raw_value"},
        "schema_id": string,
        "cred_def_id": string,
        "rev_reg_id": Optional<int>,
        "cred_rev_id": Optional<int>,
    }
non_revoc_interval:
    {
        "from": Optional<int>, // timestamp of interval beginning
        "to": Optional<int>, // timestamp of interval ending
    }
NOTE: The list of length less than the requested count means that search iterator
correspondent to the requested <item_referent> is completed.
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverCloseCredentialsSearchForProofReq \( sh \) -&gt; void

Close credentials search for proof request \(make search handle invalid\)

* `sh`: Handle (Number) - Search handle \(created by proverSearchCredentialsForProofReq\)
* __->__ void

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### proverCreateProof \( wh, proofReq, requestedCredentials, masterSecretName, schemas, credentialDefs, revStates \) -&gt; proof

Creates a proof according to the given proof request
Either a corresponding credential with optionally revealed attributes or self-attested attribute must be provided
for each requested attribute \(see proverGetCredentials\_for\_pool\_req\).
A proof request may request multiple credentials from different schemas and different issuers.
All required schemas, public keys and revocation registries must be provided.
The proof request also contains nonce.
The proof contains either proof or self-attested attribute value for each requested attribute.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `proofReq`: Json - proof request json
```
  {
      "name": string,
      "version": string,
      "nonce": string, - a decimal number represented as a string (use `generateNonce` function to generate 80-bit number)
      "requested_attributes": { // set of requested attributes
           "<attr_referent>": <attr_info>, // see below
           ...,
      },
      "requested_predicates": { // set of requested predicates
           "<predicate_referent>": <predicate_info>, // see below
           ...,
       },
      "non_revoked": Optional<<non_revoc_interval>>, // see below,
                     // If specified prover must proof non-revocation
                     // for date in this interval for each attribute
                     // (can be overridden on attribute level)
      "ver": Optional<str>  - proof request version:
          - omit to use unqualified identifiers for restrictions
          - "1.0" to use unqualified identifiers for restrictions
          - "2.0" to use fully qualified identifiers for restrictions
  }
where:
 attr_info: Describes requested attribute
     {
         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
                                              // NOTE: should either be "name" or "names", not both and not none of them.
                                              // Use "names" to specify several attributes that have to match a single credential.
         "restrictions": Optional<wql query>, // see below
         "non_revoked": Optional<<non_revoc_interval>>, // see below,
                        // If specified prover must proof non-revocation
                        // for date in this interval this attribute
                        // (overrides proof level interval)
     }
 predicate_referent: Proof-request local identifier of requested attribute predicate
 predicate_info: Describes requested attribute predicate
     {
         "name": attribute name, (case insensitive and ignore spaces)
         "p_type": predicate type (">=", ">", "<=", "<")
         "p_value": predicate value
         "restrictions": Optional<wql query>, // see below
         "non_revoked": Optional<<non_revoc_interval>>, // see below,
                        // If specified prover must proof non-revocation
                        // for date in this interval this attribute
                        // (overrides proof level interval)
     }
 non_revoc_interval: Defines non-revocation interval
     {
         "from": Optional<int>, // timestamp of interval beginning
         "to": Optional<int>, // timestamp of interval ending
     }
````
* `requestedCredentials`: Json - either a credential or self-attested attribute for each requested attribute
```
    {
        "self_attested_attributes": {
            "self_attested_attribute_referent": string
        },
        "requested_attributes": {
            "requested_attribute_referent_1": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }},
            "requested_attribute_referent_2": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }}
        },
        "requested_predicates": {
            "requested_predicates_referent_1": {"cred_id": string, "timestamp": Optional<number> }},
        }
    }
````
* `masterSecretName`: String
* `schemas`: Json - all schemas json participating in the proof request
```
    {
        <schema1_id>: <schema1_json>,
        <schema2_id>: <schema2_json>,
        <schema3_id>: <schema3_json>,
    }
````
* `credentialDefs`: Json - all credential definitions json participating in the proof request
```
    {
        "cred_def1_id": <credential_def1_json>,
        "cred_def2_id": <credential_def2_json>,
        "cred_def3_id": <credential_def3_json>,
    }
````
* `revStates`: Json - all revocation states json participating in the proof request
```
    {
        "rev_reg_def1_id  or credential_1_id"": {
            "timestamp1": <rev_state1>,
            "timestamp2": <rev_state2>,
        },
        "rev_reg_def2_id  or credential_2_id"": {
            "timestamp3": <rev_state3>
        },
        "rev_reg_def3_id  or credential_3_id"": {
            "timestamp4": <rev_state4>
        },
    } - Note: use credential_id instead rev_reg_id in case proving several credentials from the same revocation registry.
where
where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
````
* __->__ `proof`: Json - Proof json
For each requested attribute either a proof \(with optionally revealed attribute value\) or
self-attested attribute value is provided.
Each proof is associated with a credential and corresponding schema\_id, cred\_def\_id, rev\_reg\_id and timestamp.
There is also aggregated proof part common for all credential proofs.
```
    {
        "requested_proof": {
            "revealed_attrs": {
                "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string},
                "requested_attr4_id": {sub_proof_index: number: string, encoded: string},
            },
            "revealed_attr_groups": {
                "requested_attr5_id": {
                    "sub_proof_index": number,
                    "values": {
                        "attribute_name": {
                            "raw": string,
                            "encoded": string
                        }
                    },
                }
            },
            "unrevealed_attrs": {
                "requested_attr3_id": {sub_proof_index: number}
            },
            "self_attested_attrs": {
                "requested_attr2_id": self_attested_value,
            },
            "requested_predicates": {
                "requested_predicate_1_referent": {sub_proof_index: int},
                "requested_predicate_2_referent": {sub_proof_index: int},
            }
        }
        "proof": {
            "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
            "aggregated_proof": <aggregated_proof>
        } (opaque type that contains data structures internal to Ursa.
            It should not be parsed and are likely to change in future versions).
        "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
    }
````

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### verifierVerifyProof \( proofRequest, proof, schemas, credentialDefsJsons, revRegDefs, revRegs \) -&gt; valid

Verifies a proof \(of multiple credential\).
All required schemas, public keys and revocation registries must be provided.

IMPORTANT: You must use *_id's (`schema_id`, `cred_def_id`, `rev_reg_id`) listed in `proof[identifiers]`
as the keys for corresponding `schemas`, `credentialDefsJsons`, `revRegDefs`, `revRegs` objects.

* `proofRequest`: Json - proof request json
```
    {
        "name": string,
        "version": string,
        "nonce": string, - a decimal number represented as a string (use `generateNonce` function to generate 80-bit number)
        "requested_attributes": { // set of requested attributes
             "<attr_referent>": <attr_info>, // see below
             ...,
        },
        "requested_predicates": { // set of requested predicates
             "<predicate_referent>": <predicate_info>, // see below
             ...,
         },
        "non_revoked": Optional<<non_revoc_interval>>, // see below,
                       // If specified prover must proof non-revocation
                       // for date in this interval for each attribute
                       // (can be overridden on attribute level)
        "ver": Optional<str>  - proof request version:
          - omit to use unqualified identifiers for restrictions
          - "1.0" to use unqualified identifiers for restrictions
          - "2.0" to use fully qualified identifiers for restrictions
    }
````
* `proof`: Json - created for request proof json
```
    {
        "requested_proof": {
            "revealed_attrs": {
                "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
                "requested_attr4_id": {sub_proof_index: number: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
            },
            "revealed_attr_groups": {
                "requested_attr5_id": {
                    "sub_proof_index": number,
                    "values": {
                        "attribute_name": {
                            "raw": string,
                            "encoded": string
                        }
                    }, // NOTE: check that `encoded` value match to `raw` value on application level
                }
            },
            "unrevealed_attrs": {
                "requested_attr3_id": {sub_proof_index: number}
            },
            "self_attested_attrs": {
                "requested_attr2_id": self_attested_value,
            },
            "requested_predicates": {
                "requested_predicate_1_referent": {sub_proof_index: int},
                "requested_predicate_2_referent": {sub_proof_index: int},
            }
        }
        "proof": {
            "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
            "aggregated_proof": <aggregated_proof>
        }
        "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
    }
````
* `schemas`: Json - all schema jsons participating in the proof
```
    {
        <schema1_id>: <schema1_json>,
        <schema2_id>: <schema2_json>,
        <schema3_id>: <schema3_json>,
    }
````
* `credentialDefsJsons`: Json
* `revRegDefs`: Json - all revocation registry definitions json participating in the proof
```
    {
        "rev_reg_def1_id": <rev_reg_def1_json>,
        "rev_reg_def2_id": <rev_reg_def2_json>,
        "rev_reg_def3_id": <rev_reg_def3_json>,
    }
````
* `revRegs`: Json - all revocation registries json participating in the proof
```
    {
        "rev_reg_def1_id": {
            "timestamp1": <rev_reg1>,
            "timestamp2": <rev_reg2>,
        },
        "rev_reg_def2_id": {
            "timestamp3": <rev_reg3>
        },
        "rev_reg_def3_id": {
            "timestamp4": <rev_reg4>
        },
    }
````
* __->__ `valid`: Boolean - valid: true - if signature is valid, false - otherwise

Errors: `Annoncreds*`, `Common*`, `Wallet*`

#### createRevocationState \( blobStorageReaderHandle, revRegDef, revRegDelta, timestamp, credRevId \) -&gt; revState

Create revocation state for a credential that corresponds to a particular time.

Note that revocation delta must cover the whole registry existence time.
You can use `from`: `0` and `to`: `needed_time` as parameters for building request to get correct revocation delta.

The resulting revocation state and provided timestamp can be saved and reused later with applying a new
revocation delta with `updateRevocationState` function.
This new delta should be received with parameters: `from`: `timestamp` and `to`: `needed_time`.

* `blobStorageReaderHandle`: Handle (Number) - configuration of blob storage reader handle that will allow to read revocation tails
* `revRegDef`: Json - revocation registry definition json
* `revRegDelta`: Json - revocation registry delta which covers the whole registry existence time
* `timestamp`: Timestamp (Number) - time represented as a total number of seconds from Unix Epoch
* `credRevId`: String - user credential revocation id in revocation registry
* __->__ `revState`: Json - revocation state json:
```
    {
        "rev_reg": <revocation registry>,
        "witness": <witness>,
        "timestamp" : integer
    }
````

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### updateRevocationState \( blobStorageReaderHandle, revState, revRegDef, revRegDelta, timestamp, credRevId \) -&gt; updatedRevState

 Create a new revocation state for a credential based on a revocation state created before.
 Note that provided revocation delta must cover the registry gap from based state creation until the specified time
 (this new delta should be received with parameters: `from`: `state_timestamp` and `to`: `needed_time`).

 This function reduces the calculation time.

 The resulting revocation state and provided timestamp can be saved and reused later by applying a new revocation delta again.

* `blobStorageReaderHandle`: Handle (Number) - configuration of blob storage reader handle that will allow to read revocation tails
* `revState`: Json - revocation registry state json
* `revRegDef`: Json - revocation registry definition json
* `revRegDelta`: Json - revocation registry definition delta which covers the gap form original `rev_state_json` creation till the requested timestamp
* `timestamp`: Timestamp (Number) - time represented as a total number of seconds from Unix Epoch
* `credRevId`: String - user credential revocation id in revocation registry
* __->__ `updatedRevState`: Json - revocation state json:
```
    {
        "rev_reg": <revocation registry>,
        "witness": <witness>,
        "timestamp" : integer
    }
````

Errors: `Common*`, `Wallet*`, `Anoncreds*`

#### generateNonce \( \) -&gt; nonce

Generates 80-bit numbers that can be used as a nonce for proof request.

* __->__ `nonce`: Json - generated number as a string

Errors: `Common*`

#### toUnqualified \( entity \) -&gt; res

Get unqualified form (short form without method) of a fully qualified entity like DID.

This function should be used to the proper casting of fully qualified entity to unqualified form in the following cases:
1) Issuer, which works with fully qualified identifiers, creates a Credential Offer for Prover, which doesn't support fully qualified identifiers.
2) Verifier prepares a Proof Request based on fully qualified identifiers or Prover, which doesn't support fully qualified identifiers.
3) another case when casting to unqualified form needed

* `entity`: String - target entity to disqualify. Can be one of: Did, SchemaId, CredentialDefinitionId, RevocationRegistryId, Schema, CredentialDefinition, RevocationRegistryDefinition, CredentialOffer, CredentialRequest, ProofRequest.
* __->__ `res`: Json - entity either in unqualified form or original if casting isn't possible

### blob_storage

#### openBlobStorageReader \( type, config \) -&gt; handle



* `type`: String
* `config`: Json
* __->__ `handle`: Handle (Number)


#### openBlobStorageWriter \( type, config \) -&gt; handle



* `type`: String
* `config`: Json
* __->__ `handle`: Handle (Number)


### crypto

#### createKey \( wh, key \) -&gt; vk

Creates keys pair and stores in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `key`: Json - Key information as json. Example:
```
{
    "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
                               Can be UTF-8, base64 or hex string.
    "crypto_type": string, // Optional (if not set then ed25519 curve is used); Currently only 'ed25519' value is supported for this field.
}
````
* __->__ `vk`: String - Ver key of generated key pair, also used as key identifier

Errors: `Common*`, `Wallet*`, `Crypto*`

#### setKeyMetadata \( wh, verkey, metadata \) -&gt; void

Saves\/replaces the meta information for the giving key in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `verkey`: String
* `metadata`: String
* __->__ void

Errors: `Common*`, `Wallet*`, `Crypto*`

#### getKeyMetadata \( wh, verkey \) -&gt; metadata

Retrieves the meta information for the giving key in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `verkey`: String
* __->__ `metadata`: String - The meta information stored with the key; Can be null if no metadata was saved for this key.

Errors: `Common*`, `Wallet*`, `Crypto*`

#### cryptoSign \( wh, signerVk, messageRaw \) -&gt; signatureRaw

Signs a message with a key.

Note to use DID keys with this function you can call keyForDid to get key id \(verkey\)
for specific DID.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `signerVk`: String - id \(verkey\) of message signer. The key must be created by calling createKey or createAndStoreMyDid
* `messageRaw`: Buffer - a pointer to first byte of message to be signed
* __->__ `signatureRaw`: Buffer - a signature string

Errors: `Common*`, `Wallet*`, `Crypto*`

#### cryptoVerify \( signerVk, messageRaw, signatureRaw \) -&gt; valid

Verify a signature with a verkey.

Note to use DID keys with this function you can call keyForDid to get key id \(verkey\)
for specific DID.

* `signerVk`: String - verkey of the message signer
* `messageRaw`: Buffer - a pointer to first byte of message that has been signed
* `signatureRaw`: Buffer - a pointer to first byte of signature to be verified
* __->__ `valid`: Boolean - valid: true - if signature is valid, false - otherwise

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### cryptoAuthCrypt \( wh, senderVk, recipientVk, messageRaw \) -&gt; encryptedMsgRaw

  **** THIS FUNCTION WILL BE DEPRECATED USE packMessage INSTEAD ****
  
Encrypt a message by authenticated-encryption scheme.

Sender can encrypt a confidential message specifically for Recipient, using Sender's public key.
Using Recipient's public key, Sender can compute a shared secret key.
Using Sender's public key and his secret key, Recipient can compute the exact same shared secret key.
That shared secret key can be used to verify that the encrypted message was not tampered with,
before eventually decrypting it.

Note to use DID keys with this function you can call keyForDid to get key id \(verkey\)
for specific DID.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `senderVk`: String - id \(verkey\) of message sender. The key must be created by calling createKey or createAndStoreMyDid
* `recipientVk`: String - id \(verkey\) of message recipient
* `messageRaw`: Buffer - a pointer to first byte of message that to be encrypted
* __->__ `encryptedMsgRaw`: Buffer - an encrypted message as a pointer to array of bytes.

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### cryptoAuthDecrypt \( wh, recipientVk, encryptedMsgRaw \) -&gt; \[ senderVk, decryptedMsgRaw \]

  **** THIS FUNCTION WILL BE DEPRECATED USE unpackMessage INSTEAD ****
  
Decrypt a message by authenticated-encryption scheme.

Sender can encrypt a confidential message specifically for Recipient, using Sender's public key.
Using Recipient's public key, Sender can compute a shared secret key.
Using Sender's public key and his secret key, Recipient can compute the exact same shared secret key.
That shared secret key can be used to verify that the encrypted message was not tampered with,
before eventually decrypting it.

Note to use DID keys with this function you can call keyForDid to get key id \(verkey\)
for specific DID.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `recipientVk`: String - id \(verkey\) of message recipient. The key must be created by calling createKey or createAndStoreMyDid
* `encryptedMsgRaw`: Buffer - a pointer to first byte of message that to be decrypted
* __->__ [ `senderVk`: String, `decryptedMsgRaw`: Buffer ] - sender verkey and decrypted message as a pointer to array of bytes

Errors: `Common*`, `Wallet*`, `Crypto*`

#### cryptoAnonCrypt \( recipientVk, messageRaw \) -&gt; encryptedMsgRaw

Encrypts a message by anonymous-encryption scheme.

Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
Only the Recipient can decrypt these messages, using its private key.
While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.

Note to use DID keys with this function you can call keyForDid to get key id \(verkey\)
for specific DID.

Note: use packMessage function for A2A goals.

* `recipientVk`: String - verkey of message recipient
* `messageRaw`: Buffer - a pointer to first byte of message that to be encrypted
* __->__ `encryptedMsgRaw`: Buffer - an encrypted message as a pointer to array of bytes

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### cryptoAnonDecrypt \( wh, recipientVk, encryptedMsg \) -&gt; decryptedMsgRaw

Decrypts a message by anonymous-encryption scheme.

Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
Only the Recipient can decrypt these messages, using its private key.
While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.

Note to use DID keys with this function you can call keyForDid to get key id \(verkey\)
for specific DID.

Note: use unpackMessage function for A2A goals.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `recipientVk`: String - id \(verkey\) of my key. The key must be created by calling createKey or createAndStoreMyDid
* `encryptedMsg`: Buffer
* __->__ `decryptedMsgRaw`: Buffer - decrypted message as a pointer to an array of bytes

Errors: `Common*`, `Wallet*`, `Crypto*`

#### packMessage \( wh, message, receiverKeys, senderVk \) -&gt; jwe

Packs a message by encrypting the message and serializes it in a JWE-like format (Experimental)

Note to use DID keys with this function you can call keyForDid to get key id (verkey) for specific DID.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `message`: Buffer - message that to be packed
* `receiverKeys`: Array - an array of strings which contains receiver's keys the message is being encrypted for.
    Example: \['receiver edge_agent_1 verkey', 'receiver edge_agent_2 verkey'\]
* `senderVk`: String - the sender's verkey as a string When null pointer is used in this parameter, anoncrypt is used
* __->__ `jwe`: Buffer - a JWE 
```
using authcrypt alg:
{
    "protected": "b64URLencoded({
       "enc": "xsalsa20poly1305",
       "typ": "JWM/1.0",
       "alg": "Authcrypt",
       "recipients": [
           {
               "encrypted_key": base64URLencode(libsodium.crypto_box(my_key, their_vk, cek, cek_iv))
               "header": {
                    "kid": "base58encode(recipient_verkey)",
                    "sender" : base64URLencode(libsodium.crypto_box_seal(their_vk, base58encode(sender_vk)),
                    "iv" : base64URLencode(cek_iv)
               }
           },
       ],
    })",
    "iv": <b64URLencode(iv)>,
    "ciphertext": b64URLencode(encrypt_detached({'@type'...}, protected_value_encoded, iv, cek),
    "tag": <b64URLencode(tag)>
}

Alternative example in using anoncrypt alg is defined below:
{
    "protected": "b64URLencoded({
       "enc": "xsalsa20poly1305",
       "typ": "JWM/1.0",
       "alg": "Anoncrypt",
       "recipients": [
           {
               "encrypted_key": base64URLencode(libsodium.crypto_box_seal(their_vk, cek)),
               "header": {
                   "kid": base58encode(recipient_verkey),
               }
           },
       ],
    })",
    "iv": b64URLencode(iv),
    "ciphertext": b64URLencode(encrypt_detached({'@type'...}, protected_value_encoded, iv, cek),
    "tag": b64URLencode(tag)
}
````

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### unpackMessage \( wh, jwe \) -&gt; res

Unpacks a JWE-like formatted message outputted by packMessage (Experimental)

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `jwe`: Buffer - JWE to be unpacked
* __->__ `res`: Buffer - a result message
```
if authcrypt was used to pack the message returns this json structure:
{
    message: <decrypted message>,
    sender_verkey: <sender_verkey>,
    recipient_verkey: <recipient_verkey>
}

OR

if anoncrypt was used to pack the message returns this json structure:
{
    message: <decrypted message>,
    recipient_verkey: <recipient_verkey>
}
````

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

### did

#### createAndStoreMyDid \( wh, did \) -&gt; \[ did, verkey \]

Creates keys \(signing and encryption keys\) for a new
DID \(owned by the caller of the library\).
Identity's DID must be either explicitly provided, or taken as the first 16 bit of verkey.
Saves the Identity DID with keys in a secured Wallet, so that it can be used to sign
and encrypt transactions.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: Json - Identity information as json
```
{
    "did": string, (optional;
            if not provided and cid param is false then the first 16 bit of the verkey will be used as a new DID;
            if not provided and cid is true then the full verkey will be used as a new DID;
            if provided, then keys will be replaced - key rotation use case)
    "seed": string, (optional) Seed that allows deterministic did creation (if not set random one will be created).
                               Can be UTF-8, base64 or hex string.
    "crypto_type": string, (optional; if not set then ed25519 curve is used;
              currently only 'ed25519' value is supported for this field)
    "cid": bool, (optional; if not set then false is used;)
    "method_name": string, (optional) method name to create fully qualified did (Example:  `did:method_name:NcYxiDXkpYi6ov5FcYDi1e`).
}
```
* __->__ [ `did`: String, `verkey`: String ] - did: DID generated and stored in the wallet
verkey: The DIDs verification key

Errors: `Common*`, `Wallet*`, `Crypto*`

#### replaceKeysStart \( wh, did, identity \) -&gt; verkey

Generated temporary keys \(signing and encryption keys\) for an existing
DID \(owned by the caller of the library\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String - target did to rotate keys.
* `identity`: Json
* __->__ `verkey`: String - verkey: The DIDs verification key

Errors: `Common*`, `Wallet*`, `Crypto*`

#### replaceKeysApply \( wh, did \) -&gt; void

Apply temporary keys as main for an existing DID \(owned by the caller of the library\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String - DID stored in the wallet
* __->__ void

Errors: `Common*`, `Wallet*`, `Crypto*`

#### storeTheirDid \( wh, identity \) -&gt; void

Saves their DID for a pairwise connection in a secured Wallet,
so that it can be used to verify transaction.
Updates DID associated verkey in case DID already exists in the Wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `identity`: Json - Identity information as json. Example:
```
    {
       "did": string, (required)
       "verkey": string 
                     - optional is case of adding a new DID, and DID is cryptonym: did == verkey,
                     - mandatory in case of updating an existing DID   
    }
````
* __->__ void

Errors: `Common*`, `Wallet*`, `Crypto*`

#### keyForDid \( poolHandle, wh, did \) -&gt; key

Returns ver key \(key id\) for the given DID.

"keyForDid" call follow the idea that we resolve information about their DID from
the ledger with cache in the local wallet. The "openWallet" call has freshness parameter
that is used for checking the freshness of cached pool value.

Note if you don't want to resolve their DID info from the ledger you can use
"keyForLocalDid" call instead that will look only to the local wallet and skip
freshness checking.

Note that "createAndStoreMyDid" makes similar wallet record as "createKey".
As result we can use returned ver key in all generic crypto and messaging functions.

* `poolHandle`: Handle (Number) - Pool handle \(created by open\_pool\).
* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String
* __->__ `key`: String - The DIDs ver key \(key id\).

Errors: `Common*`, `Wallet*`, `Crypto*`

#### keyForLocalDid \( wh, did \) -&gt; key

Returns ver key \(key id\) for the given DID.

"keyForLocalDid" call looks data stored in the local wallet only and skips freshness
checking.

Note if you want to get fresh data from the ledger you can use "keyForDid" call
instead.

Note that "createAndStoreMyDid" makes similar wallet record as "createKey".
As result we can use returned ver key in all generic crypto and messaging functions.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String
* __->__ `key`: String - The DIDs ver key \(key id\).

Errors: `Common*`, `Wallet*`, `Crypto*`

#### setEndpointForDid \( wh, did, address, transportKey \) -&gt; void

Set\/replaces endpoint information for the given DID.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String
* `address`: String
* `transportKey`: String
* __->__ void

Errors: `Common*`, `Wallet*`, `Crypto*`

#### getEndpointForDid \( wh, poolHandle, did \) -&gt; \[ address, transportVk \]

Returns endpoint information for the given DID.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `poolHandle`: Handle (Number)
* `did`: String
* __->__ [ `address`: String, `transportVk`: String ] - The DIDs endpoint.
- transport\_vk - The DIDs transport key \(ver key, key id\).

Errors: `Common*`, `Wallet*`, `Crypto*`

#### setDidMetadata \( wh, did, metadata \) -&gt; void

Saves\/replaces the meta information for the giving DID in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String
* `metadata`: String
* __->__ void

Errors: `Common*`, `Wallet*`, `Crypto*`

#### getDidMetadata \( wh, did \) -&gt; metadata

Retrieves the meta information for the giving DID in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String
* __->__ `metadata`: String - The meta information stored with the DID; Can be null if no metadata was saved for this DID.

Errors: `Common*`, `Wallet*`, `Crypto*`

#### getMyDidWithMeta \( wh, myDid \) -&gt; didWithMeta

Retrieves the information about the giving DID in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `myDid`: String
* __->__ `didWithMeta`: Json - did\_with\_meta: {
"did": string - DID stored in the wallet,
"verkey": string - The DIDs transport key \(ver key, key id\),
"tempVerkey": string - Temporary DIDs transport key \(ver key, key id\), exist only during the rotation of the keys.
After rotation is done, it becomes a new verkey.
"metadata": string - The meta information stored with the DID
}

Errors: `Common*`, `Wallet*`, `Crypto*`

#### listMyDidsWithMeta \( wh \) -&gt; dids

Retrieves the information about all DIDs stored in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* __->__ `dids`: Json - dids: \[{
"did": string - DID stored in the wallet,
"verkey": string - The DIDs transport key \(ver key, key id\).,
"tempVerkey": string - Temporary DIDs transport key \(ver key, key id\), exist only during the rotation of the keys.
After rotation is done, it becomes a new verkey.
"metadata": string - The meta information stored with the DID
}\]

Errors: `Common*`, `Wallet*`, `Crypto*`

#### abbreviateVerkey \( did, fullVerkey \) -&gt; verkey

Retrieves abbreviated verkey if it is possible otherwise return full verkey.

* `did`: String - DID.
* `fullVerkey`: String - The DIDs verification key,
* __->__ `verkey`: String - verkey: The DIDs verification key in either abbreviated or full form

Errors: `Common*`, `Wallet*`, `Crypto*`

#### qualifyDid \( wh, did, method \) -&gt; fullQualifiedDid

Update DID stored in the wallet to make fully qualified, or to do other DID maintenance.
   - If the DID has no prefix, a prefix will be appended (prepend did:peer to a legacy did)
   - If the DID has a prefix, a prefix will be updated (migrate did:peer to did:peer-new)

Update DID related entities stored in the wallet.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `did`: String - target DID stored in the wallet.
* `method`: String - method to apply to the DID.
* __->__ `fullQualifiedDid`: String - fully qualified did

Errors: `Common*`, `Wallet*`, `Crypto*`

### ledger

#### signAndSubmitRequest \( poolHandle, wh, submitterDid, request \) -&gt; requestResult

Signs and submits request message to validator pool.

Adds submitter information to passed request json, signs it with submitter
sign key \(see wallet\_sign\), and sends signed request message
to validator pool \(see write\_request\).

* `poolHandle`: Handle (Number) - pool handle \(created by open\_pool\_ledger\).
* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - Id of Identity stored in secured Wallet.
* `request`: Json - Request data json.
* __->__ `requestResult`: Json

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### submitRequest \( poolHandle, request \) -&gt; requestResult

Publishes request message to validator pool \(no signing, unlike sign\_and\_submit\_request\).

The request is sent to the validator pool as is. It's assumed that it's already prepared.

* `poolHandle`: Handle (Number) - pool handle \(created by open\_pool\_ledger\).
* `request`: Json - Request data json.
* __->__ `requestResult`: Json

Errors: `Common*`, `Ledger*`

#### submitAction \( poolHandle, request, nodes, timeout \) -&gt; requestResult

Send action to particular nodes of validator pool.

The list of requests can be send:
POOL\_RESTART
GET\_VALIDATOR\_INFO

The request is sent to the nodes as is. It's assumed that it's already prepared.

* `poolHandle`: Handle (Number) - pool handle \(created by open\_pool\_ledger\).
* `request`: Json - Request data json.
* `nodes`: Json - \(Optional\) List of node names to send the request.
\["Node1", "Node2",...."NodeN"\]
* `timeout`: Number - \(Optional\) Time to wait respond from nodes \(override the default timeout\) \(in sec\).
Pass -1 to use default timeout
* __->__ `requestResult`: Json

Errors: `Common*`, `Ledger*`

#### signRequest \( wh, submitterDid, request \) -&gt; signedRequest

Signs request message.

Adds submitter information to passed request json, signs it with submitter
sign key \(see wallet\_sign\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - Id of Identity stored in secured Wallet.
* `request`: Json - Request data json.
* __->__ `signedRequest`: Json - Signed request json.

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### multiSignRequest \( wh, submitterDid, request \) -&gt; signedRequest

Multi signs request message.

Adds submitter information to passed request json, signs it with submitter
sign key \(see wallet\_sign\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - Id of Identity stored in secured Wallet.
* `request`: Json - Request data json.
* __->__ `signedRequest`: Json - Signed request json.

Errors: `Common*`, `Wallet*`, `Ledger*`, `Crypto*`

#### buildGetDdoRequest \( submitterDid, targetDid \) -&gt; requestResult

Builds a request to get a DDO.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `targetDid`: String - Target DID as base58-encoded string for 16 or 32 bit DID value.
* __->__ `requestResult`: Json

Errors: `Common*`

#### buildNymRequest \( submitterDid, targetDid, verkey, alias, role \) -&gt; request

Builds a NYM request. Request to create a new NYM record for a specific user.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
    Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `targetDid`: String - Target DID as base58-encoded string for 16 or 32 bit DID value.
* `verkey`: String - Target identity verification key as base58-encoded string.
* `alias`: String - NYM's alias.
* `role`: String - Role of a user NYM record:
null \(common USER\)
TRUSTEE
STEWARD
TRUST\_ANCHOR
NETWORK\_MONITOR
empty string to reset role
* __->__ `request`: Json

Errors: `Common*`

#### buildAttribRequest \( submitterDid, targetDid, hash, raw, enc \) -&gt; request

Builds an ATTRIB request. Request to add attribute to a NYM record.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `targetDid`: String - Target DID as base58-encoded string for 16 or 32 bit DID value.
* `hash`: String - \(Optional\) Hash of attribute data.
* `raw`: Json - \(Optional\) Json, where key is attribute name and value is attribute value.
* `enc`: String - \(Optional\) Encrypted value attribute data.
* __->__ `request`: Json

Errors: `Common*`

#### buildGetAttribRequest \( submitterDid, targetDid, raw, hash, enc \) -&gt; request

Builds a GET\_ATTRIB request. Request to get information about an Attribute for the specified DID.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `targetDid`: String - Target DID as base58-encoded string for 16 or 32 bit DID value.
* `raw`: String - \(Optional\) Requested attribute name.
* `hash`: String - \(Optional\) Requested attribute hash.
* `enc`: String - \(Optional\) Requested attribute encrypted value.
* __->__ `request`: Json

Errors: `Common*`

#### buildGetNymRequest \( submitterDid, targetDid \) -&gt; request

Builds a GET\_NYM request. Request to get information about a DID \(NYM\).

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `targetDid`: String - Target DID as base58-encoded string for 16 or 32 bit DID value.
* __->__ `request`: Json

Errors: `Common*`

#### parseGetNymResponse \( response \) -&gt; nymData

Parse a GET_NYM response to get NYM data.

* `response`: String - response on GET_NYM request.
* __->__ `nymData`: Json 
```
   {
       did: DID as base58-encoded string for 16 or 32 bit DID value.
       verkey: verification key as base58-encoded string.
       role: Role associated number
                               null (common USER)
                               0 - TRUSTEE
                               2 - STEWARD
                               101 - TRUST_ANCHOR
                               101 - ENDORSER - equal to TRUST_ANCHOR that will be removed soon
                               201 - NETWORK_MONITOR
   }
```

Errors: `Common*`

#### buildSchemaRequest \( submitterDid, data \) -&gt; request

Builds a SCHEMA request. Request to add Credential's schema.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `data`: Json - Credential schema.
```
{
    id: identifier of schema
    attrNames: array of attribute name strings
    name: Schema's name string (the number of attributes should be less or equal than 125)
    version: Schema's version string,
    ver: Version of the Schema json
}
````
* __->__ `request`: Json

Errors: `Common*`

#### buildGetSchemaRequest \( submitterDid, id \) -&gt; request

Builds a GET\_SCHEMA request. Request to get Credential's Schema.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `id`: String - Schema ID in ledger
* __->__ `request`: Json

Errors: `Common*`

#### parseGetSchemaResponse \( getSchemaResponse \) -&gt; \[ schemaId, schema \]

Parse a GET\_SCHEMA response to get Schema in the format compatible with Anoncreds API.

* `getSchemaResponse`: Json - response of GET\_SCHEMA request.
* __->__ [ `schemaId`: String, `schema`: Json ] - Schema Id and Schema json.
```
{
    id: identifier of schema
    attrNames: array of attribute name strings
    name: Schema's name string
    version: Schema's version string
    ver: Version of the Schema json
}
````

Errors: `Common*`

#### buildCredDefRequest \( submitterDid, data \) -&gt; request

Builds an CRED\_DEF request. Request to add a Credential Definition \(in particular, public key\),
that Issuer creates for a particular Credential Schema.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `data`: Json - credential definition json
```
{
    id: string - identifier of credential definition
    schemaId: string - identifier of stored in ledger schema
    type: string - type of the credential definition. CL is the only supported type now.
    tag: string - allows to distinct between credential definitions for the same issuer and schema
    value: Dictionary with Credential Definition's data:
{
        primary: primary credential public key,
        Optional<revocation>: revocation credential public key
    },
    ver: Version of the CredDef json
}
````
* __->__ `request`: Json

Errors: `Common*`

#### buildGetCredDefRequest \( submitterDid, id \) -&gt; request

Builds a GET\_CRED\_DEF request. Request to get a Credential Definition \(in particular, public key\),
that Issuer creates for a particular Credential Schema.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `id`: String - Credential Definition ID in ledger.
* __->__ `request`: Json

Errors: `Common*`

#### parseGetCredDefResponse \( getCredDefResponse \) -&gt; \[ credDefId, credDef \]

Parse a GET\_CRED\_DEF response to get Credential Definition in the format compatible with Anoncreds API.

* `getCredDefResponse`: Json - response of GET\_CRED\_DEF request.
* __->__ [ `credDefId`: String, `credDef`: Json ] - Credential Definition Id and Credential Definition json.
```
{
    id: string - identifier of credential definition
    schemaId: string - identifier of stored in ledger schema
    type: string - type of the credential definition. CL is the only supported type now.
    tag: string - allows to distinct between credential definitions for the same issuer and schema
    value: Dictionary with Credential Definition's data: {
        primary: primary credential public key,
        Optional<revocation>: revocation credential public key
    },
    ver: Version of the Credential Definition json
}
````

Errors: `Common*`

#### buildNodeRequest \( submitterDid, targetDid, data \) -&gt; request

Builds a NODE request. Request to add a new node to the pool, or updates existing in the pool.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `targetDid`: String - Target Node's DID.  It differs from submitter\_did field.
* `data`: Json - Data associated with the Node:
```
{
    alias: string - Node's alias
    blskey: string - (Optional) BLS multi-signature key as base58-encoded string.
    blskey_pop: string - (Optional) BLS key proof of possession as base58-encoded string.
    client_ip: string - (Optional) Node's client listener IP address.
    client_port: string - (Optional) Node's client listener port.
    node_ip: string - (Optional) The IP address other Nodes use to communicate with this Node.
    node_port: string - (Optional) The port other Nodes use to communicate with this Node.
    services: array<string> - (Optional) The service of the Node. VALIDATOR is the only supported one now.
}
````
* __->__ `request`: Json

Errors: `Common*`

#### buildGetValidatorInfoRequest \( submitterDid \) -&gt; request

Builds a GET\_VALIDATOR\_INFO request.

* `submitterDid`: String - DID of the read request sender.
* __->__ `request`: Json

Errors: `Common*`

#### buildGetTxnRequest \( submitterDid, ledgerType, seqNo \) -&gt; request

Builds a GET\_TXN request. Request to get any transaction by its seq\_no.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `ledgerType`: String - \(Optional\) type of the ledger the requested transaction belongs to:
DOMAIN - used default,
POOL,
CONFIG
any number
* `seqNo`: Number - requested transaction sequence number as it's stored on Ledger.
* __->__ `request`: Json

Errors: `Common*`

#### buildPoolConfigRequest \( submitterDid, writes, force \) -&gt; request

Builds a POOL\_CONFIG request. Request to change Pool's configuration.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `writes`: Boolean - Whether any write requests can be processed by the pool
\(if false, then pool goes to read-only state\). True by default.
* `force`: Boolean - Whether we should apply transaction \(for example, move pool to read-only state\)
without waiting for consensus of this transaction.
* __->__ `request`: Json

Errors: `Common*`

#### buildPoolRestartRequest \( submitterDid, action, datetime \) -&gt; request

Builds a POOL\_RESTART request.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `action`: String - Action that pool has to do after received transaction.
* `datetime`: String - &lt;Optional&gt; Restart time in datetime format. Skip to restart as early as possible.
* __->__ `request`: Json

Errors: `Common*`

#### buildPoolUpgradeRequest \( submitterDid, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package\_ \) -&gt; request

Builds a POOL\_UPGRADE request. Request to upgrade the Pool \(sent by Trustee\).
It upgrades the specified Nodes \(either all nodes in the Pool, or some specific ones\).

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `name`: String - Human-readable name for the upgrade.
* `version`: String - The version of indy-node package we perform upgrade to.
Must be greater than existing one \(or equal if reinstall flag is True\).
* `action`: String - Either start or cancel.
* `sha256`: String - sha256 hash of the package.
* `timeout`: Number - \(Optional\) Limits upgrade time on each Node.
* `schedule`: String - \(Optional\) Schedule of when to perform upgrade on each node. Map Node DIDs to upgrade time.
* `justification`: String - \(Optional\) justification string for this particular Upgrade.
* `reinstall`: Boolean - Whether it's allowed to re-install the same version. False by default.
* `force`: Boolean - Whether we should apply transaction \(schedule Upgrade\) without waiting
for consensus of this transaction.
* `package_`: String
* __->__ `request`: Json

Errors: `Common*`

#### buildRevocRegDefRequest \( submitterDid, data \) -&gt; request

Builds a REVOC\_REG\_DEF request. Request to add the definition of revocation registry
to an exists credential definition.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `data`: Json - Revocation Registry data:
```
    {
        "id": string - ID of the Revocation Registry,
        "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
        "tag": string - Unique descriptive ID of the Registry,
        "credDefId": string - ID of the corresponding CredentialDefinition,
        "value": Registry-specific data {
            "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
            "maxCredNum": number - Maximum number of credentials the Registry can serve.
            "tailsHash": string - Hash of tails.
            "tailsLocation": string - Location of tails file.
            "publicKeys": <public_keys> - Registry's public key.
        },
        "ver": string - version of revocation registry definition json.
    }
````
* __->__ `request`: Json

Errors: `Common*`

#### buildGetRevocRegDefRequest \( submitterDid, id \) -&gt; request

Builds a GET\_REVOC\_REG\_DEF request. Request to get a revocation registry definition,
that Issuer creates for a particular Credential Definition.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `id`: String - ID of Revocation Registry Definition in ledger.
* __->__ `request`: Json

Errors: `Common*`

#### parseGetRevocRegDefResponse \( getRevocRefDefResponse \) -&gt; \[ revocRegDefId, revocRegDef \]

Parse a GET\_REVOC\_REG\_DEF response to get Revocation Registry Definition in the format
compatible with Anoncreds API.

* `getRevocRefDefResponse`: Json
* __->__ [ `revocRegDefId`: String, `revocRegDef`: Json ] - Revocation Registry Definition Id and Revocation Registry Definition json.
```
{
    "id": string - ID of the Revocation Registry,
    "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
    "tag": string - Unique descriptive ID of the Registry,
    "credDefId": string - ID of the corresponding CredentialDefinition,
    "value": Registry-specific data {
        "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
        "maxCredNum": number - Maximum number of credentials the Registry can serve.
        "tailsHash": string - Hash of tails.
        "tailsLocation": string - Location of tails file.
        "publicKeys": <public_keys> - Registry's public key.
    },
    "ver": string - version of revocation registry definition json.
}
````

Errors: `Common*`

#### buildRevocRegEntryRequest \( submitterDid, revocRegDefId, revDefType, value \) -&gt; request

Builds a REVOC\_REG\_ENTRY request.  Request to add the RevocReg entry containing
the new accumulator value and issued\/revoked indices.
This is just a delta of indices, not the whole list.
So, it can be sent each time a new credential is issued\/revoked.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `revocRegDefId`: String - ID of the corresponding RevocRegDef.
* `revDefType`: String - Revocation Registry type \(only CL\_ACCUM is supported for now\).
* `value`: Json - Registry-specific data:
```
{
    value:
{
        prevAccum: string - previous accumulator value.
        accum: string - current accumulator value.
        issued: array<number> - an array of issued indices.
        revoked: array<number> an array of revoked indices.
    },
    ver: string - version revocation registry entry json
}
````
* __->__ `request`: Json

Errors: `Common*`

#### buildGetRevocRegRequest \( submitterDid, revocRegDefId, timestamp \) -&gt; request

Builds a GET\_REVOC\_REG request. Request to get the accumulated state of the Revocation Registry
by ID. The state is defined by the given timestamp.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `revocRegDefId`: String - ID of the corresponding Revocation Registry Definition in ledger.
* `timestamp`: Timestamp (Number) - Requested time represented as a total number of seconds from Unix Epoch
* __->__ `request`: Json

Errors: `Common*`

#### parseGetRevocRegResponse \( getRevocRegResponse \) -&gt; \[ revocRegDefId, revocReg, timestamp \]

Parse a GET\_REVOC\_REG response to get Revocation Registry in the format compatible with Anoncreds API.

* `getRevocRegResponse`: Json - response of GET\_REVOC\_REG request.
* __->__ [ `revocRegDefId`: String, `revocReg`: Json, `timestamp`: Timestamp (Number) ] - Revocation Registry Definition Id, Revocation Registry json and Timestamp.
```
{
    "value": Registry-specific data {
        "accum": string - current accumulator value.
    },
    "ver": string - version revocation registry json
}
````

Errors: `Common*`

#### buildGetRevocRegDeltaRequest \( submitterDid, revocRegDefId, from, to \) -&gt; request

Builds a GET\_REVOC\_REG\_DELTA request. Request to get the delta of the accumulated state of the Revocation Registry.
The Delta is defined by from and to timestamp fields.
If from is not specified, then the whole state till to will be returned.

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `revocRegDefId`: String - ID of the corresponding Revocation Registry Definition in ledger.
* `from`: Timestamp (Number) - Requested time represented as a total number of seconds from Unix Epoch
* `to`: Timestamp (Number) - Requested time represented as a total number of seconds from Unix Epoch
* __->__ `request`: Json

Errors: `Common*`

#### parseGetRevocRegDeltaResponse \( getRevocRegDeltaResponse \) -&gt; \[ revocRegDefId, revocRegDelta, timestamp \]

Parse a GET\_REVOC\_REG\_DELTA response to get Revocation Registry Delta in the format compatible with Anoncreds API.

* `getRevocRegDeltaResponse`: Json
* __->__ [ `revocRegDefId`: String, `revocRegDelta`: Json, `timestamp`: Timestamp (Number) ] - Revocation Registry Definition Id, Revocation Registry Delta json and Timestamp.
```
{
    "value": Registry-specific data {
        prevAccum: string - previous accumulator value.
        accum: string - current accumulator value.
        issued: array<number> - an array of issued indices.
        revoked: array<number> an array of revoked indices.
    },
    "ver": string - version revocation registry delta json
}
````

Errors: `Common*`

#### buildAuthRuleRequest \( submitterDid, txnType, action, field, oldValue, newValue, constraint \) -&gt; request

Builds a AUTH_RULE request. Request to change authentication rules for a ledger transaction.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `txnType`: String - ledger transaction alias or associated value.
* `action`: String - type of an action.
    * "ADD" - to add a new rule
    * "EDIT" - to edit an existing one
* `field`: String - transaction field.
* `oldValue`: String - \(Optional\) old value of a field, which can be changed to a new_value (mandatory for EDIT action).
* `newValue`: String - \(Optional\) new value that can be used to fill the field. 
* `constraint`: Json - set of constraints required for execution of an action in the following format:
```
 {
     constraint_id - <string> type of a constraint.
         Can be either "ROLE" to specify final constraint or  "AND"/"OR" to combine constraints.
     role - <string> (optional) role of a user which satisfy to constrain.
     sig_count - <u32> the number of signatures required to execution action.
     need_to_be_owner - <bool> (optional) if user must be an owner of transaction (false by default).
     off_ledger_signature - <bool> (optional) allow signature of unknow for ledger did (false by default). 
     metadata - <object> (optional) additional parameters of the constraint.
 }
can be combined by
 {
     'constraint_id': <"AND" or "OR">
     'auth_constraints': [<constraint_1>, <constraint_2>]
 }
```

Default ledger auth rules: https://github.com/hyperledger/indy-node/blob/master/docs/source/auth_rules.md

More about AUTH_RULE request: https://github.com/hyperledger/indy-node/blob/master/docs/source/requests.md#auth_rule   

* __->__ `request`: Json

Errors: `Common*`

#### buildAuthRulesRequest \( submitterDid, data \) -&gt; request

Builds a AUTH_RULES request. Request to change multiple authentication rules for a ledger transaction.

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `constraint`: Json - a list of auth rules:
```
[
    {
        "auth_type": ledger transaction alias or associated value,
        "auth_action": type of an action,
        "field": transaction field,
        "old_value": (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action),
        "new_value": (Optional) new value that can be used to fill the field,
        "constraint": set of constraints required for execution of an action in the format described above for `buildAuthRuleRequest` function.
    },
    ...
]
```

Default ledger auth rules: https://github.com/hyperledger/indy-node/blob/master/docs/source/auth_rules.md

More about AUTH_RULE request: https://github.com/hyperledger/indy-node/blob/master/docs/source/requests.md#auth_rules   

* __->__ `request`: Json

Errors: `Common*`


#### buildGetAuthRuleRequest \( submitterDid, txnType, action, field, oldValue, newValue \) -&gt; request

Builds a GET_AUTH_RULE request. Request to get authentication rules for a ledger transaction.

NOTE: Either none or all transaction related parameters must be specified (`oldValue` can be skipped for `ADD` action).
* none - to get all authentication rules for all ledger transactions
* all - to get authentication rules for specific action (`oldValue` can be skipped for `ADD` action)

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `txnType`: String - target ledger transaction alias or associated value.
* `action`: String - target action type. Can be either "ADD" or "EDIT".
* `field`: String - target transaction field.
* `oldValue`: String - \(Optional\) old value of field, which can be changed to a new_value (mandatory for EDIT action).
* `newValue`: String - \(Optional\) new value that can be used to fill the field. 

* __->__ `request`: Json

Errors: `Common*`

#### buildTxnAuthorAgreementRequest \( submitterDid, text, version, ratificationTimestamp, retirementTimestamp \) -&gt; request

Builds a TXN_AUTHR_AGRMT request. 
Request to add a new version of Transaction Author Agreement to the ledger.

EXPERIMENTAL

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `text`: String - \(Optional\)  a content of the TTA.
    * Mandatory in case of adding a new TAA. An existing TAA text can not be changed.
    * for Indy Node version <= 1.12.0:
        * Use empty string to reset TAA on the ledger
    * for Indy Node version > 1.12.0
        * Should be omitted in case of updating an existing TAA (setting `retirementTimestamp`)
* `version`: String - a version of the TTA (unique UTF-8 string).
* `version`: String - the date (timestamp) of TAA ratification by network government.
* `ratificationTimestamp`: Number - \(Optional\) Еhe date (timestamp) of TAA ratification by network government.
    * for Indy Node version <= 1.12.0:
        * Must be omitted
    * for Indy Node version > 1.12.0:
        * Must be specified in case of adding a new TAA
        * Can be omitted in case of updating an existing TAA
* `retirementTimestamp`: Number - \(Optional\) the date \(timestamp\) of TAA retirement.
    * for Indy Node version <= 1.12.0:
        * Must be omitted
    * for Indy Node version > 1.12.0:
        * Must be omitted in case of adding a new (latest) TAA.
        * Should be used for updating (deactivating) non-latest TAA on the ledger.

Note: Use `buildDisableAllTxnAuthorAgreementsRequest` to disable all TAA's on the ledger.

* __->__ `request`: Json

Errors: `Common*`

#### buildDisableAllTxnAuthorAgreementsRequest \( submitterDid \) -&gt; request

Builds a DISABLE_ALL_TXN_AUTHR_AGRMTS request. 
Request to disable all Transaction Author Agreement on the ledger.

EXPERIMENTAL

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)

* __->__ `request`: Json

Errors: `Common*`


#### buildGetTxnAuthorAgreementRequest \( submitterDid, data \) -&gt; request

Builds a GET_TXN_AUTHR_AGRMT request. 
Request to get a specific Transaction Author Agreement from the ledger.

EXPERIMENTAL

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `data`: Json - \(Optional\) specifies a condition for getting specific TAA.
Contains 3 mutually exclusive optional fields:
```
{
   hash: Optional<str> - hash of requested TAA,
   version: Optional<str> - version of requested TAA.
   timestamp: Optional<u64> - ledger will return TAA valid at requested timestamp.
}
```
Null data or empty JSON are acceptable here. In this case, ledger will return the latest version of TAA.

* __->__ `request`: Json

Errors: `Common*`

#### buildAcceptanceMechanismsRequest \( submitterDid, aml, version, amlContext \) -&gt; request

Builds a SET_TXN_AUTHR_AGRMT_AML request. 
Request to add a new list of acceptance mechanisms for transaction author agreement.
Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.

EXPERIMENTAL

* `submitterDid`: String - Identifier (DID) of the transaction author as base58-encoded string.
                           Actual request sender may differ if Endorser is used (look at `appendRequestEndorser`)
* `aml`: Json - a set of new acceptance mechanisms:
```
{
  “<acceptance mechanism label 1>”: { acceptance mechanism description 1},
  “<acceptance mechanism label 2>”: { acceptance mechanism description 2},
  ...
}
```
* `version`: String - a version of new acceptance mechanisms. (Note: unique on the Ledger).
* `amlContext`: String - \(Optional\) common context information about acceptance mechanisms (may be a URL to external resource).

* __->__ `request`: Json

Errors: `Common*`

#### buildGetAcceptanceMechanismsRequest \( submitterDid, timestamp \) -&gt; request

Builds a GET_TXN_AUTHR_AGRMT_AML request. 
Request to get a list of  acceptance mechanisms from the ledger valid for specified time or the latest one.

EXPERIMENTAL

* `submitterDid`: String - \(Optional\) DID of the read request sender \(if not provided then default Libindy DID will be used\).
* `timestamp`: Timestamp (Number) - \(Optional\) time to get an active acceptance mechanisms. The latest one will be returned for null.
* `version`: Timestamp (String) - \(Optional\) version of acceptance mechanisms.

NOTE: timestamp and version cannot be specified together.

* __->__ `request`: Json

Errors: `Common*`

#### appendTxnAuthorAgreementAcceptanceToRequest \( requestJson, text, version, taaDigest, accMechType, timeOfAcceptance \) -&gt; request

Append transaction author agreement acceptance data to a request.
This function should be called before signing and sending a request
if there is any transaction author agreement set on the Ledger.

EXPERIMENTAL

This function may calculate hash by itself or consume it as a parameter.
If all text, version and taaDigest parameters are specified, a check integrity of them will be done.

* `requestJson`: Json - original request data json.
* `text`: String - \(Optional\) raw data about TAA from ledger.
* `version`: String - \(Optional\) raw data about TAA from ledger.
     * `text` and `version` parameters should be passed together.
     * `text` and `version` parameters are required if taaDigest parameter is omitted.
* `taaDigest`: String - \(Optional\) hash on text and version. Digest is sha256 hash calculated on concatenated strings: version || text. This parameter is required if text and version parameters are omitted.
* `accMechType`: String - mechanism how user has accepted the TAA.
* `timeOfAcceptance`: Timestamp (Number) - UTC timestamp when user has accepted the TAA. Note that the time portion will be discarded to avoid a privacy risk. 

* __->__ `request`: Json

Errors: `Common*`

#### appendRequestEndorser \( requestJson, endorserDid \) -&gt; request

Append Endorser to an existing request.

An author of request still is a `DID` used as a `submitter_did` parameter for the building of the request.
But it is expecting that the transaction will be sent by the specified Endorser.

Note: Both Transaction Author and Endorser must sign output request after that.

More about Transaction Endorser: https://github.com/hyperledger/indy-node/blob/master/design/transaction_endorser.md
                                 https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md

* `requestJson`: Json - original request data json.
* `endorser_did`: String - DID of the Endorser that will submit the transaction.
                           The Endorser's DID must be present on the ledger.

* __->__ `request`: Json

Errors: `Common*`

#### getResponseMetadata \( response \) -&gt; responseMetadata

Parse transaction response to fetch metadata.
The important use case for this method is validation of Node's response freshens.

Distributed Ledgers can reply with outdated information for consequence read request after write.
To reduce pool load libindy sends read requests to one random node in the pool.
Consensus validation is performed based on validation of nodes multi signature for current ledger Merkle Trie root.
This multi signature contains information about the latest ldeger's transaction ordering time and sequence number that this method returns.

If node that returned response for some reason is out of consensus and has outdated ledger
it can be caught by analysis of the returned latest ledger's transaction ordering time and sequence number.

There are two ways to filter outdated responses:
1\) based on "seqNo" - sender knows the sequence number of transaction that he consider as a fresh enough.
2\) based on "txnTime" - sender knows the timestamp that he consider as a fresh enough.

Note: response of GET\_VALIDATOR\_INFO request isn't supported

* `response`: Json - response of write or get request.
* __->__ `responseMetadata`: Json - response metadata.
```
{
    "seqNo": Option<u64> - transaction sequence number,
    "txnTime": Option<u64> - transaction ordering time,
    "lastSeqNo": Option<u64> - the latest transaction seqNo for particular Node,
    "lastTxnTime": Option<u64> - the latest transaction ordering time for particular Node
}
````

Errors: `Common*`, `Ledger*`

### non_secrets

#### addWalletRecord \( wh, type, id, value, tags \) -&gt; void

Create a new non-secret record in the wallet

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `id`: String - the id of record
* `value`: String - the value of record
* `tags`: Json - \(optional\) the record tags used for search and storing meta information as json:
```
  {
    "tagName1": <str>, // string tag (will be stored encrypted)
    "tagName2": <str>, // string tag (will be stored encrypted)
    "~tagName3": <str>, // string tag (will be stored un-encrypted)
    "~tagName4": <str>, // string tag (will be stored un-encrypted)
  }
  Note that null means no tags
  If tag name starts with "~" the tag will be stored un-encrypted that will allow
  usage of this tag in complex search queries (comparison, predicates)
  Encrypted tags can be searched only for exact matching
````
* __->__ void


#### updateWalletRecordValue \( wh, type, id, value \) -&gt; void

Update a non-secret wallet record value

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `id`: String - the id of record
* `value`: String - the new value of record
* __->__ void


#### updateWalletRecordTags \( wh, type, id, tags \) -&gt; void

Update a non-secret wallet record tags

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `id`: String - the id of record
* `tags`: Json - the record tags used for search and storing meta information as json:
```
  {
    "tagName1": <str>, // string tag (will be stored encrypted)
    "tagName2": <str>, // string tag (will be stored encrypted)
    "~tagName3": <str>, // string tag (will be stored un-encrypted)
    "~tagName4": <str>, // string tag (will be stored un-encrypted)
  }
  If tag name starts with "~" the tag will be stored un-encrypted that will allow
  usage of this tag in complex search queries (comparison, predicates)
  Encrypted tags can be searched only for exact matching
````
* __->__ void


#### addWalletRecordTags \( wh, type, id, tags \) -&gt; void

Add new tags to the wallet record

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `id`: String - the id of record
* `tags`: Json - the record tags used for search and storing meta information as json:
```
  {
    "tagName1": <str>, // string tag (will be stored encrypted)
    "tagName2": <str>, // string tag (will be stored encrypted)
    "~tagName3": <str>, // string tag (will be stored un-encrypted)
    "~tagName4": <str>, // string tag (will be stored un-encrypted)
  }
  If tag name starts with "~" the tag will be stored un-encrypted that will allow
  usage of this tag in complex search queries (comparison, predicates)
  Encrypted tags can be searched only for exact matching
  Note if some from provided tags already assigned to the record than
    corresponding tags values will be replaced
````
* __->__ void


#### deleteWalletRecordTags \( wh, type, id, tagNames \) -&gt; void

Delete tags from the wallet record

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `id`: String - the id of record
* `tagNames`: Json - the list of tag names to remove from the record as json array:
\["tagName1", "tagName2", ...\]
* __->__ void


#### deleteWalletRecord \( wh, type, id \) -&gt; void

Delete an existing wallet record in the wallet

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - record type
* `id`: String - the id of record
* __->__ void


#### getWalletRecord \( wh, type, id, options \) -&gt; record

Get an wallet record by id

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `id`: String - the id of record
* `options`: Json - \/\/TODO: FIXME: Think about replacing by bitmask
```
 {
   retrieveType: (optional, false by default) Retrieve record type,
   retrieveValue: (optional, true by default) Retrieve record value,
   retrieveTags: (optional, false by default) Retrieve record tags
 }
````
* __->__ `record`: Json - wallet record json:
```
{
  id: "Some id",
  type: "Some type", // present only if retrieveType set to true
  value: "Some value", // present only if retrieveValue set to true
  tags: <tags json>, // present only if retrieveTags set to true
}
````


#### openWalletSearch \( wh, type, query, options \) -&gt; sh

Search for wallet records.

Note instead of immediately returning of fetched records
this call returns wallet\_search\_handle that can be used later
to fetch records by small batches \(with fetchWalletSearchNextRecords\).

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `type`: String - allows to separate different record types collections
* `query`: Json - MongoDB style query to wallet record tags:
```
 {
   "tagName": "tagValue",
   $or:
{
     "tagName2": { $regex: 'pattern' },
     "tagName3": { $gte: '123' },
   },
 }
````
* `options`: Json - \/\/TODO: FIXME: Think about replacing by bitmask
```
 {
   retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
   retrieveTotalCount: (optional, false by default) Calculate total count,
   retrieveType: (optional, false by default) Retrieve record type,
   retrieveValue: (optional, true by default) Retrieve record value,
   retrieveTags: (optional, false by default) Retrieve record tags,
 }
````
* __->__ `sh`: Handle (Number) - search\_handle: Wallet search handle that can be used later
to fetch records by small batches \(with fetchWalletSearchNextRecords\)


#### fetchWalletSearchNextRecords \( wh, walletSearchHandle, count \) -&gt; records

Fetch next records for wallet search.

Not if there are no records this call returns WalletNoRecords error.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `walletSearchHandle`: Handle (Number) - wallet search handle \(created by openWalletSearch\)
* `count`: Number - Count of records to fetch
* __->__ `records`: Json - wallet records json:
```
{
  totalCount: <str>, // present only if retrieveTotalCount set to true
  records: [{ // present only if retrieveRecords set to true
      id: "Some id",
      type: "Some type", // present only if retrieveType set to true
      value: "Some value", // present only if retrieveValue set to true
      tags: <tags json>, // present only if retrieveTags set to true
  }],
}
````


#### closeWalletSearch \( walletSearchHandle \) -&gt; void

Close wallet search \(make search handle invalid\)

* `walletSearchHandle`: Handle (Number) - wallet search handle
* __->__ void


### pairwise

#### isPairwiseExists \( wh, theirDid \) -&gt; exists

Check if pairwise is exists.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `theirDid`: String - encrypted DID
* __->__ `exists`: Boolean - exists: true - if pairwise is exists, false - otherwise

Errors: `Common*`, `Wallet*`

#### createPairwise \( wh, theirDid, myDid, metadata \) -&gt; void

Creates pairwise.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `theirDid`: String - encrypted DID
* `myDid`: String - encrypted DID
metadata Optional: extra information for pairwise
* `metadata`: String
* __->__ void

Errors: `Common*`, `Wallet*`

#### listPairwise \( wh \) -&gt; listPairwise

Get list of saved pairwise.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* __->__ `listPairwise`: Json - list\_pairwise: list of saved pairwise

Errors: `Common*`, `Wallet*`

#### getPairwise \( wh, theirDid \) -&gt; pairwiseInfo

Gets pairwise information for specific their\_did.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `theirDid`: String - encoded Did
* __->__ `pairwiseInfo`: Json - pairwise\_info\_json: did info associated with their did

Errors: `Common*`, `Wallet*`

#### setPairwiseMetadata \( wh, theirDid, metadata \) -&gt; void

Save some data in the Wallet for pairwise associated with Did.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `theirDid`: String - encoded Did
* `metadata`: String - some extra information for pairwise
* __->__ void

Errors: `Common*`, `Wallet*`

### payment

#### createPaymentAddress \( wh, paymentMethod, config \) -&gt; paymentAddress

Create the payment address for specified payment method


This method generates private part of payment address
and stores it in a secure place. Ideally it should be
secret in libindy wallet \(see crypto module\).

Note that payment method should be able to resolve this
secret by fully resolvable payment address format.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `paymentMethod`: String - payment method to use \(for example, 'sov'\)
* `config`: Json - payment address config as json:
```
  {
    seed: <str>, // allows deterministic creation of payment address
  }
````
* __->__ `paymentAddress`: String - payment\_address - public identifier of payment address in fully resolvable payment address format


#### listPaymentAddresses \( wh \) -&gt; paymentAddresses

Lists all payment addresses that are stored in the wallet

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* __->__ `paymentAddresses`: Json - payment\_addresses\_json - json array of string with json addresses


#### addRequestFees \( wh, submitterDid, req, inputs, outputs, extra \) -&gt; \[ reqWithFees, paymentMethod \]

Modifies Indy request by adding information how to pay fees for this transaction
according to this payment method.

This method consumes set of inputs and outputs. The difference between inputs balance
and outputs balance is the fee for this transaction.

Not that this method also produces correct fee signatures.

Format of inputs is specific for payment method. Usually it should reference payment transaction
with at least one output that corresponds to payment address that user owns.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `req`: Json - initial transaction request as json
* `inputs`: Json - The list of payment sources as json array:
\["source1", ...\]
- each input should reference paymentAddress
- this param will be used to determine payment\_method
* `outputs`: Json - The list of outputs as json array:
```
  [{
    recipient: <str>, // payment address of recipient
    amount: <int>, // amount
  }]
````
* `extra`: String - \/\/ optional information for payment operation
* __->__ [ `reqWithFees`: Json, `paymentMethod`: String ] - req\_with\_fees\_json - modified Indy request with added fees info
payment\_method - used payment method


#### parseResponseWithFees \( paymentMethod, resp \) -&gt; receipts

Parses response for Indy request with fees.

* `paymentMethod`: String - payment method to use
* `resp`: Json - response for Indy request with fees
* __->__ `receipts`: Json - receipts\_json - parsed \(payment method and node version agnostic\) receipts info as json:
```
  [{
     receipt: <str>, // receipt that can be used for payment referencing and verification
     recipient: <str>, //payment address of recipient
     amount: <int>, // amount
     extra: <str>, // optional data from payment transaction
  }]
````


#### buildGetPaymentSourcesRequest \( wh, submitterDid, paymentAddress \) -&gt; \[ getSourcesTxn, paymentMethod \]

Builds Indy request for getting sources list for payment address
according to this payment method.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `paymentAddress`: String - target payment address
* __->__ [ `getSourcesTxn`: Json, `paymentMethod`: String ] - get\_sources\_txn\_json - Indy request for getting sources list for payment address
payment\_method - used payment method


#### parseGetPaymentSourcesResponse \( paymentMethod, resp \) -&gt; sources

Parses response for Indy request for getting sources list.

* `paymentMethod`: String - payment method to use.
* `resp`: Json - response for Indy request for getting sources list
* __->__ `sources`: Json - sources\_json - parsed \(payment method and node version agnostic\) sources info as json:
```
  [{
     source: <str>, // source input
     paymentAddress: <str>, //payment address for this source
     amount: <int>, // amount
     extra: <str>, // optional data from payment transaction
  }]
````


#### buildPaymentReq \( wh, submitterDid, inputs, outputs, extra \) -&gt; \[ paymentReq, paymentMethod \]

Builds Indy request for doing payment
according to this payment method.

This method consumes set of inputs and outputs.

Format of inputs is specific for payment method. Usually it should reference payment transaction
with at least one output that corresponds to payment address that user owns.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `inputs`: Json - The list of payment sources as json array:
\["source1", ...\]
Note that each source should reference payment address
* `outputs`: Json - The list of outputs as json array:
```
  [{
    recipient: <str>, // payment address of recipient
    amount: <int>, // amount
  }]
````
* `extra`: String - \/\/ optional information for payment operation
* __->__ [ `paymentReq`: Json, `paymentMethod`: String ] - payment\_req\_json - Indy request for doing payment
payment\_method - used payment method


#### parsePaymentResponse \( paymentMethod, resp \) -&gt; receipts

Parses response for Indy request for payment txn.

* `paymentMethod`: String - payment method to use
* `resp`: Json - response for Indy request for payment txn
* __->__ `receipts`: Json - receipts\_json - parsed \(payment method and node version agnostic\) receipts info as json:
```
  [{
     receipt: <str>, // receipt that can be used for payment referencing and verification
     recipient: <str>, // payment address of recipient
     amount: <int>, // amount
     extra: <str>, // optional data from payment transaction
  }]
````

#### preparePaymentExtraWithAcceptanceData \( extraJson, text, version, taaDigest, accMechType, timeOfAcceptance \) -&gt; request

Append payment extra JSON with TAA acceptance data

EXPERIMENTAL

This function may calculate hash by itself or consume it as a parameter.
If all text, version and taaDigest parameters are specified, a check integrity of them will be done.

* `extraJson`: Json - \(Optional\) original extra json.
* `text`: String - \(Optional\) raw data about TAA from ledger.
* `version`: String - \(Optional\) raw data about TAA from ledger.
     * `text` and `version` parameters should be passed together.
     * `text` and `version` parameters are required if taaDigest parameter is omitted.
* `taaDigest`: String - \(Optional\) hash on text and version. This parameter is required if text and version parameters are omitted.
* `accMechType`: String - mechanism how user has accepted the TAA.
* `timeOfAcceptance`: Timestamp (Number) - UTC timestamp when user has accepted the TAA.

* __->__ `request`: Json

Errors: `Common*`

#### buildMintReq \( wh, submitterDid, outputs, extra \) -&gt; \[ mintReq, paymentMethod \]

Builds Indy request for doing minting
according to this payment method.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `outputs`: Json - The list of outputs as json array:
```
  [{
    recipient: <str>, // payment address of recipient
    amount: <int>, // amount
  }]
````
* `extra`: String - \/\/ optional information for mint operation
* __->__ [ `mintReq`: Json, `paymentMethod`: String ] - mint\_req\_json - Indy request for doing minting
payment\_method - used payment method


#### buildSetTxnFeesReq \( wh, submitterDid, paymentMethod, fees \) -&gt; setTxnFees

Builds Indy request for setting fees for transactions in the ledger

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `paymentMethod`: String - payment method to use
fees\_json {
txnType1: amount1,
txnType2: amount2,
.................
txnTypeN: amountN,
}
* `fees`: Json
* __->__ `setTxnFees`: Json - set\_txn\_fees\_json - Indy request for setting fees for transactions in the ledger


#### buildGetTxnFeesReq \( wh, submitterDid, paymentMethod \) -&gt; getTxnFees

Builds Indy get request for getting fees for transactions in the ledger

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `paymentMethod`: String - payment method to use
* __->__ `getTxnFees`: Json - get\_txn\_fees\_json - Indy request for getting fees for transactions in the ledger


#### parseGetTxnFeesResponse \( paymentMethod, resp \) -&gt; fees

Parses response for Indy request for getting fees

* `paymentMethod`: String - payment method to use
* `resp`: Json - response for Indy request for getting fees
* __->__ `fees`: Json - fees\_json {
txnType1: amount1,
txnType2: amount2,
.................
txnTypeN: amountN,
}


#### buildVerifyPaymentReq \( wh, submitterDid, receipt \) -&gt; \[ verifyTxn, paymentMethod \]

Builds Indy request for information to verify the payment receipt

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - \(Optional\) DID of request sender
* `receipt`: String - payment receipt to verify
* __->__ [ `verifyTxn`: Json, `paymentMethod`: String ] - verify\_txn\_json: Indy request for verification receipt
payment\_method: used payment method


#### parseVerifyPaymentResponse \( paymentMethod, resp \) -&gt; txn

Parses Indy response with information to verify receipt

* `paymentMethod`: String - payment method to use
* `resp`: Json - response of the ledger for verify txn
* __->__ `txn`: Json - txn\_json: {
sources: \[&lt;str&gt;, \]
receipts: \[ {
recipient: &lt;str&gt;, \/\/ payment address of recipient
receipt: &lt;str&gt;, \/\/ receipt that can be used for payment referencing and verification
amount: &lt;int&gt;, \/\/ amount
} \],
extra: &lt;str&gt;, \/\/optional data
}

#### getRequestInfo \( getAuthRuleResponse, requesterInfo, fees \) -&gt; requestInfo

Gets request requirements (with minimal price) correspondent to specific auth rule
in case the requester can perform this action.

*EXPERIMENTAL*

If the requester does not match to the request constraints `TransactionNotAllowed` error will be thrown.

* `getAuthRuleResponse`: String - response on GET_AUTH_RULE request returning action constraints set on the ledger.
* `requesterInfo`: Json:
```
{
    "role": string (optional) - role of a user which can sign a transaction.
    "sig_count": u64 - number of signers.
    "is_owner": bool (optional) - if user is an owner of transaction (false by default).
    "is_off_ledger_signature": bool (optional) - if user did is unknow for ledger (false by default).
}
```
* `fees`: Json - fees set on the ledger (result of `parseGetTxnFeesResponse`).
* __->__ `requestInfo`: Json - request info if a requester match to the action constraints.
```
{
    "price": u64 - fee required for the action performing,
    "requirements": [{
        "role": string (optional) - role of users who should sign,
        "sig_count": u64 - number of signers,
        "need_to_be_owner": bool - if requester need to be owner,
        "off_ledger_signature": bool - allow signature of unknow for ledger did (false by default).
    }]
}
```

Errors: `Common*`, `Ledger*`

#### signWithAddress \( wh, address, message \) -&gt; signature

Signs a message with a payment address.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `address`: String - payment address of message signer. The key must be created by calling createPaymentAddress
* `message`: Buffer - a pointer to first byte of message to be signed
* __->__ `signature`: Buffer - a signature string

Errors: `Common*`, `Wallet*`, `Payment*`

#### verifyWithAddress \( address, message, signature \) -&gt; valid

Verify a signature with a payment address.

* `address`: String - payment address of the message signer
* `message`: Buffer - a pointer to first byte of message that has been signed
* `signature`: Buffer - a pointer to first byte of signature to be verified
* __->__ `valid`: Boolean - valid: true - if signature is valid, false - otherwise

Errors: `Common*`, `Wallet*`, `Payment*`


### pool

#### createPoolLedgerConfig \( configName, config \) -&gt; void

Creates a new local pool ledger configuration that can be used later to connect pool nodes.

* `configName`: String - Name of the pool ledger configuration.
* `config`: Json? - Pool configuration json. if NULL, then default config will be used. Example:
```
{
    "genesis_txn": string (optional), A path to genesis transaction file. If NULL, then a default one will be used.
                   If file doesn't exists default one will be created.
}
````
* __->__ void

Errors: `Common*`, `Ledger*`

#### openPoolLedger \( configName, config \) -&gt; poolHandle

Opens pool ledger and performs connecting to pool nodes.

Pool ledger configuration with corresponded name must be previously created
with createPoolLedgerConfig method.
It is impossible to open pool with the same name more than once.

config\_name: Name of the pool ledger configuration.
config \(optional\): Runtime pool configuration json.
if NULL, then default config will be used. Example:
```
{
    "timeout": int (optional), timeout for network request (in sec).
    "extended_timeout": int (optional), extended timeout for network request (in sec).
    "preordered_nodes": array<string> -  (optional), names of nodes which will have a priority during request sending:
        ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
        This can be useful if a user prefers querying specific nodes.
        Assume that `Node1` and `Node2` nodes reply faster.
        If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
        Note: Nodes not specified will be placed randomly.
    "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
        By default Libindy sends a read requests to 2 nodes in the pool.
        If response isn't received or `state proof` is invalid Libindy sends the request again but to 2 (`number_read_nodes`) * 2 = 4 nodes and so far until completion.
}
````

* `configName`: String
* `config`: Json
* __->__ `poolHandle`: Handle (Number) - Handle to opened pool to use in methods that require pool connection.

Errors: `Common*`, `Ledger*`

#### refreshPoolLedger \( handle \) -&gt; void

Refreshes a local copy of a pool ledger and updates pool nodes connections.

* `handle`: Handle (Number) - pool handle returned by openPoolLedger
* __->__ void

Errors: `Common*`, `Ledger*`

#### listPools \(  \) -&gt; pools

Lists names of created pool ledgers

* __->__ `pools`: Json


#### closePoolLedger \( handle \) -&gt; void

Closes opened pool ledger, opened nodes connections and frees allocated resources.

* `handle`: Handle (Number) - pool handle returned by openPoolLedger.
* __->__ void

Errors: `Common*`, `Ledger*`

#### deletePoolLedgerConfig \( configName \) -&gt; void

Deletes created pool ledger configuration.

* `configName`: String - Name of the pool ledger configuration to delete.
* __->__ void

Errors: `Common*`, `Ledger*`

#### setProtocolVersion \( protocolVersion \) -&gt; void

Set PROTOCOL\_VERSION to specific version.

There is a global property PROTOCOL\_VERSION that used in every request to the pool and
specified version of Indy Node which Libindy works.

By default PROTOCOL\_VERSION=1.

* `protocolVersion`: Number - Protocol version will be used:
1 - for Indy Node 1.3
2 - for Indy Node 1.4 and greater
* __->__ void

Errors: `Common*`

### wallet

#### createWallet \( config, credentials \) -&gt; void

Create a new secure wallet.

* `config`: Json - Wallet configuration json.
```
{
  "id": string, Identifier of the wallet.
        Configured storage uses this identifier to lookup exact wallet data placement.
  "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
                 'Default' storage type allows to store wallet data in the local file.
                 Custom storage types can be registered with indy_register_wallet_storage call.
  "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
                    Can be optional if storage supports default configuration.
                    For 'default' storage type configuration is:
  {
    "path": optional<string>, Path to the directory with wallet files.
            Defaults to $HOME/.indy_client/wallet.
            Wallet will be stored in the file {path}/{id}/sqlite.db
  }
}
````
* `credentials`: Json - Wallet credentials json
```
{
  "key": string, Key or passphrase used for wallet key derivation.
                 Look to key_derivation_method param for information about supported key derivation methods.
  "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
                         Can be optional if storage supports default configuration.
                         For 'default' storage type should be empty.
  "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
                         ARGON2I_MOD - derive secured wallet master key (used by default)
                         ARGON2I_INT - derive secured wallet master key (less secured but faster)
                         RAW - raw wallet key master provided (skip derivation).
                               RAW keys can be generated with generateWalletKey call
}
````
* __->__ void

Errors: `Common*`, `Wallet*`

#### openWallet \( config, credentials \) -&gt; handle

Open the wallet.

Wallet must be previously created with createWallet method.

* `config`: Json - Wallet configuration json.
```
  {
      "id": string, Identifier of the wallet.
            Configured storage uses this identifier to lookup exact wallet data placement.
      "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
                      'Default' storage type allows to store wallet data in the local file.
                      Custom storage types can be registered with indy_register_wallet_storage call.
      "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
                        Can be optional if storage supports default configuration.
                        For 'default' storage type configuration is:
          {
             "path": optional<string>, Path to the directory with wallet files.
                     Defaults to $HOME/.indy_client/wallet.
                     Wallet will be stored in the file {path}/{id}/sqlite.db
          }
  }
````
* `credentials`: Json - Wallet credentials json
```
  {
      "key": string, Key or passphrase used for wallet key derivation.
                     Look to key_derivation_method param for information about supported key derivation methods.
      "rekey": optional<string>, If present than wallet master key will be rotated to a new one.
      "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
                             Can be optional if storage supports default configuration.
                             For 'default' storage type should be empty.
      "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
                         ARGON2I_MOD - derive secured wallet master key (used by default)
                         ARGON2I_INT - derive secured wallet master key (less secured but faster)
                         RAW - raw wallet key master provided (skip derivation).
                               RAW keys can be generated with generateWalletKey call
      "rekey_derivation_method": optional<string> Algorithm to use for wallet rekey derivation:
                         ARGON2I_MOD - derive secured wallet master rekey (used by default)
                         ARGON2I_INT - derive secured wallet master rekey (less secured but faster)
                         RAW - raw wallet rekey master provided (skip derivation).
                               RAW keys can be generated with generateWalletKey call
  }
````
* __->__ `handle`: Handle (Number) - err: Error code
handle: Handle to opened wallet to use in methods that require wallet access.

Errors: `Common*`, `Wallet*`

#### exportWallet \( wh, exportConfig \) -&gt; void

Exports opened wallet

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `exportConfig`: Json
* __->__ void

Errors: `Common*`, `Wallet*`

#### importWallet \( config, credentials, importConfig \) -&gt; void

Creates a new secure wallet and then imports its content
according to fields provided in import\_config
This can be seen as an createWallet call with additional content import

* `config`: Json - Wallet configuration json.
```
{
  "id": string, Identifier of the wallet.
        Configured storage uses this identifier to lookup exact wallet data placement.
  "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
                 'Default' storage type allows to store wallet data in the local file.
                 Custom storage types can be registered with indy_register_wallet_storage call.
  "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
                    Can be optional if storage supports default configuration.
                    For 'default' storage type configuration is:
  {
    "path": optional<string>, Path to the directory with wallet files.
            Defaults to $HOME/.indy_client/wallet.
            Wallet will be stored in the file {path}/{id}/sqlite.db
  }
}
````
* `credentials`: Json - Wallet credentials json
```
{
  "key": string, Key or passphrase used for wallet key derivation.
                 Look to key_derivation_method param for information about supported key derivation methods.
  "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
                         Can be optional if storage supports default configuration.
                         For 'default' storage type should be empty.
  "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
                            ARGON2I_MOD - derive secured wallet master key (used by default)
                            ARGON2I_INT - derive secured wallet master key (less secured but faster)
                            RAW - raw wallet key master provided (skip derivation).
                               RAW keys can be generated with generateWalletKey call
}
````
* `importConfig`: Json
* __->__ void

Errors: `Common*`, `Wallet*`

#### closeWallet \( wh \) -&gt; void

Closes opened wallet and frees allocated resources.

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* __->__ void

Errors: `Common*`, `Wallet*`

#### deleteWallet \( config, credentials \) -&gt; void

Deletes created wallet.

* `config`: Json - Wallet configuration json.
```
{
  "id": string, Identifier of the wallet.
        Configured storage uses this identifier to lookup exact wallet data placement.
  "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
                 'Default' storage type allows to store wallet data in the local file.
                 Custom storage types can be registered with indy_register_wallet_storage call.
  "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
                    Can be optional if storage supports default configuration.
                    For 'default' storage type configuration is:
  {
    "path": optional<string>, Path to the directory with wallet files.
            Defaults to $HOME/.indy_client/wallet.
            Wallet will be stored in the file {path}/{id}/sqlite.db
  }
}
````
* `credentials`: Json - Wallet credentials json
```
{
  "key": string, Key or passphrase used for wallet key derivation.
                 Look to key_derivation_method param for information about supported key derivation methods.
  "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
                         Can be optional if storage supports default configuration.
                         For 'default' storage type should be empty.
  "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
                            ARGON2I_MOD - derive secured wallet master key (used by default)
                            ARGON2I_INT - derive secured wallet master key (less secured but faster)
                            RAW - raw wallet key master provided (skip derivation).
                               RAW keys can be generated with generateWalletKey call
}
````
* __->__ void

Errors: `Common*`, `Wallet*`

#### generateWalletKey \( config \) -&gt; key

Generate wallet master key.
Returned key is compatible with "RAW" key derivation method.
It allows to avoid expensive key derivation for use cases when wallet keys can be stored in a secure enclave.

* `config`: Json - \(optional\) key configuration json.
```
{
  "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
                             Can be UTF-8, base64 or hex string.
}
````
* __->__ `key`: String - err: Error code

Errors: `Common*`, `Wallet*`


### logger

WARNING: You can only set the logger **once**. Call `setLogger`, `setDefaultLogger`, not both. Once it's been set, libindy won't let you change it.

#### setDefaultLogger \( pattern \)

Calling this turns on the default logger and libindy will write logs to stdout.

* `pattern`: String - pattern that corresponds with the log messages to show.

Errors: `Common*`

NOTE: This is a synchronous function (does not return a promise.)

#### setLogger \( logFn \)

Set a function to be called every time a log is emitted from libindy.

* `logFn`: Function(Int level, String target, String message, String module_path, String file, Int line)

Example:
```js
indy.setLogger(function (level, target, message, modulePath, file, line) {
    console.log('libindy said:', level, target, message, modulePath, file, line)
})
```

Errors: `Common*`

NOTE: This is a synchronous function (does not return a promise) but may call `logFn` asynchronously many times.

### cache

#### getSchema \( poolHandle, wh, submitterDid, id, options \) -&gt; schema

Get schema json data for specified schema id.
If data is present inside of cache, cached data is returned.
Otherwise data is fetched from the ledger and stored inside of cache for future use.

EXPERIMENTAL

* `poolHandle`:
* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - DID of the read request sender.
* `id`: String - Schema ID in ledger
* `options`: Json
```
 {
    noCache: (bool, optional, false by default) Skip usage of cache,
    noUpdate: (bool, optional, false by default) Use only cached data, do not try to update.
    noStore: (bool, optional, false by default) Skip storing fresh data if updated,
    minFresh: (int, optional, -1 by default) Return cached data if not older than this many seconds. -1 means do not check age.
 }

```
__->__ schema: Json
```
{
    id: identifier of schema
    attrNames: array of attribute name strings
    name: Schema's name string
    version: Schema's version string
    ver: Version of the Schema json
}
```

Errors: `Common*`, `Wallet*`, `Ledger*`

#### getCredDef \( poolHandle, wh, submitterDid, id, options \) -&gt; credDef

EXPERIMENTAL

Get credential definition json data for specified credential definition id.
If data is present inside of cache, cached data is returned.
Otherwise data is fetched from the ledger and stored inside of cache for future use.

* `poolHandle`:
* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `submitterDid`: String - DID of the read request sender.
* `id`: String - Credential Definition ID in ledger.
* `options`: Json
```
 {
    noCache: (bool, optional, false by default) Skip usage of cache,
    noUpdate: (bool, optional, false by default) Use only cached data, do not try to update.
    noStore: (bool, optional, false by default) Skip storing fresh data if updated,
    minFresh: (int, optional, -1 by default) Return cached data if not older than this many seconds. -1 means do not check age.
 }

```
__->__ credDef: Json
```
{
    id: string - identifier of credential definition
    schemaId: string - identifier of stored in ledger schema
    type: string - type of the credential definition. CL is the only supported type now.
    tag: string - allows to distinct between credential definitions for the same issuer and schema
    value: Dictionary with Credential Definition's data: {
        primary: primary credential public key,
        Optional<revocation>: revocation credential public key
    },
    ver: Version of the Credential Definition json
}
```

Errors: `Common*`, `Wallet*`, `Ledger*`

#### purgeSchemaCache \( wh, options \) -&gt; void

Purge schema cache.

EXPERIMENTAL

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `options`: Json
```
 {
   maxAge: (int, optional, -1 by default) Purge cached data if older than this many seconds. -1 means purge all.
 }
```
* __->__ void

Errors: `Common*`, `Wallet*`

#### purgeCredDefCache \( wh, options \) -&gt; void

Purge credential definition cache.

EXPERIMENTAL

* `wh`: Handle (Number) - wallet handle (created by openWallet)
* `options`: Json
```
 {
   maxAge: (int, optional, -1 by default) Purge cached data if older than this many seconds. -1 means purge all.
 }
```
* __->__ void

Errors: `Common*`, `Wallet*`

### mod

#### setRuntimeConfig \( config \)

Set libindy runtime configuration. Can be optionally called to change current params.

* `config`: Json
```
{
  "crypto_thread_pool_size": Optional<int> - size of thread pool for the most expensive crypto operations. (4 by default)
  "collect_backtrace": Optional<bool> - whether errors backtrace should be collected.
    Capturing of backtrace can affect library performance.
    NOTE: must be set before invocation of any other API functions.
}
```

Errors: `Common*`

NOTE: This is a synchronous function (does not return a promise.)

## Advanced

If you need to get closer to the metal, you can access the node bindings directly.

* The function names and parameters are the same.
* It will not json stringify or parse for you.
* Only callbacks.
* Errors are plain numbers for indy error codes.

```js
var indy = require('indy-sdk')

indy.capi.abbreviateVerkey(did, fullVerkey, function(err, verkey){
  // err will be 0 on success, and a code number on failure
  // verkey will be the result string on success
})
```

## Contributing

[![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg)](https://standardjs.com)

Setup an Indy SDK environment, and start a local pool.

 * [ubuntu](https://github.com/hyperledger/indy-sdk/blob/master/docs/ubuntu-build.md)
 * [osx](https://github.com/hyperledger/indy-sdk/blob/master/docs/mac-build.md)
 * [windows](https://github.com/hyperledger/indy-sdk/blob/master/docs/windows-build.md)

```sh
# You will need libindy in your system library path. (i.e. /usr/lib/libindy.so for linux)
# or in this directory (i.e. wrappers/nodejs/libindy.so)

# Install dependencies and do the initial build.
npm install

# Run the tests
TEST_POOL_IP=10.0.0.2 npm test

# If you built with libindy locally (i.e. wrappers/nodejs/libindy.so) you need to set LD_LIBRARY_PATH
LD_LIBRARY_PATH=./ TEST_POOL_IP=10.0.0.2 npm test

# To recompile the native bindings
npm run rebuild
```
