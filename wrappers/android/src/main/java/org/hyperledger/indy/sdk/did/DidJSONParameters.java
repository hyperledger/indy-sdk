package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyJava;

/**
 * did.rs JSON parameters
 */
public final class DidJSONParameters {

	private DidJSONParameters() {

	}

	public static class CreateAndStoreMyDidJSONParameter extends IndyJava.JsonParameter {

		public CreateAndStoreMyDidJSONParameter(String did, String seed, String cryptoType, Boolean cid) {

			if (did != null) this.map.put("did", did);
			if (seed != null) this.map.put("seed", seed);
			if (cryptoType != null) this.map.put("crypto_type", cryptoType);
			if (cid != null) this.map.put("cid", cid);
		}
	}
}
