package org.hyperledger.indy.sdk.ledger;

import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;
import org.hyperledger.indy.sdk.JsonObjectSimilar;

public class GetTxnRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	@Test
	public void testBuildGetTxnRequestWorks() throws Exception {
		int seq_no = 1;

		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "3")
								.put("ledgerId", 1)
								.put("data", seq_no)
				);

		String getTxnRequest = Ledger.buildGetTxnRequest(DID, null, seq_no).get();
		assert (new JSONObject(getTxnRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	public void testBuildGetTxnRequestWorksForLedgerType() throws Exception {
		int seq_no = 1;

		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "3")
								.put("ledgerId", 0)
								.put("data", seq_no)
				);

		String getTxnRequest = Ledger.buildGetTxnRequest(DID, "POOL", seq_no).get();
		assert (new JSONObject(getTxnRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testGetTxnRequestWorks() throws Exception {
		String did = createStoreAndPublishDidFromTrustee();

		String schemaRequest = Ledger.buildSchemaRequest(did, SCHEMA_DATA).get();
		String schemaResponse = Ledger.signAndSubmitRequest(pool, wallet, did, schemaRequest).get();

		JSONObject schemaResponseObj = new JSONObject(schemaResponse);
		int seqNo = schemaResponseObj.getJSONObject("result").getJSONObject("txnMetadata").getInt("seqNo");

		String getTxnRequest = Ledger.buildGetTxnRequest(did, null, seqNo).get();
		String expectedData = "{\"name\":\"gvt\",\"version\":\"1.0\",\"attr_names\": [\"name\"]}";

		String getTxnResponse = PoolUtils.ensurePreviousRequestApplied(pool, getTxnRequest, response -> {
			JSONObject getTxnResponseObj = new JSONObject(response);
			JSONObject schemaTransactionObj =
					getTxnResponseObj.getJSONObject("result").getJSONObject("data").getJSONObject("txn").getJSONObject("data").getJSONObject("data");

			return JsonObjectSimilar.similar(new JSONObject(expectedData), schemaTransactionObj);
		});
		assertNotNull(getTxnResponse);
	}
}
