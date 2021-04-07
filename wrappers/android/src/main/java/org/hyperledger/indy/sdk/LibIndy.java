package org.hyperledger.indy.sdk;

import com.sun.jna.*;
import com.sun.jna.ptr.PointerByReference;

import java.io.File;
import java.util.HashMap;
import java.util.Map;

import static com.sun.jna.Native.detach;

public abstract class LibIndy {

	public static final String LIBRARY_NAME = "indy";
	private static final String LIB_INDY_LOGGER_PREFIX = String.format("%s.native", LibIndy.class.getName());
	static final DefaultTypeMapper MAPPER = new DefaultTypeMapper();

	/*
	 * Native library interface
	 */

	/**
	 * JNA method signatures for calling SDK function.
	 */
	public interface API extends Library {

		// pool.rs

		public int indy_create_pool_ledger_config(int command_handle, String config_name, String config, Callback cb);
		public int indy_open_pool_ledger(int command_handle, String config_name, String config, Callback cb);
		public int indy_refresh_pool_ledger(int command_handle, int handle, Callback cb);
		public int indy_close_pool_ledger(int command_handle, int handle, Callback cb);
		public int indy_delete_pool_ledger_config(int command_handle, String config_name, Callback cb);
		public int indy_set_protocol_version(int command_handle, int protocol_version, Callback cb);
		public int indy_list_pools(int command_handle, Callback cb);

		// wallet.rs

		public int indy_create_wallet(int command_handle, String config, String credentials, Callback cb);
		public int indy_open_wallet(int command_handle, String config, String credentials, Callback cb);
		public int indy_close_wallet(int command_handle, int handle, Callback cb);
		public int indy_delete_wallet(int command_handle, String config, String credentials, Callback cb);
		public int indy_export_wallet(int command_handle, int handle, String exportConfigJson, Callback cb);
		public int indy_import_wallet(int command_handle, String config, String credentials, String importConfigJson, Callback cb);
		public int indy_generate_wallet_key(int command_handle, String config, Callback cb);

		// ledger.rs

		public int indy_sign_and_submit_request(int command_handle, int pool_handle, int wallet_handle, String submitter_did, String request_json, Callback cb);
		public int indy_submit_request(int command_handle, int pool_handle, String request_json, Callback cb);
		public int indy_submit_action(int command_handle, int pool_handle, String request_json, String nodes, int timeout, Callback cb);
		public int indy_sign_request(int command_handle, int wallet_handle, String submitter_did, String request_json, Callback cb);
		public int indy_multi_sign_request(int command_handle, int wallet_handle, String submitter_did, String request_json, Callback cb);
		public int indy_build_get_ddo_request(int command_handle, String submitter_did, String target_did, Callback cb);
		public int indy_build_nym_request(int command_handle, String submitter_did, String target_did, String verkey, String alias, String role, Callback cb);
		public int indy_build_attrib_request(int command_handle, String submitter_did, String target_did, String hash, String raw, String enc, Callback cb);
		public int indy_build_get_attrib_request(int command_handle, String submitter_did, String target_did, String raw, String hash, String enc, Callback cb);
		public int indy_build_get_nym_request(int command_handle, String submitter_did, String target_did, Callback cb);
		public int indy_parse_get_nym_response(int command_handle, String response, Callback cb);
		public int indy_build_schema_request(int command_handle, String submitter_did, String data, Callback cb);
		public int indy_build_get_schema_request(int command_handle, String submitter_did, String id, Callback cb);
		public int indy_parse_get_schema_response(int command_handle, String get_schema_response, Callback cb);
		public int indy_build_cred_def_request(int command_handle, String submitter_did, String data, Callback cb);
		public int indy_build_get_cred_def_request(int command_handle, String submitter_did, String id, Callback cb);
		public int indy_parse_get_cred_def_response(int command_handle, String get_cred_def_response, Callback cb);
		public int indy_build_node_request(int command_handle, String submitter_did, String target_did, String data, Callback cb);
		public int indy_build_get_validator_info_request(int command_handle, String submitter_did, Callback cb);
		public int indy_build_get_txn_request(int command_handle, String submitter_did, String ledger_type, int data, Callback cb);
		public int indy_build_pool_config_request(int command_handle, String submitter_did, boolean writes, boolean force, Callback cb);
		public int indy_build_pool_restart_request(int command_handle, String submitter_did, String action, String datetime, Callback cb);
		public int indy_build_pool_upgrade_request(int command_handle, String submitter_did, String name, String version, String action, String sha256, int timeout, String schedule, String justification, boolean reinstall, boolean force, String package_, Callback cb);
		public int indy_build_revoc_reg_def_request(int command_handle, String submitter_did, String data, Callback cb);
		public int indy_build_get_revoc_reg_def_request(int command_handle, String submitter_did, String id, Callback cb);
		public int indy_parse_get_revoc_reg_def_response(int command_handle, String get_revoc_ref_def_response, Callback cb);
		public int indy_build_revoc_reg_entry_request(int command_handle, String submitter_did, String revoc_reg_def_id, String rev_def_type, String value, Callback cb);
		public int indy_build_get_revoc_reg_request(int command_handle, String submitter_did, String revoc_reg_def_id, long timestamp, Callback cb);
		public int indy_parse_get_revoc_reg_response(int command_handle, String get_revoc_reg_response, Callback cb);
		public int indy_build_get_revoc_reg_delta_request(int command_handle, String submitter_did, String revoc_reg_def_id, long from, long to, Callback cb);
		public int indy_parse_get_revoc_reg_delta_response(int command_handle, String get_revoc_reg_delta_response, Callback cb);
		public int indy_get_response_metadata(int command_handle, String response, Callback cb);
		public int indy_build_auth_rule_request(int command_handle, String submitter_did, String txn_type, String action, String field, String old_value, String new_value, String constraint, Callback cb);
		public int indy_build_auth_rules_request(int command_handle, String submitter_did, String data, Callback cb);
		public int indy_build_get_auth_rule_request(int command_handle, String submitter_did, String txn_type, String action, String field, String old_value, String new_value, Callback cb);
		public int indy_build_txn_author_agreement_request(int command_handle, String submitter_did, String text, String version, long ratification_ts, long retirement_ts, Callback cb);
		public int indy_build_disable_all_txn_author_agreements_request(int command_handle, String submitter_did, Callback cb);
		public int indy_build_get_txn_author_agreement_request(int command_handle, String submitter_did, String data, Callback cb);
		public int indy_build_acceptance_mechanisms_request(int command_handle, String submitter_did, String aml, String version, String aml_context, Callback cb);
		public int indy_build_get_acceptance_mechanisms_request(int command_handle, String submitter_did, int timestamp, String version, Callback cb);
		public int indy_append_txn_author_agreement_acceptance_to_request(int command_handle, String request_json, String text, String version, String hash, String acc_mech_type, long time_of_acceptance, Callback cb);
		public int indy_append_request_endorser(int command_handle, String request_json, String endorser_did, Callback cb);
		public int indy_build_ledgers_freeze_request(int command_handle, String submitter_did, String ledgers_ids, Callback cb);
		public int indy_build_get_frozen_ledgers_request(int command_handle, String submitter_did, Callback cb);

		// did.rs

		public int indy_create_and_store_my_did(int command_handle, int wallet_handle, String did_json, Callback cb);
		public int indy_replace_keys_start(int command_handle, int wallet_handle, String did, String identity_json, Callback cb);
		public int indy_replace_keys_apply(int command_handle, int wallet_handle, String did, Callback cb);
		public int indy_store_their_did(int command_handle, int wallet_handle, String identity_json, Callback cb);
		public int indy_key_for_did(int command_handle, int pool_handle, int wallet_handle, String did, Callback cb);
        public int indy_key_for_local_did(int command_handle, int wallet_handle, String did, Callback cb);
		public int indy_set_endpoint_for_did(int command_handle, int wallet_handle, String did, String address, String transportKey, Callback cb);
		public int indy_get_endpoint_for_did(int command_handle, int wallet_handle, int pool_handle, String did, Callback cb);
		public int indy_set_did_metadata(int command_handle, int wallet_handle, String did, String metadata, Callback cb);
		public int indy_get_did_metadata(int command_handle, int wallet_handle, String did, Callback cb);
		public int indy_get_my_did_with_meta(int command_handle, int wallet_handle, String did, Callback cb);
		public int indy_list_my_dids_with_meta(int command_handle, int wallet_handle, Callback cb);
		public int indy_abbreviate_verkey(int command_handle, String did, String full_verkey, Callback cb);
		public int indy_qualify_did(int command_handle, int wallet_handle, String did, String method, Callback cb);

		// crypto.rs

		public int indy_create_key(int command_handle, int wallet_handle, String key_json, Callback cb);
		public int indy_set_key_metadata(int command_handle, int wallet_handle, String verkey, String metadata, Callback cb);
		public int indy_get_key_metadata(int command_handle, int wallet_handle, String verkey, Callback cb);
		public int indy_crypto_sign(int command_handle, int wallet_handle, String my_vk, byte[] message_raw, int message_len, Callback cb);
		public int indy_crypto_verify(int command_handle, String their_vk, byte[] message_raw, int message_len, byte[] signature_raw, int signature_len, BoolCallback cb);
		public int indy_crypto_auth_crypt(int command_handle, int wallet_handle, String my_vk, String their_vk, byte[] message_raw, int message_len, Callback cb);
		public int indy_crypto_auth_decrypt(int command_handle, int wallet_handle, String my_vk, byte[] encrypted_msg_raw, int encrypted_msg_len, Callback cb);
		public int indy_crypto_anon_crypt(int command_handle, String their_vk, byte[] message_raw, int message_len, Callback cb);
		public int indy_crypto_anon_decrypt(int command_handle, int wallet_handle, String my_vk, byte[] encrypted_msg_raw, int encrypted_msg_len, Callback cb);
		public int indy_pack_message(int command_handle, int wallet_handle, byte[] message, int message_len, String receiver_keys, String sender, Callback cb);
		public int indy_unpack_message(int command_handle, int wallet_handle, byte[] jwe_data, int jwe_len, Callback cb);

		// anoncreds.rs

		public int indy_issuer_create_schema(int command_handle, String issuer_did, String name, String version, String attr_names, Callback cb);
		public int indy_issuer_create_and_store_credential_def(int command_handle, int wallet_handle, String issuer_did, String schema_json, String tag, String signature_type, String config_json, Callback cb);
		public int indy_issuer_rotate_credential_def_start(int command_handle, int wallet_handle, String cred_def_id, String config_json, Callback cb);
		public int indy_issuer_rotate_credential_def_apply(int command_handle, int wallet_handle, String cred_def_id, Callback cb);
		public int indy_issuer_create_and_store_revoc_reg(int command_handle, int wallet_handle, String issuer_did, String revoc_def_type, String tag, String cred_def_id, String config_json, int blob_storage_writer_handle, Callback cb);
		public int indy_issuer_create_credential_offer(int command_handle, int wallet_handle, String cred_def_id, Callback cb);
		public int indy_issuer_create_credential(int command_handle, int wallet_handle, String cred_offer_json, String cred_req_json, String cred_values_json, String rev_reg_id, int blob_storage_reader_handle, Callback cb);
		public int indy_issuer_revoke_credential(int command_handle, int wallet_handle, int blob_storage_reader_handle, String rev_reg_id, String cred_revoc_id, Callback cb);
//		public int indy_issuer_recover_credential(int command_handle, int wallet_handle, int blob_storage_reader_handle, String rev_reg_id, String cred_revoc_id, Callback cb);
		public int indy_issuer_merge_revocation_registry_deltas(int command_handle, String rev_reg_delta_json, String other_rev_reg_delta_json, Callback cb);
		public int indy_prover_create_master_secret(int command_handle, int wallet_handle, String master_secret_id, Callback cb);
		public int indy_prover_create_credential_req(int command_handle, int wallet_handle, String prover_did, String cred_offer_json, String cred_def_json, String master_secret_id, Callback cb);
		public int indy_prover_store_credential(int command_handle, int wallet_handle, String cred_id, String cred_req_metadata_json, String cred_json, String cred_def_json, String rev_reg_def_json, Callback cb);
		public int indy_prover_get_credentials(int command_handle, int wallet_handle, String filter_json, Callback cb);
		public int indy_prover_get_credential(int command_handle, int wallet_handle, String cred_id, Callback cb);
		public int indy_prover_delete_credential(int command_handle, int wallet_handle, String cred_id, Callback cb);
		public int indy_prover_search_credentials(int command_handle, int wallet_handle, String query_json, Callback cb);
		public int indy_prover_fetch_credentials(int command_handle, int search_handle, int count, Callback cb);
		public int indy_prover_close_credentials_search(int command_handle, int search_handle, Callback cb);
		public int indy_prover_get_credentials_for_proof_req(int command_handle, int wallet_handle, String proof_request_json, Callback cb);
		public int indy_prover_search_credentials_for_proof_req(int command_handle, int wallet_handle, String proof_request_json, String extra_query_json, Callback cb);
		public int indy_prover_fetch_credentials_for_proof_req(int command_handle, int search_handle, String item_referent, int count, Callback cb);
		public int indy_prover_close_credentials_search_for_proof_req(int command_handle, int search_handle, Callback cb);
		public int indy_prover_create_proof(int command_handle, int wallet_handle, String proof_req_json, String requested_credentials_json, String master_secret_name, String schemas_json, String credential_defs_json, String rev_infos_json, Callback cb);
		public int indy_verifier_verify_proof(int command_handle, String proof_request_json, String proof_json, String schemas_json, String cred_defs_jsons, String rev_reg_defs_json, String revoc_regs_json, Callback cb);
		public int indy_create_revocation_state(int command_handle, int blob_storage_reader_handle, String rev_reg_def_json, String rev_reg_delta_json, long timestamp, String cred_rev_id, Callback cb);
		public int indy_update_revocation_state(int command_handle, int blob_storage_reader_handle, String rev_state_json, String rev_reg_def_json, String rev_reg_delta_json, long timestamp, String cred_rev_id, Callback cb);
		public int indy_generate_nonce(int command_handle, Callback cb);
		public int indy_to_unqualified(int command_handle, String entity, Callback cb);


		// pairwise.rs

		public int indy_is_pairwise_exists(int command_handle, int wallet_handle, String their_did, Callback cb);
		public int indy_create_pairwise(int command_handle, int wallet_handle, String their_did, String my_did, String metadata, Callback cb);
		public int indy_list_pairwise(int command_handle, int wallet_handle, Callback cb);
		public int indy_get_pairwise(int command_handle, int wallet_handle, String their_did, Callback cb);
		public int indy_set_pairwise_metadata(int command_handle, int wallet_handle, String their_did, String metadata, Callback cb);

		// blob_storage.rs
		public int indy_open_blob_storage_reader(int command_handle, String type, String config_json, Callback cb);
		public int indy_open_blob_storage_writer(int command_handle, String type, String config_json, Callback cb);

		// non_secrets.rs
		public int indy_add_wallet_record(int command_handle, int wallet_handle, String type, String id, String value, String tags_json, Callback cb);
		public int indy_update_wallet_record_value(int command_handle, int wallet_handle, String type, String id, String value, Callback cb);
		public int indy_update_wallet_record_tags(int command_handle, int wallet_handle, String type, String id, String tags_json, Callback cb);
		public int indy_add_wallet_record_tags(int command_handle, int wallet_handle, String type, String id, String tags_json, Callback cb);
		public int indy_delete_wallet_record_tags(int command_handle, int wallet_handle, String type, String id, String tag_names_json, Callback cb);
		public int indy_delete_wallet_record(int command_handle, int wallet_handle, String type, String id, Callback cb);
		public int indy_get_wallet_record(int command_handle, int wallet_handle, String type, String id, String options_json, Callback cb);
		public int indy_open_wallet_search(int command_handle, int wallet_handle, String type, String query_json, String options_json, Callback cb);
		public int indy_fetch_wallet_search_next_records(int command_handle, int wallet_handle, int wallet_search_handle, int count, Callback cb);
		public int indy_close_wallet_search(int command_handle, int wallet_search_handle, Callback cb);
		public int indy_get_schema(int command_handle, int pool_handle, int wallet_handle, String submitter_did, String id, String options_json, Callback cb);
		public int indy_get_cred_def(int command_handle, int pool_handle, int wallet_handle, String submitter_did, String id, String options_json, Callback cb);
		public int indy_purge_schema_cache(int command_handle, int wallet_handle, String options_json, Callback cb);
		public int indy_purge_cred_def_cache(int command_handle, int wallet_handle, String options_json, Callback cb);

		// payments.rs
		int indy_create_payment_address(int command_handle, int wallet_handle, String payment_method, String config, Callback cb);
		int indy_list_payment_addresses(int command_handle, int wallet_handle, Callback cb);
		int indy_add_request_fees(int command_handle, int wallet_handle, String submitter_did, String req_json, String inputs_json, String outputs_json, String extra, Callback cb);
		int indy_parse_response_with_fees(int command_handle, String payment_method, String resp_json, Callback cb);
		int indy_build_get_payment_sources_request(int command_handle, int wallet_handle, String submitter_did, String payment_address, Callback cb);
		int indy_build_get_payment_sources_with_from_request(int command_handle, int wallet_handle, String submitter_did, String payment_address, int from, Callback cb);
		int indy_parse_get_payment_sources_response(int command_handle, String payment_method, String resp_json, Callback cb);
		int indy_parse_get_payment_sources_with_from_response(int command_handle, String payment_method, String resp_json, Callback cb);
		int indy_build_payment_req(int command_handle, int wallet_handle, String submitter_did, String inputs_json, String outputs_json, String extra, Callback cb);
		int indy_parse_payment_response(int command_handle, String payment_method, String resp_json, Callback cb);
		int indy_build_mint_req(int command_handle, int wallet_handle, String submitter_did, String outputs_json, String extra, Callback cb);
		int indy_build_set_txn_fees_req(int command_handle, int wallet_handle, String submitter_did, String payment_method, String fees_json, Callback cb);
		int indy_build_get_txn_fees_req(int command_handle, int wallet_handle, String submitter_did, String payment_method, Callback cb);
		int indy_parse_get_txn_fees_response(int command_handle, String payment_method, String resp_json, Callback cb);
		int indy_build_verify_payment_req(int command_handle, int wallet_handle, String submitter_did, String receipt, Callback cb);
		int indy_parse_verify_payment_response(int command_handle, String payment_method, String resp_json, Callback cb);
		int indy_prepare_payment_extra_with_acceptance_data(int command_handle, String extra_json, String text, String version, String hash, String acc_mech_type, long time_of_acceptance, Callback cb);
		int indy_get_request_info(int command_handle, String get_auth_rule_response_json, String requester_info_json, String fees_json, Callback cb);
		int indy_sign_with_address(int command_handle, int wallet_handle, String address, byte[] message_raw, int message_len, Callback cb);
		int indy_verify_with_address(int command_handle, String address, byte[] message_raw, int message_len, byte[] signature_raw, int signature_len, Callback cb);

		// metrics.rs
		int indy_collect_metrics(int command_handle, Callback cb);

		int indy_set_logger(Pointer context, Callback enabled, Callback log, Callback flush);
		int indy_set_logger_with_max_lvl(Pointer context, Callback enabled, Callback log, Callback flush, int max_lvl);
		int indy_set_log_max_lvl(int max_lvl);

		int indy_set_runtime_config(String config);
		int indy_get_current_error(PointerByReference error);

		interface BoolCallback extends Callback {
			void callback(int xcommand_handle, int err, IndyBool valid);
		}
	}

	/*
	 * Initialization
	 */

	public static API api = null;

	static {
		MAPPER.addTypeConverter(IndyBool.class, IndyBool.MAPPER);

		try {

			init();
		} catch (UnsatisfiedLinkError ex) {

			// Library could not be found in standard OS locations.
			// Call init(File file) explicitly with absolute library path.
		}
	}

	/**
	 * Initializes the API with the path to the C-Callable library.
	 *
	 * @param searchPath The path to the directory containing the C-Callable library file.
	 */
	public static void init(String searchPath) {

		NativeLibrary.addSearchPath(LIBRARY_NAME, searchPath);
		init();
	}

	/**
	 * Initializes the API with the path to the C-Callable library.
	 * Warning: This is not platform-independent.
	 *
	 * @param file The absolute path to the C-Callable library file.
	 */
	public static void init(File file) {
		Map<String, Object> options = new HashMap<String, Object>();
		options.put(Library.OPTION_TYPE_MAPPER, MAPPER);

		api = Native.loadLibrary(file.getAbsolutePath(), API.class, options);
		initLogger();
	}

	/**
	 * Initializes the API with the default library.
	 */
	public static void init() {
		Map<String, Object> options = new HashMap<String, Object>();
		options.put(Library.OPTION_TYPE_MAPPER, MAPPER);

		api = Native.loadLibrary(LIBRARY_NAME, API.class, options);
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
				detach(false);

				org.slf4j.Logger logger = org.slf4j.LoggerFactory.getLogger(LIB_INDY_LOGGER_PREFIX + target.replace("::", "."));

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
		org.slf4j.Logger logger = org.slf4j.LoggerFactory.getLogger(LIB_INDY_LOGGER_PREFIX);
		int logLevel;
		if (logger.isTraceEnabled()) {
			logLevel = 5;
		} else if (logger.isDebugEnabled()) {
			logLevel = 4;
		} else if (logger.isInfoEnabled()) {
			logLevel = 3;
		} else if (logger.isWarnEnabled()) {
			logLevel = 2;
		} else if (logger.isErrorEnabled()) {
			logLevel = 1;
		} else { // Off
			logLevel = 0;
		}
		api.indy_set_logger_with_max_lvl(null, Logger.enabled, Logger.log, Logger.flush, logLevel);
	}

	/**
	 * Set libindy runtime configuration. Can be optionally called to change current params.
	 *
	 * @param config config: {
	 *     "crypto_thread_pool_size": Optional[int] - size of thread pool for the most expensive crypto operations. (4 by default)
	 *     "collect_backtrace": Optional[bool] - whether errors backtrace should be collected.
	 *         Capturing of backtrace can affect library performance.
	 *         NOTE: must be set before invocation of any other API functions.
	 *  }
	 */
	public static void setRuntimeConfig(String config) {
		api.indy_set_runtime_config(config);
	}
}
