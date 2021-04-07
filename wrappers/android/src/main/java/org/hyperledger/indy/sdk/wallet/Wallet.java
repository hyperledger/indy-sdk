package org.hyperledger.indy.sdk.wallet;

import java9.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;

import com.sun.jna.Callback;

/**
 * wallet.rs API
 */
/**
 * High level wrapper for wallet SDK functions.
 */
public class Wallet extends IndyJava.API implements AutoCloseable {

	private final int walletHandle;

	private Wallet(int walletHandle) {

		this.walletHandle = walletHandle;
	}

	/**
	 * Gets the handle for the wallet.
	 *
	 * @return The handle for the wallet.
	 */
	public int getWalletHandle() {

		return this.walletHandle;
	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when function returning void completes.
	 */
	private static Callback voidCb = new Callback() {

		@SuppressWarnings({ "unused", "unchecked" })
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when function returning string completes.
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
	 * Callback used when openWallet completes.
	 */
	private static Callback openWalletCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int handle) {

			CompletableFuture<Wallet> future = (CompletableFuture<Wallet>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Wallet wallet = new Wallet(handle);

			Wallet result = wallet;
			future.complete(result);
		}
	};

	/*
	 * STATIC METHODS
	 */

	/**
	 * Creates a new secure wallet with the given unique name.
	 *
	 * @param config Wallet configuration json.
	 * {
	 *   "id": string, Identifier of the wallet.
	 *         Configured storage uses this identifier to lookup exact wallet data placement.
	 *   "storage_type": optional["string"], Type of the wallet storage. Defaults to 'default'.
	 *                  'Default' storage type allows to store wallet data in the local file.
	 *                  Custom storage types can be registered with indy_register_wallet_storage call.
	 *   "storage_config": optional[{config json}], Storage configuration json. Storage type defines set of supported keys.
	 *                     Can be optional if storage supports default configuration.
	 *                      For 'default' storage type configuration is:
	 *   {
	 *     "path": optional["string"], Path to the directory with wallet files.
	 *             Defaults to $HOME/.indy_client/wallet.
	 *             Wallet will be stored in the file {path}/{id}/sqlite.db
	 *   }
	 * }
	 * @param credentials Wallet credentials json
	 * {
	 *   "key": string, Key or passphrase used for wallet key derivation.
	 *                  Look to key_derivation_method param for information about supported key derivation methods.
	 *   "storage_credentials": optional[{credentials json}] Credentials for wallet storage. Storage type defines set of supported keys.
	 *                          Can be optional if storage supports default configuration.
	 *                           For 'default' storage type should be empty.
	 *   "key_derivation_method": optional[string] Algorithm to use for wallet key derivation:
	 *                           ARGON2I_MOD - derive secured wallet master key (used by default)
	 *                           ARGON2I_INT - derive secured wallet master key (less secured but faster)
	 *                           RAW - raw wallet key master provided (skip derivation).
	 *                              RAW keys can be generated with generateWalletKey call
	 * }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<Void> createWallet(
			String config,
			String credentials) throws IndyException {

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_create_wallet(
				commandHandle,
				config,
				credentials,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Opens the wallet with specific name.
	 *
	 * @param config Wallet configuration json.
	 * {
	 *   "id": string, Identifier of the wallet.
	 *         Configured storage uses this identifier to lookup exact wallet data placement.
	 *   "storage_type": optional["string"], Type of the wallet storage. Defaults to 'default'.
	 *                  'Default' storage type allows to store wallet data in the local file.
	 *                  Custom storage types can be registered with indy_register_wallet_storage call.
	 *   "storage_config": optional[{config json}], Storage configuration json. Storage type defines set of supported keys.
	 *                     Can be optional if storage supports default configuration.
	 *                      For 'default' storage type configuration is:
	 *   {
	 *     "path": optional["string"], Path to the directory with wallet files.
	 *             Defaults to $HOME/.indy_client/wallet.
	 *             Wallet will be stored in the file {path}/{id}/sqlite.db
	 *   }
	 * }
	 * @param credentials Wallet credentials json
	 *   {
	 *       "key": string, Key or passphrase used for wallet key derivation.
	 *                      Look to key_derivation_method param for information about supported key derivation methods.
	 *       "rekey": optional["string"], If present than wallet master key will be rotated to a new one.
	 *       "storage_credentials": optional[{credentiails object}] Credentials for wallet storage. Storage type defines set of supported keys.
	 *                              Can be optional if storage supports default configuration.
	 *                               For 'default' storage type should be empty.
	 *   "key_derivation_method": optional[string] Algorithm to use for wallet key derivation:
	 *                           ARGON2I_MOD - derive secured wallet master key (used by default)
	 *                           ARGON2I_INT - derive secured wallet master key (less secured but faster)
	 *                           RAW - raw wallet key master provided (skip derivation).
	 *                              RAW keys can be generated with generateWalletKey call
	 *   "rekey_derivation_method": optional[string] Algorithm to use for wallet rekey derivation:
	 *                           ARGON2I_MOD - derive secured wallet master rekey (used by default)
	 *                           ARGON2I_INT - derive secured wallet master rekey (less secured but faster)
	 *                           RAW - raw wallet master rekey provided (skip derivation).
	 *                              RAW keys can be generated with generateWalletKey call
	 *
	 *   }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<Wallet> openWallet(
			String config,
			String credentials) throws IndyException {


		CompletableFuture<Wallet> future = new CompletableFuture<Wallet>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_open_wallet(
				commandHandle,
				config,
				credentials,
				openWalletCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Closes the specified open wallet and frees allocated resources.
	 *
	 * @param wallet The wallet to close.
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	private static CompletableFuture<Void> closeWallet(
			Wallet wallet) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int handle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_close_wallet(
				commandHandle,
				handle,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Deletes an existing wallet.
	 *
	 * @param config Wallet configuration json.
	 * {
	 *   "id": string, Identifier of the wallet.
	 *         Configured storage uses this identifier to lookup exact wallet data placement.
	 *   "storage_type": optional["string"], Type of the wallet storage. Defaults to 'default'.
	 *                  'Default' storage type allows to store wallet data in the local file.
	 *                  Custom storage types can be registered with indy_register_wallet_storage call.
	 *   "storage_config": optional[{config json}], Storage configuration json. Storage type defines set of supported keys.
	 *                     Can be optional if storage supports default configuration.
	 *                      For 'default' storage type configuration is:
	 *   {
	 *     "path": optional["string"], Path to the directory with wallet files.
	 *             Defaults to $HOME/.indy_client/wallet.
	 *             Wallet will be stored in the file {path}/{id}/sqlite.db
	 *   }
	 * }
	 * @param credentials Wallet credentials json
	 *   {
	 *       "key": string, Key or passphrase used for wallet key derivation.
	 *                      Look to key_derivation_method param for information about supported key derivation methods.
	 *       "storage_credentials": optional[{credentials json}] Credentials for wallet storage. Storage type defines set of supported keys.
	 *                              Can be optional if storage supports default configuration.
	 *                               For 'default' storage type should be empty.
	 *       "key_derivation_method": optional[string] Algorithm to use for wallet key derivation:
	 *                           ARGON2I_MOD - derive secured wallet master key (used by default)
	 *                           ARGON2I_INT - derive secured wallet master key (less secured but faster)
	 *                           RAW - raw wallet key master provided (skip derivation).
	 *                              RAW keys can be generated with generateWalletKey call
	 *   }
	 *                       
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<Void> deleteWallet(
			String config,
			String credentials) throws IndyException {

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_delete_wallet(
				commandHandle,
				config,
				credentials,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Exports opened wallet to the file.
	 *
	 * @param wallet The wallet to export.
	 * @param exportConfigJson: JSON containing settings for input operation.
	 *   {
	 *     "path": "string", Path of the file that contains exported wallet content
	 *     "key": string, Key or passphrase used for wallet export key derivation.
	 *                    Look to key_derivation_method param for information about supported key derivation methods.
	 *     "key_derivation_method": optional[string] algorithm to use for export key derivation:
	 *                           ARGON2I_MOD - derive secured wallet export key (used by default)
	 *                           ARGON2I_INT - derive secured wallet export key (less secured but faster)
	 *                           RAW - raw wallet export master provided (skip derivation).
	 *                              RAW keys can be generated with generateWalletKey call
	 *   }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<Void> exportWallet(
			Wallet wallet,
			String exportConfigJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(exportConfigJson, "exportConfigJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int handle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_export_wallet(
				commandHandle,
				handle,
				exportConfigJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Creates a new secure wallet with the given unique name and then imports its content
	 * according to fields provided in import_config
	 * This can be seen as an indy_create_wallet call with additional content import
	 *
	 * @param config Wallet configuration json. List of supported keys are defined by wallet type.
	 * @param credentials Wallet configuration json.
	 * {
	 *   "id": string, Identifier of the wallet.
	 *         Configured storage uses this identifier to lookup exact wallet data placement.
	 *   "storage_type": optional["string"], Type of the wallet storage. Defaults to 'default'.
	 *                  'Default' storage type allows to store wallet data in the local file.
	 *                  Custom storage types can be registered with indy_register_wallet_storage call.
	 *   "storage_config": optional[{config json}], Storage configuration json. Storage type defines set of supported keys.
	 *                     Can be optional if storage supports default configuration.
	 *                      For 'default' storage type configuration is:
	 *   {
	 *     "path": optional["string"], Path to the directory with wallet files.
	 *             Defaults to $HOME/.indy_client/wallet.
	 *             Wallet will be stored in the file {path}/{id}/sqlite.db
	 *   }
	 * }
	 * @param credentials Wallet credentials json
	 * {
	 *    "key": string, Key or passphrase used for wallet key derivation.
	 *                   Look to key_derivation_method param for information about supported key derivation methods.
	 *   "storage_credentials": optional[{credentials json}] Credentials for wallet storage. Storage type defines set of supported keys.
	 *                          Can be optional if storage supports default configuration.
	 *                          For 'default' storage type should be empty.
	 *   "key_derivation_method": optional[string] Algorithm to use for wallet key derivation:
	 *                           ARGON2I_MOD - derive secured wallet master key (used by default)
	 *                           ARGON2I_INT - derive secured wallet master key (less secured but faster)
	 *                           RAW - raw wallet key master provided (skip derivation).
	 *                              RAW keys can be generated with generateWalletKey call
	 * }	
	 * @param importConfigJson Import settings json.
	 * {
	 *   "path": "string", Path of the file that contains exported wallet content
	 *   "key": "string",  Key used for export of the wallet
	 * }
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<Void> importWallet(
			String config,
			String credentials,
			String importConfigJson) throws IndyException {

		ParamGuard.notNull(importConfigJson, "importConfigJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_import_wallet(
				commandHandle,
				config,
				credentials,
				importConfigJson,
				voidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Generate wallet master key.
	 * Returned key is compatible with "RAW" key derivation method.
	 * It allows to avoid expensive key derivation for use cases when wallet keys can be stored in a secure enclave.
	 *
	 * @param config (optional) key configuration json.
	 * {
	 *   "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
	 *                              Can be UTF-8, base64 or hex string.
	 * }
	 *   
	 * @return A future that resolves to key.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public static CompletableFuture<String> generateWalletKey(
			String config) throws IndyException {

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_generate_wallet_key(
				commandHandle,
				config,
				stringCb);

		checkResult(future, result);

		return future;
	}
	
	/*
	 * INSTANCE METHODS
	 */

	/**
	 * Closes the wallet and frees allocated resources.
	 *
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if a call to the underlying SDK fails.
	 */
	public CompletableFuture<Void> closeWallet(
			) throws IndyException {

		return closeWallet(this);
	}

	@Override
	public void close() throws InterruptedException, ExecutionException, IndyException {
		closeWallet().get();
	}
}
