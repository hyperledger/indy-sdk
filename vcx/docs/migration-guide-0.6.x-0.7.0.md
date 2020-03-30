# LibVCX migration guide from 0.6.x to 0.7.0 and 0.7.1

## A Developer Guide for LibVCX migration

This document is written for developers using LibVCX to provide necessary information and
to simplify their transition to LibVCX 0.7.x from LibVCX 0.6.x.

* [API](#api)
    * [Message Getters](#message-getters)
    * [Connection](#connection)

#### Message Getters

Removed `connection_handle` from functions to get protocol messages. Here is the list of affected functions:
* `vcx_credential_get_request_msg` - gets the credential request message.
* `vcx_issuer_get_credential_offer_msg` - gets the offer message.
* `vcx_issuer_get_credential_msg` - gets the credential message.
* `vcx_proof_get_request_msg` - gets the proof request message.
* `vcx_get_proof` - gets the proof message.
   
#### Connection

Added new functions to get information about connection object:
* `vcx_connection_get_pw_did` - gets the DID of the current side (`pw_did`) associated with this connection.
* `vcx_connection_get_their_pw_did` - gets DID of the other side (`their_pw_did`) associated with this connection.
* `vcx_connection_info` - gets all information about this connection.
   
Added ability to accept a duplicate connection by redirecting to the already existing one instead of forming a duplicate connection. 
* `vcx_connection_redirect` - redirect a new connection to already existing one.
* `vcx_connection_get_redirect_details` - gets redirection information.
   
Example flow:   
1. Faber sends invite to ALice.
2. Alice creates a connection with Faber.
3. Faber exchange messages with ALice.
3. Faber sends a new invite to Alice. 
4. Alice creates a new Connection object and redirects it to existing ont with `vcx_connection_redirect`.
5. Faber receives redirection response congaing old Alice DIDs/Keys.
6. Faber exchange messages with ALice.

#### Disclose Proof
 
Added a new function `vcx_disclosed_proof_decline_presentation_request` to explicitly reject a presentation request.

There are two rejecting options:
- Prover wants to propose using a different presentation - pass `proposal` parameter
- Prover doesn't want to continue interaction - pass `reason` parameter.

#### Protocols compatibility
Supported `protocol_version`: `3.0` which actually is an alternative to combination of settings: `protocol_version`: `3.0` and `communication_method`: `aries`.

Added a new enum variant `Pending` for IssuerCredentials/Credentials/Proofs/DisclosedProofs objects.
Initially create `Pending` objects and convert them to V1/V3 after receiving the connection handle.
`Pending` objects have `3.0` versions during serialization.
