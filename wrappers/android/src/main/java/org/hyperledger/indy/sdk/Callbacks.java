package org.hyperledger.indy.sdk;

import java9.util.concurrent.CompletableFuture;

public class Callbacks extends IndyJava.API {

	/**
	 * Callback used when boolean callback completes.
	 */
	public static LibIndy.API.BoolCallback boolCallback = new LibIndy.API.BoolCallback() {

		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommand_handle, int err, IndyBool value) {
			CompletableFuture<Boolean> future = (CompletableFuture<Boolean>) removeFuture(xcommand_handle);
			if (! checkResult(future, err)) return;

			Boolean result = value.value();
			future.complete(result);
		}
	};
}
