# Indy SDK configuration

Indy SDK contains several components, some of which can be configured. 
This configuration is useful when you need to define some specific customizations on your project. 
Like predefined config files, additional plugins, logging and else.

This document contains information on how Indy-SDK components can be configured.

* [Libindy](#libindy)
    * [Pool](#pool)
    * [Wallet](#wallet)
    * [Payment](#payment)
    * [Logging](#logging)
    * [Error Handling](#error-handling)
    * [Runtime Configuration](#runtime-configuration)
    * [Transaction Endorser](#transaction-endorser)
    * [Transaction Author Agreement](#transaction-author-agreement)
    * [Fully-Qualified Identifiers](#fully-qualified-identifiers)

* [Indy-CLI](#indy-cli)
    * [Options](#options)
    * [Config](#config)
    * [Execution mode](#execution-mode)
    * [Transaction Author Agreement](#transaction-author-agreement)

* [Vcx](#vcx)
    * [Configuration options](#configuration-options)
    * [Logging](#logging)

## Libindy

#### Pool

* Genesis Transactions - predefined transactions which will be used for a connection to Pool.
`indy_create_pool_ledger_config` API function accepts a `config` parameter that looks like:
    ```
    {
        "genesis_txn": string (optional), Path to a file containg genesis transactions.
            - If NULL, then a default one will be used (<pool_name>.txn). 
    }
    ```
    An example of genesis transactions can be found [here](../cli/docker_pool_transactions_genesis)

* Connection
`indy_open_pool_ledger` API function opens the pool ledger and forms connections to the nodes defined in the genesis transactions file.
This function accepts a `config` parameter that defines the behavior of the client-side connection and looks like:
    ```
    {
        "timeout": int (optional) - specifies the maximum number of seconds to wait for pool response (ACK, REPLY).
        "extended_timeout": int (optional), an additional number of seconds to wait for REPLY in case ACK has been received.
        "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default). 
            Libindy sends write transactions (like `NYM)` to all nodes in the ledger. 
            In case of read request (like `GET_NYM`) it's enough to receive a reply with valid `state proof` only from one node.
            By default Libindy sends a read requests to 2 (`number_read_nodes`) nodes in the pool. 
            If Reply isn't received or response `state proof` is invalid Libindy sends the request again but to 2 (`number_read_nodes`) * 2 = 4 nodes and so far until completion.
            So using `number_read_nodes` parameter you can set the number of nodes to send read requests.  
        "preordered_nodes": array<string> -  (optional), names of nodes which will have priority during request sending.
            This can be useful if a user prefers querying specific nodes.
            Assume that `Node1` and `Node2` nodes reply faster. 
            If you pass them to `preordered_nodes` parameter Libindy always sends a read request to these nodes first and only then (if not enough) to others.
            Note: Nodes not specified will be placed randomly.
            
    }
    ```

* Protocol Version - specifies the version of Indy Node which Libindy works with (There is a global property PROTOCOL_VERSION that used in every request to the pool).
Use `indy_set_protocol_version` API function to - set PROTOCOL_VERSION value which can be one of:
    ```
    1 - for Indy Node 1.3
    2 - for Indy Node 1.4 and greater
    ```

* Refresh
`indy_refresh_pool_ledger` API function updates a list of active nodes in the pool.
This function can be useful for applications working with an Indy network that is run continuously for a long time.
Note that active requests will be dropped.

* State Proof
There are some types of requests to Nodes in the Pool which support State Proof optimization in
Client-Node communication. Instead of sending requests to all nodes in the Pool, a client can send a request
to a single Node and expect a State Proof.

    Libindy allows the building and sending of custom requests via a pluggable interface (like Payment).
    `indy_register_transaction_parser_for_sp` API function allows registering of State Proof parsers for these custom requests.

#### Wallet

* Storage Type - libindy allows plugging different wallet implementations to handle storage layers. 
Libindy uses Sqlite as the default storage layer.
`indy_register_wallet_storage` API function allows registering of custom wallet storage implementation, passing the set of callbacks.

* Wallet Configuration
    ```
    {
      "id": string, Identifier of the wallet.
            Configured storage uses this identifier to lookup exact wallet data placement.
      "storage_type": string (optional), Type of the wallet storage. Defaults to 'default'.
                     'Default' storage type allows to store wallet data in the local file.
                     Custom storage types can be registered with indy_register_wallet_storage call.
      "storage_config": object (optional), Storage configuration json. Storage type defines set of supported keys.
                        Can be optional if storage supports default configuration.
                        Configuration for 'default' storage type:
      {
        "path": string (optional), Path to the directory with wallet files.
                Defaults to $HOME/.indy_client/wallet.
                Wallet will be stored in the file {path}/{id}/sqlite.db
      }
    }
    ```

* Wallet Credentials
    ```
    {
      "key": string, Key or passphrase used for wallet key derivation.
                     Look to key_derivation_method param for information about supported key derivation methods.
      "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
                             Can be optional if storage supports default configuration.
                             Should be empty for 'default' storage type.
      "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
                             ARGON2I_MOD - derive secured wallet master key (used by default)
                             ARGON2I_INT - derive secured wallet master key (less secure but faster)
                             RAW - raw wallet key master provided (skip derivation).
                                   RAW keys can be generated with indy_generate_wallet_key call
    }
    ```

#### Payment

Libindy provides a generic API for building payment-related transactions. 
These functions look at registered payment methods and call corresponding handlers.
Libindy doesn't provide default support of any payment method. There is `indy_register_payment_method` API function to register a payment method.
Any payment method (e.g., Sovrin tokens, Bitcoin, Visa, etc.) may be added to Libindy through plugins.

#### Logging
Libindy provides two options for Logger initialization:

* `indy_set_default_logger` API function sets default logger implementation. 
Rust `env_logger` is used as a default logging library.  This is a simple logger which writes to stdout (can be configured via `RUST_LOG` environment variable).
More details about `env_logger` and its customization can be found [here](https://crates.io/crates/env_logger).

* `indy_set_logger` API function registers custom logger implementation. 
Library user can provide a custom logger implementation by passing a set of handlers which will be called in correspondent cases.

WARNING: You can only set the logger **once**. Call `indy_set_default_logger`, `indy_set_logger`, not both. Once it's been set, libindy won't let you change it.

#### Error Handling

Every Libindy API function returns an error code that indicates result status of function execution. 
Some errors occur in only one specific case but some are common for different functions.
Libindy provides `indy_get_current_error` function which allows getting of details for last occurred error in addition to an error code.

This function should be called in two places to handle both cases of error occurrence:
1) synchronous  - in the same application thread
2) asynchronous - inside of function callback

#### Runtime Configuration
`indy_set_runtime_config` APIfunction allows setting of Libindy runtime configuration. 
This function accepts a `config` parameter that looks like:
```
{
    "crypto_thread_pool_size": int (optional) - size of thread pool for the most expensive crypto operations like keys generation. (4 by default)
    "collect_backtrace": Optional<bool> - whether errors backtrace should be collected.
        Capturing of backtrace can affect library performance.
        NOTE: must be set before invocation of any other API functions.
}
```

#### Transaction Endorser
As a transaction author, I need my transactions to be written to the ledger preserving me as the author without my needing to accept the responsibilities of an endorser.
Instead, I will have a relationship with an endorser who will endorse my transactions.

##### Endorse Transaction Workflow
1. Transaction Author builds a new request (indy_build_xxx_reqeust).
1. If no endorser is needed for a transaction (for example, the transaction author is an endorser, or auth rules are configured in a way that transaction author can send requests in permissionless mode), then the author signs and submits the transaction.
1. Otherwise the author chooses an Endorser and adds Endorser's DID into the request calling `indy_append_request_endorser`.
1. Transaction author signs the request (`indy_multi_sign_request` or `indy_sign_request`) with the added endorser field (output of `indy_append_request_endorser`).
1. Transaction author sends the request to the Endorser (out of scope).
1. Transaction Endorser signs the request (as of now `indy_multi_sign_request` must be called, not `indy_sign_request`) and submits it to the ledger.

#### Transaction Author Agreement
See [document](./how-tos/transaction-author-agreement.md)

#### Fully-Qualified Identifiers
See [document](./how-tos/fully-qualified-did-support.md)

## Indy-CLI
There is a Command Line Interface (CLI) built over Libindy which provides a set of commands to:
* Manage wallets
* Manage pool configurations
* Manage DIDs
* Send transactions to the ledger

#### Options
* -h and --help - Print usage. (usage: `indy-cli --help`)

* --logger-config - Init logger according to a config file. \
                    Indy Cli uses [log4rs](https://crates.io/crates/log4rs) logging framework \
                    Usage: `indy-cli --logger-config <path-to-config-file>` \
                    Example: `indy-cli --logger-config logger.yml` \
                    An example config file can be found [here](../cli/logger.yml) \
                    By default no logger initialized.
            
* --plugins - Load custom plugins in Libindy (e.g., wallet storage type, payment, etc.) \
              Usage: `indy-cli --plugins <lib-1-name>:<init-func-1-name>,...,<lib-n-name>:<init-func-n-name>)` \
              Example: `indy-cli --plugins libnullpay.so:nullpay_init`

* --config - Define config file for CLI initialization. \
             Usage: `indy-cli --config <path-to-config-json-file>` \
             Example: `indy-cli --config linux-sample-config.json`

#### Config
Indy-CLI supports initialization with a predefined config file. 
A config file can contain the following fields:
```
{
  "loggerConfig": string, - path to a logger config file (is equal to usage of "--logger-config" option)
  "plugins": string, - a list of plugins to load in Libindy (is equal to usage of "--plugins" option)
  "taaAcceptanceMechanism": string - transaction author agreement acceptance mechanism to be used when sending write transactions to the Ledger.
}
```
An example config file can be found [here](../cli/linux-sample-config.json)

#### Execution mode
Indy-CLI supports two execution modes:
* Interactive. In this mode CLI will read commands from terminal interactively (command by command).
* Batch. In this mode all commands will be read from file or pipe and executed in series. \
Usage: `indy-cli <path-to-file>` \
An example of a batch script:
    ```
    wallet create w1 key=k1
    wallet open w1 key=k1
    did new
    did list
    wallet close
    exit
    ```

#### Transaction Author Agreement
See [document](./how-tos/transaction-author-agreement.md)

## Vcx

Libvcx library must be initialized with one of the functions:
* `vcx_init_with_config` -  initializes with <configuration> passed as JSON string (wallet must be already created). 
* `vcx_init` -  initializes with a path to the file containing <configuration> (wallet must be already created). 
* `vcx_init_minimal` - initializes with the minimal <configuration> (wallet, pool must already be set with vcx_wallet_set_handle() and vcx_pool_set_handle()) and without any agency configuration.

If the library works with an agency `vcx_agent_provision` function must be called before initialization.
This function does: 
* provisions an agent in the agency.
* populates of configuration and wallet for this agent.
The result of this function is <configuration> JSON which can be extended and used for initialization.

Every library call after initialization will use this <configuration>. 
An example of <configuration> file can be found [here](../vcx/libvcx/sample_config/config.json)

To change <configuration> a user must call `vcx_shutdown` and then call initialization function again.

### Configuration options
The config json must match to the following format: `{"key": "value as string"}`. Note, that values are always strings. 
It can accept multiple options that are listed below. 
Almost all of them are optional and depend on the way you use Vcx (with agency or not).

##### Common library related options
* `payment_method` - name of used payment method which was registered by a plugin (plugin must be registered independently).

* `threadpool_size` - size of thread pool used for command execution (8 by default). 

* `protocol_version` - message protocol to use for agent to agency and agent to agent communication. 
Can be one of:
    * "1.0" - use bundled messages, auth/anon cryptography.
    * "2.0" - use aries cross domain message format, pack/unpack functions. 
    * "3.0" - use aries cross domain message format, pack/unpack functions and aries communication protocols (is alternative to the combination of settings "protocol_version":"2.0" and "communication_method":"aries"). 
    
* `author_agreement` - accept and use transaction author agreement data containing the following fields:
    * `acceptanceMechanismType` - (string) mechanism how user has accepted the TAA 
        (must be one of the keys taken from GET_TRANSACTION_AUTHOR_AGREEMENT_AML response['result']['data']['aml'] map).
    * `timeOfAcceptance` - (u64) UTC timestamp when user has accepted the TAA.
    * `text` and `version` - (string) text and version of TAA.
    * `taaDigest` - (string) sha256 hash calculated on concatenated strings: `version || text`.

    NOTE that either pair `text` `version` or `taaDigest` must be used 
    This TAA data will be appended for every write transaction sending to the ledger.

    Example: 
    ```
    ... other config fields
    ...
    ...
    "author_agreement": "{\"taaDigest\": \"string\", \"acceptanceMechanismType\":\"string\", \"timeOfAcceptance\": u64}” }",
    ```

* `did_method` - method name to use for fully qualified DIDs.

##### User info options
* `institution_did` - DID associated with institution.
* `institution_verkey` - Verkey associated with institution.
* `institution_name` - name associated with institution (it is used like a label for connection/credential offer/proof request). 
* `institution_logo_url` - url containing institution logo.

##### Pool related options
* `pool_name` - name of the pool ledger configuration will be created.
* `genesis_path` - path to the genesis transaction file to use fot pool creation.
* `pool_config` - runtime pool configuration json (see `config` parameter of `indy_open_pool_ledger` function). 

##### Wallet related options
All these options are part of Indy wallet `config`/`credential` parameters.

* `wallet_name` - name of the wallet to use. Note that wallet must be already created.
* `wallet_key` - key or passphrase used for wallet creation.
* `wallet_key_derivation` - key derivation method.
* `wallet_type` - type of the wallet (default Libindy or some plugged one).
* `storage_config` - an addition configuration related to the wallet storage.
* `storage_credentials` - storage configuration json.
* `backup_key` - key or passphrase used for wallet import.
* `exported_wallet_path` - key or passphrase used for wallet export.
* `wallet_handle` - handle to the already opened wallet to use.

##### Agency related options
* `agency_endpoint` - agency endpoint to connect.
* `agency_did` - agency DID.
* `agency_verkey` - agency Verkey.
* `remote_to_sdk_did` - agent DID
* `remote_to_sdk_verkey` - agent Verkey
* `sdk_to_remote_did` - pairwise DID for agent
* `sdk_to_remote_verkey` - pairwise Verkey for Agent

##### Protocol related options
* `communication_method` - the version of protocols to use (can be `aries` or `proprietary`) for connection establishment and messages exchange.
    * `aries` - the public protocols described in the [repository](https://github.com/hyperledger/aries-rfcs).
    * `proprietary` - the proprietary protocols.
* `actors` - the set of actors which application supports. This setting is used within the `Feature Discovery` protocol to discover which features are supported by another connection side.

    The following actors are supported by default: `[inviter, invitee, issuer, holder, prover, verifier, sender, receiver]`. 
    You need to edit this list and add to an initialization config in case the application supports the fewer number of actors.

    Note that option is applicable for `aries` communication method only.

### Logging
libVCX provides two options for Logger initialization:

* `vcx_set_default_logger` API function sets default logger implementation. 
Rust `env_logger` is used as a default logging library.  This is a simple logger which writes to stdout (can be configured via `RUST_LOG` environment variable).
More details about `env_logger` and its customization can be found [here](https://crates.io/crates/env_logger).

* `vcx_set_logger` API function registers custom logger implementation. 
Library user can provide a custom logger implementation by passing a set of handlers which will be called in correspondent cases.
This function will also be used by `indy` and `plugins` for logging.

WARNING: You can only set the logger **once**. Once it's been set, vcx won't let you change it.

##### Wrappers
* The Python wrapper uses default Python logging module. So, to enable logs you need just to configure its usual way. 
 Note: there is an additional log level=0 that is equal to `trace` level. 

    Example: `logging.basicConfig(level=logging.DEBUG)`
* The Java wrapper uses slf4j as a facade for various logging frameworks, such as java.util.logging, logback and log4j.
* The NodeJs provides to functions for logger initialization:
  * setDefaultLogger ( pattern ) - turns on the default logger.
  * setLogger ( logFn ) - sets a function to be called every time a log is emitted. Example:
  
    ```
    indy.setLogger(function (level, target, message, modulePath, file, line) {
      console.log('libindy said:', level, target, message, modulePath, file, line)
    })
    ```
