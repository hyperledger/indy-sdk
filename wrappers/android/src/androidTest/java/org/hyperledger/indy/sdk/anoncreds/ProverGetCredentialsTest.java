package org.hyperledger.indy.sdk.anoncreds;


import org.hyperledger.indy.sdk.wallet.WalletInvalidQueryException;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;

public class ProverGetCredentialsTest extends AnoncredsIntegrationTest {

	public ProverGetCredentialsTest() throws JSONException {
	}

	@Test
	public void testProverGetCredentialsWorksForEmptyFilter() throws Exception {

		JSONObject json = new JSONObject();
		String filter = json.toString();

		String credentials = Anoncreds.proverGetCredentials(wallet, filter).get();

		JSONArray credentialsArray = new JSONArray(credentials);

		assertEquals(3, credentialsArray.length());
	}

	@Test
	public void testProverGetCredentialsWorksForFilterByIssuer() throws Exception {

		JSONObject json = new JSONObject();
		String filter = json.put("issuer_did", issuerDid).toString();

		String credentials = Anoncreds.proverGetCredentials(wallet, filter).get();

		JSONArray credentialsArray = new JSONArray(credentials);

		assertEquals(2, credentialsArray.length());
	}

	@Test
	public void testProverGetCredentialsWorksForEmptyResult() throws Exception {

		JSONObject json = new JSONObject();
		String filter = json.put("issuer_did", issuerDid + "a").toString();

		String credentials = Anoncreds.proverGetCredentials(wallet, filter).get();

		JSONArray credentialsArray = new JSONArray(credentials);

		assertEquals(0, credentialsArray.length());
	}

	@Test
	public void testProverGetCredentialsWorksForInvalidFilterJson() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletInvalidQueryException.class));

		JSONObject json = new JSONObject();
		String filter = json.put("issuer_did", 1).toString();

		Anoncreds.proverGetCredentials(wallet, filter).get();
	}
}
