package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyJava;

/**
 * crypto.rs JSON parameters
 */
public final class CryptoJSONParameters {

	private CryptoJSONParameters() {

	}

	public static class CreateAndStoreMyDidJSONParameter extends IndyJava.JsonParameter {

		public CreateAndStoreMyDidJSONParameter(String did, String seed, String cryptoType, Boolean cid) {

			if (did != null) this.map.put("did", did);
			if (seed != null) this.map.put("seed", seed);
			if (cryptoType != null) this.map.put("crypto_type", cryptoType);
			if (cid != null) this.map.put("cid", cid);
		}
	}


	public static class CreateKeyJSONParameter extends IndyJava.JsonParameter {

		public CreateKeyJSONParameter(String seed, String cryptoType) {

			if (seed != null) this.map.put("seed", seed);
			if (cryptoType != null) this.map.put("crypto_type", cryptoType);
		}
	}
}
