package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.*;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;

public class SchemaRequestsTest extends LedgerIntegrationTest {

	@Test
	public void testBuildSchemaRequestWorks() throws Exception {
		String expectedResult = "\"operation\": {\n" +
				"            \"type\": \"101\",\n" +
				"            \"data\": {\"name\": \"gvt\", \"version\": \"1.0\", \"attr_names\": [\"name\"]}\n" +
				"        }";

		String schemaRequest = Ledger.buildSchemaRequest(DID, SCHEMA_DATA).get();

		assertTrue(schemaRequest.replaceAll("\\s+", "").contains(expectedResult.replaceAll("\\s+", "")));
	}

	@Test
	public void testBuildGetSchemaRequestWorks() throws Exception {
		String id = String.format("%s:1:%s:%s", DID, GVT_SCHEMA_NAME, SCHEMA_VERSION);

		String expectedResult = "\"operation\":{\"type\":\"107\",\"dest\":\"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW\",\"data\":{\"name\":\"gvt\",\"version\":\"1.0\"}}";

		String getSchemaRequest = Ledger.buildGetSchemaRequest(DID, id).get();

		assertTrue(getSchemaRequest.contains(expectedResult));
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
