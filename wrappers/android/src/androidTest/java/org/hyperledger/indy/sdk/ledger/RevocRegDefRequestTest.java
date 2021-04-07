package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertTrue;

public class RevocRegDefRequestTest extends LedgerIntegrationTest {

	@Test
	public void testBuildRevocRegDefRequestWorks() throws Exception {
		String expectedResult =
				"\"operation\":{" +
						"\"type\":\"113\"," +
						"\"id\":\"NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:CL_ACCUM:TAG_1\"," +
						"\"revocDefType\":\"CL_ACCUM\"," +
						"\"tag\":\"TAG1\"," +
						"\"credDefId\":\"NcYxiDXkpYi6ov5FcYDi1e:3:CL:1\"," +
						"\"value\":{" +
						"   \"issuanceType\":\"ISSUANCE_ON_DEMAND\"," +
						"   \"maxCredNum\":5," +
						"   \"publicKeys\":{" +
						"       \"accumKey\":{" +
						"           \"z\":\"1 0000000000000000000000000000000000000000000000000000000000001111 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000\"" +
						"       }" +
						"   }";

		String data = "{\n" +
				"        \"ver\": \"1.0\",\n" +
				"        \"id\": \"NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:CL_ACCUM:TAG_1\",\n" +
				"        \"revocDefType\": \"CL_ACCUM\",\n" +
				"        \"tag\": \"TAG1\",\n" +
				"        \"credDefId\": \"NcYxiDXkpYi6ov5FcYDi1e:3:CL:1\",\n" +
				"        \"value\": {\n" +
				"            \"issuanceType\": \"ISSUANCE_ON_DEMAND\",\n" +
				"            \"maxCredNum\": 5,\n" +
				"            \"tailsHash\": \"s\",\n" +
				"            \"tailsLocation\": \"http://tails.location.com\",\n" +
				"            \"publicKeys\": {\n" +
				"                \"accumKey\": {\n" +
				"                    \"z\": \"1 0000000000000000000000000000000000000000000000000000000000001111 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000\"\n" +
				"                }\n" +
				"            }\n" +
				"        }\n" +
				"    }";

		String request = Ledger.buildRevocRegDefRequest(DID, data).get();
		assertTrue(request.replaceAll("\\s+","").contains(expectedResult.replaceAll("\\s+","")));
	}

	@Test
	public void testRevocRegDefRequestsWorks() throws Exception {
		postEntities();

		String getRevRegDefRequest = Ledger.buildGetRevocRegDefRequest(DID, revRegDefId).get();
		String getRevRegDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDefRequest, response -> {
			JSONObject responseObject = new JSONObject(response);
			return !responseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetRevocRegDefResponse(getRevRegDefResponse).get();
	}
}
