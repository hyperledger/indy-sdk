#ifndef __VCX_H
#define __VCX_H

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned int vcx_error_t;
typedef unsigned int vcx_schema_handle_t;
typedef unsigned int vcx_credentialdef_handle_t;
typedef unsigned int vcx_issuer_credential_handle_t;
typedef unsigned int vcx_disclosed_proof_handle_t;
typedef unsigned int vcx_connection_handle_t;
typedef unsigned int vcx_credential_handle_t;
typedef unsigned int vcx_proof_handle_t;
typedef unsigned int vcx_command_handle_t;
typedef unsigned int vcx_payment_handle_t;
typedef unsigned int vcx_wallet_search_handle_t;
typedef unsigned int vcx_bool_t;
typedef unsigned int count_t;
typedef unsigned long vcx_price_t;
typedef unsigned int vcx_u32_t;

typedef enum
{
  none = 0,
  initialized,
  offer_sent,
  request_received,
  accepted,
  unfulfilled,
  expired,
  revoked,
} vcx_state_t;

typedef enum
{
  undefined = 0,
  validated = 1,
  invalid = 2,
} vcx_proof_state_t;

// Initialize payment plugin
//
// #Returns
// Success
vcx_error_t nullpay_init();

// Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
//
// libvcx will be inoperable and must be initialized again with vcx_init_with_config
//
// #Params
// delete: specify whether wallet/pool should be deleted

// Provision an agent in the agency, populate configuration and wallet for this agent.
// NOTE: for synchronous call use vcx_provision_agent
//
// #Params
// command_handle: command handle to map callback to user context.
//
// json: configuration
//
// cb: Callback that provides configuration or error status
//
// #Returns
// Configuration (wallet also populated), on error returns NULL
vcx_error_t vcx_agent_provision_async(vcx_command_handle_t command_handle,
                                   const char *json,
                                   void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Update information on the agent (ie, comm method and type)
//
// #Params
// command_handle: command handle to map callback to user context.
//
// json: updated configuration
//
// cb: Callback that provides configuration or error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_agent_update_info(vcx_command_handle_t command_handle,
                               const char *json,
                               void (*cb)(vcx_command_handle_t, vcx_error_t));

// Establishes connection between institution and its user
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: Connection handle that identifies connection object
//
// connection_options: Provides details indicating if the connection will be established by text or QR Code
//
// # Examples connection_options -> "{"connection_type":"SMS","phone":"123"}" OR: "{"connection_type":"QR","phone":""}"
//
// cb: Callback that provides error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_connect(vcx_command_handle_t command_handle,
                                vcx_connection_handle_t connection_handle,
                                const char *connection_options,
                                void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// -> Create a Connection object that provides a pairwise connection for an institution's user
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: institution's personal identification for the user
//
// cb: Callback that provides connection handle and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_create(vcx_command_handle_t command_handle,
                               const char *source_id,
                               void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_connection_handle_t));

// Create a Connection object from the given invite_details that provides a pairwise connection.
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: institution's personal identification for the user
//
// invite_details: Provided via the other end of the connection calling "vcx_connection_connect" or "vcx_connection_invite_details"
//
// cb: Callback that provides connection handle and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_create_with_invite(vcx_command_handle_t command_handle,
                                           const char *source_id,
                                           const char *invite_details,
                                           void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_connection_handle_t));

// Delete a Connection object and release its handle
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: handle of the connection to delete.
//
// cb: Callback that provides feedback of the api call.
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_delete_connection(vcx_command_handle_t command_handle,
                                          vcx_connection_handle_t connection_handle,
                                          void (*cb)(vcx_command_handle_t, vcx_error_t));

// Takes a json string representing a connection object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_data: json string representing a connection object
//
// cb: Callback that provides credential handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_deserialize(vcx_command_handle_t command_handle,
                                    const char *connection_data,
                                    void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_connection_handle_t));

// Get the current state of the connection object
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Connection handle that was provided during creation. Used to access connection object
//
// cb: Callback that provides most current state of the connection and error status of request
//
// #Returns
vcx_error_t vcx_connection_get_state(vcx_command_handle_t command_handle,
                                  vcx_connection_handle_t connection_handle,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Gets the current connection details
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: was provided during creation. Used to identify connection object
//
// abbreviated: abbreviated connection details for QR codes or not
//
// cb: Callback that provides the json string of details
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_invite_details(vcx_command_handle_t command_handle,
                                       vcx_connection_handle_t connection_handle,
                                       vcx_bool_t abbreviated,
                                       void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Releases the connection object by de-allocating memory
//
// #Params
// connection_handle: was provided during creation. Used to identify connection object
//
// #Returns
// Success
vcx_error_t vcx_connection_release(vcx_connection_handle_t connection_handle);

// Takes the Connection object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: Callback that provides json string of the connection's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_serialize(vcx_command_handle_t command_handle,
                                  vcx_connection_handle_t connection_handle,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Checks for any state change in the connection and updates the the state attribute
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: was provided during creation. Used to identify connection object
//
// cb: Callback that provides most current state of the credential and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_connection_update_state(vcx_command_handle_t command_handle,
                                     vcx_connection_handle_t connection_handle,
                                     void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Create a Credential object that requests and receives a credential for an institution
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Institution's personal identification for the credential, should be unique.
//
// connection_handle: connection to query for credential offer
//
// msg_id: msg_id that contains the credential offer
//
// cb: Callback that provides credential handle or error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_create_with_msgid(vcx_command_handle_t command_handle,
                                          const char *source_id,
                                          vcx_credential_handle_t connection_handle,
                                          const char *msg_id,
                                          void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_credential_handle_t, const char*));

// Create a Credential object that requests and receives a credential for an institution
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Institution's personal identification for the credential, should be unique.
//
// offer: credential offer received via "vcx_credential_get_offers"
//
// # Example offer -> "[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]
//
// cb: Callback that provides credential handle or error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_create_with_offer(vcx_command_handle_t command_handle,
                                          const char *source_id,
                                          const char *offer,
                                          void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_credential_handle_t));

// Takes a json string representing an credential object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_data: json string representing a credential object
//
//
// cb: Callback that provides credential handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_deserialize(vcx_command_handle_t command_handle,
                                    const char *credential_data,
                                    void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_credential_handle_t));

// Queries agency for credential offers from the given connection.
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: Connection to query for credential offers.
//
// cb: Callback that provides any credential offers and error status of query
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_get_offers(vcx_command_handle_t command_handle,
                                   vcx_credential_handle_t connection_handle,
                                   void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Retrieves Payment Info from a Credential
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: credential handle that was provided during creation. Used to identify credential object
//
// cb: Callback that provides Payment Info of a Credential
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_get_payment_info(vcx_command_handle_t command_handle,
                                         vcx_credential_handle_t credential_handle,
                                         void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Retrieve the txn associated with paying for the credential
//
// #param
// handle: credential handle that was provided during creation.  Used to access credential object.
//
// #Callback returns
// PaymentTxn json
// example: {
// "amount":25,
// "inputs":[
// "pay:null:1_3FvPC7dzFbQKzfG",
// "pay:null:1_lWVGKc07Pyc40m6"
// ],
// "outputs":[
// {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null},
// {"recipient":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j","amount":25,"extra":null}
// ]
// }
vcx_error_t vcx_credential_get_payment_txn(vcx_command_handle_t command_handle,
                                        vcx_credential_handle_t handle,
                                        void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Get the current state of the credential object
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Credential handle that was provided during creation.
//
// cb: Callback that provides most current state of the credential and error status of request
//
// #Returns
vcx_error_t vcx_credential_get_state(vcx_command_handle_t command_handle,
                                  vcx_credential_handle_t handle,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Releases the credential object by de-allocating memory
//
// #Params
// handle: Credential handle that was provided during creation. Used to access credential object
//
// #Returns
// Success
vcx_error_t vcx_credential_release(vcx_credential_handle_t handle);

// Send a credential request to the connection, called after having received a credential offer
//
// #params
// command_handle: command handle to map callback to user context
//
// credential_handle: credential handle that was provided during creation. Used to identify credential object
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: Callback that provides error status of credential request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_send_request(vcx_command_handle_t command_handle,
                                     vcx_credential_handle_t credential_handle,
                                     vcx_connection_handle_t connection_handle,
                                     vcx_payment_handle_t payment_handle,
                                     void (*cb)(vcx_command_handle_t, vcx_error_t));

// Takes the credential object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// handle: Credential handle that was provided during creation. Used to identify credential object
//
// cb: Callback that provides json string of the credential's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_serialize(vcx_command_handle_t command_handle,
                                  vcx_credential_handle_t handle,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Checks for any state change in the credential and updates the the state attribute.  If it detects a credential it
// will store the credential in the wallet and update the state.
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: Credential handle that was provided during creation. Used to identify credential object
//
// cb: Callback that provides most current state of the credential and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credential_update_state(vcx_command_handle_t command_handle,
                                     vcx_credential_handle_t credential_handle,
                                     void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Create a new CredentialDef object that can create credential definitions on the ledger
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Enterprise's personal identification for the user.
//
// credentialdef_name: Name of credential definition
//
// schema_id: The schema id given during the creation of the schema
//
// issuer_did: did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
//
// tag: way to create a unique credential def with the same schema and issuer did.
//
// config: revocation info
//
// cb: Callback that provides CredentialDef handle and error status of request.
//
// payment_handle: future use (currently uses any address in wallet)
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credentialdef_create(vcx_command_handle_t command_handle,
                                  const char *source_id,
                                  const char *credentialdef_name,
                                  const char *schema_id,
                                  const char *issuer_did,
                                  const char *tag,
                                  const char *config,
                                  vcx_payment_handle_t payment_handle,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_credential_handle_t));

// Takes a json string representing a credentialdef object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credentialdef_data: json string representing a credentialdef object
//
// cb: Callback that provides credentialdef handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credentialdef_deserialize(vcx_command_handle_t command_handle,
                                       const char *credentialdef_data,
                                       void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_credential_handle_t));

// Retrieves credential definition's id
//
// #Params
// cred_def_handle: CredDef handle that was provided during creation. Used to access proof object
//
// cb: Callback that provides credential definition id and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credentialdef_get_cred_def_id(vcx_command_handle_t command_handle,
                                           vcx_credential_handle_t cred_def_handle,
                                           void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Retrieve the txn associated with paying for the credential_def
//
// #param
// handle: credential_def handle that was provided during creation.  Used to access credential_def object.
//
// #Callback returns
// PaymentTxn json
// example: {
// "amount":25,
// "inputs":[
// "pay:null:1_3FvPC7dzFbQKzfG",
// "pay:null:1_lWVGKc07Pyc40m6"
// ],
// "outputs":[
// {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null},
// {"recipient":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j","amount":25,"extra":null}
// ]
// }
vcx_error_t vcx_credentialdef_get_payment_txn(vcx_command_handle_t command_handle,
                                           vcx_credential_handle_t handle,
                                           void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Releases the credentialdef object by de-allocating memory
//
// #Params
// handle: Proof handle that was provided during creation. Used to access credential object
//
// #Returns
// Success
vcx_error_t vcx_credentialdef_release(vcx_credential_handle_t credentialdef_handle);

// Takes the credentialdef object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
//
// cb: Callback that provides json string of the credentialdef's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_credentialdef_serialize(vcx_command_handle_t command_handle,
                                     vcx_credential_handle_t credentialdef_handle,
                                     void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Create a proof for fulfilling a corresponding proof request
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Institution's personal identification for the proof, should be unique.
//
// connection: connection to query for proof request
//
// msg_id: msg_id that contains the proof request
//
// cb: Callback that provides proof handle and proof request or error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_create_with_msgid(vcx_command_handle_t command_handle,
                                               const char *source_id,
                                               vcx_connection_handle_t connection_handle,
                                               const char *msg_id,
                                               void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_disclosed_proof_handle_t, const char*));

// Create a proof for fulfilling a corresponding proof request
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Institution's identification for the proof, should be unique.
//
// req: proof request received via "vcx_get_proof_requests"
//
// cb: Callback that provides proof handle or error status
//
// #Returns
// Error code as u32
vcx_error_t vcx_disclosed_proof_create_with_request(vcx_command_handle_t command_handle,
                                                 const char *source_id,
                                                 const char *proof_req,
                                                 void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_disclosed_proof_handle_t));

// Takes a json string representing an disclosed proof object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// data: json string representing a disclosed proof object
//
//
// cb: Callback that provides handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_deserialize(vcx_command_handle_t command_handle,
                                         const char *proof_data,
                                         void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_disclosed_proof_handle_t));

// Takes the disclosed proof object and generates a proof from the selected credentials and self attested attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
//
// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
//
// selected_credentials: a json string with a credential for each proof request attribute.
// List of possible credentials for each attribute is returned from vcx_disclosed_proof_retrieve_credentials
// # Examples selected_credential -> "{"req_attr_0":cred_info}" Where cred_info is returned from retrieve credentials
//
// self_attested_attrs: a json string with attributes self attested by user
// # Examples self_attested_attrs -> "{"self_attested_attr_0":"attested_val"}"
//
// cb: Callback that returns error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_generate_proof(vcx_command_handle_t command_handle,
                                            vcx_disclosed_proof_handle_t proof_handle,
                                            const char *selected_credentials,
                                            const char *self_attested_attrs,
                                            void (*cb)(vcx_command_handle_t, vcx_error_t));

// Queries agency for proof requests from the given connection.
//
// #Params
// command_handle: command handle to map callback to user context.
//
// connection_handle: Connection to query for proof requests.
//
// cb: Callback that provides any proof requests and error status of query
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_get_requests(vcx_command_handle_t command_handle,
                                          vcx_disclosed_proof_handle_t connection_handle,
                                          void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Get the current state of the disclosed proof object
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Proof handle that was provided during creation. Used to access disclosed proof object
//
// cb: Callback that provides most current state of the disclosed proof and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_get_state(vcx_command_handle_t command_handle,
                                       vcx_disclosed_proof_handle_t proof_handle,
                                       void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Releases the disclosed proof object by de-allocating memory
//
// #Params
// handle: Proof handle that was provided during creation. Used to access proof object
//
// #Returns
// Success
vcx_error_t vcx_disclosed_proof_release(vcx_disclosed_proof_handle_t handle);

// Takes the disclosed proof object and returns a json string of all credentials matching associated proof request from wallet
//
// #Params
// command_handle: command handle to map callback to user context.
//
// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
//
// cb: Callback that provides json string of the credentials in wallet associated with proof request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_retrieve_credentials(vcx_command_handle_t command_handle,
                                                  vcx_disclosed_proof_handle_t proof_handle,
                                                  void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Send a proof to the connection, called after having received a proof request
//
// #params
// command_handle: command handle to map callback to API user context.
//
// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: Callback that provides error status of proof send request
//
// #Returns
// Error code as u32
vcx_error_t vcx_disclosed_proof_send_proof(vcx_command_handle_t command_handle,
                                        vcx_disclosed_proof_handle_t proof_handle,
                                        vcx_connection_handle_t connection_handle,
                                        void (*cb)(vcx_command_handle_t, vcx_error_t));

// Takes the disclosed proof object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
//
// cb: Callback that provides json string of the disclosed proof's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_serialize(vcx_command_handle_t command_handle,
                                       vcx_disclosed_proof_handle_t proof_handle,
                                       void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Checks for any state change in the disclosed proof and updates the the state attribute
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Credential handle that was provided during creation. Used to identify disclosed proof object
//
// cb: Callback that provides most current state of the disclosed proof and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_disclosed_proof_update_state(vcx_command_handle_t command_handle,
                                          vcx_disclosed_proof_handle_t proof_handle,
                                          void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

const char *vcx_error_c_message(vcx_error_t error_code);

// Retrieve information about a stored credential in user's wallet, including credential id and the credential itself.
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: credential handle that was provided during creation. Used to identify credential object
//
// cb: Callback that provides error status of api call, or returns the credential in json format of "{uuid:credential}".
//
// #Returns
// Error code as a u32
vcx_error_t vcx_get_credential(vcx_command_handle_t command_handle,
                            vcx_credential_handle_t credential_handle,
                            void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Get Proof
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Proof handle that was provided during creation. Used to identify proof object
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: Callback that provides Proof attributes and error status of sending the credential
//
// #Returns
// Error code as a u32
vcx_error_t vcx_get_proof(vcx_command_handle_t command_handle,
                       vcx_proof_handle_t proof_handle,
                       vcx_connection_handle_t connection_handle,
                       void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_proof_state_t, const char*));

// Initializes VCX with config file
//
// An example file is at libvcx/sample_config/config.json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// config_path: path to a config file to populate config attributes
//
// cb: Callback that provides error status of initialization
//
// #Returns
// Error code as a u32
vcx_error_t vcx_init(vcx_command_handle_t command_handle, const char *config_path, void (*cb)(vcx_command_handle_t, vcx_error_t));

// Initializes VCX with config settings
//
// example configuration is in libvcx/sample_config/config.json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// config_path: path to a config file to populate config attributes
//
// cb: Callback that provides error status of initialization
//
// #Returns
// Error code as a u32
vcx_error_t vcx_init_with_config(vcx_command_handle_t command_handle,
                              const char *config,
                              void (*cb)(vcx_command_handle_t, vcx_error_t));

// Create a Issuer Credential object that provides a credential for an enterprise's user
// Assumes a credential definition has been written to the ledger.
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Enterprise's personal identification for the user.
//
// cred_def_id: id of credential definition given during creation of the credential definition
//
// issuer_did: did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
//
// credential_data: data attributes offered to person in the credential
//
// credential_name: Name of the credential - ex. Drivers Licence
//
// price: price of credential
//
// cb: Callback that provides credential handle and error status of request
//
// #Returns
// Error code as a u32
//
// # Example credential_data -> "{"state":["UT"]}"
vcx_error_t vcx_issuer_create_credential(vcx_command_handle_t command_handle,
                                      const char *source_id,
                                      const char *cred_def_id,
                                      const char *issuer_did,
                                      const char *credential_data,
                                      const char *credential_name,
                                      vcx_payment_handle_t price,
                                      void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_issuer_credential_handle_t));

// Takes a json string representing an issuer credential object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_data: json string representing a credential object
//
// cb: Callback that provides credential handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_issuer_credential_deserialize(vcx_command_handle_t command_handle,
                                           const char *credential_data,
                                           void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_issuer_credential_handle_t));

// Retrieve the txn associated with paying for the issuer_credential
//
// #param
// handle: issuer_credential handle that was provided during creation.  Used to access issuer_credential object.
//
// #Callback returns
// PaymentTxn json
// example: {
// "amount":25,
// "inputs":[
// "pay:null:1_3FvPC7dzFbQKzfG",
// "pay:null:1_lWVGKc07Pyc40m6"
// ],
// "outputs":[
// {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null},
// {"recipient":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j","amount":25,"extra":null}
// ]
// }
vcx_error_t vcx_issuer_credential_get_payment_txn(vcx_command_handle_t command_handle,
                                               vcx_issuer_credential_handle_t handle,
                                               void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Get the current state of the issuer credential object
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Issuer Credential handle that was provided during creation.
//
// cb: Callback that provides most current state of the issuer credential and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_issuer_credential_get_state(vcx_command_handle_t command_handle,
                                         vcx_issuer_credential_handle_t credential_handle,
                                         void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Releases the issuer credential object by deallocating memory
//
// #Params
// credential_handle: Credential handle that was provided during creation. Used to identify credential object
//
// #Returns
// Success
vcx_error_t vcx_issuer_credential_release(vcx_issuer_credential_handle_t credential_handle);

// Takes the credential object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: Credential handle that was provided during creation. Used to identify credential object
//
// cb: Callback that provides json string of the credential's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_issuer_credential_serialize(vcx_command_handle_t command_handle,
                                         vcx_issuer_credential_handle_t credential_handle,
                                         void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Checks for any state change in the credential and updates the the state attribute
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: Credential handle that was provided during creation. Used to identify credential object
//
// cb: Callback that provides most current state of the credential and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_issuer_credential_update_state(vcx_command_handle_t command_handle,
                                            vcx_issuer_credential_handle_t credential_handle,
                                            void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Send Credential that was requested by user
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: Credential handle that was provided during creation. Used to identify credential object
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: Callback that provides error status of sending the credential
//
// #Returns
// Error code as a u32
vcx_error_t vcx_issuer_send_credential(vcx_command_handle_t command_handle,
                                    vcx_issuer_credential_handle_t credential_handle,
                                    vcx_connection_handle_t connection_handle,
                                    void (*cb)(vcx_command_handle_t, vcx_error_t));

// Send a credential offer to user showing what will be included in the actual credential
//
// #Params
// command_handle: command handle to map callback to user context.
//
// credential_handle: Credential handle that was provided during creation. Used to identify credential object
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: Callback that provides error status of credential offer
//
// #Returns
// Error code as a u32
vcx_error_t vcx_issuer_send_credential_offer(vcx_command_handle_t command_handle,
                                          vcx_issuer_credential_handle_t credential_handle,
                                          vcx_connection_handle_t connection_handle,
                                          void (*cb)(vcx_command_handle_t, vcx_error_t));

// Get ledger fees from the sovrin network
//
// #Params
// command_handle: command handle to map callback to user context.
//
// cb: Callback that provides the fee structure for the sovrin network
//
// #Returns
// Error code as a u32
vcx_error_t vcx_ledger_get_fees(vcx_command_handle_t command_handle, void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Retrieve messages from the specified connection
//
// #params
//
// command_handle: command handle to map callback to user context.
//
// message_status: optional - query for messages with the specified status
//
// uids: optional, comma separated - query for messages with the specified uids
//
// cb: Callback that provides array of matching messages retrieved
//
// #Returns
// Error code as a u32
vcx_error_t vcx_messages_download(vcx_command_handle_t command_handle,
                               const char *message_status,
                               const char *uids,
                               const char *pw_dids,
                               void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Update the status of messages from the specified connection
//
// #params
//
// command_handle: command handle to map callback to user context.
//
// message_status: updated status
//
// msg_json: messages to update: [{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]},...]
//
// cb: Callback that provides success or failure of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_messages_update_status(vcx_command_handle_t command_handle,
                                    const char *message_status,
                                    const char *msg_json,
                                    void (*cb)(vcx_command_handle_t, vcx_error_t));

// Create a new Proof object that requests a proof for an enterprise
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Enterprise's personal identification for the user.
//
// requested_attrs: attributes/claims prover must provide in proof
//
// # Example requested_attrs -> "[{"name":"attrName","restrictions":["issuer_did":"did","schema_id":"id","schema_issuer_did":"did","schema_name":"name","schema_version":"1.1.1","cred_def_id":"id"}]]"
//
// requested_predicates: predicate specifications prover must provide claim for
//
// # Example requested_predicates -> "[{"name":"attrName","p_type":"GE","p_value":9,"restrictions":["issuer_did":"did","schema_id":"id","schema_issuer_did":"did","schema_name":"name","schema_version":"1.1.1","cred_def_id":"id"}]]"
//
//
// cb: Callback that provides proof handle and error status of request.
//
// #Returns
// Error code as a u32
vcx_error_t vcx_proof_create(vcx_command_handle_t command_handle,
                          const char *source_id,
                          const char *requested_attrs,
                          const char *requested_predicates,
                          const char *name,
                          void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_proof_handle_t));

// Takes a json string representing a proof object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_data: json string representing a proof object
//
// cb: Callback that provides proof handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_proof_deserialize(vcx_command_handle_t command_handle,
                               const char *proof_data,
                               void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_proof_handle_t));

// Get the current state of the proof object
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Proof handle that was provided during creation. Used to access proof object
//
// cb: Callback that provides most current state of the proof and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_proof_get_state(vcx_command_handle_t command_handle,
                             vcx_proof_handle_t proof_handle,
                             void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Releases the proof object by de-allocating memory
//
// #Params
// proof_handle: Proof handle that was provided during creation. Used to access proof object
//
// #Returns
// Success
vcx_error_t vcx_proof_release(vcx_proof_handle_t proof_handle);

// Sends a proof request to pairwise connection
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Proof handle that was provided during creation. Used to access proof object
//
// connection_handle: Connection handle that identifies pairwise connection
//
// cb: provides any error status of the proof_request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_proof_send_request(vcx_command_handle_t command_handle,
                                vcx_proof_handle_t proof_handle,
                                vcx_connection_handle_t connection_handle,
                                void (*cb)(vcx_command_handle_t, vcx_error_t));

// Takes the proof object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Proof handle that was provided during creation. Used to access proof object
//
// cb: Callback that provides json string of the proof's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_proof_serialize(vcx_command_handle_t command_handle,
                             vcx_proof_handle_t proof_handle,
                             void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Checks for any state change and updates the proof state attribute
//
// #Params
// command_handle: command handle to map callback to user context.
//
// proof_handle: Proof handle that was provided during creation. Used to access proof object
//
// cb: Callback that provides most current state of the proof and error status of request
//
// #Returns
// Error code as a u32
vcx_error_t vcx_proof_update_state(vcx_command_handle_t command_handle,
                                vcx_proof_handle_t proof_handle,
                                void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

// Provision an agent in the agency, populate configuration and wallet for this agent.
// NOTE: for asynchronous call use vcx_agent_provision_async
//
// #Params
// json: configuration
//
// #Returns
// Configuration (wallet also populated), on error returns NULL
char *vcx_provision_agent(const char *json);

// Create a new Schema object that can create or look up schemas on the ledger
//
// #Params
// command_handle: command handle to map callback to user context.
//
// source_id: Enterprise's personal identification for the user.
//
// schema_name: Name of schema
//
// version: version of schema
//
// schema_data: list of attributes that will make up the schema
//
// # Example schema_data -> "["attr1", "attr2", "attr3"]"
//
// payment_handle: future use (currently uses any address in the wallet)
//
// cb: Callback that provides Schema handle and error status of request.
//
// #Returns
// Error code as a u32
vcx_error_t vcx_schema_create(vcx_command_handle_t command_handle,
                           const char *source_id,
                           const char *schema_name,
                           const char *version,
                           const char *schema_data,
                           vcx_payment_handle_t payment_handle,
                           void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_schema_handle_t));

// Takes a json string representing a schema object and recreates an object matching the json
//
// #Params
// command_handle: command handle to map callback to user context.
//
// schema_data: json string representing a schema object
//
// cb: Callback that provides schema handle and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_schema_deserialize(vcx_command_handle_t command_handle,
                                const char *schema_data,
                                void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_schema_handle_t));

// Retrieves all of the data associated with a schema on the ledger.
//
// #Params
// source_id: Enterprise's personal identification for the user.
//
// schema_id: id of schema given during the creation of the schema
//
// cb: Callback contains the error status (if the schema cannot be found)
// and it will also contain a json string representing all of the data of a
// schema already on the ledger.
//
// #Returns
// Error code as a u32
vcx_error_t vcx_schema_get_attributes(vcx_command_handle_t command_handle,
                                   const char *source_id,
                                   const char *schema_id,
                                   void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_schema_handle_t, const char*));

// Retrieve the txn associated with paying for the schema
//
// #param
// handle: schema handle that was provided during creation.  Used to access schema object.
//
// #Callback returns
// PaymentTxn json
// example: {
// "amount":25,
// "inputs":[
// "pay:null:1_3FvPC7dzFbQKzfG",
// "pay:null:1_lWVGKc07Pyc40m6"
// ],
// "outputs":[
// {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null},
// {"recipient":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j","amount":25,"extra":null}
// ]
// }
vcx_error_t vcx_schema_get_payment_txn(vcx_command_handle_t command_handle,
                                    vcx_schema_handle_t handle,
                                    void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Retrieves schema's id
//
// #Params
// schema_handle: Schema handle that was provided during creation. Used to access proof object
//
// cb: Callback that provides schema id and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_schema_get_schema_id(vcx_command_handle_t command_handle,
                                  vcx_schema_handle_t schema_handle,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Releases the schema object by de-allocating memory
//
// #Params
// schema_handle: Schema handle that was provided during creation. Used to access schema object
//
// #Returns
// Success
vcx_error_t vcx_schema_release(vcx_schema_handle_t schema_handle);

// Takes the schema object and returns a json string of all its attributes
//
// #Params
// command_handle: command handle to map callback to user context.
//
// schema_handle: Schema handle that was provided during creation. Used to access schema object
//
// cb: Callback that provides json string of the schema's attributes and provides error status
//
// #Returns
// Error code as a u32
vcx_error_t vcx_schema_serialize(vcx_command_handle_t command_handle,
                              vcx_schema_handle_t schema_handle,
                              void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
//
// libvcx will be inoperable and must be initialized again with vcx_init_with_config
//
// #Params
// delete: specify whether wallet/pool should be deleted
//
// #Returns
// Success
vcx_error_t vcx_shutdown(vcx_bool_t delete);

const char *vcx_version();

// Adds a record to the wallet
// Assumes there is an open wallet.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// value: value of the record with the associated id.
//
// tags_json: the record tags used for search and storing meta information as json:
// {
// "tagName1": <str>, // string tag (will be stored encrypted)
// "tagName2": <int>, // int tag (will be stored encrypted)
// "~tagName3": <str>, // string tag (will be stored un-encrypted)
// "~tagName4": <int>, // int tag (will be stored un-encrypted)
// }
// The tags_json must be valid json, and if no tags are to be associated with the
// record, then the empty '{}' json must be passed.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
//
vcx_error_t vcx_wallet_add_record(vcx_command_handle_t command_handle,
                               const char *type_,
                               const char *id,
                               const char *value,
                               const char *tags_json,
                               void (*cb)(vcx_command_handle_t, vcx_error_t));

// Adds tags to a record.
// Assumes there is an open wallet and that a type and id pair already exists.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// tags: Tags for the record with the associated id and type.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
//
vcx_error_t vcx_wallet_add_record_tags(vcx_command_handle_t command_handle,
                                    const char *type_,
                                    const char *id,
                                    const char *tags,
                                    void (*cb)(vcx_command_handle_t, vcx_error_t));

// Close a search
//
// #Params
//
// command_handle: command handle to map callback to user context.
//
// search_handle: for future use
//
// cb: Callback that provides wallet balance
//
// #Returns
// Error code as a u32
vcx_error_t vcx_wallet_close_search(vcx_command_handle_t command_handle,
                                 vcx_wallet_search_handle_t search_handle,
                                 void (*cb)(vcx_command_handle_t, vcx_error_t));

// Add a payment address to the wallet
//
// #params
//
// cb: Callback that provides payment address info
//
// #Returns
// Error code as u32
vcx_error_t vcx_wallet_create_payment_address(vcx_command_handle_t command_handle,
                                           const char *seed,
                                           void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Deletes an existing record.
// Assumes there is an open wallet and that a type and id pair already exists.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
// Error will be a libindy error code
//
vcx_error_t vcx_wallet_delete_record(vcx_command_handle_t command_handle,
                                  const char *type_,
                                  const char *id,
                                  void (*cb)(vcx_command_handle_t, vcx_error_t));

// Deletes tags from a record.
// Assumes there is an open wallet and that a type and id pair already exists.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// tags: Tags to remove from the record with the associated id and type.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
//
vcx_error_t vcx_wallet_delete_record_tags(vcx_command_handle_t command_handle,
                                       const char *type_,
                                       const char *id,
                                       const char *tags,
                                       void (*cb)(vcx_command_handle_t, vcx_error_t));

// Exports opened wallet
//
// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
// in the future releases.
//
// #Params:
// command_handle: Handle for User's Reference only.
// path: Path to export wallet to User's File System.
// backup_key: String representing the User's Key for securing (encrypting) the exported Wallet.
// cb: Callback that provides the success/failure of the api call.
// #Returns
// Error code - success indicates that the api call was successfully created and execution
// is scheduled to begin in a separate thread.
vcx_error_t vcx_wallet_export(vcx_command_handle_t command_handle,
                           const char *path,
                           const char *backup_key,
                           void (*cb)(vcx_command_handle_t, vcx_error_t));

// Deletes an existing record.
// Assumes there is an open wallet and that a type and id pair already exists.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
// Error will be a libindy error code
//
vcx_error_t vcx_wallet_get_record(vcx_command_handle_t command_handle,
                               const char *type_,
                               const char *id,
                               const char *options_json,
                               void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Get the total balance from all addresses contained in the configured wallet
//
// #Params
//
// command_handle: command handle to map callback to user context.
//
// payment_handle: for future use
//
// cb: Callback that provides wallet balance
//
// #Returns
// Error code as a u32
vcx_error_t vcx_wallet_get_token_info(vcx_command_handle_t command_handle,
                                   vcx_payment_handle_t payment_handle,
                                   void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Creates a new secure wallet and then imports its content
// according to fields provided in import_config
// Cannot be used if wallet is already opened (Especially if vcx_init has already been used).
//
// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
// in the future releases.
//
// config: "{"wallet_name":"","wallet_key":"","exported_wallet_path":"","backup_key":""}"
// exported_wallet_path: Path of the file that contains exported wallet content
// backup_key: Key used when creating the backup of the wallet (For encryption/decrption)
// cb: Callback that provides the success/failure of the api call.
// #Returns
// Error code - success indicates that the api call was successfully created and execution
// is scheduled to begin in a separate thread.
vcx_error_t vcx_wallet_import(vcx_command_handle_t command_handle,
                           const char *config,
                           void (*cb)(vcx_command_handle_t, vcx_error_t));

// Opens a storage search handle
//
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// query_json: MongoDB style query to wallet record tags:
// {
// "tagName": "tagValue",
// $or: {
// "tagName2": { $regex: 'pattern' },
// "tagName3": { $gte: 123 },
// },
// }
// options_json:
// {
// retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
// retrieveTotalCount: (optional, false by default) Calculate total count,
// retrieveType: (optional, false by default) Retrieve record type,
// retrieveValue: (optional, true by default) Retrieve record value,
// retrieveTags: (optional, true by default) Retrieve record tags,
// }
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
vcx_error_t vcx_wallet_open_search(int32_t command_handle,
                                const char *type_,
                                const char *query_json,
                                const char *options_json,
                                void (*cb)(int32_t, vcx_error_t, int32_t));

// Fetch next records for wallet search.
//
// Not if there are no records this call returns WalletNoRecords error.
//
// #Params
// wallet_handle: wallet handle (created by open_wallet)
// wallet_search_handle: wallet wallet handle (created by indy_open_wallet_search)
// count: Count of records to fetch
//
// #Returns
// wallet records json:
// {
// totalCount: <int>, // present only if retrieveTotalCount set to true
// records: [{ // present only if retrieveRecords set to true
// id: "Some id",
// type: "Some type", // present only if retrieveType set to true
// value: "Some value", // present only if retrieveValue set to true
// tags: <tags json>, // present only if retrieveTags set to true
// }],
// }
vcx_error_t vcx_wallet_search_next_records(int32_t command_handle,
                                        int32_t wallet_search_handle,
                                        count_t count,
                                        void (*cb)(int32_t, vcx_error_t, const char*));

// Send tokens to a specific address
//
// #Params
//
// command_handle: command handle to map callback to user context.
//
// payment_handle: for future use (currently uses any address in the wallet)
//
// tokens: number of tokens to send
//
// recipient: address of recipient
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
vcx_error_t vcx_wallet_send_tokens(vcx_command_handle_t command_handle,
                                vcx_payment_handle_t payment_handle,
                                const char *tokens,
                                const char *recipient,
                                void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

// Updates the value of a record already in the wallet.
// Assumes there is an open wallet and that a type and id pair already exists.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// tags: New tags for the record with the associated id and type.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
//
vcx_error_t vcx_wallet_update_record_tags(vcx_command_handle_t command_handle,
                                       const char *type_,
                                       const char *id,
                                       const char *tags,
                                       void (*cb)(vcx_command_handle_t, vcx_error_t));

// Updates the value of a record already in the wallet.
// Assumes there is an open wallet and that a type and id pair already exists.
// #Params
//
// command_handle: command handle to map callback to user context.
//
// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
//
// id: the id ("key") of the record.
//
// value: New value of the record with the associated id.
//
// cb: Callback that any errors or a receipt of transfer
//
// #Returns
// Error code as a u32
//
vcx_error_t vcx_wallet_update_record_value(vcx_command_handle_t command_handle,
                                        const char *type_,
                                        const char *id,
                                        const char *value,
                                        void (*cb)(vcx_command_handle_t, vcx_error_t));

// Validates a Payment address
//
// #Params
//
// command_handle: command handle to map callback to user context.
//
// payment_address: value to be validated as a payment address
//
// cb: Callback that any errors
//
// #Returns
// Error code as a u32
vcx_error_t vcx_wallet_validate_payment_address(int32_t command_handle,
                                             const char *payment_address,
                                             void (*cb)(int32_t, vcx_error_t));


vcx_error_t vcx_set_default_logger( const char * pattern );
vcx_error_t vcx_set_logger( const void* context,
                            vcx_bool_t (*enabledFn)(const void*  context,
                                                      vcx_u32_t level,
                                                      const char* target),
                            void (*logFn)(const void*  context,
                                          vcx_u32_t level,
                                          const char* target,
                                          const char* message,
                                          const char* module_path,
                                          const char* file,
                                          vcx_u32_t line),
                            void (*flushFn)(const void*  context));
vcx_error_t vcx_get_logger(const void*  vcx_get_logger,
                           vcx_bool_t (**enabledFn)(const void*  context,
                                                     vcx_u32_t level,
                                                     const char* target),
                           void (**logFn)(const void*  context,
                                          vcx_u32_t level,
                                          const char* target,
                                          const char* message,
                                          const char* module_path,
                                          const char* file,
                                          vcx_u32_t line),
                           void (**flushFn)(const void*  context) );

#ifdef __cplusplus
} // extern "C"
#endif
#endif
