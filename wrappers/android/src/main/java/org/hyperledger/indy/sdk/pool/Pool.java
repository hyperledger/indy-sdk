package org.hyperledger.indy.sdk.pool;

import com.sun.jna.Callback;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;

import java.util.concurrent.ExecutionException;

import java9.util.concurrent.CompletableFuture;

/**
 * pool.rs API
 */

/**
 * High level wrapper around SDK Pool functionality.
 */
public class Pool extends IndyJava.API implements AutoCloseable {

	private final int poolHandle;

	private Pool(int poolHandle) {

		this.poolHandle = poolHandle;
	}

	/**
	 * Gets the handle for the pool instance.
	 * 
	 * @return The handle for the pool instance.
	 */
	public int getPoolHandle() {

		return this.poolHandle;
	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when openPoolLedger completes.
	 */
	private static Callback openPoolLedgerCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int pool_handle) {

			CompletableFuture<Pool> future = (CompletableFuture<Pool>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Pool pool = new Pool(pool_handle);

			Pool result = pool;
			future.complete(result);
		}
	};

	/**
	 * Callback used when void function completes.
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
	 * Callback used when listPools completes.
	 */
	private static Callback listPoolsCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String metadata) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = metadata;
			future.complete(result);
		}
	};
	
	/*
	 * STATIC METHODS
	 */

	/**
	 * Creates a new local pool ledger configuration that can be used later to connect pool nodes.
	 * 
	 * @param configName Name of the pool ledger configuration.
	 * @param config Pool configuration json. if NULL, then default config will be used.
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> createPoolLedgerConfig(
			String configName,
			String config) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(configName, "configName");		
		
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_create_pool_ledger_config(
				commandHandle, 
				configName, 
				config,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Opens pool ledger and performs connecting to pool nodes.
	 * 
	 * @param configName Name of the pool ledger configuration.
	 * @param config Runtime pool configuration json. Optional. If NULL, then default config will be used.
	 *               Example:
	 * {
	 *     "timeout": int (optional), timeout for network request (in sec).
	 *     "extended_timeout": int (optional), extended timeout for network request (in sec).
	 *     "preordered_nodes": array(string) -  (optional), names of nodes which will have a priority during request sending:
	 *          ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
	 *          This can be useful if a user prefers querying specific nodes.
	 *          Assume that `Node1` and `Node2` nodes reply faster.
	 *          If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
	 *          Note: Nodes not specified will be placed randomly.
	 *     "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
	 *          By default Libindy sends a read requests to 2 nodes in the pool.
	 *          If response isn't received or `state proof` is invalid Libindy sends the request again but to 2 (`number_read_nodes`) * 2 = 4 nodes and so far until completion.
	 * }
	 *
	 * @return A future that resolves to an opened Pool instance.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Pool> openPoolLedger(
			String configName,
			String config) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(configName, "configName");	
		
		CompletableFuture<Pool> future = new CompletableFuture<Pool>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_open_pool_ledger(
				commandHandle, 
				configName, 
				config, 
				openPoolLedgerCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Refreshes a local copy of a pool ledger and updates pool nodes connections.
	 * 
	 * @param pool The pool to refresh.
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	private static CompletableFuture<Void> refreshPoolLedger(
			Pool pool) throws IndyException {

		ParamGuard.notNull(pool, "pool");	
		
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int handle = pool.getPoolHandle();

		int result = LibIndy.api.indy_refresh_pool_ledger(
				commandHandle, 
				handle,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Closes opened pool ledger, opened nodes connections and frees allocated resources.
	 * 
	 * @param pool The pool to close.
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	private static CompletableFuture<Void> closePoolLedger(
			Pool pool) throws IndyException {

		ParamGuard.notNull(pool, "pool");	
		
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int handle = pool.getPoolHandle();

		int result = LibIndy.api.indy_close_pool_ledger(
				commandHandle, 
				handle,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Deletes created pool ledger configuration.
	 * 
	 * @param configName Name of the pool ledger configuration to delete.
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> deletePoolLedgerConfig(
			String configName) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(configName, "configName");	
		
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_delete_pool_ledger_config(
				commandHandle, 
				configName,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Set PROTOCOL_VERSION to specific version.
	 *
	 * There is a global property PROTOCOL_VERSION that used in every request to the pool and
	 * specified version of Indy Node which Libindy works.
	 *
	 * By default PROTOCOL_VERSION=1.
	 *
	 * @param protocolVersion Protocol version will be used:
	 *      1 - for Indy Node 1.3
	 *      2 - for Indy Node 1.4 and greater
	 *
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> setProtocolVersion(
			int protocolVersion) throws IndyException {

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_set_protocol_version(
				commandHandle,
				protocolVersion,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Lists names of created pool ledgers
	 *
	 * @return A future resolving to a list of pools: [{
	 *     "pool": string - Name of pool ledger stored in the wallet.
	 *   }]
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> listPools() throws IndyException {

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_list_pools(
				commandHandle,
				listPoolsCb);

		checkResult(future, result);

		return future;
	}

	/*
	 * INSTANCE METHODS
	 */

	/**
	 * Refreshes a local copy of a pool ledger and updates pool nodes connections.
	 * 
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public CompletableFuture<Void> refreshPoolLedger(
			) throws IndyException {

		return refreshPoolLedger(this);
	}

	/**
	 * Closes opened pool ledger, opened nodes connections and frees allocated resources.
	 * 
	 * @return A future that does not resolve a value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public CompletableFuture<Void> closePoolLedger(
			) throws IndyException {

		return closePoolLedger(this);
	}

	@Override
	public void close() throws InterruptedException, ExecutionException, IndyException {
		closePoolLedger().get();
	}
}
