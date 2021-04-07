package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.JsonObjectSimilar;
import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.TransactionNotAllowedException;
import org.json.JSONArray;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;


public class GetRequestInfoTest extends IndyIntegrationTest {

	private String getAuthRuleResponseJson = "{\n" +
			"                \"result\":{\n" +
			"                    \"data\":[{\n" +
			"                        \"new_value\":\"0\",\n" +
			"                        \"constraint\":{\n" +
			"                            \"need_to_be_owner\":false,\n" +
			"                            \"sig_count\":1,\n" +
			"                            \"metadata\":{\n" +
			"                                \"fees\": \"1\"\n" +
			"                            },\n" +
			"                            \"role\":\"0\",\n" +
			"                            \"constraint_id\":\"ROLE\"\n" +
			"                        },\n" +
			"                        \"field\":\"role\",\n" +
			"                        \"auth_type\":\"1\",\n" +
			"                        \"auth_action\":\"ADD\"\n" +
			"                    }],\n" +
			"                    \"identifier\":\"LibindyDid111111111111\",\n" +
			"                    \"auth_action\":\"ADD\",\n" +
			"                    \"new_value\":\"0\",\n" +
			"                    \"reqId\":15616,\n" +
			"                    \"auth_type\":\"1\",\n" +
			"                    \"type\":\"121\",\n" +
			"                    \"field\":\"role\"\n" +
			"                },\n" +
			"                \"op\":\"REPLY\"\n" +
			"            }";
	private String requesterInfo = "{\n" +
			"                \"role\": \"0\",\n" +
			"                \"need_to_be_owner\":false,\n" +
			"                \"sig_count\":1\n" +
			"            }";
	private String fees = "{\n" +
			"                \"1\": 100\n" +
			"            }";

	@Test
	public void testGetRequestInfoTest() throws Exception {
		String requestInfoJson = Payments.getRequestInfo(getAuthRuleResponseJson, requesterInfo, fees).get();
		JSONObject requestInfo = new JSONObject(requestInfoJson);

		JSONObject expectedRequestInfo = new JSONObject()
				.put("price", 100)
				.put("requirements",
						new JSONArray()
								.put(new JSONObject()
										.put("role", "0")
										.put("need_to_be_owner", false)
										.put("sig_count", 1))
				);

		assertTrue(JsonObjectSimilar.similar(expectedRequestInfo, requestInfo));
	}

	@Test
	public void testGetRequestInfoTestForRequesterNotMatchToConstraint() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(TransactionNotAllowedException.class));

		String requesterInfo = "{\n" +
				"                \"role\": \"101\",\n" +
				"                \"need_to_be_owner\":false,\n" +
				"                \"sig_count\":1\n" +
				"            }";

		Payments.getRequestInfo(getAuthRuleResponseJson, requesterInfo, fees).get();
	}
}