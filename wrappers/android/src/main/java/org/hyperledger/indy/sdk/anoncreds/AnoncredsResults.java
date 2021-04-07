package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.IndyJava;

/**
 * anoncreds.rs results
 */

/**
 * Result classes related to anonymous credentials calls.
 */
public final class AnoncredsResults {

	private AnoncredsResults() {

	}

	/**
	 * Result from calling issuerCreateSchema.
	 */
	public static class IssuerCreateSchemaResult extends IndyJava.Result {

		private String schemaId, schemaJson;

		IssuerCreateSchemaResult(String schemaId, String schemaJson) {
			this.schemaId = schemaId;
			this.schemaJson = schemaJson;
		}

		/**
		 * Gets the schema Id.
		 *
		 * @return Schema Id.
		 */
		public String getSchemaId() {
			return this.schemaId;
		}

		/**
		 * Gets the schema JSON.
		 *
		 * @return The schema JSON.
		 */
		public String getSchemaJson() {
			return this.schemaJson;
		}
	}

	/**
	 * Result from calling IssuerCreateAndStoreCredentialDefResult.
	 */
	public static class IssuerCreateAndStoreCredentialDefResult extends IndyJava.Result {

		private String credDefId, credDefJson;

		IssuerCreateAndStoreCredentialDefResult(String credDefId, String credDefJson) {
			this.credDefId = credDefId;
			this.credDefJson = credDefJson;
		}

		/**
		 * Gets the credential def Id.
		 *
		 * @return credential def Id.
		 */
		public String getCredDefId() {
			return this.credDefId;
		}

		/**
		 * Gets the credential definition JSON.
		 *
		 * @return The credential definition JSON.
		 */
		public String getCredDefJson() {
			return this.credDefJson;
		}
	}

	/**
	 * Result from calling issuerCreateAndStoreRevocReg.
	 */
	public static class IssuerCreateAndStoreRevocRegResult extends IndyJava.Result {

		private String revRegid, revRegDefJson, revRegEntryJson;

		IssuerCreateAndStoreRevocRegResult(String revRegid, String revRegDefJson, String revRegEntryJson) {
			this.revRegid = revRegid;
			this.revRegDefJson = revRegDefJson;
			this.revRegEntryJson = revRegEntryJson;
		}

		/**
		 * Gets the revocation registry Id.
		 *
		 * @return revocation registry Id.
		 */
		public String getRevRegId() {
			return this.revRegid;
		}

		/**
		 * Gets the revocation registry definition JSON.
		 *
		 * @return The revocation registry definition JSON.
		 */
		public String getRevRegDefJson() {
			return this.revRegDefJson;
		}

		/**
		 * Gets the revocation registry entry JSON.
		 *
		 * @return The revocation registry entry JSON.
		 */
		public String getRevRegEntryJson() {
			return this.revRegEntryJson;
		}
	}

	/**
	 * Result from calling issuerCreateCredential.
	 */
	public static class IssuerCreateCredentialResult extends IndyJava.Result {

		private String credentialJson, revocId, revocRegDeltaJson;

		IssuerCreateCredentialResult(String credentialJson, String revocId, String revocRegUpdateJson) {
			this.credentialJson = credentialJson;
			this.revocId = revocId;
			this.revocRegDeltaJson = revocRegUpdateJson;
		}

		/**
		 * Gets the credential JSON.
		 *
		 * @return The credential JSON.
		 */
		public String getCredentialJson() {
			return this.credentialJson;
		}

		/**
		 * Gets the credential revocation Id.
		 *
		 * @return The credential revocation Id.
		 */
		public String getRevocId() {
			return this.revocId;
		}

		/**
		 * Gets the revocation registration delta JSON.
		 *
		 * @return The revocation registration delta JSON.
		 */
		public String getRevocRegDeltaJson() {
			return this.revocRegDeltaJson;
		}
	}

	/**
	 * Result from calling proverCreateCredentialReq.
	 */
	public static class ProverCreateCredentialRequestResult extends IndyJava.Result {

		private String credentialRequestJson, credentialRequestMetadataJson;

		ProverCreateCredentialRequestResult(String credentialRequestJson, String credentialRequestMetadataJson) {
			this.credentialRequestJson = credentialRequestJson;
			this.credentialRequestMetadataJson = credentialRequestMetadataJson;
		}

		/**
		 * Gets the credential request JSON.
		 *
		 * @return The credential request JSON.
		 */
		public String getCredentialRequestJson() {
			return this.credentialRequestJson;
		}

		/**
		 * Gets the credential request metadata JSON.
		 *
		 * @return The credential request metadata JSON.
		 */
		public String getCredentialRequestMetadataJson() {
			return this.credentialRequestMetadataJson;
		}
	}
}
