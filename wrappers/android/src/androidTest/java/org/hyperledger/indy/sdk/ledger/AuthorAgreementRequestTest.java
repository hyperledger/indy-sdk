package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.utils.JsonTestUtils;
import org.json.JSONObject;
import org.junit.Test;


public class AuthorAgreementRequestTest extends IndyIntegrationTest {

	private String TEXT = "indy agreement";
	private String VERSION = "1.0.0";

	@Test
	public void testBuildTxnAuthorAgreementRequest() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "4")
								.put("text", TEXT)
								.put("version", VERSION)
				);

		String request = Ledger.buildTxnAuthorAgreementRequest(DID, TEXT, VERSION, -1, -1).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildTxnAuthorAgreementRequestForRetiredAndRatificatedWoText() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "4")
								.put("text", TEXT)
								.put("version", VERSION)
								.put("ratification_ts", 12345)
								.put("retirement_ts", 54321)
				);

		String request = Ledger.buildTxnAuthorAgreementRequest(DID, TEXT, VERSION, 12345, 54321).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildGetTxnAuthorAgreementRequest() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("operation",
						new JSONObject()
								.put("type", "6")
				);

		String request = Ledger.buildGetTxnAuthorAgreementRequest(null, null).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildGetTxnAuthorAgreementRequestForVersion() throws Exception {
		JSONObject data = new JSONObject()
				.put("version", VERSION);

		JSONObject expectedResult = new JSONObject()
				.put("operation",
						new JSONObject()
								.put("type", "6")
								.put("version", VERSION)
				);

		String request = Ledger.buildGetTxnAuthorAgreementRequest(null, data.toString()).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildDisableAllTxnAuthorAgreementsRequest() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("operation",
						new JSONObject()
								.put("type", "8")
				);

		String request = Ledger.buildDisableAllTxnAuthorAgreementsRequest(DID).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}
}
