package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;

public class RevocRegDefRequestTest extends LedgerIntegrationTest {

	@Test
	public void testBuildRevocRegDefRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "113")
								.put("id", "NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:CL_ACCUM:TAG_1")
								.put("revocDefType", "CL_ACCUM")
								.put("tag", "TAG1")
								.put("tag", "TAG1")
								.put("credDefId", "NcYxiDXkpYi6ov5FcYDi1e:3:CL:1")
								.put("credDefId", "NcYxiDXkpYi6ov5FcYDi1e:3:CL:1")
								.put("value", new JSONObject()
										.put("issuanceType", "ISSUANCE_ON_DEMAND")
										.put("maxCredNum", 5)
										.put("tailsHash", "s")
										.put("tailsLocation", "http://tails.location.com")
										.put("publicKeys", new JSONObject()
												.put("accumKey", new JSONObject()
														.put("z", "1 0000000000000000000000000000000000000000000000000000000000001111 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000")
												)
										)
								)
				);

		JSONObject data = new JSONObject()
				.put("ver", "1.0")
				.put("id", "NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:CL_ACCUM:TAG_1")
				.put("revocDefType", "CL_ACCUM")
				.put("tag", "TAG1")
				.put("credDefId", "NcYxiDXkpYi6ov5FcYDi1e:3:CL:1")
				.put("value",
						new JSONObject()
								.put("issuanceType", "ISSUANCE_ON_DEMAND")
								.put("maxCredNum", 5)
								.put("tailsHash", "s")
								.put("tailsLocation", "http://tails.location.com")
								.put("publicKeys", new JSONObject()
										.put("accumKey", new JSONObject()
												.put("z", "1 0000000000000000000000000000000000000000000000000000000000001111 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000")
										)
								)
				);

		String request = Ledger.buildRevocRegDefRequest(DID, data.toString()).get();

		assert (new JSONObject(request).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	public void testRevocRegDefRequestsWorks() throws Exception {
		postEntities();

		String getRevRegDefRequest = Ledger.buildGetRevocRegDefRequest(DID, revRegDefId).get();
		String getRevRegDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDefRequest, response -> {
			JSONObject responseObject = new JSONObject(response);
			return ! responseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetRevocRegDefResponse(getRevRegDefResponse).get();
	}
}
