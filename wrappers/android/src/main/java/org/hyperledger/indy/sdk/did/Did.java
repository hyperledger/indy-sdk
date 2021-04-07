package org.hyperledger.indy.sdk.did;

import java9.util.concurrent.CompletableFuture;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.did.DidResults.CreateAndStoreMyDidResult;
import org.hyperledger.indy.sdk.did.DidResults.EndpointForDidResult;
import org.hyperledger.indy.sdk.wallet.Wallet;

import com.sun.jna.Callback;

/**
 * did.rs API
 */

/**
 * High level wrapper around did SDK functions.
 */
public class Did extends IndyJava.API {

	private Did() {

	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when createAndStoreMyDid completes.
	 */
	private static Callback createAndStoreMyDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String did, String verkey) {

			CompletableFuture<CreateAndStoreMyDidResult> future = (CompletableFuture<CreateAndStoreMyDidResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			CreateAndStoreMyDidResult result = new CreateAndStoreMyDidResult(did, verkey);
			future.complete(result);
		}
	};

	/**
	 * Callback used when replaceKeysStart completes.
	 */
	private static Callback replaceKeysStartCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String verkey) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = verkey;
			future.complete(result);
		}
	};

	/**
	 * Callback used when replaceKeysApply completes.
	 */
	private static Callback replaceKeysApplyCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when storeTheirDid completes.
	 */
	private static Callback storeTheirDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when keyForDid completes.
	 */
	private static Callback keyForDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String key) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = key;
			future.complete(result);
		}
	};

	/**
	 * Callback used when keyForLocalDid completes.
	 */
	private static Callback keyForLocalDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String key) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = key;
			future.complete(result);
		}
	};

	/**
	 * Callback used when setEndpointForDid completes.
	 */
	private static Callback setEndpointForDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when getEndpointForDid completes.
	 */
	private static Callback getEndpointForDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String endpoint, String transport_vk) {

			CompletableFuture<EndpointForDidResult> future = (CompletableFuture<EndpointForDidResult>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			EndpointForDidResult result = new EndpointForDidResult(endpoint, transport_vk);
			future.complete(result);
		}
	};

	/**
	 * Callback used when setDidMetadata completes.
	 */
	private static Callback setDidMetadataCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err) {

			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Void result = null;
			future.complete(result);
		}
	};

	/**
	 * Callback used when getDidMetadata completes.
	 */
	private static Callback getDidMetadataCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String metadata) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = metadata;
			future.complete(result);
		}
	};

	/**
	 * Callback used when getAttrVerkey completes.
	 */
	private static Callback getAttrVerkeyCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String verkey) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = verkey;
			future.complete(result);
		}
	};

	/**
	 * Callback used when qualifyDid completes.
	 */
	private static Callback qualifyDidCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, String did) {

			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			String result = did;
			future.complete(result);
		}
	};

	/*
	 * STATIC METHODS
	 */

	/**
	 * Creates keys (signing and encryption keys) for a new
	 * DID (owned by the caller of the library).
	 *
	 * Identity's DID must be either explicitly provided, or taken as the first 16 bit of verkey.
	 * Saves the Identity DID with keys in a secured Wallet, so that it can be used to sign
	 * and encrypt transactions.
	 *
	 * @param wallet  The wallet.
	 * @param didJson Identity information as json.
	 * {
	 *     "did": string, (optional;
	 *             if not provided and cid param is false then the first 16 bit of the verkey will be used as a new DID;
	 *             if not provided and cid is true then the full verkey will be used as a new DID;
	 *             if provided, then keys will be replaced - key rotation use case)
	 *     "seed": string, (optional) Seed that allows deterministic did creation (if not set random one will be created).
	 *                                Can be UTF-8, base64 or hex string.
	 *     "crypto_type": string, (optional; if not set then ed25519 curve is used;
	 *               currently only 'ed25519' value is supported for this field)
	 *     "cid": bool, (optional; if not set then false is used;)
	 *     "method_name": string, (optional) method name to create fully qualified did.
	 * }
	 * @return A future that resolves to a CreateAndStoreMyDidResult containing did and verkey.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<CreateAndStoreMyDidResult> createAndStoreMyDid(
			Wallet wallet,
			String didJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(didJson, "didJson");

		CompletableFuture<CreateAndStoreMyDidResult> future = new CompletableFuture<CreateAndStoreMyDidResult>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_create_and_store_my_did(
				commandHandle,
				walletHandle,
				didJson,
				createAndStoreMyDidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Generated new signing and encryption keys for an existing DID owned by the caller.
	 *
	 * @param wallet       The wallet.
	 * @param did          The DID
	 * @param identityJson identity information as json.
	 * @return A future that resolves to a temporary verkey.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> replaceKeysStart(
			Wallet wallet,
			String did,
			String identityJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");
		ParamGuard.notNullOrWhiteSpace(identityJson, "identityJson");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_replace_keys_start(
				commandHandle,
				walletHandle,
				did,
				identityJson,
				replaceKeysStartCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Apply temporary keys as main for an existing DID (owned by the caller of the library).
	 *
	 * @param wallet The wallet.
	 * @param did    The DID
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> replaceKeysApply(
			Wallet wallet,
			String did) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_replace_keys_apply(
				commandHandle,
				walletHandle,
				did,
				replaceKeysApplyCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Saves their DID for a pairwise connection in a secured Wallet so that it can be used to verify transaction.
	 * Updates DID associated verkey in case DID already exists in the Wallet.
	 *
	 * @param wallet       The wallet.
	 * @param identityJson Identity information as json.
	 *     {
	 *        "did": string, (required)
	 *        "verkey": string
	 *                     - optional is case of adding a new DID, and DID is cryptonym: did == verkey,
	 *                     - mandatory in case of updating an existing DID
	 *     }
	 *
	 * @return A future that does not resolve any value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> storeTheirDid(
			Wallet wallet,
			String identityJson) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(identityJson, "identityJson");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_store_their_did(
				commandHandle,
				walletHandle,
				identityJson,
				storeTheirDidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Returns ver key (key id) for the given DID.
	 *
	 * "keyForDid" call follow the idea that we resolve information about their DID from
	 * the ledger with cache in the local wallet. The "openWallet" call has freshness parameter
	 * that is used for checking the freshness of cached pool value.
	 *
	 * Note if you don't want to resolve their DID info from the ledger you can use
	 * "keyForLocalDid" call instead that will look only to local wallet and skip
	 * freshness checking.
	 *
	 * Note that "createAndStoreMyDid" makes similar wallet record as "createKey".
	 * As result we can use returned ver key in all generic crypto and messaging functions.
	 *
	 * @param pool   The pool.
	 * @param wallet The wallet.
	 * @param did    The DID to resolve key.
	 * @return A future resolving to a verkey
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> keyForDid(
			Pool pool,
			Wallet wallet,
			String did) throws IndyException {

		ParamGuard.notNull(pool, "pool");
		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();
		int poolHandle = pool.getPoolHandle();

		int result = LibIndy.api.indy_key_for_did(
				commandHandle,
				poolHandle,
				walletHandle,
				did,
				keyForDidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Returns ver key (key id) for the given DID.
	 *
	 * "keyForLocalDid" call looks data stored in the local wallet only and skips freshness checking.
	 *
	 * Note if you want to get fresh data from the ledger you can use "keyForDid" call
	 * instead.
	 *
	 * Note that "createAndStoreMyDid" makes similar wallet record as "createKey".
	 * As result we can use returned ver key in all generic crypto and messaging functions.
	 *
	 * @param wallet The wallet.
	 * @param did    The DID to resolve key.
	 * @return A future resolving to a verkey
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> keyForLocalDid(
			Wallet wallet,
			String did) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_key_for_local_did(
				commandHandle,
				walletHandle,
				did,
				keyForLocalDidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Set/replaces endpoint information for the given DID.
	 *
	 * @param wallet       The wallet.
	 * @param did          The DID to resolve endpoint.
	 * @param address      The DIDs endpoint address.
	 * @param transportKey The DIDs transport key (ver key, key id).
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> setEndpointForDid(
			Wallet wallet,
			String did,
			String address,
			String transportKey) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");
		ParamGuard.notNull(address, "address");
		ParamGuard.notNullOrWhiteSpace(transportKey, "transportKey");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_set_endpoint_for_did(
				commandHandle,
				walletHandle,
				did,
				address,
				transportKey,
				setEndpointForDidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Returns endpoint information for the given DID.
	 *
	 * @param wallet The wallet.
	 * @param pool The pool.
	 * @param did  The DID to resolve endpoint.
	 * @return A future resolving to a object containing endpoint and transportVk
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<EndpointForDidResult> getEndpointForDid(
			Wallet wallet,
			Pool pool,
			String did) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNull(pool, "pool");
		ParamGuard.notNullOrWhiteSpace(did, "did");

		CompletableFuture<EndpointForDidResult> future = new CompletableFuture<EndpointForDidResult>();
		int commandHandle = addFuture(future);

		int poolHandle = pool.getPoolHandle();
		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_get_endpoint_for_did(
				commandHandle,
				walletHandle,
				poolHandle,
				did,
				getEndpointForDidCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Saves/replaces the meta information for the giving DID in the wallet.
	 *
	 * @param wallet   The wallet.
	 * @param did      The DID to store metadata.
	 * @param metadata The meta information that will be store with the DID.
	 * @return A future that resolves no value.
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<Void> setDidMetadata(
			Wallet wallet,
			String did,
			String metadata) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");
		ParamGuard.notNull(metadata, "metadata");

		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_set_did_metadata(
				commandHandle,
				walletHandle,
				did,
				metadata,
				setDidMetadataCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Retrieves the meta information for the giving DID in the wallet.
	 *
	 * @param wallet The wallet.
	 * @param did    The DID to retrieve metadata.
	 * @return A future resolving to a metadata
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> getDidMetadata(
			Wallet wallet,
			String did) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_get_did_metadata(
				commandHandle,
				walletHandle,
				did,
				getDidMetadataCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Retrieves the information about the giving DID in the wallet.
	 *
	 * @param wallet The wallet.
	 * @param did    The DID to retrieve metadata.
	 * @return A future resolving to a did data: {
	 *     "did": string - DID stored in the wallet,
	 *     "verkey": string - The DIDs transport key (ver key, key id),
	 *     "tempVerkey": string - Future DIDs transport key (ver key, key id), after rotation of keys is done.
	 *     "metadata": string - The meta information stored with the DID
	 *   }
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> getDidWithMeta(
			Wallet wallet,
			String did) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");
		ParamGuard.notNullOrWhiteSpace(did, "did");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_get_my_did_with_meta(
				commandHandle,
				walletHandle,
				did,
				getDidMetadataCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Retrieves the information about all DIDs stored in the wallet.
	 *
	 * @param wallet The wallet.
	 * @return A future resolving to a list of dids: [{
	 *     "did": string - DID stored in the wallet,
	 *     "verkey": string - The DIDs transport key (ver key, key id).,
	 *     "metadata": string - The meta information stored with the DID
	 *   }]
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> getListMyDidsWithMeta(
			Wallet wallet) throws IndyException {

		ParamGuard.notNull(wallet, "wallet");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_list_my_dids_with_meta(
				commandHandle,
				walletHandle,
				getDidMetadataCb);

		checkResult(future, result);

		return future;
	}
	
	/**
	 * Retrieves abbreviated verkey if it is possible otherwise return full verkey.
	 *
	 * @param did   DID.
	 * @param verkey    The DIDs verification key,
	 * @return A future resolving to the DIDs verification key in either abbreviated or full form
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> AbbreviateVerkey(
			String did,
			String verkey) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(did, "did");
		ParamGuard.notNullOrWhiteSpace(did, "verkey");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_abbreviate_verkey(
				commandHandle,
				did,
				verkey,
				getAttrVerkeyCb);

		checkResult(future, result);

		return future;
	}

	/**
	 * Update DID stored in the wallet to make fully qualified, or to do other DID maintenance.
	 *     - If the DID has no method, a method will be appended (prepend did:peer to a legacy did)
	 *     - If the DID has a method, a method will be updated (migrate did:peer to did:peer-new)
	 *
	 * Update DID related entities stored in the wallet.
	 *
	 * @param wallet The wallet.
	 * @param did The target DID stored in the wallet.
	 * @param method The method to apply to the DID.
	 * @return A future resolving to a fully qualified did
	 * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
	 */
	public static CompletableFuture<String> qualifyDid(
			Wallet wallet,
			String did,
			String method) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(did, "did");
		ParamGuard.notNull(method, "method");

		CompletableFuture<String> future = new CompletableFuture<String>();
		int commandHandle = addFuture(future);

		int walletHandle = wallet.getWalletHandle();

		int result = LibIndy.api.indy_qualify_did(
				commandHandle,
				walletHandle,
				did,
				method,
				getDidMetadataCb);

		checkResult(future, result);

		return future;
	}
}
