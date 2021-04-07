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
public class WalletRecord extends IndyJava.API {

	private WalletRecord() {

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

	/*
	 * STATIC METHODS
	 */

	/**
	 * Create a new non-secret record in the wallet
	 *
	 * @param wallet   The wallet.
	 * @param type     Allows to separate different record types collections
	 * @param id       The id of record
	 * @param value    The value of record
	 * @param tagsJson The record tags used for search and storing meta information as json:
	 *                 {
	 *                     "tagName1": "str", // string tag (will be stored encrypted)
	 *                     "tagName2": "str", // string tag (will be stored encrypted)
	 *                 }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> add(
			Wallet wallet,
			String type,
			String id,
			String value,
			String tagsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(value, "value");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_add_wallet_record(
				commandHandle,
				walletHandle,
				type,
				id,
				value,
				tagsJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Update a non-secret wallet record value
	 *
	 * @param wallet The wallet.
	 * @param type   Allows to separate different record types collections
	 * @param id     The id of record
	 * @param value  The value of record
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> updateValue(
			Wallet wallet,
			String type,
			String id,
			String value) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(value, "value");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_update_wallet_record_value(
				commandHandle,
				walletHandle,
				type,
				id,
				value,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Update a non-secret wallet record tags
	 *
	 * @param wallet   The wallet.
	 * @param type     Allows to separate different record types collections
	 * @param id       The id of record
	 * @param tagsJson The record tags used for search and storing meta information as json:
	 *                 {
	 *                     "tagName1": "str", // string tag (will be stored encrypted)
	 *                     "tagName2": "str", // string tag (will be stored encrypted)
	 *                 }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> updateTags(
			Wallet wallet,
			String type,
			String id,
			String tagsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(tagsJson, "tagsJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_update_wallet_record_tags(
				commandHandle,
				walletHandle,
				type,
				id,
				tagsJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Update a non-secret wallet record tags
	 *
	 * @param wallet   The wallet.
	 * @param type     Allows to separate different record types collections
	 * @param id       The id of record
	 * @param tagsJson The record tags used for search and storing meta information as json:
	 *                 {
	 *                     "tagName1": "str", // string tag (will be stored encrypted)
	 *                     "tagName2": "str", // string tag (will be stored encrypted)
	 *                 }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> addTags(
			Wallet wallet,
			String type,
			String id,
			String tagsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(tagsJson, "tagsJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_add_wallet_record_tags(
				commandHandle,
				walletHandle,
				type,
				id,
				tagsJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Delete tags from the wallet record
	 *
	 * @param wallet       The wallet.
	 * @param type         Allows to separate different record types collections
	 * @param id           The id of record
	 * @param tagNamesJson The list of tag names to remove from the record as json array:
	 *                     ["tagName1", "tagName2", ...]
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> deleteTags(
			Wallet wallet,
			String type,
			String id,
			String tagNamesJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(tagNamesJson, "tagNamesJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_delete_wallet_record_tags(
				commandHandle,
				walletHandle,
				type,
				id,
				tagNamesJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Delete an existing wallet record in the wallet
	 *
	 * @param wallet The wallet.
	 * @param type   Allows to separate different record types collections
	 * @param id     The id of record
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> delete(
			Wallet wallet,
			String type,
			String id) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_delete_wallet_record(
				commandHandle,
				walletHandle,
				type,
				id,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Delete an existing wallet record in the wallet
	 *
	 * @param wallet The wallet.
	 * @param type   Allows to separate different record types collections
	 * @param id     The id of record
	 * @param optionsJson
	 *  {
	 *    retrieveType: (optional, false by default) Retrieve record type,
	 *    retrieveValue: (optional, true by default) Retrieve record value,
	 *    retrieveTags: (optional, false by default) Retrieve record tags
	 *  }
	 * @return A future that resolves to wallet records json:
	 *  {
	 *    id: "Some id",
	 *    type: "Some type", // present only if retrieveType set to true
	 *    value: "Some value", // present only if retrieveValue set to true
	 *    tags: "Some tags json", // present only if retrieveTags set to true
	 *  }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> get(
			Wallet wallet,
			String type,
			String id,
			String optionsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(optionsJson, "optionsJson");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_get_wallet_record(
				commandHandle,
				walletHandle,
				type,
				id,
				optionsJson,
				stringCb);

		checkResult(future, result);

		return future;
	}
}
