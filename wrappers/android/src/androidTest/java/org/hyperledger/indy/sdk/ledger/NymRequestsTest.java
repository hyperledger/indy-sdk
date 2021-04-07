package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.utils.JsonTestUtils;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

public class NymRequestsTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	private String dest = "FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4";
	private String role = "STEWARD";
	private String alias = "some_alias";

	@Test
	public void testBuildNymRequestWorksForOnlyRequiredFields() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("dest", dest)
								.put("type", "1")
				);

		String request = Ledger.buildNymRequest(DID, dest, null, null, null).get();
		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildNymRequestWorksForOnlyOptionalFields() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("alias", alias)
								.put("dest", dest)
								.put("verkey", VERKEY_TRUSTEE)
								.put("role", "2")
								.put("type", "1")
				);


		String request = Ledger.buildNymRequest(DID, dest, VERKEY_TRUSTEE, alias, role).get();
		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildGetNymRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("dest", dest)
								.put("type", "105")
				);

		String request = Ledger.buildGetNymRequest(DID, dest).get();
		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testBuildGetNymRequestWorksForDefaultSubmitter() throws Exception {
		Ledger.buildGetNymRequest(null, dest).get();
	}

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testNymRequestsWorks() throws Exception {
		DidResults.CreateAndStoreMyDidResult trusteeDidResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String trusteeDid = trusteeDidResult.getDid();

		DidResults.CreateAndStoreMyDidResult myDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String myDid = myDidResult.getDid();
		String myVerkey = myDidResult.getVerkey();

		String nymRequest = Ledger.buildNymRequest(trusteeDid, myDid, myVerkey, null, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDid, nymRequest).get();

		String getNymRequest = Ledger.buildGetNymRequest(myDid, myDid).get();
		String getNymResponse = PoolUtils.ensurePreviousRequestApplied(pool, getNymRequest,
				innerResponse -> {
					JSONObject innerResponseObject = new JSONObject(innerResponse);
					return !innerResponseObject.getJSONObject("result").isNull("seqNo");
				});
		assertNotNull(getNymResponse);

		String nymDataJson = Ledger.parseGetNymResponse(getNymResponse).get();
		JSONObject nymData = new JSONObject(nymDataJson);
		assertEquals(myDid, nymData.getString("did"));
		assertEquals(myVerkey, nymData.getString("verkey"));
	}
}
