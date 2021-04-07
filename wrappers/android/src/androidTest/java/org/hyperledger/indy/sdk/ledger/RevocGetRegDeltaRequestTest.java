package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.utils.JsonTestUtils;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;

import java.util.Date;

public class RevocGetRegDeltaRequestTest extends LedgerIntegrationTest {

	@Test
	public void testBuildGetRevocRegDeltaRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("operation", new JSONObject()
						.put("type", "117")
						.put("revocRegDefId", revRegDefId)
						.put("to", 100)
				);

		String request = Ledger.buildGetRevocRegDeltaRequest(DID, revRegDefId, - 1, 100).get();

		assert (JsonTestUtils.toJsonMap(request).entrySet()
				.containsAll(
						JsonTestUtils.toJsonMap(expectedResult).entrySet()));
	}

	@Test
	public void testRevocRegRequestsDeltaWorks() throws Exception {
		postEntities();

		int to = (int) (new Date().getTime()/1000) + 100;

		String getRevRegRequest = Ledger.buildGetRevocRegDeltaRequest(DID, revRegDefId, -1, to).get();
		String getRevReResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegRequest, response -> {
			JSONObject responseObject = new JSONObject(response);
			return !responseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetRevocRegDeltaResponse(getRevReResponse).get();
	}
}
