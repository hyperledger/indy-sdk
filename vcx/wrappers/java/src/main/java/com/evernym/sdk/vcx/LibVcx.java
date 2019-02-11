package com.evernym.sdk.vcx;

import com.sun.jna.*;

import java.io.File;

public abstract class LibVcx {
    private static final String LIBRARY_NAME = "vcx";
    /*
     * Native library interface
     */


    /**
     * JNA method signatures for calling SDK function.
     */
    public interface API extends Library {

        // pool.rs
        public int vcx_init_with_config(int command_handle,
                                        String config,
                                        Callback cb);
        public int vcx_init(int command_handle, String config_path, Callback cb);

        public String vcx_error_c_message(int error_code);
        public int vcx_shutdown(boolean delete);
        public int vcx_reset();

    /**
     * nullpay
     */
        public int nullpay_init();

    /**
     * Helper API for testing purposes.
     */
        public void vcx_set_next_agency_response(int msg);


    /**
     * Schema object
     *
     * For creating, validating and committing a schema to the sovrin ledger.
     */

        /**
         * Creates a schema from a json string. Populates a handle to the new schema.
         */
        public int vcx_schema_create(int command_handle, String source_id, String schema_name, String version, String schema_data, int payment_handle, Callback cb);

        /**
         * Populates status with the current State of this claim.
         */
        public int vcx_schema_serialize(int command_handle, int schema_handle, Callback cb);

        /**
         * Re-creates a claim object from the specified serialization.
         */
        public int vcx_schema_deserialize(int command_handle, String serialized_schema, Callback cb);

        /**
         * Populates data with the contents of the schema handle.
         */
        public int vcx_schema_get_attributes(int command_handle, String source_id, String schema_id, Callback cb);

        /**
         * Populates sequence_no with the actual sequence number of the schema on the sovrin ledger.
         */
        public int vcx_schema_get_schema_id(int command_handle, int schema_handle, Callback cb);

        /**
         * Release memory associated with schema object.
         */
        public int vcx_schema_release(int handle);




    /**
     * connection object
     *
     * For creating a connection with an identity owner for interactions such as exchanging
     * claims and proofs.
     */

        /**
         * Creates a connection object to a specific identity owner. Populates a handle to the new connection.
         */
        public int vcx_connection_create(int command_handle, String source_id, Callback cb);

        /**
         * Asynchronously request a connection be made.
         */
        public int vcx_connection_connect(int command_handle, int connection_handle, String connection_type, Callback cb);

        /**
         * Returns the contents of the connection handle or null if the connection does not exist.
         */
        public int vcx_connection_serialize(int command_handle, int connection_handle, Callback cb);

        /**
         * Re-creates a connection object from the specified serialization.
         */
        public int vcx_connection_deserialize(int command_handle, String serialized_claim, Callback cb);

        /**
         * Request a State update from the agent for the given connection.
         */
        public int vcx_connection_update_state(int command_handle, int connection_handle, Callback cb);

        /**
         * Retrieves the State of the connection
         */
        public int vcx_connection_get_state(int command_handle, int connection_handle, Callback cb);

        /**
         * Releases the connection from memory.
         */
        public int vcx_connection_release(int connection_handle);

        /**
         * Get the invite details for the connection.
         */
        public int vcx_connection_invite_details(int command_handle, int connection_handle, int abbreviated, Callback cb);

        /**
         * Creates a connection from the invite details.
         */
        public int vcx_connection_create_with_invite(int command_handle, String source_id, String invite_details, Callback cb);

        /**
         * Deletes a connection and send a delete API request to backend to delete connection
         */
        public int vcx_connection_delete_connection(int command_handle, int connection_handle, Callback cb);


    /**
     * credential issuer object
     *
     * Used for offering and managing a credential with an identity owner.
     */

        /** Creates a credential object from the specified credentialdef handle. Populates a handle the new credential. */
        public int vcx_issuer_create_credential(int command_handle, String source_id, String cred_def_id, String issuer_did, String credential_data, String credential_name, long price, Callback cb);

        /** Asynchronously sends the credential offer to the connection. */
        public int vcx_issuer_send_credential_offer(int command_handle, int credential_handle, int connection_handle, Callback cb);

        /** Updates the state of the credential from the agency. */
        public int vcx_issuer_credential_update_state(int command_handle, int credential_handle, Callback cb);

        /** Retrieves the state of the issuer_credential. */
        public int vcx_issuer_credential_get_state(int command_handle, int credential_handle, Callback cb);

        /** Asynchronously send the credential to the connection. Populates a handle to the new transaction. */
        public int vcx_issuer_send_credential(int command_handle, int credential_handle, int connection_handle, Callback cb);

        /** Populates status with the current state of this credential. */
        public int vcx_issuer_credential_serialize(int command_handle, int credential_handle, Callback cb);

        /** Re-creates a credential object from the specified serialization. */
        public int vcx_issuer_credential_deserialize(int command_handle, String serialized_credential, Callback cb);

        /** Terminates a credential for the specified reason. */
        public int vcx_issuer_terminate_credential(int command_handle, int credential_handle, int state_type, String msg);

        /** Releases the credential from memory. */
        public int vcx_issuer_credential_release(int credential_handle);

        /** Populates credential_request with the latest credential request received. (not in MVP) */
        public int vcx_issuer_get_credential_request(int credential_handle, String credential_request);

        /** Sets the credential request in an accepted state. (not in MVP) */
        public int vcx_issuer_accept_credential(int credential_handle);


    /**
     * proof object
     *
     * Used for requesting and managing a proof request with an identity owner.
     */

        /**
         * Creates a proof object.  Populates a handle to the new proof.
         */
        public int vcx_proof_create(int command_handle, String source_id, String requested_attrs, String requested_predicates, String revocationInterval, String name, Callback cb);

        /**
         * Asynchronously send a proof request to the connection.
         */
        public int vcx_proof_send_request(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Populate response_data with the latest proof offer received.
         */
        public int vcx_get_proof(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Set proof offer as accepted.
         */
        public int vcx_proof_accepted(int proof_handle, String response_data);

        /**
         * Populates status with the current State of this proof request.
         */
        public int vcx_proof_update_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Retrieves the State of the proof.
         */
        public int vcx_proof_get_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Populates status with the current State of this proof.
         */
        public int vcx_proof_serialize(int command_handle, int proof_handle, Callback cb);

        /**
         * Re-creates a proof object from the specified serialization.
         */
        public int vcx_proof_deserialize(int command_handle, String serialized_proof, Callback cb);

        /**
         * Releases the proof from memory.
         */
        public int vcx_proof_release(int proof_handle);

    /**
     * disclosed_proof object
     *
     * Used for sending a disclosed_proof to an identity owner.
     */

        /**
         * Creates a disclosed_proof object.  Populates a handle to the new disclosed_proof.
         */
        public int vcx_disclosed_proof_create_with_request(int command_handle, String source_id, String requested_attrs, String requested_predicates, String name, Callback cb);

        /**
         * Create a proof object with proof request
         */
        public int vcx_disclosed_proof_create_with_request(int command_handle, String source_id, String proof_req, Callback cb);

        /**
         * Asynchronously send a proof to the connection.
         */
        public int vcx_disclosed_proof_send_proof(int command_handle, int proof_handle, int connection_handle, Callback cb);

        /**
         * Populates status with the current State of this disclosed_proof request.
         */
        public int vcx_disclosed_proof_update_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Check for any proof requests from the connection.
         */
        public int vcx_disclosed_proof_get_requests(int command_handle, int connection_handle, Callback cb);

        /**
         * Retrieves the State of the disclosed_proof.
         */
        public int vcx_disclosed_proof_get_state(int command_handle, int proof_handle, Callback cb);

        /**
         * Populates status with the current State of this disclosed_proof.
         */
        public int vcx_disclosed_proof_serialize(int command_handle, int proof_handle, Callback cb);

        /**
         * Re-creates a disclosed_proof object from the specified serialization.
         */
        public int vcx_disclosed_proof_deserialize(int command_handle, String serialized_proof, Callback cb);

        /**
         * Releases the disclosed_proof from memory.
         */
        public int vcx_disclosed_proof_release(int proof_handle);

        /**
         * Create proof instance with a message id
         */
        public int vcx_disclosed_proof_create_with_msgid(int command_handle, String source_id, int connection_handle, String msd_id, Callback cb);

        /**
         * Retrieve credentials that matches with the proof request
         */
        public int vcx_disclosed_proof_retrieve_credentials(int command_handle, int proof_handle, Callback cb);

        /**
         * Generate a proof that can be sent later
         */
        public int vcx_disclosed_proof_generate_proof(int command_handle, int proof_handle, String selected_credentials, String self_attested_attributes, Callback cb);


        /**
         * UtilsApi object
         *
         */
        public String vcx_provision_agent(String json);

        public int vcx_agent_provision_async(int command_handle, String json,Callback cb);

        public int vcx_agent_update_info(int command_handle,String json,Callback cb);

        public int vcx_ledger_get_fees(int command_handle, Callback cb);

        /**
         * credential object
         *
         * Used for accepting and requesting a credential with an identity owner.
         */

        /** Creates a credential object from the specified credentialdef handle. Populates a handle the new credential. */
        public int vcx_credential_create_with_offer(int command_handle, String source_id, String credential_offer,Callback cb);

        /** Creates a credential object from the connection and msg id. Populates a handle the new credential. */
        public int vcx_credential_create_with_msgid(int command_handle, String source_id, int connection, String msg_id,Callback cb);

        /** Asynchronously sends the credential request to the connection. */
        public int vcx_credential_send_request(int command_handle, int credential_handle, int connection_handle,int payment_handle, Callback cb);

        /** Check for any credential offers from the connection. */
        public int vcx_credential_get_offers(int command_handle, int connection_handle,Callback cb);

        /** Updates the State of the credential from the agency. */
        public int vcx_credential_update_state(int command_handle, int credential_handle,Callback cb);

        /** Retrieves the State of the credential - including storing the credential if it has been sent. */
        public int vcx_credential_get_state(int command_handle, int credential_handle, Callback cb);

        /** Populates status with the current State of this credential. */
        public int vcx_credential_serialize(int command_handle, int credential_handle, Callback cb);

        /** Re-creates a credential from the specified serialization. */
        public int vcx_credential_deserialize(int command_handle, String serialized_credential, Callback cb);

        /** Releases the credential from memory. */
        public int vcx_credential_release(int credential_handle);

        /** Retrieve information about a stored credential in user's wallet, including credential id and the credential itself. */
        public int vcx_get_credential(int command_handle, int credential_handle, Callback cb);

        /**
         * wallet object
         *
         * Used for exporting and importing and managing the wallet.
         */

        /** Export the wallet as an encrypted file */
        public int vcx_wallet_export(int command_handle, String path, String backup_key, Callback cb);

        /** Import an encrypted file back into the wallet */
        public int vcx_wallet_import(int command_handle, String config, Callback cb);

        /** Add a record into wallet */
        public int vcx_wallet_add_record(int command_handle, String recordType, String recordId, String recordValue, String recordTag, Callback cb);

        /** Delete a record from wallet */
        public int vcx_wallet_delete_record(int command_handle, String recordType, String recordId, Callback cb);

        /** Get a record from wallet */
        public int vcx_wallet_get_record(int command_handle, String recordType, String recordId, String optionsJson, Callback cb);

        /** Update a record in wallet */
        public int vcx_wallet_update_record_value(int command_handle, String recordType, String recordId, String recordValue, Callback cb);

        /**
         * token object
         *
         * Used for sending tokens and getting token related info
         */

        /** Gets token Balance and payment addresses info */
        public int vcx_wallet_get_token_info(int command_handle, int payment_handle, Callback cb);

        /** Sends token to recipient */
        public int vcx_wallet_send_tokens(int command_handle, int payment_handle, String tokens, String recipient, Callback cb);

        /** Create a payment address and returns it */
        public int vcx_wallet_create_payment_address(int command_handle, String seed, Callback cb);

        /**
         * message object
         *
         * Used for getting and updating messages
         */

        /** Get messages for given uids or pairwise did from agency endpoint */
        public int vcx_messages_download(int command_handle, String messageStatus, String uids, String pwdids, Callback cb);

        /** Update message status for a object of uids */
        public int vcx_messages_update_status(int command_handle, String messageStatus, String msgJson, Callback cb);

        /**
         * credentialdef object
         *
         * For creating, validating and committing a credential definition to the sovrin ledger.
         */

        /** Creates a credential definition from the given schema.  Populates a handle to the new credentialdef. */
        int vcx_credentialdef_create(int command_handle, String source_id, String credentialdef_name, String schema_id, String issuer_did, String tag,  String config, int payment_handle, Callback cb);


        /** Populates status with the current state of this credential. */
        int vcx_credentialdef_serialize(int command_handle, int credentialdef_handle, Callback cb);

        /** Re-creates a credential object from the specified serialization. */
        int vcx_credentialdef_deserialize(int command_handle, String serialized_credentialdef, Callback cb);

        /** Release memory associated with credentialdef object. */
        int vcx_credentialdef_release(int handle);

        /** Retrieves cred_def_id from credentialdef object. */
        int vcx_credentialdef_get_cred_def_id(int command_handle, int cred_def_handle, Callback cb);

        /**
         * logger
         *
         */

        /** Set custom logger implementation.. */
        int vcx_set_logger(Pointer context, Callback enabled, Callback log, Callback flush);
        int vcx_set_default_logger(String log_level);

    }

    /*
     * Initialization
     */

    public static API api = null;

    static {

        try {
            init();
        } catch (UnsatisfiedLinkError ex) {
            // Library could not be found in standard OS locations.
            // Call init(File file) explicitly with absolute library path.
            ex.printStackTrace();
        }
    }

    /**
     * Initializes the API with the path to the C-Callable library.
     *
     * @param searchPath The path to the directory containing the C-Callable library file.
     */
    public static void init(String searchPath, String libraryName) {

        NativeLibrary.addSearchPath(libraryName, searchPath);
        api = Native.loadLibrary(libraryName, API.class);
        initLogger();
    }

    /**
     * Initializes the API with the path to the C-Callable library.
     * Warning: This is not platform-independent.
     *
     * @param file The absolute path to the C-Callable library file.
     */
    public static void init(File file) {

        api = Native.loadLibrary(file.getAbsolutePath(), API.class);
        initLogger();
    }

    /**
     * Initializes the API with the default library.
     */
    public static void init() {

        api = Native.loadLibrary(LIBRARY_NAME, API.class);
        initLogger();
    }

    public static void initByLibraryName(String libraryName) {

        System.loadLibrary(libraryName);
        api = Native.loadLibrary(libraryName, API.class);
        initLogger();
    }

    /**
     * Indicates whether or not the API has been initialized.
     *
     * @return true if the API is initialize, otherwise false.
     */
    public static boolean isInitialized() {

        return api != null;
    }

    private static class Logger {
        private static Callback enabled = null;

        private static Callback log = new Callback() {

            @SuppressWarnings({"unused", "unchecked"})
            public void callback(Pointer context, int level, String target, String message, String module_path, String file, int line) {
                org.slf4j.Logger logger = org.slf4j.LoggerFactory.getLogger(String.format("%s.native.%s", LibVcx.class.getName(), target.replace("::", ".")));

                String logMessage = String.format("%s:%d | %s", file, line, message);

                switch (level) {
                    case 1:
                        logger.error(logMessage);
                        break;
                    case 2:
                        logger.warn(logMessage);
                        break;
                    case 3:
                        logger.info(logMessage);
                        break;
                    case 4:
                        logger.debug(logMessage);
                        break;
                    case 5:
                        logger.trace(logMessage);
                        break;
                    default:
                        break;
                }
            }
        };

        private static Callback flush = null;
    }

    private static void initLogger() {
        api.vcx_set_logger(null, Logger.enabled, Logger.log, Logger.flush);
    }
}