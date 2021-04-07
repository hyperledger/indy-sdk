package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.JsonObjectSimilar;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.Timeout;

import java.util.concurrent.TimeUnit;

import static org.junit.Assert.assertTrue;

public class CredDefRequestsTest extends LedgerIntegrationTest {

	@Rule
	public Timeout globalTimeout = new Timeout(10, TimeUnit.MINUTES);

	@Test
	public void testBuildCredDefRequestWorks() throws Exception {
		String data = "{\n" +
				"        \"ver\": \"1.0\",\n" +
				"        \"id\": \"NcYxiDXkpYi6ov5FcYDi1e:3:CL:1\",\n" +
				"        \"schemaId\": \"1\",\n" +
				"        \"type\": \"CL\",\n" +
				"        \"tag\": \"TAG_1\",\n" +
				"        \"value\": {\n" +
				"            \"primary\": {\n" +
				"                \"n\": \"1\",\n" +
				"                \"s\": \"2\",\n" +
				"                \"r\": {\"name\": \"1\",\"master_secret\": \"3\"},\n" +
				"                \"rctxt\": \"1\",\n" +
				"                \"z\": \"1\"\n" +
				"            }\n" +
				"        }\n" +
				"    }";

		String expectedResult = "{\n" +
				"            \"ref\": 1,\n" +
				"            \"data\": {\n" +
				"                \"primary\": {\"n\": \"1\", \"s\": \"2\", \"r\": {\"name\": \"1\",\"master_secret\": \"3\"}, \"rctxt\": \"1\", \"z\": \"1\"}\n" +
				"            },\n" +
				"            \"type\": \"102\",\n" +
				"            \"signature_type\": \"CL\",\n" +
				"            \"tag\": \"TAG_1\"\n" +
				"        }";

		String credDefRequest = Ledger.buildCredDefRequest(DID, data).get();

		assertTrue(JsonObjectSimilar.similar(new JSONObject(credDefRequest).getJSONObject("operation"), new JSONObject(expectedResult)));
	}

	@Test
	public void testBuildGetCredDefRequestWorks() throws Exception {
		int seqNo = 1;
		String id = DID + ":3:" + SIGNATURE_TYPE + ":" + seqNo + ":" + TAG;
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"108\"," +
				"\"ref\":%d," +
				"\"signature_type\":\"%s\"," +
				"\"origin\":\"%s\"," +
				"\"tag\":\"%s\"" +
				"}", DID, seqNo, SIGNATURE_TYPE, DID, TAG);

		String getCredDefRequest = Ledger.buildGetCredDefRequest(DID, id).get();

		assertTrue(getCredDefRequest.replace("\\", "").contains(expectedResult));
	}

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testCredDefRequestsWorks() throws Exception {
		postEntities();

		String getCredDefRequest = Ledger.buildGetCredDefRequest(DID, credDefId).get();
		String getCredDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getCredDefRequest, response -> {
			JSONObject responseObject = new JSONObject(response);
			return !responseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetCredDefResponse(getCredDefResponse).get();
	}
}
