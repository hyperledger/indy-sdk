package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyJava;

/**
 * did.rs results
 */
/**
 * Result classes for Did operations.
 */
public final class DidResults {

	private DidResults() {

	}

	/**
	 * Result from calling createAndStoreMyDid.
	 */
	public static class CreateAndStoreMyDidResult extends IndyJava.Result {

		private String did, verkey;
		CreateAndStoreMyDidResult(String did, String verkey) { this.did = did; this.verkey = verkey; }
		
		/**
		 * Gets the DID.
		 * 
		 * @return The DID.
		 */
		public String getDid() { return this.did; }
		
		/**
		 * Gets the verification key.
		 * 
		 * @return The verification key.
		 */
		public String getVerkey() { return this.verkey; }
	}

	/**
	 * Result from calling encrypt.
	 */
	public static class EncryptResult extends IndyJava.Result {

		private byte[] encryptedMessage, nonce;
		EncryptResult(byte[] encryptedMessage, byte[] nonce) { this.encryptedMessage = encryptedMessage; this.nonce = nonce; }

		/**
		 * Gets the encrypted message.
		 *
		 * @return The encrypted message.
		 */
		public byte[] getEncryptedMessage() { return this.encryptedMessage; }

		/**
		 * Gets the nonce.
		 *
		 * @return The nonce.
		 */
		public byte[] getNonce() { return this.nonce; }
	}

	/**
	 * Result from calling endpointForDid.
	 */
	public static class EndpointForDidResult extends IndyJava.Result {

		private String address, transportKey;
		EndpointForDidResult(String address, String transportKey) { this.address = address; this.transportKey = transportKey;}

		/**
		 * Gets the Endpoint.
		 *
		 * @return The Endpoint.
		 */
		public String getAddress() { return this.address; }

		/**
		 * Gets the transport key.
		 *
		 * @return The transport key.
		 */
		public String getTransportKey() { return this.transportKey; }
	}
}
