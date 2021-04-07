package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.blob_storage.BlobStorageReader;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageWriter;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import static org.hyperledger.indy.sdk.utils.EnvironmentUtils.getIndyHomePath;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;


public class IssuerRevokeCredentialTest extends AnoncredsIntegrationTest {

	/* FIXME: getIndyHomePath hard coded forward slash "/". It will not work for Windows. */
	private String tailsWriterConfig = new JSONObject(String.format("{\"base_dir\":\"%s\", \"uri_pattern\":\"\"}", getIndyHomePath("tails")).replace('\\', '/')).toString();

	public IssuerRevokeCredentialTest() throws JSONException {
	}

	@Test
	public void testIssuerRevokeProofWorks() throws Exception {
		// Create wallet, get wallet handle
		String walletConfig = new JSONObject().put("id", "revocationWallet").toString();
		Wallet.createWallet(walletConfig, CREDENTIALS).get();
		Wallet wallet = Wallet.openWallet(walletConfig, CREDENTIALS).get();

		// Issuer create Schema
		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, gvtSchemaName, schemaVersion, gvtSchemaAttributes).get();
		String schemaId = createSchemaResult.getSchemaId();
		String schemaJson = createSchemaResult.getSchemaJson();

		// Issuer create issuer1GvtCredential definition
		String revocationCredentialDefConfig = "{\"support_revocation\":true}";
		AnoncredsResults.IssuerCreateAndStoreCredentialDefResult createCredentialDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid, schemaJson, tag, null, revocationCredentialDefConfig).get();
		String credDefId = createCredentialDefResult.getCredDefId();
		String credDefJson = createCredentialDefResult.getCredDefJson();

		// Issuer create revocation registry
		BlobStorageWriter tailsWriter = BlobStorageWriter.openWriter("default", tailsWriterConfig).get();
		String revRegConfig = "{\"issuance_type\":null,\"max_cred_num\":5}";
		AnoncredsResults.IssuerCreateAndStoreRevocRegResult createRevRegResult = Anoncreds.issuerCreateAndStoreRevocReg(wallet, issuerDid, null, tag, credDefId, revRegConfig, tailsWriter).get();
		String revRegId = createRevRegResult.getRevRegId();
		String revRegDef = createRevRegResult.getRevRegDefJson();

		// Prover create Master Secret
		Anoncreds.proverCreateMasterSecret(wallet, masterSecretId).get();

		// Issuer create Credential Offer
		String credOfferJson = Anoncreds.issuerCreateCredentialOffer(wallet, credDefId).get();

		// Prover create Credential Request
		AnoncredsResults.ProverCreateCredentialRequestResult createCredReqResult =
				Anoncreds.proverCreateCredentialReq(wallet, proverDid, credOfferJson, credDefJson, masterSecretId).get();
		String credentialReqJson = createCredReqResult.getCredentialRequestJson();
		String credentialReqMetadataJson = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer open TailsReader
		BlobStorageReader blobReaderCfg = BlobStorageReader.openReader("default", tailsWriterConfig).get();
		int blobStorageReaderHandleCfg = blobReaderCfg.getBlobStorageReaderHandle();

		//9. Issuer create Credential
		AnoncredsResults.IssuerCreateCredentialResult createCredentialResult =
				Anoncreds.issuerCreateCredential(wallet, credOfferJson, credentialReqJson, gvtCredentialValuesJson, revRegId, blobStorageReaderHandleCfg).get();
		String credJson = createCredentialResult.getCredentialJson();
		String credRevocId = createCredentialResult.getRevocId();
		String revRegDelta = createCredentialResult.getRevocRegDeltaJson();

		// Prover create RevocationState
		int timestamp = 100;
		String revStateJson = Anoncreds.createRevocationState(blobStorageReaderHandleCfg, revRegDef, revRegDelta, timestamp, credRevocId).get();

		// Prover store received Credential
		Anoncreds.proverStoreCredential(wallet, credentialId1, credentialReqMetadataJson, credJson, credDefJson, revRegDef).get();

		// Prover gets Credentials for Proof Request
		String credentialsJson = Anoncreds.proverGetCredentialsForProofReq(wallet, proofRequest).get();
		JSONObject credentials = new JSONObject(credentialsJson);
		JSONArray credentialsForAttr1 = credentials.getJSONObject("attrs").getJSONArray("attr1_referent");

		String credentialUuid = credentialsForAttr1.getJSONObject(0).getJSONObject("cred_info").getString("referent");

		// Prover create Proof
		String requestedCredentialsJson = String.format("{" +
				"\"self_attested_attributes\":{}," +
				"\"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%s\", \"revealed\":true, \"timestamp\":%d }}," +
				"\"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%s\", \"timestamp\":%d}}" +
				"}", credentialUuid, timestamp, credentialUuid, timestamp);

		String schemasJson = new JSONObject(String.format("{\"%s\":%s}", schemaId, schemaJson)).toString();
		String credentialDefsJson = new JSONObject(String.format("{\"%s\":%s}", credDefId, credDefJson)).toString();
		String revStatesJson = new JSONObject(String.format("{\"%s\": { \"%s\":%s }}", revRegId, timestamp, revStateJson)).toString();

		String proofJson = Anoncreds.proverCreateProof(wallet, proofRequest, requestedCredentialsJson, masterSecretId, schemasJson, credentialDefsJson, revStatesJson).get();
		JSONObject proof = new JSONObject(proofJson);

		// Issuer revoke Credential
		revRegDelta = Anoncreds.issuerRevokeCredential(wallet, blobStorageReaderHandleCfg, revRegId, credRevocId).get();

		// Verifier verify proof
		JSONObject revealedAttr1 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr1_referent");
		assertEquals("Alex", revealedAttr1.getString("raw"));

		String revRegDefsJson = new JSONObject(String.format("{\"%s\":%s}", revRegId, revRegDef)).toString();
		String revRegs = new JSONObject(String.format("{\"%s\": { \"%s\":%s }}", revRegId, timestamp, revRegDelta)).toString();

		boolean valid = Anoncreds.verifierVerifyProof(proofRequest, proofJson, schemasJson, credentialDefsJson, revRegDefsJson, revRegs).get();
		assertFalse(valid);

		// Close and Delete Wallet
		wallet.closeWallet().get();
		Wallet.deleteWallet(walletConfig, CREDENTIALS).get();
	}
}
