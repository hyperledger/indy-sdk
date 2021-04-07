package org.hyperledger.indy.sdk.anoncreds;

import com.sun.jna.Callback;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.wallet.Wallet;

import java9.util.concurrent.CompletableFuture;

public class CredentialsSearch extends IndyJava.API implements AutoCloseable {
	private final int searchHandle;
	private final int totalCount;

	private CredentialsSearch(int searchHandle, int totalCount) {
		this.searchHandle = searchHandle;
		this.totalCount = totalCount;
	}

	/**
	 * Callback used when proverSearchCredentials completes.
	 */
	private static Callback proverSearchCredentialsCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int search_handle, int total_count) {

			CompletableFuture<CredentialsSearch> future = (CompletableFuture<CredentialsSearch>) removeFuture(xcommand_handle);
			if (!checkResult(future, err)) return;

			CredentialsSearch result = new CredentialsSearch(search_handle, total_count);
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
	 * Search for credentials stored in wallet.
	 * Credentials can be filtered by tags created during saving of credential.
	 *
	 * Instead of immediately returning of fetched credentials {@link Anoncreds#proverGetCredentials(Wallet, String)}
	 * this call returns CredentialsSearch that can be used later
	 * to fetch records by small batches (with {@link CredentialsSearch#fetchNextCredentials(int)}).
	 *
	 * @param wallet 	 A wallet
	 * @param queryJson Wql style filter for credentials searching based on tags.
	 *         where wql query: indy-sdk/doc/design/011-wallet-query-language/README.md
	 * @return CredentialsSearch to fetch method
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<CredentialsSearch> open(
			Wallet wallet,
			String queryJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(queryJson, "queryJson");

		CompletableFuture<CredentialsSearch> future = new CompletableFuture<CredentialsSearch>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_prover_search_credentials(
				commandHandle,
				walletHandle,
				queryJson,
				proverSearchCredentialsCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Fetch next records for credential search.
	 *
	 * @param count count of records to fetch
	 * @return credentials_json: List of credentials:
	 *     [{
	 *         "referent": string, // cred_id in the wallet
	 *         "attrs": {"key1":"raw_value1", "key2":"raw_value2"},
	 *         "schema_id": string,
	 *         "cred_def_id": string,
	 *         "rev_reg_id": Optional["string"],
	 *         "cred_rev_id": Optional["string"]
	 *     }]
	 * NOTE: The list of length less than the requested count means credentials search iterator is completed.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public CompletableFuture<String> fetchNextCredentials(
			int count
	) throws IndyException {
		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_prover_fetch_credentials(
				commandHandle,
				searchHandle,
				count,
				Anoncreds.stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Close credentials search
	 *
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public CompletableFuture<Void> closeSearch() throws IndyException {
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_prover_close_credentials_search(
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

	public int totalCount() {
		return totalCount;
	}
}
