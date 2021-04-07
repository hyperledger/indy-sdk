package org.hyperledger.indy.sdk.blob_storage;

import com.sun.jna.Callback;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;

import java9.util.concurrent.CompletableFuture;

/**
 * blob_storage.rs API
 */

/**
 * High level wrapper for wallet SDK functions.
 */
public class BlobStorageReader extends IndyJava.API {

	private final int blobStorageReaderHandle;

	private BlobStorageReader(int blobStorageReaderHandle) {

		this.blobStorageReaderHandle = blobStorageReaderHandle;
	}

	/**
	 * Gets the handle for the blob storage.
	 *
	 * @return The handle for the blob storage.
	 */
	public int getBlobStorageReaderHandle() {

		return this.blobStorageReaderHandle;
	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when openReader completes.
	 */
	private static Callback openReaderCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int handle) {

			CompletableFuture<BlobStorageReader> future = (CompletableFuture<BlobStorageReader>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			BlobStorageReader tailsReader = new BlobStorageReader(handle);

			future.complete(tailsReader);
		}
	};

	/*
	 * STATIC METHODS
	 */

	public static CompletableFuture<BlobStorageReader> openReader(
			String type,
			String config) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNullOrWhiteSpace(config, "config");

		CompletableFuture<BlobStorageReader> future = new CompletableFuture<BlobStorageReader>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_open_blob_storage_reader(
				commandHandle,
				type,
				config,
				openReaderCb);

		checkResult(future, result);

		return future;
	}
}
