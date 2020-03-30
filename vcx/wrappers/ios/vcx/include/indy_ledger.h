#ifndef __indy__ledger_included__
#define __indy__ledger_included__

#include "indy_mod.h"
#include "indy_types.h"

#ifdef __cplusplus
extern "C" {
#endif
    
    /// Signs and submits request message to validator pool.
    ///
    /// Adds submitter information to passed request json, signs it with submitter
    /// sign key (see wallet_sign), and sends signed request message
    /// to validator pool (see write_request).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// pool_handle: pool handle (created by open_pool_ledger).
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*
    
    extern indy_error_t indy_sign_and_submit_request(indy_handle_t command_handle,
                                                     indy_handle_t pool_handle,
                                                     indy_handle_t wallet_handle,
                                                     const char *  submitter_did,
                                                     const char *  request_json,

                                                     void           (*cb)(indy_handle_t command_handle_,
                                                                          indy_error_t  err,
                                                                          const char*   request_result_json)
                                                     );
    
    /// Publishes request message to validator pool (no signing, unlike sign_and_submit_request).
    ///
    /// The request is sent to the validator pool as is. It's assumed that it's already prepared.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// pool_handle: pool handle (created by open_pool_ledger).
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    
    extern indy_error_t indy_submit_request(indy_handle_t command_handle,
                                            indy_handle_t pool_handle,
                                            const char *  request_json,

                                            void           (*cb)(indy_handle_t command_handle_,
                                                                 indy_error_t  err,
                                                                 const char*   request_result_json)
                                           );

    /// Send action to particular nodes of validator pool.
    ///
    /// The list of requests can be send:
    ///     POOL_RESTART
    ///     GET_VALIDATOR_INFO
    ///
    /// The request is sent to the nodes as is. It's assumed that it's already prepared.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// pool_handle: pool handle (created by open_pool_ledger).
    /// request_json: Request data json.
    /// nodes: (Optional) List of node names to send the request.
    ///        ["Node1", "Node2",...."NodeN"]
    /// timeout: (Optional) Time to wait respond from nodes (override the default timeout) (in sec).
    ///                     Pass -1 to use default timeout
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    /// Ledger*

    extern indy_error_t indy_submit_action(indy_handle_t command_handle,
                                           indy_handle_t pool_handle,
                                           const char *  request_json,
                                           const char *  nodes,
                                           indy_i32_t    timeout,

                                           void           (*cb)(indy_handle_t command_handle_,
                                                                indy_error_t  err,
                                                                const char*   request_result_json)
                                           );

    /// Signs request message.
    ///
    /// Adds submitter information to passed request json, signs it with submitter
    /// sign key (see wallet_sign).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Signed request json.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*

    extern indy_error_t indy_sign_request(indy_handle_t command_handle,
                                         indy_handle_t  wallet_handle,
                                         const char *   submitter_did,
                                         const char *   request_json,

                                         void           (*cb)(indy_handle_t command_handle_,
                                                              indy_error_t  err,
                                                              const char*   signed_request_json)
                                         );


    /// Multi signs request message.
    ///
    /// Adds submitter information to passed request json, signs it with submitter
    /// sign key (see wallet_sign).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// wallet_handle: wallet handle (created by open_wallet).
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// request_json: Request data json.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Signed request json.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    /// Ledger*
    /// Crypto*

    extern indy_error_t indy_multi_sign_request(indy_handle_t command_handle,
                                                indy_handle_t  wallet_handle,
                                                const char *   submitter_did,
                                                const char *   request_json,

                                                void           (*cb)(indy_handle_t command_handle_,
                                                                     indy_error_t  err,
                                                                     const char*   signed_request_json)
                                                );

    /// Builds a request to get a DDO.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Id of Identity stored in secured Wallet.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    
    extern indy_error_t indy_build_get_ddo_request(indy_handle_t command_handle,
                                                   const char *  submitter_did,
                                                   const char *  target_did,

                                                   void           (*cb)(indy_handle_t command_handle_,
                                                                        indy_error_t  err,
                                                                        const char*   request_result_json)
                                                  );
    
    /// Builds a NYM request. Request to create a new NYM record for a specific user.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// verkey: Target identity verification key as base58-encoded string.
    /// alias: NYM's alias.
    /// role: Role of a user NYM record:
    ///                             null (common USER)
    ///                             TRUSTEE
    ///                             STEWARD
    ///                             TRUST_ANCHOR
    ///                             ENDORSER - equal to TRUST_ANCHOR that will be removed soon
    ///                             empty string to reset role
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_nym_request(indy_handle_t command_handle,
                                               const char *  submitter_did,
                                               const char *  target_did,
                                               const char *  verkey,
                                               const char *  alias,
                                               const char *  role,

                                               void           (*cb)(indy_handle_t command_handle_,
                                                                    indy_error_t  err,
                                                                    const char*   request_json)
                                              );

    /// Builds an ATTRIB request. Request to add attribute to a NYM record.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// hash: (Optional) Hash of attribute data.
    /// raw: (Optional) Json, where key is attribute name and value is attribute value.
    /// enc: (Optional) Encrypted value attribute data.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_attrib_request(indy_handle_t command_handle,
                                                  const char *  submitter_did,
                                                  const char *  target_did,
                                                  const char *  hash,
                                                  const char *  raw,
                                                  const char *  enc,

                                                  void           (*cb)(indy_handle_t command_handle_,
                                                                       indy_error_t  err,
                                                                       const char*   request_json)
                                                  );

    /// Builds a GET_ATTRIB request. Request to get information about an Attribute for the specified DID.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// raw: (Optional) Requested attribute name.
    /// hash: (Optional) Requested attribute hash.
    /// enc: (Optional) Requested attribute encrypted value.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    
    extern indy_error_t indy_build_get_attrib_request(indy_handle_t command_handle,
                                                      const char *  submitter_did,
                                                      const char *  target_did,
                                                      const char *  hash,
                                                      const char *  raw,
                                                      const char *  enc,

                                                      void           (*cb)(indy_handle_t command_handle_,
                                                                           indy_error_t  err,
                                                                           const char*   request_json)
                                                     );

    /// Builds a GET_NYM request. Request to get information about a DID (NYM).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did:(Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_get_nym_request(indy_handle_t command_handle,
                                                   const char *  submitter_did,
                                                   const char *  target_did,

                                                   void           (*cb)(indy_handle_t command_handle_,
                                                                        indy_error_t  err,
                                                                        const char*   request_json)
                                                  );

    /// Builds a SCHEMA request. Request to add Credential's schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// data: Credential schema.
    /// {
    ///     id: identifier of schema
    ///     attrNames: array of attribute name strings (the number of attributes should be less or equal than 125)
    ///     name: Schema's name string
    ///     version: Schema's version string,
    ///     ver: Version of the Schema json
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_schema_request(indy_handle_t command_handle,
                                                  const char *  submitter_did,
                                                  const char *  data,

                                                  void           (*cb)(indy_handle_t command_handle_,
                                                                       indy_error_t  err,
                                                                       const char*   request_json)
                                                 );

    /// Builds a GET_SCHEMA request. Request to get Credential's Schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// id: Schema ID in ledger
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    
    extern indy_error_t indy_build_get_schema_request(indy_handle_t command_handle,
                                                      const char *  submitter_did,
                                                      const char *  id,

                                                      void           (*cb)(indy_handle_t command_handle_,
                                                                           indy_error_t  err,
                                                                           const char*   request_json)
                                                     );

    /// Parse a GET_SCHEMA response to get Schema in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_schema_response: response of GET_SCHEMA request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Schema Id and Schema json.
    /// {
    ///     id: identifier of schema
    ///     attrNames: array of attribute name strings
    ///     name: Schema's name string
    ///     version: Schema's version string
    ///     ver: Version of the Schema json
    /// }
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_parse_get_schema_response(indy_handle_t command_handle,
                                                       const char *  get_schema_response,

                                                       void           (*cb)(indy_handle_t command_handle_,
                                                                            indy_error_t  err,
                                                                            const char*   schema_id,
                                                                            const char*   schema_json)
                                                       );
    
    /// Builds an CRED_DEF request. Request to add a Credential Definition (in particular, public key),
    /// that Issuer creates for a particular Credential Schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// data: credential definition json
    /// {
    ///     id: string - identifier of credential definition
    ///     schemaId: string - identifier of stored in ledger schema
    ///     type: string - type of the credential definition. CL is the only supported type now.
    ///     tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///     value: Dictionary with Credential Definition's data: {
    ///         primary: primary credential public key,
    ///         Optional<revocation>: revocation credential public key
    ///     },
    ///     ver: Version of the CredDef json
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    
    extern indy_error_t indy_build_cred_def_request(indy_handle_t command_handle,
                                                    const char *  submitter_did,
                                                    const char *  data,

                                                    void           (*cb)(indy_handle_t command_handle_,
                                                                         indy_error_t  err,
                                                                         const char*   request_json)
                                                    );
    
    /// Builds a GET_CRED_DEF request. Request to get a Credential Definition (in particular, public key),
    /// that Issuer creates for a particular Credential Schema.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// id: Credential Definition ID in ledger.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

     extern indy_error_t indy_build_get_cred_def_request(indy_handle_t command_handle,
                                                         const char *  submitter_did,
                                                         const char *  id,

                                                         void           (*cb)(indy_handle_t command_handle_,
                                                                              indy_error_t  err,
                                                                              const char*   request_json)
                                                         );

    /// Parse a GET_CRED_DEF response to get Credential Definition in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_cred_def_response: response of GET_CRED_DEF request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Credential Definition Id and Credential Definition json.
    /// {
    ///     id: string - identifier of credential definition
    ///     schemaId: string - identifier of stored in ledger schema
    ///     type: string - type of the credential definition. CL is the only supported type now.
    ///     tag: string - allows to distinct between credential definitions for the same issuer and schema
    ///     value: Dictionary with Credential Definition's data: {
    ///         primary: primary credential public key,
    ///         Optional<revocation>: revocation credential public key
    ///     },
    ///     ver: Version of the Credential Definition json
    /// }
    ///
    /// #Errors
    /// Common*

     extern indy_error_t indy_parse_get_cred_def_response(indy_handle_t command_handle,
                                                          const char *  get_cred_def_response,
                                                          void           (*cb)(indy_handle_t command_handle_,
                                                                               indy_error_t  err,
                                                                               const char*   cred_def_id,
                                                                               const char*   cred_def_json)
                                                          );

    /// Builds a NODE request. Request to add a new node to the pool, or updates existing in the pool.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// target_did: Target Node's DID.  It differs from submitter_did field.
    /// data: Data associated with the Node: {
    ///     alias: string - Node's alias
    ///     blskey: string - (Optional) BLS multi-signature key as base58-encoded string.
    ///     blskey_pop: string - (Optional) BLS key proof of possession as base58-encoded string.
    ///     client_ip: string - (Optional) Node's client listener IP address.
    ///     client_port: string - (Optional) Node's client listener port.
    ///     node_ip: string - (Optional) The IP address other Nodes use to communicate with this Node.
    ///     node_port: string - (Optional) The port other Nodes use to communicate with this Node.
    ///     services: array<string> - (Optional) The service of the Node. VALIDATOR is the only supported one now.
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    
    extern indy_error_t indy_build_node_request(indy_handle_t command_handle,
                                                const char *  submitter_did,
                                                const char *  target_did,
                                                const char *  data,

                                                void           (*cb)(indy_handle_t command_handle_,
                                                                     indy_error_t  err,
                                                                     const char*   request_json)
                                               );

        /// Builds a GET_VALIDATOR_INFO request.
        ///
        /// #Params
        /// command_handle: command handle to map callback to caller context.
        /// submitter_did: Id of Identity stored in secured Wallet.
        /// cb: Callback that takes command result as parameter.
        ///
        /// #Returns
        /// Request result as json.
        ///
        /// #Errors
        /// Common*

        extern indy_error_t indy_build_get_validator_info_request(indy_handle_t command_handle,
                                                       const char *  submitter_did,
                                                       void           (*cb)(indy_handle_t command_handle_,
                                                                            indy_error_t  err,
                                                                            const char*   request_json)
                                                       );


    /// Builds a GET_TXN request. Request to get any transaction by its seq_no.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// ledger_type: (Optional) type of the ledger the requested transaction belongs to:
    ///     DOMAIN - used default,
    ///     POOL,
    ///     CONFIG
    ///     any number
    /// seq_no: seq_no of transaction in ledger.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_get_txn_request(indy_handle_t command_handle,
                                                   const char *  submitter_did,
                                                   const char *  ledger_type,
                                                   indy_i32_t    seq_no,

                                                   void           (*cb)(indy_handle_t command_handle_,
                                                                        indy_error_t  err,
                                                                        const char*   request_json)
                                                   );

    /// Builds a POOL_CONFIG request. Request to change Pool's configuration.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// writes: Whether any write requests can be processed by the pool
    ///         (if false, then pool goes to read-only state). True by default.
    /// force: Whether we should apply transaction (for example, move pool to read-only state)
    ///        without waiting for consensus of this transaction.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_pool_config_request(indy_handle_t command_handle,
                                                       const char *  submitter_did,
                                                       indy_bool_t    writes,
                                                       indy_bool_t    force,

                                                       void           (*cb)(indy_handle_t command_handle_,
                                                                            indy_error_t  err,
                                                                            const char*   request_json)
                                                       );

    /// Builds a POOL_RESTART request.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: Id of Identity stored in secured Wallet.
    /// action: Either start or cancel
    /// datetime:
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_pool_restart_request(indy_handle_t command_handle,
                                                        const char *  submitter_did,
                                                        const char *  action,
                                                        const char *  datetime,
                                                        void           (*cb)(indy_handle_t command_handle_,
                                                                             indy_error_t  err,
                                                                             const char*   request_json)
                                                        );

    /// Builds a POOL_UPGRADE request. Request to upgrade the Pool (sent by Trustee).
    /// It upgrades the specified Nodes (either all nodes in the Pool, or some specific ones).
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// name: Human-readable name for the upgrade.
    /// version: The version of indy-node package we perform upgrade to.
    ///          Must be greater than existing one (or equal if reinstall flag is True).
    /// action: Either start or cancel.
    /// sha256: sha256 hash of the package.
    /// timeout: (Optional) Limits upgrade time on each Node.
    /// schedule: (Optional) Schedule of when to perform upgrade on each node. Map Node DIDs to upgrade time.
    /// justification: (Optional) justification string for this particular Upgrade.
    /// reinstall: Whether it's allowed to re-install the same version. False by default.
    /// force: Whether we should apply transaction (schedule Upgrade) without waiting
    ///        for consensus of this transaction.
    /// package: (Optional) Package to be upgraded.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_pool_upgrade_request(indy_handle_t command_handle,
                                                        const char *  submitter_did,
                                                        const char *  name,
                                                        const char *  version,
                                                        const char *  action,
                                                        const char *  sha256,
                                                        indy_i32_t    timeout,
                                                        const char *  schedule,
                                                        const char *  justification,
                                                        indy_bool_t   reinstall,
                                                        indy_bool_t   force,
                                                        const char *  package_,

                                                        void           (*cb)(indy_handle_t command_handle_,
                                                                             indy_error_t  err,
                                                                             const char*   request_json)
                                                        );

    /// Builds a REVOC_REG_DEF request. Request to add the definition of revocation registry
    /// to an exists credential definition.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// data: Revocation Registry data:
    ///     {
    ///         "id": string - ID of the Revocation Registry,
    ///         "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
    ///         "tag": string - Unique descriptive ID of the Registry,
    ///         "credDefId": string - ID of the corresponding CredentialDefinition,
    ///         "value": Registry-specific data {
    ///             "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
    ///             "maxCredNum": number - Maximum number of credentials the Registry can serve.
    ///             "tailsHash": string - Hash of tails.
    ///             "tailsLocation": string - Location of tails file.
    ///             "publicKeys": <public_keys> - Registry's public key.
    ///         },
    ///         "ver": string - version of revocation registry definition json.
    ///     }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_revoc_reg_def_request(indy_handle_t command_handle,
                                                         const char *  submitter_did,
                                                         const char *  data,

                                                         void           (*cb)(indy_handle_t command_handle_,
                                                                              indy_error_t  err,
                                                                              const char*   request_json)
                                                         );

    /// Builds a GET_REVOC_REG_DEF request. Request to get a revocation registry definition,
    /// that Issuer creates for a particular Credential Definition.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// id:  ID of Revocation Registry Definition in ledger.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_get_revoc_reg_def_request(indy_handle_t command_handle,
                                                             const char *  submitter_did,
                                                             const char *  id,

                                                             void           (*cb)(indy_handle_t command_handle_,
                                                                                  indy_error_t  err,
                                                                                  const char*   request_json)
                                                            );

    /// Parse a GET_REVOC_REG_DEF response to get Revocation Registry Definition in the format
    /// compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_revoc_reg_def_response: response of GET_REVOC_REG_DEF request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Revocation Registry Definition Id and Revocation Registry Definition json.
    /// {
    ///     "id": string - ID of the Revocation Registry,
    ///     "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
    ///     "tag": string - Unique descriptive ID of the Registry,
    ///     "credDefId": string - ID of the corresponding CredentialDefinition,
    ///     "value": Registry-specific data {
    ///         "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
    ///         "maxCredNum": number - Maximum number of credentials the Registry can serve.
    ///         "tailsHash": string - Hash of tails.
    ///         "tailsLocation": string - Location of tails file.
    ///         "publicKeys": <public_keys> - Registry's public key.
    ///     },
    ///     "ver": string - version of revocation registry definition json.
    /// }
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_parse_get_revoc_reg_def_response(indy_handle_t command_handle,
                                                              const char *  get_revoc_ref_def_response,

                                                              void           (*cb)(indy_handle_t command_handle_,
                                                                                   indy_error_t  err,
                                                                                   const char*   revoc_reg_def_id,
                                                                                   const char*   revoc_reg_def_json)
                                                             );

    /// Builds a REVOC_REG_ENTRY request.  Request to add the RevocReg entry containing
    /// the new accumulator value and issued/revoked indices.
    /// This is just a delta of indices, not the whole list.
    /// So, it can be sent each time a new credential is issued/revoked.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the submitter stored in secured Wallet.
    /// revoc_reg_def_id: ID of the corresponding RevocRegDef.
    /// rev_def_type: Revocation Registry type (only CL_ACCUM is supported for now).
    /// value: Registry-specific data: {
    ///     value: {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> - an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     ver: string - version revocation registry entry json
    ///
    /// }
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_revoc_reg_entry_request(indy_handle_t command_handle,
                                                           const char *  submitter_did,
                                                           const char *  revoc_reg_def_id,
                                                           const char *  rev_def_type,
                                                           const char *  value,

                                                           void           (*cb)(indy_handle_t command_handle_,
                                                                                indy_error_t  err,
                                                                                const char*   request_json)
                                                          );

    /// Builds a GET_REVOC_REG request. Request to get the accumulated state of the Revocation Registry
    /// by ID. The state is defined by the given timestamp.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// revoc_reg_def_id:  ID of the corresponding Revocation Registry Definition in ledger.
    /// timestamp: Requested time represented as a total number of seconds from Unix Epoch
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_get_revoc_reg_request(indy_handle_t command_handle,
                                                         const char *  submitter_did,
                                                         const char *  revoc_reg_def_id,
                                                         long long    timestamp,

                                                         void           (*cb)(indy_handle_t command_handle_,
                                                                              indy_error_t  err,
                                                                              const char*   request_json)
                                                        );

    /// Parse a GET_REVOC_REG response to get Revocation Registry in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_revoc_reg_response: response of GET_REVOC_REG request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Revocation Registry Definition Id, Revocation Registry json and Timestamp.
    /// {
    ///     "value": Registry-specific data {
    ///         "accum": string - current accumulator value
    ///     },
    ///     "ver": string - version revocation registry json
    /// }
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_parse_get_revoc_reg_response(indy_handle_t command_handle,
                                                          const char *  get_revoc_reg_response,

                                                          void           (*cb)(indy_handle_t command_handle_,
                                                                               indy_error_t  err,
                                                                               const char*   revoc_reg_def_id,
                                                                               const char*   revoc_reg_json,
                                                                               unsigned long long      timestamp)
                                                         );

    /// Builds a GET_REVOC_REG_DELTA request. Request to get the delta of the accumulated state of the Revocation Registry.
    /// The Delta is defined by from and to timestamp fields.
    /// If from is not specified, then the whole state till to will be returned.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
    /// revoc_reg_def_id:  ID of the corresponding Revocation Registry Definition in ledger.
    /// from: Requested time represented as a total number of seconds from Unix Epoch
    /// to: Requested time represented as a total number of seconds from Unix Epoch
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_build_get_revoc_reg_delta_request(indy_handle_t command_handle,
                                                               const char *  submitter_did,
                                                               const char *  revoc_reg_def_id,
                                                               long long    from,
                                                               long long    to,

                                                               void           (*cb)(indy_handle_t command_handle_,
                                                                                    indy_error_t  err,
                                                                                    const char*   request_json)
                                                              );

    /// Parse a GET_REVOC_REG_DELTA response to get Revocation Registry Delta in the format compatible with Anoncreds API.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// get_revoc_reg_response: response of GET_REVOC_REG_DELTA request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Revocation Registry Definition Id, Revocation Registry Delta json and Timestamp.
    /// {
    ///     "value": Registry-specific data {
    ///         prevAccum: string - previous accumulator value.
    ///         accum: string - current accumulator value.
    ///         issued: array<number> - an array of issued indices.
    ///         revoked: array<number> an array of revoked indices.
    ///     },
    ///     "ver": string - version revocation registry delta json
    /// }
    ///
    /// #Errors
    /// Common*

    extern indy_error_t indy_parse_get_revoc_reg_delta_response(indy_handle_t command_handle,
                                                                const char *  get_revoc_reg_delta_response,

                                                                void           (*cb)(indy_handle_t command_handle_,
                                                                                     indy_error_t  err,
                                                                                     const char*   revoc_reg_def_id,
                                                                                     const char*   revoc_reg_delta_json,
                                                                                     unsigned long long      timestamp)
                                                               );


    /// Parse transaction response to fetch metadata.
    /// The important use case for this method is validation of Node's response freshens.
    ///
    /// Distributed Ledgers can reply with outdated information for consequence read request after write.
    /// To reduce pool load libindy sends read requests to one random node in the pool.
    /// Consensus validation is performed based on validation of nodes multi signature for current ledger Merkle Trie root.
    /// This multi signature contains information about the latest ldeger's transaction ordering time and sequence number that this method returns.
    ///
    /// If node that returned response for some reason is out of consensus and has outdated ledger
    /// it can be caught by analysis of the returned latest ledger's transaction ordering time and sequence number.
    ///
    /// There are two ways to filter outdated responses:
    ///     1) based on "seqNo" - sender knows the sequence number of transaction that he consider as a fresh enough.
    ///     2) based on "txnTime" - sender knows the timestamp that he consider as a fresh enough.
    ///
    /// Note: response of GET_VALIDATOR_INFO request isn't supported
    ///
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// response: response of write or get request.
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// response metadata.
    /// {
    ///     "seqNo": Option<u64> - transaction sequence number,
    ///     "txnTime": Option<u64> - transaction ordering time,
    ///     "lastSeqNo": Option<u64> - the latest transaction seqNo for particular Node,
    ///     "lastTxnTime": Option<u64> - the latest transaction ordering time for particular Node
    /// }
    ///
    /// #Errors
    /// Common*
    /// Ledger*
    extern indy_error_t indy_get_response_metadata(indy_handle_t command_handle,
                                                   const char *  response,

                                                   void           (*cb)(indy_handle_t command_handle_,
                                                                        indy_error_t  err,
                                                                        const char*   response_metadata)
                                                  );

    /// Builds a AUTH_RULE request. Request to change authentication rules for a ledger transaction.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// txn_type: ledger transaction alias or associated value.
    /// action: type of an action.
    ///     Can be either "ADD" (to add a new rule) or "EDIT" (to edit an existing one).
    /// field: transaction field.
    /// old_value: (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action).
    /// new_value: (Optional) new value that can be used to fill the field.
    /// constraint: set of constraints required for execution of an action in the following format
    ///     {
    ///         constraint_id - <string> type of a constraint.
    ///             Can be either "ROLE" to specify final constraint or  "AND"/"OR" to combine constraints.
    ///         role - <string> role of a user which satisfy to constrain.
    ///         sig_count - <u32> the number of signatures required to execution action.
    ///         need_to_be_owner - <bool> if user must be an owner of transaction.
    ///         metadata - <object> additional parameters of the constraint.
    ///     }
    /// can be combined by
    ///     {
    ///         'constraint_id': <"AND" or "OR">
    ///         'auth_constraints': [<constraint_1>, <constraint_2>]
    ///     }
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_build_auth_rule_request(indy_handle_t command_handle,
                                                     const char *  submitter_did,
                                                     const char *  txn_type,
                                                     const char *  action,
                                                     const char *  field,
                                                     const char *  old_value,
                                                     const char *  new_value,
                                                     const char *  constraint,

                                                     void           (*cb)(indy_handle_t command_handle_,
                                                                          indy_error_t  err,
                                                                          const char*   request_json)
                                                    );

    /// Builds a GET_AUTH_RULE request. Request to get authentication rules for a ledger transaction.
    ///
    /// NOTE: Either none or all transaction related parameters must be specified (`old_value` can be skipped for `ADD` action).
    ///     * none - to get all authentication rules for all ledger transactions
    ///     * all - to get authentication rules for specific action (`old_value` can be skipped for `ADD` action)
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the read request sender.
    /// txn_type: (Optional) target ledger transaction alias or associated value.
    /// action: (Optional) target action type. Can be either "ADD" or "EDIT".
    /// field: (Optional) target transaction field.
    /// old_value: (Optional) old value of field, which can be changed to a new_value (mandatory for EDIT action).
    /// new_value: (Optional) new value that can be used to fill the field.
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_build_get_auth_rule_request(indy_handle_t command_handle,
                                                         const char *  submitter_did,
                                                         const char *  txn_type,
                                                         const char *  action,
                                                         const char *  field,
                                                         const char *  old_value,
                                                         const char *  new_value,

                                                         void           (*cb)(indy_handle_t command_handle_,
                                                                              indy_error_t  err,
                                                                              const char*   request_json)
                                                        );

    /// Builds a TXN_AUTHR_AGRMT request. Request to add a new version of Transaction Author Agreement to the ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the request sender.
    /// text: a content of the TTA.
    /// version: a version of the TTA (unique UTF-8 string).
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_build_txn_author_agreement_request(indy_handle_t command_handle,
                                                                const char *  submitter_did,
                                                                const char *  text,
                                                                const char *  version,

                                                                void           (*cb)(indy_handle_t command_handle_,
                                                                                     indy_error_t  err,
                                                                                     const char*   request_json)
                                                               );

    /// Builds a GET_TXN_AUTHR_AGRMT request. Request to get a specific Transaction Author Agreement from the ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the request sender.
    /// data: (Optional) specifies a condition for getting specific TAA.
    /// Contains 3 mutually exclusive optional fields:
    /// {
    ///     hash: Optional<str> - hash of requested TAA,
    ///     version: Optional<str> - version of requested TAA.
    ///     timestamp: Optional<u64> - ledger will return TAA valid at requested timestamp.
    /// }
    /// Null data or empty JSON are acceptable here. In this case, ledger will return the latest version of TAA.
    ///
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_build_get_txn_author_agreement_request(indy_handle_t command_handle,
                                                         const char *  submitter_did,
                                                         const char *  data,

                                                         void           (*cb)(indy_handle_t command_handle_,
                                                                              indy_error_t  err,
                                                                              const char*   request_json)
                                                        );

    /// Builds a SET_TXN_AUTHR_AGRMT_AML request. Request to add a new list of acceptance mechanisms for transaction author agreement.
    /// Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: DID of the request sender.
    /// aml: a set of new acceptance mechanisms:
    /// {
    ///     “<acceptance mechanism label 1>”: { acceptance mechanism description 1},
    ///     “<acceptance mechanism label 2>”: { acceptance mechanism description 2},
    ///     ...
    /// }
    /// version: a version of new acceptance mechanisms. (Note: unique on the Ledger)
    /// aml_context: (Optional) common context information about acceptance mechanisms (may be a URL to external resource).
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_build_acceptance_mechanisms_request(indy_handle_t command_handle,
                                                                 const char *  submitter_did,
                                                                 const char *  aml,
                                                                 const char *  version,
                                                                 const char *  aml_context,

                                                                 void           (*cb)(indy_handle_t command_handle_,
                                                                                      indy_error_t  err,
                                                                                      const char*   request_json)
                                                                );

    /// Builds a GET_TXN_AUTHR_AGRMT_AML request. Request to get a list of  acceptance mechanisms from the ledger
    /// valid for specified time or the latest one.
    ///
    /// EXPERIMENTAL
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// submitter_did: (Optional) DID of the request sender.
    /// timestamp: i64 - time to get an active acceptance mechanisms. Pass -1 to get the latest one.
    /// version: (Optional) version of acceptance mechanisms.
    /// cb: Callback that takes command result as parameter.
    ///
    /// NOTE: timestamp and version cannot be specified together.
    ///
    /// #Returns
    /// Request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_build_get_acceptance_mechanisms_request(indy_handle_t command_handle,
                                                                     const char *  submitter_did,
                                                                     indy_i64_t  timestamp,
                                                                     const char *  version,

                                                                     void           (*cb)(indy_handle_t command_handle_,
                                                                                          indy_error_t  err,
                                                                                          const char*   request_json)
                                                                    );

    /// Append transaction author agreement acceptance data to a request.
    /// This function should be called before signing and sending a request
    /// if there is any transaction author agreement set on the Ledger.
    ///
    /// EXPERIMENTAL
    ///
    /// This function may calculate hash by itself or consume it as a parameter.
    /// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
    ///
    /// #Params
    /// command_handle: command handle to map callback to caller context.
    /// request_json: original request data json.
    /// text and version - (optional) raw data about TAA from ledger.
    ///     These parameters should be passed together.
    ///     These parameters are required if taa_digest parameter is omitted.
    /// taa_digest - (optional) hash on text and version. This parameter is required if text and version parameters are omitted.
    /// mechanism - mechanism how user has accepted the TAA
    /// time - UTC timestamp when user has accepted the TAA
    /// cb: Callback that takes command result as parameter.
    ///
    /// #Returns
    /// Updated request result as json.
    ///
    /// #Errors
    /// Common*
    extern indy_error_t indy_append_txn_author_agreement_acceptance_to_request(indy_handle_t command_handle,
                                                                               const char *  request_json,
                                                                               const char *  text,
                                                                               const char *  version,
                                                                               const char *  taa_digest,
                                                                               const char *  mechanism,
                                                                               indy_u64_t    time,

                                                                               void           (*cb)(indy_handle_t command_handle_,
                                                                                                    indy_error_t  err,
                                                                                                    const char*   request_with_meta_json)
                                                                               );

#ifdef __cplusplus
}
#endif

#endif
