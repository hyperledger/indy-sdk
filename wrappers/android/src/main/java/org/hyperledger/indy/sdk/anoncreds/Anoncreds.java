package org.hyperledger.indy.sdk.anoncreds;

import java9.util.concurrent.CompletableFuture;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateSchemaResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateAndStoreCredentialDefResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateAndStoreRevocRegResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateCredentialResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.ProverCreateCredentialRequestResult;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageWriter;
import org.hyperledger.indy.sdk.wallet.Wallet;

import com.sun.jna.Callback;

import static org.hyperledger.indy.sdk.Callbacks.boolCallback;

/**
 * anoncreds.rs API
 */

/**
 * Functionality for anonymous credentials
 * 
 * These functions wrap the Ursa algorithm as documented in this paper:
 * https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
 *
 * And is documented in this HIPE:
 * https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
 * 
 */
public class Anoncreds extends IndyJava.API {

	private Anoncreds() {

	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when issuerCreateSchema completes.
	 */
	private static Callback issuerCreateSchemaCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String schema_id, String schema_json) {

			CompletableFuture<IssuerCreateSchemaResult> future = (CompletableFuture<IssuerCreateSchemaResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			IssuerCreateSchemaResult result = new IssuerCreateSchemaResult(schema_id, schema_json);
			future.complete(result);
		}
	};

	/**
	 * Callback used when issuerCreateAndStoreCredentialDef completes.
	 */
	private static Callback issuerCreateAndStoreCredentialDefCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String credential_def_id, String credential_def_json) {

			CompletableFuture<IssuerCreateAndStoreCredentialDefResult> future = (CompletableFuture<IssuerCreateAndStoreCredentialDefResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			IssuerCreateAndStoreCredentialDefResult result = new IssuerCreateAndStoreCredentialDefResult(credential_def_id, credential_def_json);
			future.complete(result);
		}
	};

	/**
	 * Callback used when issuerCreateAndStoreRevocReg completes.
	 */
	private static Callback issuerCreateAndStoreRevocRegCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String revoc_reg_id, String revoc_reg_def_json, String revoc_reg_entry_json) {

			CompletableFuture<IssuerCreateAndStoreRevocRegResult> future = (CompletableFuture<IssuerCreateAndStoreRevocRegResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			IssuerCreateAndStoreRevocRegResult result = new IssuerCreateAndStoreRevocRegResult(revoc_reg_id, revoc_reg_def_json, revoc_reg_entry_json);
			future.complete(result);
		}
	};

	/**
	 * Callback used when function returning string completes.
	 */
	static Callback stringCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String str) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = str;
			future.complete(result);
		}
	};

	/**
	 * Callback used when issuerCreateCredential completes.
	 */
	private static Callback issuerCreateCredentialCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String cred_json, String cred_rev_id, String revoc_reg_delta_json) {

			CompletableFuture<IssuerCreateCredentialResult> future = (CompletableFuture<IssuerCreateCredentialResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			IssuerCreateCredentialResult result = new IssuerCreateCredentialResult(cred_json, cred_rev_id, revoc_reg_delta_json);
			future.complete(result);
		}
	};


	/**
	 * Callback used when proverCreateCredentialReq completes.
	 */
	private static Callback proverCreateCredentialReqCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String credential_req_json, String credential_req_metadata_json) {

			CompletableFuture<ProverCreateCredentialRequestResult> future = (CompletableFuture<ProverCreateCredentialRequestResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			ProverCreateCredentialRequestResult result = new ProverCreateCredentialRequestResult(credential_req_json, credential_req_metadata_json);
			future.complete(result);
		}
	};

	/**
	 * Callback used when function with empty result completes.
	 */
	private static Callback voidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/*
	 * STATIC METHODS
	 */

	/**
	 * Create credential schema entity that describes credential attributes list and allows credentials
	 * interoperability.
	 *
	 * Schema is public and intended to be shared with all anoncreds workflow actors usually by publishing SCHEMA transaction
	 * to Indy distributed ledger.
	 *
	 * It is IMPORTANT for current version POST Schema in Ledger and after that GET it from Ledger
	 * with correct seq_no to save compatibility with Ledger.
	 * After that can call indy_issuer_create_and_store_credential_def to build corresponding Credential Definition.
	 *
	 * @param issuerDid The DID of the issuer.
	 * @param name      Human-readable name of schema.
	 * @param version   Version of schema.
	 * @param attrs:     list of schema attributes descriptions (the number of attributes should be less or equal than 125)
	 *     `["attr1", "attr2"]`
	 * @return A future resolving to IssuerCreateSchemaResult object containing:
	 * schemaId: identifier of created schema
	 * schemaJson: schema as json
	 * {
	 *     id: identifier of schema
	 *     attrNames: array of attribute name strings
	 *     name: schema's name string
	 *     version: schema's version string,
	 *     ver: version of the Schema json
	 * }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<IssuerCreateSchemaResult> issuerCreateSchema(
			String issuerDid,
			String name,
			String version,
			String attrs) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(issuerDid, "issuerDid");
		ParamGuard.notNullOrWhiteSpace(name, "name");
		ParamGuard.notNullOrWhiteSpace(version, "version");
		ParamGuard.notNullOrWhiteSpace(attrs, "attrs");

		CompletableFuture<IssuerCreateSchemaResult> future = new CompletableFuture<IssuerCreateSchemaResult>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_issuer_create_schema(
				commandHandle,
				issuerDid,
				name,
				version,
				attrs,
				issuerCreateSchemaCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Create credential definition entity that encapsulates credentials issuer DID, credential schema, secrets used for signing credentials
	 * and secrets used for credentials revocation.
	 * 
	 * Credential definition entity contains private and public parts. Private part will be stored in the wallet. Public part
	 * will be returned as json intended to be shared with all anoncreds workflow actors usually by publishing CRED_DEF transaction
	 * to Indy distributed ledger.
	 * 
	 * It is IMPORTANT for current version GET Schema from Ledger with correct seq_no to save compatibility with Ledger.
	 *
	 * Note: Use combination of `issuerRotateCredentialDefStart` and `issuerRotateCredentialDefApply` functions
	 * to generate new keys for an existing credential definition.
	 *
	 * @param wallet     The wallet.
	 * @param issuerDid  DID of the issuer signing cred_def transaction to the Ledger
	 * @param schemaJson Credential schema as a json: {
	 *     id: identifier of schema
	 *     attrNames: array of attribute name strings
	 *     name: schema's name string
	 *     version: schema's version string,
	 *     seqNo: (Optional) schema's sequence number on the ledger,
	 *     ver: version of the Schema json
	 * }
	 * @param tag        Allows to distinct between credential definitions for the same issuer and schema
	 * @param signatureType       Credential definition signatureType (optional, 'CL' by default) that defines credentials signature and revocation math.
	 *                   Supported types are:
	 *                   - 'CL': Camenisch-Lysyanskaya credential signature type that is implemented according to the algorithm in this paper:
	 *                              https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
	 *                          And is documented in this HIPE:
	 *                              https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
	 * @param configJson (optional) Type-specific configuration of credential definition as json:
	 *                   - 'CL':
	 *                      {
	 *                          "support_revocation" - bool (optional, default false) whether to request non-revocation credential
	 *                      }
	 * @return A future resolving to IssuerCreateAndStoreCredentialDefResult containing:.
	 * credDefId: identifier of created credential definition.
	 * credDefJson: public part of created credential definition
	 * {
	 *     id: string - identifier of credential definition
	 *     schemaId: string - identifier of stored in ledger schema
	 *     type: string - type of the credential definition. CL is the only supported type now.
	 *     tag: string - allows to distinct between credential definitions for the same issuer and schema
	 *     value: Dictionary with Credential Definition's data is depended on the signature type: {
	 *         primary: primary credential public key,
	 *         Optional(revocation): revocation credential public key
	 *     },
	 *     ver: Version of the CredDef json
	 * }
	 * Note: `primary` and `revocation` fields of credential definition are complex opaque types that contain data structures internal to Ursa.
	 * They should not be parsed and are likely to change in future versions.
	 * 
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<IssuerCreateAndStoreCredentialDefResult> issuerCreateAndStoreCredentialDef(
			Wallet wallet,
			String issuerDid,
			String schemaJson,
			String tag,
			String signatureType,
			String configJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(issuerDid, "issuerDid");
		ParamGuard.notNullOrWhiteSpace(schemaJson, "schemaJson");
		ParamGuard.notNullOrWhiteSpace(tag, "tag");

		CompletableFuture<IssuerCreateAndStoreCredentialDefResult> future = new CompletableFuture<IssuerCreateAndStoreCredentialDefResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_create_and_store_credential_def(
				commandHandle,
				walletHandle,
				issuerDid,
				schemaJson,
				tag,
				signatureType,
				configJson,
				issuerCreateAndStoreCredentialDefCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Generate temporary credential definitional keys for an existing one (owned by the caller of the library).
	 *
	 * Use `issuerRotateCredentialDefApply` function to set temporary keys as the main.
	 *
	 * WARNING: Rotating the credential definitional keys will result in making all credentials issued under the previous keys unverifiable.
	 *
	 * @param wallet     The wallet.
	 * @param credDefId  An identifier of created credential definition stored in the wallet
	 * @param configJson (optional) Type-specific configuration of credential definition as json:
	 *                   - 'CL':
	 *                      {
	 *                          "support_revocation" - bool (optional, default false) whether to request non-revocation credential
	 *                      }
	 *
	 * @return A future resolving to IssuerCreateAndStoreCredentialDefResult containing:.
	 * credDefJson: public part of temporary created credential definition
	 *
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> issuerRotateCredentialDefStart(
			Wallet wallet,
			String credDefId,
			String configJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credDefId, "credDefId");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_rotate_credential_def_start(
				commandHandle,
				walletHandle,
				credDefId,
				configJson,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Apply temporary keys as main for an existing Credential Definition (owned by the caller of the library).
	 *
	 * WARNING: Rotating the credential definitional keys will result in making all credentials issued under the previous keys unverifiable.
	 *
	 * @param wallet     The wallet.
	 * @param credDefId  An identifier of created credential definition stored in the wallet
	 *
	 * @return A future resolving to no value
	 *
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> issuerRotateCredentialDefApply(
			Wallet wallet,
			String credDefId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credDefId, "credDefId");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_rotate_credential_def_apply(
				commandHandle,
				walletHandle,
				credDefId,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Create a new revocation registry for the given credential definition as tuple of entities:
	 * - Revocation registry definition that encapsulates credentials definition reference, revocation revoc_def_type specific configuration and
	 * secrets used for credentials revocation
	 * - Revocation registry state that stores the information about revoked entities in a non-disclosing way. The state can be
	 * represented as ordered list of revocation registry entries were each entry represents the list of revocation or issuance operations.
	 * 
	 * Revocation registry definition entity contains private and public parts. Private part will be stored in the wallet. Public part
	 * will be returned as json intended to be shared with all anoncreds workflow actors usually by publishing REVOC_REG_DEF transaction
	 * to Indy distributed ledger.
	 * 
	 * Revocation registry state is stored on the wallet and also intended to be shared as the ordered list of REVOC_REG_ENTRY transactions.
	 * This call initializes the state in the wallet and returns the initial entry.
	 * 
	 * Some revocation registry types (for example, 'CL_ACCUM') can require generation of binary blob called tails used to hide information about revoked credentials in public
	 * revocation registry and intended to be distributed out of leger (REVOC_REG_DEF transaction will still contain uri and hash of tails).
	 * This call requires access to pre-configured blob storage writer instance handle that will allow to write generated tails.
	 *
	 * @param wallet      The wallet.
	 * @param issuerDid   The DID of the issuer.
	 * @param revoc_def_type        Revocation registry revoc_def_type (optional, default value depends on credential definition revoc_def_type). Supported types are:
	 *                    - 'CL_ACCUM': Type-3 pairing based accumulator implemented according to the algorithm in this paper:
	 *                          https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
	 *                          This type is default for 'CL' credential definition type.
	 * @param tag         Allows to distinct between revocation registries for the same issuer and credential definition
	 * @param credDefId   Id of stored in ledger credential definition
	 * @param configJson  revoc_def_type-specific configuration of revocation registry as json:
	 * - 'CL_ACCUM': {
	 *     "issuance_type": (optional) revoc_def_type of issuance. Currently supported:
	 *         1) ISSUANCE_BY_DEFAULT: all indices are assumed to be issued and initial accumulator is calculated over all indices;
	 *            Revocation Registry is updated only during revocation.
	 *         2) ISSUANCE_ON_DEMAND: nothing is issued initially accumulator is 1 (used by default);
	 *     "max_cred_num": maximum number of credentials the new registry can process (optional, default 100000)
	 * }
	 * @param tailsWriter Handle of blob storage to store tails
	 *
	 * NOTE:
	 *      Recursive creation of folder for Default Tails Writer (correspondent to `tailsWriter`)
	 *      in the system-wide temporary directory may fail in some setup due to permissions: `IO error: Permission denied`.
	 *      In this case use `TMPDIR` environment variable to define temporary directory specific for an application.
	 *
	 * @return A future resolving to:
	 * revocRegId: identifier of created revocation registry definition
	 * revocRegDefJson: public part of revocation registry definition
	 *     {
	 *         "id": string - ID of the Revocation Registry,
	 *         "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
	 *         "tag": string - Unique descriptive ID of the Registry,
	 *         "credDefId": string - ID of the corresponding CredentialDefinition,
	 *         "value": Registry-specific data {
	 *             "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
	 *             "maxCredNum": number - Maximum number of credentials the Registry can serve.
	 *             "tailsHash": string - Hash of tails.
	 *             "tailsLocation": string - Location of tails file.
	 *             "publicKeys": (public_keys) - Registry's public key (opaque type that contains data structures internal to Ursa.
	 *                                                                  It should not be parsed and are likely to change in future versions).
	 *         },
	 *         "ver": string - version of revocation registry definition json.
	 *     }
	 * revocRegEntryJson: revocation registry entry that defines initial state of revocation registry
	 * {
	 *     value: {
	 *         prevAccum: string - previous accumulator value.
	 *         accum: string - current accumulator value.
	 *         issued: array(number) - an array of issued indices.
	 *         revoked: array(number) an array of revoked indices.
	 *     },
	 *     ver: string - version revocation registry entry json
	 * }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<IssuerCreateAndStoreRevocRegResult> issuerCreateAndStoreRevocReg(
			Wallet wallet,
			String issuerDid,
			String revoc_def_type,
			String tag,
			String credDefId,
			String configJson,
			BlobStorageWriter tailsWriter) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(issuerDid, "issuerDid");
		ParamGuard.notNullOrWhiteSpace(tag, "tag");
		ParamGuard.notNullOrWhiteSpace(credDefId, "credDefId");
		ParamGuard.notNullOrWhiteSpace(configJson, "configJson");

		CompletableFuture<IssuerCreateAndStoreRevocRegResult> future = new CompletableFuture<IssuerCreateAndStoreRevocRegResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_create_and_store_revoc_reg(
				commandHandle,
				walletHandle,
				issuerDid,
				revoc_def_type,
				tag,
				credDefId,
				configJson,
				tailsWriter.getBlobStorageWriterHandle(),
				issuerCreateAndStoreRevocRegCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Create credential offer that will be used by Prover for
	 * credential request creation. Offer includes nonce and key correctness proof
	 * for authentication between protocol steps and integrity checking.
	 *
	 * @param wallet    The wallet.
	 * @param credDefId Id of stored in ledger credential definition.
	 * @return A future resolving to a JSON string containing the credential offer.
	 *     {
	 *         "schema_id": string,
	 *         "cred_def_id": string,
	 *         // Fields below can depend on Cred Def type
	 *         "nonce": string,
	 *         "key_correctness_proof" : key correctness proof for credential definition correspondent to cred_def_id
	 *                                   (opaque type that contains data structures internal to Ursa.
	 *                                   It should not be parsed and are likely to change in future versions).
	 *     }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> issuerCreateCredentialOffer(
			Wallet wallet,
			String credDefId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credDefId, "credDefId");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_create_credential_offer(
				commandHandle,
				walletHandle,
				credDefId,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Check Cred Request for the given Cred Offer and issue Credential for the given Cred Request.
	 * 
	 * Cred Request must match Cred Offer. The credential definition and revocation registry definition
	 * referenced in Cred Offer and Cred Request must be already created and stored into the wallet.
	 * 
	 * Information for this credential revocation will be store in the wallet as part of revocation registry under
	 * generated cred_revoc_id local for this wallet.
	 * 
	 * This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY transaction.
	 * Note that it is possible to accumulate deltas to reduce ledger load.
	 *
	 * @param wallet                  The wallet.
	 * @param credOfferJson           Cred offer created by issuerCreateCredentialOffer
	 * @param credReqJson             Credential request created by proverCreateCredentialReq
	 * @param credValuesJson          Credential containing attribute values for each of requested attribute names.
	 *                                Example:
	 *                                {
	 *                                  "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
	 *                                  "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
	 *                                }
	 *                                If you want to use empty value for some credential field, you should set "raw" to "" and "encoded" should not be empty
	 * @param revRegId                (Optional) id of stored in ledger revocation registry definition
	 * @param blobStorageReaderHandle Pre-configured blob storage reader instance handle that will allow to read revocation tails
	 * @return A future resolving to a IssuerCreateCredentialResult containing:
	 * credentialJson: Credential json containing signed credential values
	 *     {
	 *         "schema_id": string,
	 *         "cred_def_id": string,
	 *         "rev_reg_def_id", Optional[string],
	 *         "values": "see credValuesJson above",
	 *         // Fields below can depend on Cred Def type
	 *         "signature": {signature} 
	 *                      (opaque type that contains data structures internal to Ursa.
	 *                       It should not be parsed and are likely to change in future versions).
	 *         "signature_correctness_proof": {signature_correctness_proof}
	 *                      (opaque type that contains data structures internal to Ursa.
	 *                       It should not be parsed and are likely to change in future versions).
	 *         "rev_reg" - (Optional) revocation registry accumulator value on the issuing moment.
	 *                      (opaque type that contains data structures internal to Ursa.
	 *                       It should not be parsed and are likely to change in future versions).
	 *         "witness" - (Optional) revocation related data
	 *                      (opaque type that contains data structures internal to Ursa.
	 *                       It should not be parsed and are likely to change in future versions).
	 *     }
	 * credRevocId: local id for revocation info (Can be used for revocation of this cred)
	 * revocRegDeltaJson: Revocation registry delta json with a newly issued credential
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<IssuerCreateCredentialResult> issuerCreateCredential(
			Wallet wallet,
			String credOfferJson,
			String credReqJson,
			String credValuesJson,
			String revRegId,
			int blobStorageReaderHandle) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credOfferJson, "credOfferJson");
		ParamGuard.notNullOrWhiteSpace(credReqJson, "credReqJson");
		ParamGuard.notNullOrWhiteSpace(credValuesJson, "credValuesJson");

		CompletableFuture<IssuerCreateCredentialResult> future = new CompletableFuture<IssuerCreateCredentialResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_create_credential(
				commandHandle,
				walletHandle,
				credOfferJson,
				credReqJson,
				credValuesJson,
				revRegId,
				blobStorageReaderHandle,
				issuerCreateCredentialCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Revoke a credential identified by a cred_revoc_id (returned by issuerCreateCredential).
	 *
	 * The corresponding credential definition and revocation registry must be already
	 * created an stored into the wallet.
	 *
	 * This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY transaction.
	 * Note that it is possible to accumulate deltas to reduce ledger load.
	 *
	 * @param wallet                  A wallet.
	 * @param blobStorageReaderHandle Pre-configured blob storage reader instance handle that will allow to read revocation tails
	 * @param revRegId                Id of revocation registry stored in wallet.
	 * @param credRevocId             Local id for revocation info
	 * @return A future resolving to a revocation registry delta json with a revoked credential
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> issuerRevokeCredential(
			Wallet wallet,
			int blobStorageReaderHandle,
			String revRegId,
			String credRevocId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(revRegId, "revRegId");
		ParamGuard.notNull(credRevocId, "credRevocId");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_issuer_revoke_credential(
				commandHandle,
				walletHandle,
				blobStorageReaderHandle,
				revRegId,
				credRevocId,
				stringCb);

		checkResult(future, result);

		return future;
	}

//	/**
//	 * Recover a credential identified by a cred_revoc_id (returned by indy_issuer_create_credential).
//	 * <p>
//	 * The corresponding credential definition and revocation registry must be already
//	 * created an stored into the wallet.
//	 * <p>
//	 * This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY transaction.
//	 * Note that it is possible to accumulate deltas to reduce ledger load.
//	 *
//	 * @param wallet                  A wallet.
//	 * @param blobStorageReaderHandle Pre-configured blob storage reader instance handle that will allow to read revocation tails
//	 * @param revRegId                Id of revocation registry stored in wallet.
//	 * @param credRevocId             Local id for revocation info
//	 * @return A future resolving to a revocation registry update json with a recovered credential
//	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
//	 */
//	public static CompletableFuture<String> issuerRecoverCredential(
//			Wallet wallet,
//			int blobStorageReaderHandle,
//			String revRegId,
//			String credRevocId) throws IndyException {
//
//		ParamGuard.notNull(wallet, "wallet");
//		ParamGuard.notNull(revRegId, "revRegId");
//		ParamGuard.notNull(credRevocId, "credRevocId");
//
//		CompletableFuture<String> future = new CompletableFuture<String>();
//		int commandHandle = addFuture(future);
//
//		int walletHandle = wallet.getWalletHandle();
//
//		int result = LibIndy.api.indy_issuer_recover_credential(
//				commandHandle,
//				walletHandle,
//				blobStorageReaderHandle,
//				revRegId,
//				credRevocId,
//				issuerRecoverCredentialCb);
//
//		checkResult(future, result);
//
//		return future;
//	}

	/**
	 * Merge two revocation registry deltas (returned by issuerCreateCredential or issuerRevokeCredential) to accumulate common delta.
	 * Send common delta to ledger to reduce the load.
	 *
	 * @param revRegDelta      Revocation registry delta json.
	 * @param otherRevRegDelta Revocation registry delta for which PrevAccum value  is equal to current accum value of revRegDelta.
	 * @return A future resolving to a merged revocation registry delta.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> issuerMergeRevocationRegistryDeltas(
			String revRegDelta,
			String otherRevRegDelta) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(revRegDelta, "revRegDelta");
		ParamGuard.notNullOrWhiteSpace(otherRevRegDelta, "otherRevRegDelta");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_issuer_merge_revocation_registry_deltas(
				commandHandle,
				revRegDelta,
				otherRevRegDelta,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Creates a master secret with a given name and stores it in the wallet.
	 *
	 * @param wallet         A wallet.
	 * @param masterSecretId (Optional, if not present random one will be generated) New master id
	 * @return A future resolving to id of generated master secret.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> proverCreateMasterSecret(
			Wallet wallet,
			String masterSecretId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_create_master_secret(
				commandHandle,
				walletHandle,
				masterSecretId,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Creates a credential request for the given credential offer.
	 * 
	 * The method creates a blinded master secret for a master secret identified by a provided name.
	 * The master secret identified by the name must be already stored in the secure wallet (see proverCreateMasterSecret)
	 * The blinded master secret is a part of the credential request.
	 *
	 * @param wallet              A wallet.
	 * @param proverDid           The DID of the prover.
	 * @param credentialOfferJson Credential offer as a json containing information about the issuer and a credential
	 *     {
	 *         "schema_id": string, - identifier of schema
	 *         "cred_def_id": string, - identifier of credential definition
	 *          ...
	 *         Other fields that contains data structures internal to Ursa.
	 *         These fields should not be parsed and are likely to change in future versions.
	 *     }
	 * @param credentialDefJson   Credential definition json realted to cred_def_id in credentialOfferJson
	 * @param masterSecretId      The id of the master secret stored in the wallet
	 * @return A future that resolves to:
	 * * credReqJson: Credential request json for creation of credential by Issuer
	 *     {
	 *      "prover_did" : string,
	 *      "cred_def_id" : string,
	 *         // Fields below can depend on Cred Def type
	 *      "blinded_ms" : {blinded_master_secret},
	 *                      (opaque type that contains data structures internal to Ursa.
	 *                       It should not be parsed and are likely to change in future versions).
	 *      "blinded_ms_correctness_proof" : {blinded_ms_correctness_proof},
	 *                      (opaque type that contains data structures internal to Ursa.
	 *                       It should not be parsed and are likely to change in future versions).
	 *      "nonce": string
	 *    }
	 * credReqMetadataJson: Credential request metadata json for further processing of received form Issuer credential.
	 *     Credential request metadata contains data structures internal to Ursa.
	 *     Credential request metadata mustn't be shared with Issuer.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<ProverCreateCredentialRequestResult> proverCreateCredentialReq(
			Wallet wallet,
			String proverDid,
			String credentialOfferJson,
			String credentialDefJson,
			String masterSecretId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(proverDid, "proverDid");
		ParamGuard.notNullOrWhiteSpace(credentialOfferJson, "credentialOfferJson");
		ParamGuard.notNullOrWhiteSpace(credentialDefJson, "credentialDefJson");
		ParamGuard.notNullOrWhiteSpace(masterSecretId, "masterSecretId");

		CompletableFuture<ProverCreateCredentialRequestResult> future = new CompletableFuture<ProverCreateCredentialRequestResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_create_credential_req(
				commandHandle,
				walletHandle,
				proverDid,
				credentialOfferJson,
				credentialDefJson,
				masterSecretId,
				proverCreateCredentialReqCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Check credential provided by Issuer for the given credential request,
	 * updates the credential by a master secret and stores in a secure wallet.
	 *
	 * To support efficient and flexible search the following tags will be created for stored credential:
	 *     {
	 *         "schema_id": "credential schema id",
	 *         "schema_issuer_did": "credential schema issuer did",
	 *         "schema_name": "credential schema name",
	 *         "schema_version": "credential schema version",
	 *         "issuer_did": "credential issuer did",
	 *         "cred_def_id": "credential definition id",
	 *         // for every attribute in credValuesJson
	 *         "attr::{attribute name}::marker": "1",
	 *         "attr::{attribute name}::value": "attribute raw value",
	 *     }
	 * 
	 * @param wallet              A Wallet.
	 * @param credId              (optional, default is a random one) Identifier by which credential will be stored in the wallet
	 * @param credReqMetadataJson Credential request metadata created by proverCreateCredentialReq
	 * @param credJson            Credential json received from issuer
	 * @param credDefJson         Credential definition json related to cred_def_id in credJson
	 * @param revRegDefJson       Revocation registry definition json related to rev_reg_def_id in credJson
	 * @return A future that  resolve to identifier by which credential is stored in the wallet.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> proverStoreCredential(
			Wallet wallet,
			String credId,
			String credReqMetadataJson,
			String credJson,
			String credDefJson,
			String revRegDefJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credReqMetadataJson, "credReqMetadataJson");
		ParamGuard.notNullOrWhiteSpace(credJson, "credJson");
		ParamGuard.notNullOrWhiteSpace(credDefJson, "credDefJson");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_store_credential(
				commandHandle,
				walletHandle,
				credId,
				credReqMetadataJson,
				credJson,
				credDefJson,
				revRegDefJson,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Gets human readable credentials according to the filter.
	 * If filter is NULL, then all credentials are returned.
	 * Credentials can be filtered by tags created during saving of credential.
	 *
	 * NOTE: This method is deprecated because immediately returns all fetched credentials.
	 * Use {@link CredentialsSearch#open(Wallet, String)} to fetch records by small batches.
	 *
	 * @param wallet A wallet.
	 * @param filter filter for credentials
	 *        {
	 *            "schema_id": string, (Optional)
	 *            "schema_issuer_did": string, (Optional)
	 *            "schema_name": string, (Optional)
	 *            "schema_version": string, (Optional)
	 *            "issuer_did": string, (Optional)
	 *            "cred_def_id": string, (Optional)
	 *        }
	 * @return A future that resolves to a credentials json
	 *     [{
	 *         "referent": string, - id of credential in the wallet
	 *         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
	 *         "schema_id": string, - identifier of schema
	 *         "cred_def_id": string, - identifier of credential definition
	 *         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
	 *         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
	 *     }]
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> proverGetCredentials(
			Wallet wallet,
			String filter) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(filter, "filter");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_get_credentials(
				commandHandle,
				walletHandle,
				filter,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Gets human readable credential by the given id.
	 *
	 * @param wallet A wallet.
	 * @param credId Identifier by which requested credential is stored in the wallet
	 * @return credential json
	 * {
	 *      "referent": string, - id of credential in the wallet
	 *      "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
	 *      "schema_id": string, - identifier of schema
	 *      "cred_def_id": string, - identifier of credential definition
	 *      "rev_reg_id": Optional<string>, - identifier of revocation registry definition
	 *      "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
	 * }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> proverGetCredential(
			Wallet wallet,
			String credId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credId, "credId");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_get_credential(
				commandHandle,
				walletHandle,
				credId,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Deletes credential by given id.
	 *
	 * @param wallet A wallet.
	 * @param credId Identifier by which requested credential is stored in the wallet
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> proverDeleteCredential(
			Wallet wallet,
			String credId) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(credId, "credId");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_delete_credential(
				commandHandle,
				walletHandle,
				credId,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Gets human readable credentials matching the given proof request.
	 *
	 * NOTE: This method is deprecated because immediately returns all fetched credentials.
	 * Use {@link CredentialsSearchForProofReq#open(Wallet, String, String)} to fetch records by small batches.
	 *
	 * @param wallet       A wallet.
	 * @param proofRequest proof request json
	 *     {
	 *         "name": string,
	 *         "version": string,
	 *         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
	 *         "requested_attributes": { // set of requested attributes
	 *              "<attr_referent>": <attr_info>, // see below
	 *              ...,
	 *         },
	 *         "requested_predicates": { // set of requested predicates
	 *              "<predicate_referent>": <predicate_info>, // see below
	 *              ...,
	 *          },
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval for each attribute
	 *                        // (applies to every attribute and predicate but can be overridden on attribute level)
	 *         "ver": Optional<str>  - proof request version:
	 *             - omit to use unqualified identifiers for restrictions
	 *             - "1.0" to use unqualified identifiers for restrictions
	 *             - "2.0" to use fully qualified identifiers for restrictions
	 *     }
	 * where
	 * attr_referent: Proof-request local identifier of requested attribute
	 * attr_info: Describes requested attribute
	 *     {
	 *         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
	 *         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
	 *                                              // NOTE: should either be "name" or "names", not both and not none of them.
	 *                                              // Use "names" to specify several attributes that have to match a single credential.
	 *         "restrictions": Optional<wql query>, // see below
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval this attribute
	 *                        // (overrides proof level interval)
	 *     }
	 * predicate_referent: Proof-request local identifier of requested attribute predicate
	 * predicate_info: Describes requested attribute predicate
	 *     {
	 *         "name": attribute name, (case insensitive and ignore spaces)
	 *         "p_type": predicate type (">=", ">", "<=", "<")
	 *         "p_value": int predicate value
	 *         "restrictions": Optional<filter_json>, // see below
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval this attribute
	 *                        // (overrides proof level interval)
	 *     }
	 * non_revoc_interval: Defines non-revocation interval
	 *     {
	 *         "from": Optional<int>, // timestamp of interval beginning
	 *         "to": Optional<int>, // timestamp of interval ending
	 *     }
	 *  filter_json:
	 *     {
	 *        "schema_id": string, (Optional)
	 *        "schema_issuer_did": string, (Optional)
	 *        "schema_name": string, (Optional)
	 *        "schema_version": string, (Optional)
	 *        "issuer_did": string, (Optional)
	 *        "cred_def_id": string, (Optional)
	 *     }
	 *                        
	 * @return A future that resolves to a json with credentials for the given proof request.
	 *     {
	 *         "attrs": {
	 *             "<attr_referent>": [{ cred_info: <credential_info>, interval: Optional<non_revoc_interval> }],
	 *             ...,
	 *         },
	 *         "predicates": {
	 *             "requested_predicates": [{ cred_info: <credential_info>, timestamp: Optional<integer> }, { cred_info: <credential_2_info>, timestamp: Optional<integer> }],
	 *             "requested_predicate_2_referent": [{ cred_info: <credential_2_info>, timestamp: Optional<integer> }]
	 *         }
	 *     }, where <credential_info> is
	 *     {
	 *         "referent": string, - id of credential in the wallet
	 *         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
	 *         "schema_id": string, - identifier of schema
	 *         "cred_def_id": string, - identifier of credential definition
	 *         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
	 *         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
	 *     }
	 *     
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> proverGetCredentialsForProofReq(
			Wallet wallet,
			String proofRequest) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(proofRequest, "proofRequest");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_get_credentials_for_proof_req(
				commandHandle,
				walletHandle,
				proofRequest,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Creates a proof according to the given proof request.
	 *
	 * @param wallet               A wallet.
	 * @param proofRequest proof request json
	 *     {
	 *         "name": string,
	 *         "version": string,
	 *         "nonce": string, - a decimal number represented as a string (use `generateNonce` function to generate 80-bit number)
	 *         "requested_attributes": { // set of requested attributes
	 *              "<attr_referent>": <attr_info>, // see below
	 *              ...,
	 *         },
	 *         "requested_predicates": { // set of requested predicates
	 *              "<predicate_referent>": <predicate_info>, // see below
	 *              ...,
	 *          },
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval for each attribute
	 *                        // (applies to every attribute and predicate but can be overridden on attribute level)
	 *                        // (can be overridden on attribute level)
	 *         "ver": Optional<str>  - proof request version:
	 *             - omit to use unqualified identifiers for restrictions
	 *             - "1.0" to use unqualified identifiers for restrictions
	 *             - "2.0" to use fully qualified identifiers for restrictions
	 *     }
	 * @param requestedCredentials either a credential or self-attested attribute for each requested attribute
	 *     {
	 *         "self_attested_attributes": {
	 *             "self_attested_attribute_referent": string
	 *         },
	 *         "requested_attributes": {
	 *             "requested_attribute_referent_1": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }},
	 *             "requested_attribute_referent_2": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }}
	 *         },
	 *         "requested_predicates": {
	 *             "requested_predicates_referent_1": {"cred_id": string, "timestamp": Optional<number> }},
	 *         }
	 *     }
	 * @param masterSecret         Id of the master secret stored in the wallet
	 * @param schemas              All schemas json participating in the proof request
	 *     {
	 *         "schema1_id": {schema1},
	 *         "schema2_id": {schema2},
	 *         "schema3_id": {schema3},
	 *     }
	 * @param credentialDefs       All credential definitions json participating in the proof request
	 *     {
	 *         "cred_def1_id": {credential_def1},
	 *         "cred_def2_id": {credential_def2},
	 *         "cred_def3_id": {credential_def3},
	 *     }
	 * @param revStates            All revocation states json participating in the proof request
	 *     {
	 *         "rev_reg_def1_id  or credential_1_id": {
	 *             "timestamp1": {rev_state1},
	 *             "timestamp2": {rev_state2},
	 *         },
	 *         "rev_reg_def2_id  or credential_2_id": {
	 *             "timestamp3": {rev_state3}
	 *         },
	 *         "rev_reg_def3_id  or credential_3_id": {
	 *             "timestamp4": {rev_state4}
	 *         },
	 *     } - Note: use credential_id instead rev_reg_id in case proving several credentials from the same revocation registry.
	 * where
	 * attr_referent: Proof-request local identifier of requested attribute
	 * attr_info: Describes requested attribute
	 *     {
	 *         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
	 *         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
	 *                                              // NOTE: should either be "name" or "names", not both and not none of them.
	 *                                              // Use "names" to specify several attributes that have to match a single credential.
	 *         "restrictions": Optional<wql query>, // see below
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval this attribute
	 *                        // (overrides proof level interval)
	 *     }
	 * predicate_referent: Proof-request local identifier of requested attribute predicate
	 * predicate_info: Describes requested attribute predicate
	 *     {
	 *         "name": attribute name, (case insensitive and ignore spaces)
	 *         "p_type": predicate type (">=", ">", "<=", "<")
	 *         "p_value": predicate value
	 *         "restrictions": Optional<wql query>, // see below
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval this attribute
	 *                        // (overrides proof level interval)
	 *     }
	 * non_revoc_interval: Defines non-revocation interval
	 *     {
	 *         "from": Optional<int>, // timestamp of interval beginning
	 *         "to": Optional<int>, // timestamp of interval ending
	 *     }
	 * where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
	 *     The list of allowed fields:
	 *         "schema_id": <credential schema id>,
	 *         "schema_issuer_did": <credential schema issuer did>,
	 *         "schema_name": <credential schema name>,
	 *         "schema_version": <credential schema version>,
	 *         "issuer_did": <credential issuer did>,
	 *         "cred_def_id": <credential definition id>,
	 *         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
	 *                       	
	 * @return A future resolving to a Proof json
	 * For each requested attribute either a proof (with optionally revealed attribute value) or
	 * self-attested attribute value is provided.
	 * Each proof is associated with a credential and corresponding schema_id, cred_def_id, rev_reg_id and timestamp.
	 * There is also aggregated proof part common for all credential proofs.
	 *     {
	 *         "requested_proof": {
	 *             "revealed_attrs": {
	 *                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string},
	 *                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string},
	 *             },
	 *             "revealed_attr_groups": {
	 *                 "requested_attr5_id": {
	 *                     "sub_proof_index": number,
	 *                     "values": {
	 *                         "attribute_name": {
	 *                             "raw": string,
	 *                             "encoded": string
	 *                         }
	 *                     },
	 *                 }
	 *             },
	 *             "unrevealed_attrs": {
	 *                 "requested_attr3_id": {sub_proof_index: number}
	 *             },
	 *             "self_attested_attrs": {
	 *                 "requested_attr2_id": self_attested_value,
	 *             },
	 *             "predicates": {
	 *                 "requested_predicate_1_referent": {sub_proof_index: int},
	 *                 "requested_predicate_2_referent": {sub_proof_index: int},
	 *             }
	 *         }
	 *         "proof": {
	 *             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
	 *             "aggregated_proof": <aggregated_proof>
	 *         } (opaque type that contains data structures internal to Ursa.
	 *           It should not be parsed and are likely to change in future versions).
	 *         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
	 *     }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> proverCreateProof(
			Wallet wallet,
			String proofRequest,
			String requestedCredentials,
			String masterSecret,
			String schemas,
			String credentialDefs,
			String revStates) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(proofRequest, "proofRequest");
		ParamGuard.notNullOrWhiteSpace(requestedCredentials, "requestedCredentials");
		ParamGuard.notNullOrWhiteSpace(schemas, "schemas");
		ParamGuard.notNullOrWhiteSpace(masterSecret, "masterSecret");
		ParamGuard.notNullOrWhiteSpace(credentialDefs, "credentialDefs");
		ParamGuard.notNullOrWhiteSpace(revStates, "revStates");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_create_proof(
				commandHandle,
				walletHandle,
				proofRequest,
				requestedCredentials,
				masterSecret,
				schemas,
				credentialDefs,
				revStates,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Verifies a proof (of multiple credential).
	 * All required schemas, public keys and revocation registries must be provided.
	 *
	 * IMPORTANT: You must use *_id's (`schema_id`, `cred_def_id`, `rev_reg_id`) listed in `proof[identifiers]`
	 * as the keys for corresponding `schemas`, `credentialDefs`, `revocRegDefs`, `revocRegs` objects.
	 *
	 * @param proofRequest   proof request json
	 *     {
	 *         "name": string,
	 *         "version": string,
	 *         "nonce": string, - a decimal number represented as a string (use `generateNonce` function to generate 80-bit number)
	 *         "requested_attributes": { // set of requested attributes
	 *              "<attr_referent>": <attr_info>, // see below
	 *              ...,
	 *         },
	 *         "requested_predicates": { // set of requested predicates
	 *              "<predicate_referent>": <predicate_info>, // see below
	 *              ...,
	 *          },
	 *         "non_revoked": Optional<<non_revoc_interval>>, // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval for each attribute
	 *                        // (can be overridden on attribute level)
	 *         "ver": Optional<str>  - proof request version:
	 *             - omit to use unqualified identifiers for restrictions
	 *             - "1.0" to use unqualified identifiers for restrictions
	 *             - "2.0" to use fully qualified identifiers for restrictions
	 *     }
	 * @param proof  Proof json
	 *     {
	 *         "requested_proof": {
	 *             "revealed_attrs": {
	 *                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
	 *                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
	 *             },
	 *             "revealed_attr_groups": {
	 *                 "requested_attr5_id": {
	 *                     "sub_proof_index": number,
	 *                     "values": {
	 *                         "attribute_name": {
	 *                             "raw": string,
	 *                             "encoded": string
	 *                         }
	 *                     }, // NOTE: check that `encoded` value match to `raw` value on application level
	 *                 }
	 *             },
	 *             "unrevealed_attrs": {
	 *                 "requested_attr3_id": {sub_proof_index: number}
	 *             },
	 *             "self_attested_attrs": {
	 *                 "requested_attr2_id": self_attested_value,
	 *             },
	 *             "requested_predicates": {
	 *                 "requested_predicate_1_referent": {sub_proof_index: int},
	 *                 "requested_predicate_2_referent": {sub_proof_index: int},
	 *             }
	 *         }
	 *         "proof": {
	 *             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
	 *             "aggregated_proof": <aggregated_proof>
	 *         }
	 *         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
	 *     }
	 * @param schemas        All schemas json participating in the proof request
	 *     {
	 *         "schema1_id": {schema1},
	 *         "schema2_id": {schema2},
	 *         "schema3_id": {schema3},
	 *     }
	 * @param credentialDefs  All credential definitions json participating in the proof request
	 *     {
	 *         "cred_def1_id": {credential_def1},
	 *         "cred_def2_id": {credential_def2},
	 *         "cred_def3_id": {credential_def3},
	 *     }
	 * @param revocRegDefs   All revocation registry definitions json participating in the proof
	 *     {
	 *         "rev_reg_def1_id": {rev_reg_def1},
	 *         "rev_reg_def2_id": {rev_reg_def2},
	 *         "rev_reg_def3_id": {rev_reg_def3},
	 *     }
	 * @param revocRegs      all revocation registries json participating in the proof
	 *     {
	 *         "rev_reg_def1_id": {
	 *             "timestamp1": {rev_reg1},
	 *             "timestamp2": {rev_reg2},
	 *         },
	 *         "rev_reg_def2_id": {
	 *             "timestamp3": {rev_reg3}
	 *         },
	 *         "rev_reg_def3_id": {
	 *             "timestamp4": {rev_reg4}
	 *         },
	 *     }
	 * @return A future resolving to true if signature is valid, otherwise false.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Boolean> verifierVerifyProof(
			String proofRequest,
			String proof,
			String schemas,
			String credentialDefs,
			String revocRegDefs,
			String revocRegs) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(proofRequest, "proofRequest");
		ParamGuard.notNullOrWhiteSpace(proof, "proof");
		ParamGuard.notNullOrWhiteSpace(schemas, "schemas");
		ParamGuard.notNullOrWhiteSpace(credentialDefs, "credentialDefs");
		ParamGuard.notNullOrWhiteSpace(revocRegDefs, "revocRegDefs");
		ParamGuard.notNullOrWhiteSpace(revocRegs, "revocRegs");

		CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_verifier_verify_proof(
				commandHandle,
				proofRequest,
				proof,
				schemas,
				credentialDefs,
				revocRegDefs,
				revocRegs,
				boolCallback);

		checkResult(future, result);

		return future;
	}

	/**
	 * Create revocation state for a credential that corresponds to a particular time.
	 *
	 * Note that revocation delta must cover the whole registry existence time.
	 * You can use `from`: `0` and `to`: `needed_time` as parameters for building request to get correct revocation delta.
	 *
	 * The resulting revocation state and provided timestamp can be saved and reused later with applying a new
	 * revocation delta with `updateRevocationState` function.
	 * This new delta should be received with parameters: `from`: `timestamp` and `to`: `needed_time`.
	 *
	 * @param blobStorageReaderHandle Configuration of blob storage reader handle that will allow to read revocation tails
	 * @param revRegDef               Revocation registry definition json
	 * @param revRegDelta             Revocation registry delta which covers the whole registry existence time
	 * @param timestamp               Time represented as a total number of seconds from Unix Epoch
	 * @param credRevId               user credential revocation id in revocation registry
	 * @return A future that resolves to a revocation state json:
	 *     {
	 *         "rev_reg": {revocation registry},
	 *         "witness": {witness},
	 *         "timestamp" : integer
	 *     }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> createRevocationState(
			int blobStorageReaderHandle,
			String revRegDef,
			String revRegDelta,
			long timestamp,
			String credRevId) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(revRegDef, "revRegDef");
		ParamGuard.notNullOrWhiteSpace(revRegDelta, "revRegDelta");
		ParamGuard.notNullOrWhiteSpace(credRevId, "credRevId");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_create_revocation_state(
				commandHandle,
				blobStorageReaderHandle,
				revRegDef,
				revRegDelta,
				timestamp,
				credRevId,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Create a new revocation state for a credential based on a revocation state created before.
	 * Note that provided revocation delta must cover the registry gap from based state creation until the specified time
	 * (this new delta should be received with parameters: `from`: `state_timestamp` and `to`: `needed_time`).
	 *
	 * This function reduces the calculation time.
	 *
	 * The resulting revocation state and provided timestamp can be saved and reused later by applying a new revocation delta again.
	 *
	 * @param blobStorageReaderHandle Configuration of blob storage reader handle that will allow to read revocation tails
	 * @param revState                Rrevocation registry state json
	 * @param revRegDef               Revocation registry definition json
	 * @param revRegDelta             Revocation registry definition delta which covers the gap form original `rev_state_json` creation till the requested timestamp
	 * @param timestamp               Time represented as a total number of seconds from Unix Epoch
	 * @param credRevId               user credential revocation id in revocation registry
	 * @return A future that resolves to a revocation state json:
	 *     {
	 *         "rev_reg": {revocation registry},
	 *         "witness": {witness},
	 *         "timestamp" : integer
	 *     }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> updateRevocationState(
			int blobStorageReaderHandle,
			String revState,
			String revRegDef,
			String revRegDelta,
			long timestamp,
			String credRevId) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(revState, "revState");
		ParamGuard.notNullOrWhiteSpace(revRegDef, "revRegDef");
		ParamGuard.notNullOrWhiteSpace(revRegDelta, "revRegDelta");
		ParamGuard.notNullOrWhiteSpace(credRevId, "credRevId");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_update_revocation_state(
				commandHandle,
				blobStorageReaderHandle,
				revState,
				revRegDef,
				revRegDelta,
				timestamp,
				credRevId,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Generates 80-bit numbers that can be used as a nonce for proof request.
	 *
	 * @return A future that resolves to a generated number as a string
	 *
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> generateNonce() throws IndyException {

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_generate_nonce(
				commandHandle,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Get unqualified form (short form without method) of a fully qualified entity like DIDs..
	 *
	 * This function should be used to the proper casting of fully qualified entity to unqualified form in the following cases:
	 *     Issuer, which works with fully qualified identifiers, creates a Credential Offer for Prover, which doesn't support fully qualified identifiers.
	 *     Verifier prepares a Proof Request based on fully qualified identifiers or Prover, which doesn't support fully qualified identifiers.
	 *     another case when casting to unqualified form needed
	 *
	 * @param entity target entity to toUnqualified. Can be one of:
	 *             Did
	 *             SchemaId
	 *             CredentialDefinitionId
	 *             RevocationRegistryId
	 *             Schema
	 *             CredentialDefinition
	 *             RevocationRegistryDefinition
	 *             CredentialOffer
	 *             CredentialRequest
	 *             ProofRequest
	 * @return A future that resolves to entity either in unqualified form or original if casting isn't possible
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> toUnqualified(
			String entity) throws IndyException {

		ParamGuard.notNull(entity, "entity");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_to_unqualified(
				commandHandle,
				entity,
				stringCb);

		checkResult(future, result);

		return future;
	}
}
