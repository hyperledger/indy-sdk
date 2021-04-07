package org.hyperledger.indy.sdk.anoncreds;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class ProverSearchCredentialsForProofRequestTest extends AnoncredsIntegrationTest {

	public ProverSearchCredentialsForProofRequestTest() throws JSONException {
	}

	@Test
	public void testProverSearchCredentialsForProofRequestWorks() throws Exception {

		String proofRequest = "{" +
				"              \"nonce\":\"123432421212\"," +
				"              \"name\":\"proof_req_1\"," +
				"              \"version\":\"0.1\"," +
				"              \"requested_attributes\":{" +
				"                   \"attr1_referent\":{\"name\":\"name\"}" +
				"               }," +
				"              \"requested_predicates\":{}" +
				"          }";

		CredentialsSearchForProofReq credentialsSearch = CredentialsSearchForProofReq.open(wallet, new JSONObject(proofRequest).toString(), null).get();

		JSONArray credentialsForAttribute1 = new JSONArray(credentialsSearch.fetchNextCredentials("attr1_referent", 100).get());

		assertEquals(2, credentialsForAttribute1.length());

		credentialsSearch.close();
	}

	@Test
	public void testProverSearchCredentialsForProofRequestWorksForNotFound() throws Exception {

		String proofRequest = "{" +
				"              \"nonce\":\"123432421212\"," +
				"              \"name\":\"proof_req_1\"," +
				"              \"version\":\"0.1\"," +
				"              \"requested_attributes\":{" +
				"                   \"attr1_referent\":{\"name\":\"not_found_attr\"}" +
				"               }," +
				"              \"requested_predicates\":{}" +
				"          }";

		CredentialsSearchForProofReq credentialsSearch = CredentialsSearchForProofReq.open(wallet, new JSONObject(proofRequest).toString(), null).get();

		JSONArray credentialsForAttribute1 = new JSONArray(credentialsSearch.fetchNextCredentials("attr1_referent", 100).get());

		assertEquals(0, credentialsForAttribute1.length());

		credentialsSearch.close();
	}

	@Test
	public void testProverSearchCredentialsForProofRequestWorksForRevealedAttributeAndExtraQuery() throws Exception {

		String proofRequest = "{" +
				"              \"nonce\":\"123432421212\"," +
				"              \"name\":\"proof_req_1\"," +
				"              \"version\":\"0.1\"," +
				"              \"requested_attributes\":{" +
				"                   \"attr1_referent\":{\"name\":\"name\"}" +
				"               }," +
				"              \"requested_predicates\":{}" +
				"          }";

		String extraQuery = "{\"attr1_referent\": { \"attr::name::value\": \"Alex\"}}";

		CredentialsSearchForProofReq credentialsSearch = CredentialsSearchForProofReq.open(wallet, new JSONObject(proofRequest).toString(),
				new JSONObject(extraQuery).toString()).get();

		JSONArray credentialsForAttribute1 = new JSONArray(credentialsSearch.fetchNextCredentials("attr1_referent", 100).get());

		assertEquals(1, credentialsForAttribute1.length());

		credentialsSearch.close();
	}
}
