package org.hyperledger.indy.sdk.pool;

import org.hyperledger.indy.sdk.IndyJava;

/**
 * pool.rs JSON parameters
 */
public final class PoolJSONParameters {

	private PoolJSONParameters() {

	}

	public static class CreatePoolLedgerConfigJSONParameter extends IndyJava.JsonParameter {

		public CreatePoolLedgerConfigJSONParameter(String genesisTxn) {

			if (genesisTxn != null) this.map.put("genesis_txn", genesisTxn);
		}
	}

	public static class OpenPoolLedgerJSONParameter extends IndyJava.JsonParameter {

		public OpenPoolLedgerJSONParameter(Integer timeout, Integer extended_timeout) {

			if (timeout != null) this.map.put("timeout", timeout);
			if (extended_timeout != null) this.map.put("extended_timeout", extended_timeout);
		}
	}
}
