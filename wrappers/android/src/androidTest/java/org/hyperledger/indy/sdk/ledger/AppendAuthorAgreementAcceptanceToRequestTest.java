package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.JsonObjectSimilar;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertTrue;


public class AppendAuthorAgreementAcceptanceToRequestTest extends IndyIntegrationTest {


	private String text = "some agreement text";
	private String version = "1.0.0";
	private String acceptanceMechanismType = "acceptance type 1";
	private String hash = "050e52a57837fff904d3d059c8a123e3a04177042bf467db2b2c27abd8045d5e";
	private int timeOfAcceptance = 123379200;

	private void checkRequestAcceptance(String request) throws JSONException {
		JSONObject expectedAcceptance = new JSONObject()
				.put("mechanism", acceptanceMechanismType)
				.put("taaDigest", hash)
				.put("time", timeOfAcceptance);

		JSONObject actualRequest = new JSONObject(request).getJSONObject("taaAcceptance");
		assertTrue(JsonObjectSimilar.similar(actualRequest, expectedAcceptance));
	}

	@Test
	public void testAppendAuthorAgreementAcceptanceToRequestForTextVersion() throws Exception {
		String requestWithAcceptance = Ledger.appendTxnAuthorAgreementAcceptanceToRequest(REQUEST.toString(), text, version, null, acceptanceMechanismType, timeOfAcceptance).get();
		checkRequestAcceptance(requestWithAcceptance);
	}

	@Test
	public void testAppendAuthorAgreementAcceptanceToRequestForHash() throws Exception {
		String requestWithAcceptance = Ledger.appendTxnAuthorAgreementAcceptanceToRequest(REQUEST.toString(), null, null, hash, acceptanceMechanismType, timeOfAcceptance).get();
		checkRequestAcceptance(requestWithAcceptance);
	}
}
