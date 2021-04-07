package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyJava;

/**
 * ledger.rs results
 */
public final class LedgerResults {

	private LedgerResults() {

	}

	/**
	 * Result from calling parseResponse functions.
	 */
	public static class ParseResponseResult extends IndyJava.Result {

		private String id, objectJson;

		ParseResponseResult(String id, String objectJson) {
			this.id = id;
			this.objectJson = objectJson;
		}

		/**
		 * Gets the Id.
		 *
		 * @return Id.
		 */
		public String getId() {
			return this.id;
		}

		/**
		 * Gets the object JSON.
		 *
		 * @return The object JSON.
		 */
		public String getObjectJson() {
			return this.objectJson;
		}
	}

	/**
	 * Result from calling parseRegistryResponse functions.
	 */
	public static class ParseRegistryResponseResult extends IndyJava.Result {

		private String id, objectJson;
		long timestamp;

		ParseRegistryResponseResult(String id, String objectJson, long timestamp) {
			this.id = id;
			this.objectJson = objectJson;
			this.timestamp = timestamp;
		}

		/**
		 * Gets the Id.
		 *
		 * @return Id.
		 */
		public String getId() {
			return this.id;
		}

		/**
		 * Gets the object JSON.
		 *
		 * @return The object JSON.
		 */
		public String getObjectJson() {
			return this.objectJson;
		}

		/**
		 * Gets the timestamp.
		 *
		 * @return The timestamp.
		 */
		public long getTimestamp() {
			return this.timestamp;
		}
	}
}
