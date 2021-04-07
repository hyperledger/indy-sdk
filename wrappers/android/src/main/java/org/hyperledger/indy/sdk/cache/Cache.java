package org.hyperledger.indy.sdk.cache;

import com.sun.jna.Callback;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.wallet.Wallet;

import java9.util.concurrent.CompletableFuture;

/**
 * cache.rs API
 */

/**
 * High level wrapper around did SDK functions.
 */
public class Cache extends IndyJava.API {
    private Cache() {

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
	 * Purge schema cache.
	 *
	 * EXPERIMENTAL
	 *
	 * @param wallet      The wallet.
	 * @param optionsJson The record tags used for search and storing meta information as json:
	 *                    {
	 *                        "maxAge": -1, // (int, optional, -1 by default) Purge cached data if older than this many seconds. -1 means purge all.
	 *                    }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> purgeSchemaCache(
			Wallet wallet,
			String optionsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(optionsJson, "optionsJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_purge_schema_cache(
				commandHandle,
				walletHandle,
				optionsJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Purge credential definition cache.
	 *
	 * EXPERIMENTAL
	 *
	 * @param wallet      The wallet.
	 * @param optionsJson The record tags used for search and storing meta information as json:
	 *                    {
	 *                        "maxAge": -1, // (int, optional, -1 by default) Purge cached data if older than this many seconds. -1 means purge all.
	 *                    }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> purgeCredDefCache(
			Wallet wallet,
			String optionsJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(optionsJson, "optionsJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_purge_cred_def_cache(
				commandHandle,
				walletHandle,
				optionsJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

    /**
	 * Gets schema json data for specified schema id.
	 * If data is present inside of cache, cached data is returned.
	 * Otherwise data is fetched from the ledger and stored inside of cache for future use.
	 *
	 * EXPERIMENTAL
	 *
	 * @param pool           The pool.
	 * @param wallet         The wallet.
	 * @param submitterDid   DID of the submitter stored in secured Wallet
	 * @param id             The id of schema.
	 * @param optionsJson
	 *  {
	 *    noCache: (optional, false by default) Skip usage of cache,
	 *    noUpdate: (optional, false by default) Use only cached data, do not try to update.
	 *    noStore: (optional, false by default) Skip storing fresh data if updated
	 *    minFresh: (optional, -1 by default) Return cached data if not older than this many seconds. -1 means do not check age.
	 *  }
	 * @return A future that resolves to Schema json:
	 * {
	 *     id: identifier of schema
	 *     attrNames: array of attribute name strings
	 *     name: Schema's name string
	 *     version: Schema's version string
	 *     ver: Version of the Schema json
	 * }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> getSchema(
	        Pool pool,
			Wallet wallet,
			String submitterDid,
			String id,
			String optionsJson) throws IndyException {

		ParamGuard.notNull(pool, "pool");
		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(submitterDid, "submitterDid");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(optionsJson, "optionsJson");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

        int poolHandle = pool.getPoolHandle();
		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_get_schema(
				commandHandle,
				poolHandle,
				walletHandle,
				submitterDid,
				id,
				optionsJson,
				stringCb);

		checkResult(future, result);

		return future;
	}

    /**
	 * Gets credential definition json data for specified credential definition id.
	 * If data is present inside of cache, cached data is returned.
	 * Otherwise data is fetched from the ledger and stored inside of cache for future use.
	 *
	 * EXPERIMENTAL
	 *
	 * @param pool           The pool.
	 * @param wallet         The wallet.
	 * @param submitterDid   DID of the submitter stored in secured Wallet
	 * @param id             The id of credential definition.
	 * @param optionsJson
	 *  {
	 *    noCache: (optional, false by default) Skip usage of cache,
	 *    noUpdate: (optional, false by default) Use only cached data, do not try to update.
	 *    noStore: (optional, false by default) Skip storing fresh data if updated
	 *    minFresh: (optional, -1 by default) Return cached data if not older than this many seconds. -1 means do not check age.
	 *  }
	 * @return A future that resolves to Credential Definition json:
	 * {
	 *     id: string - identifier of credential definition
	 *     schemaId: string - identifier of stored in ledger schema
	 *     type: string - type of the credential definition. CL is the only supported type now.
	 *     tag: string - allows to distinct between credential definitions for the same issuer and schema
	 *     value: Dictionary with Credential Definition's data: {
	 *         primary: primary credential public key,
	 *         Optional[revocation]: revocation credential public key
	 *     },
	 *     ver: Version of the Credential Definition json
	 * }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> getCredDef(
	        Pool pool,
			Wallet wallet,
			String submitterDid,
			String id,
			String optionsJson) throws IndyException {

		ParamGuard.notNull(pool, "pool");
		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(submitterDid, "submitterDid");
		ParamGuard.notNull(id, "id");
		ParamGuard.notNull(optionsJson, "optionsJson");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

        int poolHandle = pool.getPoolHandle();
		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_get_cred_def(
				commandHandle,
				poolHandle,
				walletHandle,
				submitterDid,
				id,
				optionsJson,
				stringCb);

		checkResult(future, result);

		return future;
	}
}
