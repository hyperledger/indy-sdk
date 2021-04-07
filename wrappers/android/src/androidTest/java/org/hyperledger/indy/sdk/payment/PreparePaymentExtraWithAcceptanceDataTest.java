package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.JsonObjectSimilar;
import org.hyperledger.indy.sdk.payments.Payments;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertTrue;


public class PreparePaymentExtraWithAcceptanceDataTest extends IndyIntegrationTest {

	private String text = "some agreement text";
	private String version = "1.0.0";
	private String acceptanceMechanismType = "acceptance type 1";
	private String hash = "050e52a57837fff904d3d059c8a123e3a04177042bf467db2b2c27abd8045d5e";
	private int timeOfAcceptance = 123379200;
	private JSONObject taaAcceptance = new JSONObject()
			.put("mechanism", acceptanceMechanismType)
			.put("taaDigest", hash)
			.put("time", timeOfAcceptance);

	public PreparePaymentExtraWithAcceptanceDataTest() throws JSONException {
	}

	@Test
	public void testPreparePaymentExtraWithAcceptanceData() throws Exception {
		JSONObject extra = new JSONObject().put("data", "someData");

		String extraWithTaaJson = Payments.preparePaymentExtraWithAcceptanceData(extra.toString(), text, version, null, acceptanceMechanismType, timeOfAcceptance).get();
		JSONObject actualExtra = new JSONObject(extraWithTaaJson);

		JSONObject expectedExtra = new JSONObject().put("data", "someData").put("taaAcceptance", taaAcceptance);

		assertTrue(JsonObjectSimilar.similar(expectedExtra, actualExtra));

	}

	@Test
	public void testPreparePaymentExtraWithAcceptanceDataForEmptyExtra() throws Exception {
		String extraWithTaaJson = Payments.preparePaymentExtraWithAcceptanceData(null, text, version, null, acceptanceMechanismType, timeOfAcceptance).get();
		JSONObject actualExtra = new JSONObject(extraWithTaaJson);

		JSONObject expectedExtra = new JSONObject().put("taaAcceptance", taaAcceptance);

		assertTrue(JsonObjectSimilar.similar(expectedExtra, actualExtra));

	}
}
