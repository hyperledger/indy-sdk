package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.*;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

public class AttribRequestsTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	private String endpoint = "{\"endpoint\":{\"ha\":\"127.0.0.1:5555\"}}";
	private String hash = "83d907821df1c87db829e96569a11f6fc2e7880acba5e43d07ab786959e13bd3";
	private String enc = "aa3f41f619aa7e5e6b6d0de555e05331787f9bf9aa672b94b57ab65b9b66c3ea960b18a98e3834b1fc6cebf49f463b81fd6e3181";

	@Test
	public void testBuildAttribRequestWorksForRawValue() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"100\"," +
				"\"dest\":\"%s\"," +
				"\"raw\":\"%s\"" +
				"}", DID_TRUSTEE, DID_TRUSTEE, endpoint);

		String attribRequest = Ledger.buildAttribRequest(DID_TRUSTEE, DID_TRUSTEE, null, endpoint, null).get();

		assertTrue(attribRequest.replace("\\", "").contains(expectedResult));
	}

	@Test
	public void testBuildAttribRequestWorksForHashValue() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"100\"," +
				"\"dest\":\"%s\"," +
				"\"hash\":\"%s\"" +
				"}", DID_TRUSTEE, DID_TRUSTEE, hash);

		String attribRequest = Ledger.buildAttribRequest(DID_TRUSTEE, DID_TRUSTEE, hash, null, null).get();

		assertTrue(attribRequest.replace("\\", "").contains(expectedResult));
	}

	@Test
	public void testBuildAttribRequestWorksForEncValue() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"100\"," +
				"\"dest\":\"%s\"," +
				"\"enc\":\"%s\"" +
				"}", DID_TRUSTEE, DID_TRUSTEE, enc);

		String attribRequest = Ledger.buildAttribRequest(DID_TRUSTEE, DID_TRUSTEE, null, null, enc).get();

		assertTrue(attribRequest.replace("\\", "").contains(expectedResult));
	}

	@Test
	public void testBuildAttribRequestWorksForMissedAttribute() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Ledger.buildAttribRequest(DID_TRUSTEE, DID_TRUSTEE, null, null, null).get();
	}

	@Test
	public void testBuildGetAttribRequestWorksForRawValue() throws Exception {
		String raw = "endpoint";

		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"104\"," +
				"\"dest\":\"%s\"," +
				"\"raw\":\"%s\"" +
				"}", DID_TRUSTEE, DID_TRUSTEE, raw);

		String getAttribRequest = Ledger.buildGetAttribRequest(DID_TRUSTEE, DID_TRUSTEE, raw, null, null).get();

		assertTrue(getAttribRequest.contains(expectedResult));
	}

	@Test
	public void testBuildGetAttribRequestWorksForHashValue() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"104\"," +
				"\"dest\":\"%s\"," +
				"\"hash\":\"%s\"" +
				"}", DID_TRUSTEE, DID_TRUSTEE, hash);

		String getAttribRequest = Ledger.buildGetAttribRequest(DID_TRUSTEE, DID_TRUSTEE, null, hash, null).get();

		assertTrue(getAttribRequest.contains(expectedResult));
	}

	@Test
	public void testBuildGetAttribRequestWorksForEncValue() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"104\"," +
				"\"dest\":\"%s\"," +
				"\"enc\":\"%s\"" +
				"}", DID_TRUSTEE, DID_TRUSTEE, enc);

		String getAttribRequest = Ledger.buildGetAttribRequest(DID_TRUSTEE, DID_TRUSTEE, null, null, enc).get();

		assertTrue(getAttribRequest.contains(expectedResult));
	}

	@Test
	public void testBuildGetAttribRequestWorksForDefaultSubmitter() throws Exception {
		Ledger.buildGetAttribRequest(null, DID_TRUSTEE, "endpoint", null, null).get();
	}

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testAttribRequestsWorksForRawValue() throws Exception {
		DidResults.CreateAndStoreMyDidResult trusteeDidResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String trusteeDid = trusteeDidResult.getDid();

		DidResults.CreateAndStoreMyDidResult myDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String myDid = myDidResult.getDid();
		String myVerkey = myDidResult.getVerkey();

		String nymRequest = Ledger.buildNymRequest(trusteeDid, myDid, myVerkey, null, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDid, nymRequest).get();

		String attribRequest = Ledger.buildAttribRequest(myDid, myDid, null, endpoint, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, attribRequest).get();

		String getAttribRequest = Ledger.buildGetAttribRequest(myDid, myDid, "endpoint", null, null).get();

		String getAttribResponse = PoolUtils.ensurePreviousRequestApplied(pool, getAttribRequest, response -> {
			JSONObject getAttribResponseObject = new JSONObject(response);
			return endpoint.equals(getAttribResponseObject.getJSONObject("result").getString("data"));
		});
		assertNotNull(getAttribResponse);
	}

	@Test
	public void testBuildAttribRequestWorksForInvalidIdentifier() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Ledger.buildAttribRequest("invalid_base58_identifier", DID_TRUSTEE, null, endpoint, null).get();
	}
}