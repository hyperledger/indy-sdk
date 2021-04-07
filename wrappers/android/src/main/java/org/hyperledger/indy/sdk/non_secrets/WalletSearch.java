package org.hyperledger.indy.sdk.non_secrets;

import com.sun.jna.Callback;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.wallet.Wallet;

import java9.util.concurrent.CompletableFuture;

/**
 * non_secrets.rs API
 */

/**
 * High level wrapper around did SDK functions.
 */
public class WalletSearch extends IndyJava.API implements AutoCloseable {

	private final int searchHandle;

	private WalletSearch(int searchHandle) {

		this.searchHandle = searchHandle;
	}

	/**
	 * Gets the handle for the search.
	 *
	 * @return The handle for the search.
	 */
	public int getSearchHandle() {

		return this.searchHandle;
	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when a function returning Void completes.
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

	/**
	 * Callback used when a function returning String completes.
	 */
	private static Callback stringCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String str) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = str;
			future.complete(result);
		}
	};

	/**
	 * Callback used when a openSearch function completes.
	 */
	private static Callback searchCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int handle) {

			CompletableFuture<WalletSearch> future = (CompletableFuture<WalletSearch>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			WalletSearch result = new WalletSearch(handle);
			future.complete(result);
		}
	};

	/*
	 * STATIC METHODS
	 */

	/**
	 * Delete an existing wallet record in the wallet
	 *
	 * @param wallet      The wallet.
	 * @param type        Allows to separate different record types collections
	 * @param queryJson   MongoDB style query to wallet record tags:
	 *                    {
	 *                      "tagName": "tagValue",
	 *                      $or: {
	 *                          "tagName2": { $regex: 'pattern' },
	 *                          "tagName3": { $gte: '123' },
	 *                      }
	 *                    }
	 * @param optionsJson {
	 *                      retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
	 *                      retrieveTotalCount: (optional, false by default) Calculate total count,
	 *                      retrieveType: (optional, false by default) Retrieve record type,
	 *                      retrieveValue: (optional, true by default) Retrieve record value,
	 *                      retrieveTags: (optional, false by default) Retrieve record tags,
	 *                    }
	 * @return A future that resolves to WalletSearch instance.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<WalletSearch> open(
			Wallet wallet,
			String type,
			String queryJson,
			String optionsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(queryJson, "queryJson");
		ParamGuard.notNull(optionsJson, "optionsJson");

		CompletableFuture<WalletSearch> future = new CompletableFuture<WalletSearch>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_open_wallet_search(
				commandHandle,
				walletHandle,
				type,
				queryJson,
				optionsJson,
				searchCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Fetch next records for wallet search.
	 *
	 * @param wallet The wallet.
	 * @param search The wallet search.
	 * @param count  Count of records to fetch
	 * @return A future resolving to the wallet records json:
	 * {
	 *      totalCount: int, // present only if retrieveTotalCount set to true
	 *      records: [{ // present only if retrieveRecords set to true
	 *          id: "Some id",
	 *          type: "Some type", // present only if retrieveType set to true
	 *          value: "Some value", // present only if retrieveValue set to true
	 *          tags: "Some tags json", // present only if retrieveTags set to true
	 *      }],
	 * }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> searchFetchNextRecords(
			Wallet wallet,
			WalletSearch search,
			int count) throws IndyException {

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();
		int searchHandle = search.getSearchHandle();

		int result = LibIndy.api.indy_fetch_wallet_search_next_records(
				commandHandle,
				walletHandle,
				searchHandle,
				count,
				stringCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Close wallet search (make search handle invalid)
	 *
	 * @param search The wallet search.
	 * @return A future resolving to no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> closeSearch(
			WalletSearch search) throws IndyException {

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int searchHandle = search.getSearchHandle();

		int result = LibIndy.api.indy_close_wallet_search(
				commandHandle,
				searchHandle,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/*
	 * INSTANCE METHODS
	 */

	/**
	 * Fetch next records for wallet search.
	 *
	 * @param wallet The wallet.
	 * @param count  Count of records to fetch
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public CompletableFuture<String> fetchNextRecords(
			Wallet wallet,
			int count
	) throws IndyException {

		return searchFetchNextRecords(wallet, this, count);
	}

	/**
	 * Closes opened wallet search.
	 *
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public CompletableFuture<Void> closeSearch() throws IndyException {
		return closeSearch(this);
	}

	@Override
	public void close() throws Exception {
		closeSearch().get();
	}
}
