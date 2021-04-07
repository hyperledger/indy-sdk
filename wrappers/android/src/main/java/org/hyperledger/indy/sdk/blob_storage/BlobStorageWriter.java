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
public class BlobStorageWriter extends IndyJava.API {

	private final int blobStorageWriterHandle;

	private BlobStorageWriter(int blobStorageWriterHandle) {

		this.blobStorageWriterHandle = blobStorageWriterHandle;
	}

	/**
	 * Gets the handle for the blob storage.
	 *
	 * @return The handle for the blob storage.
	 */
	public int getBlobStorageWriterHandle() {

		return this.blobStorageWriterHandle;
	}

	/*
	 * STATIC CALLBACKS
	 */

	/**
	 * Callback used when openReader completes.
	 */
	private static Callback openWriterCb = new Callback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, int handle) {

			CompletableFuture<BlobStorageWriter> future = (CompletableFuture<BlobStorageWriter>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			BlobStorageWriter blobStorageWriter = new BlobStorageWriter(handle);

			future.complete(blobStorageWriter);
		}
	};

	/*
	 * STATIC METHODS
	 */

	public static CompletableFuture<BlobStorageWriter> openWriter(
			String type,
			String config) throws IndyException {

		ParamGuard.notNullOrWhiteSpace(type, "type");
		ParamGuard.notNullOrWhiteSpace(config, "config");

		CompletableFuture<BlobStorageWriter> future = new CompletableFuture<BlobStorageWriter>();
		int commandHandle = addFuture(future);

		int result = LibIndy.api.indy_open_blob_storage_writer(
				commandHandle,
				type,
				config,
				openWriterCb);

		checkResult(future, result);

		return future;
	}
}
