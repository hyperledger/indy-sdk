package org.hyperledger.indy.sdk.anoncreds;


import org.hyperledger.indy.sdk.JsonObjectSimilar;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;

public class ProverGetCredentialTest extends AnoncredsIntegrationTest {

	public ProverGetCredentialTest() throws JSONException {
	}

	@Test
	public void testProverGetCredentialWorks() throws Exception {
		String credentialJson = Anoncreds.proverGetCredential(wallet, credentialId1).get();

		JSONObject credential = new JSONObject(credentialJson);

		JSONObject expected = new JSONObject();
		expected.put("schema_id", gvtSchemaId);
		expected.put("cred_def_id", issuer1gvtCredDefId);
		expected.put("referent", credentialId1);
		expected.put("rev_reg_id", JSONObject.NULL);
		expected.put("cred_rev_id", JSONObject.NULL);
		JSONObject attrs = new JSONObject();
		attrs.put("sex", "male");
		attrs.put("name", "Alex");
		attrs.put("height", "175");
		attrs.put("age", "28");
		expected.put("attrs", attrs);

		assertTrue(JsonObjectSimilar.similar(expected, credential));
	}

	@Test
	public void testProverGetCredentialWorksForNotFound() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Anoncreds.proverGetCredential(wallet, "other_cred_id").get();
	}

}
