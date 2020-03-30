package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.*;
import org.junit.rules.Timeout;

import java.util.concurrent.TimeUnit;

import static org.junit.Assert.assertTrue;

import org.hyperledger.indy.sdk.JsonObjectSimilar;

public class CredDefRequestsTest extends LedgerIntegrationTest {

	@Rule
	public Timeout globalTimeout = new Timeout(1, TimeUnit.MINUTES);

	@Test
	public void testBuildCredDefRequestWorks() throws Exception {
		JSONObject value = new JSONObject()
				.put("primary", new JSONObject()
						.put("n", "1")
						.put("s", "2")
						.put("rctxt", "1")
						.put("z", "1")
						.put("r", new JSONObject()
								.put("name", "1")
								.put("master_secret", "3")
						)

				);

		JSONObject data = new JSONObject()
				.put("ver", "1.0")
				.put("id", "NcYxiDXkpYi6ov5FcYDi1e:3:CL:1")
				.put("schemaId", "1")
				.put("type", "CL")
				.put("tag", "TAG_1")
				.put("value", value);

		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("ref", 1)
								.put("tag", "TAG_1")
								.put("signature_type", "CL")
								.put("data", value)
								.put("type", "102")
				);

		String credDefRequest = Ledger.buildCredDefRequest(DID, data.toString()).get();
		assert (new JSONObject(credDefRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	public void testBuildGetCredDefRequestWorks() throws Exception {
		int seqNo = 1;
		String id = DID + ":3:" + SIGNATURE_TYPE + ":" + seqNo + ":" + TAG;

		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("ref", seqNo)
								.put("origin", DID)
								.put("signature_type", SIGNATURE_TYPE)
								.put("tag", TAG)
								.put("type", "108")
				);

		String getCredDefRequest = Ledger.buildGetCredDefRequest(DID, id).get();
		assert (new JSONObject(getCredDefRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testCredDefRequestsWorks() throws Exception {
		postEntities();

		String getCredDefRequest = Ledger.buildGetCredDefRequest(DID, credDefId).get();
		String getCredDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getCredDefRequest, response -> {
			JSONObject responseObject = new JSONObject(response);
			return ! responseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetCredDefResponse(getCredDefResponse).get();
	}
}
