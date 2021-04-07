package org.hyperledger.indy.sdk.anoncreds;

import com.sun.jna.Callback;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.wallet.Wallet;

import java9.util.concurrent.CompletableFuture;

public class CredentialsSearchForProofReq extends IndyJava.API implements AutoCloseable {
	private final int searchHandle;

	private CredentialsSearchForProofReq(int searchHandle) {
		this.searchHandle = searchHandle;
	}

	/**
	 * Callback used when proverSearchCredentials completes.
	 */
	private static Callback proverSearchCredentialsForProofReqCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int search_handle) {

			CompletableFuture<CredentialsSearchForProofReq> future = (CompletableFuture<CredentialsSearchForProofReq>) removeFuture(xcommand_handle);
			if (!checkResult(future, err)) return;

			CredentialsSearchForProofReq result = new CredentialsSearchForProofReq(search_handle);
			future.complete(result);
		}
	};

	/**
	 * Callback used when a function returning Void completes.
	 */
	private static Callback voidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (!checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Search for credentials matching the given proof request.
	 *
	 * Instead of immediately returning of fetched credentials {@link Anoncreds#proverGetCredentialsForProofReq(Wallet, String)}
	 * this call returns search_handle that can be used later
	 * to fetch records by small batches (with {@link CredentialsSearchForProofReq#fetchNextCredentials(String, int)}).
	 *
	 * @param wallet 		A wallet.
	 * @param proofReqJson	proof request json
	 *     {
	 *         "name": string,
	 *         "version": string,
	 *         "nonce": string,
	 *         "requested_attributes": { // set of requested attributes
	 *              "attr_referent": {attr_info}, // see below
	 *              ...,
	 *         },
	 *         "requested_predicates": { // set of requested predicates
	 *              "predicate_referent": {predicate_info}, // see below
	 *              ...,
	 *          },
	 *         "non_revoked": Optional[{non_revoc_interval}], // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval for each attribute
	 *                        // (can be overridden on attribute level)
	 *     }
	 *     where
	 *     attr_referent: Describes requested attribute
	 *     {
	 *         "name": string, // attribute name, (case insensitive and ignore spaces)
	 *         "restrictions": Optional[{wql query}],
	 *                          // if specified, credential must satisfy to one of the given restriction.
	 *         "non_revoked": Optional[{non_revoc_interval}], // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval this attribute
	 *                        // (overrides proof level interval)
	 *     }
	 *     predicate_referent: Describes requested attribute predicate
	 *     {
	 *         "name": attribute name, (case insensitive and ignore spaces)
	 *         "p_type": predicate type (Currently {@code ">=" } only)
	 *         "p_value": predicate value
	 *         "restrictions": Optional[{wql query}],
	 *                         // if specified, credential must satisfy to one of the given restriction.
	 *         "non_revoked": Optional[{non_revoc_interval}], // see below,
	 *                        // If specified prover must proof non-revocation
	 *                        // for date in this interval this attribute
	 *                        // (overrides proof level interval)
	 *     }
	 *     non_revoc_interval: Defines non-revocation interval
	 *     {
	 *         "from": Optional[int], // timestamp of interval beginning
	 *         "to": Optional[int], // timestamp of interval ending
	 *     }
	 * @param extraQueryJson (Optional) List of extra queries that will be applied to correspondent attribute/predicate:
	 *     {
	 *         "attr_referent": {wql query},
	 *         "predicate_referent": {wql query},
	 *     }
	 * where wql query: indy-sdk/doc/design/011-wallet-query-language/README.md
	 * @return Future CredentialsSearchForProofReq to fetch credentials
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<CredentialsSearchForProofReq> open(
			Wallet wallet,
			String proofReqJson,
			String extraQueryJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(proofReqJson, "proofReqJson");

		CompletableFuture<CredentialsSearchForProofReq> future = new CompletableFuture<CredentialsSearchForProofReq>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_search_credentials_for_proof_req(
				commandHandle,
				walletHandle,
				proofReqJson,
				extraQueryJson,
				proverSearchCredentialsForProofReqCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Fetch next records for the requested item using CredentialsSearchForProofReq.
	 *
	 * @param itemRef Referent of attribute/predicate in the proof request
	 * @param count Count of records to fetch
	 * @return List of credentials for the given proof request.
	 *     [{
	 *         cred_info: {credential_info},
	 *         interval: Optional[{non_revoc_interval}]
	 *     }]
	 * where credential_info is
	 *     {
	 *         "referent": "string",
	 *         "attrs": [{"attr_name" : "attr_raw_value"}],
	 *         "schema_id": string,
	 *         "cred_def_id": string,
	 *         "rev_reg_id": Optional[int],
	 *         "cred_rev_id": Optional[int],
	 *     }
	 * NOTE: The list of length less than the requested count means that search iterator
	 * correspondent to the requested itemRef is completed.
	 *
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public CompletableFuture<String> fetchNextCredentials(
			String itemRef,
			int count
	) throws IndyException {
		ParamGuard.notNullOrWhiteSpace(itemRef, "itemRef");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_prover_fetch_credentials_for_proof_req(
				commandHandle,
				searchHandle,
				itemRef,
				count,
				Anoncreds.stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Close credentials search for proof request
	 *
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public CompletableFuture<Void> closeSearch() throws IndyException {
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_prover_close_credentials_search_for_proof_req(
				commandHandle,
				searchHandle,
				voidCb);

		checkResult(future, result);

		return future;
	}

	@Override
	public void close() throws Exception {
		closeSearch().get();
	}
}
