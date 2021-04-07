package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertNotNull;

public class ProverCreateProofTest extends AnoncredsIntegrationTest {

	private String requestedCredentialsJson = String.format("{" +
			"\"self_attested_attributes\":{}," +
			"\"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%s\", \"revealed\":true}}," +
			"\"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%s\"}}" +
			"}", credentialId1, credentialId1);

	public ProverCreateProofTest() throws JSONException {
	}

	@Test
	public void testProverCreateProofWorks() throws Exception {

		String schemasJson = new JSONObject().put(gvtSchemaId, new JSONObject(gvtSchema)).toString();
		String credentialDefsJson = new JSONObject().put(issuer1gvtCredDefId, new JSONObject(issuer1gvtCredDef)).toString();
		String revocStatesJson = new JSONObject().toString();

		String proofJson = Anoncreds.proverCreateProof(wallet, proofRequest, new JSONObject(requestedCredentialsJson).toString(),
				masterSecretId, schemasJson, credentialDefsJson, revocStatesJson).get();
		assertNotNull(proofJson);
	}

	@Test
	public void testProverCreateProofWorksForInvalidMasterSecret() throws Exception {

		String schemasJson = new JSONObject().put(gvtSchemaId, new JSONObject(gvtSchema)).toString();
		String credentialDefsJson = new JSONObject().put(issuer1gvtCredDefId, new JSONObject(issuer1gvtCredDef)).toString();
		String revocStatesJson = new JSONObject().toString();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Anoncreds.proverCreateProof(wallet, proofRequest, new JSONObject(requestedCredentialsJson).toString(),
				"wrong_master_secret", schemasJson, credentialDefsJson, revocStatesJson).get();
	}
}
