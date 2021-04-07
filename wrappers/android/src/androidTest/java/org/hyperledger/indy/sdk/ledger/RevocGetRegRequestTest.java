package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.utils.JsonTestUtils;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;

import java.util.Date;

public class RevocGetRegRequestTest extends LedgerIntegrationTest {

	@Test
	public void testBuildGetRevocRegRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("operation", new JSONObject()
						.put("type", "116")
						.put("revocRegDefId", revRegDefId)
						.put("timestamp", 100)
				);

		String request = Ledger.buildGetRevocRegRequest(DID, revRegDefId, 100).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testRevocRegRequestsWorks() throws Exception {
		postEntities();

		int timestamp = (int) (new Date().getTime()/1000) + 100;

		String getRevRegRequest = Ledger.buildGetRevocRegRequest(DID, revRegDefId, timestamp).get();
		String getRevReResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegRequest, response -> {
			JSONObject responseObject = new JSONObject(response);
			return !responseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetRevocRegResponse(getRevReResponse).get();
	}
}
