package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONArray;
import org.json.JSONObject;
import org.junit.*;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class SchemaRequestsTest extends LedgerIntegrationTest {

	@Test
	public void testBuildSchemaRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "101")
								.put("data", new JSONObject()
										.put("name", "gvt")
										.put("version", "1.0")
										.put("attr_names", new JSONArray().put("name"))

								)
				);

		String schemaRequest = Ledger.buildSchemaRequest(DID, SCHEMA_DATA).get();
		assert (new JSONObject(schemaRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	public void testBuildGetSchemaRequestWorks() throws Exception {
		String id = String.format("%s:1:%s:%s", DID, GVT_SCHEMA_NAME, SCHEMA_VERSION);

		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "107")
								.put("dest", "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW")
								.put("data", new JSONObject()
										.put("name", "gvt")
										.put("version", "1.0")

								)
				);

		String getSchemaRequest = Ledger.buildGetSchemaRequest(DID, id).get();

		assert (new JSONObject(getSchemaRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	public void testSchemaRequestWorksWithoutSignature() throws Exception {
		String schemaRequest = Ledger.buildSchemaRequest(DID, SCHEMA_DATA).get();
		String response = Ledger.submitRequest(pool, schemaRequest).get();
		checkResponseType(response, "REQNACK");
	}

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testSchemaRequestsWorks() throws Exception {
		postEntities();

		String getSchemaRequest = Ledger.buildGetSchemaRequest(DID, String.valueOf(schemaId)).get();
		String getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, response -> {
			JSONObject getSchemaResponseObject = new JSONObject(response);
			return ! getSchemaResponseObject.getJSONObject("result").isNull("seqNo");
		});

		Ledger.parseGetSchemaResponse(getSchemaResponse).get();
	}

	@Test
	public void testBuildSchemaRequestWorksForMissedFields() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		String data = "{\"name\":\"name\",\"version\":\"1.0\"}";

		Ledger.buildSchemaRequest(DID, data).get();
	}
}
