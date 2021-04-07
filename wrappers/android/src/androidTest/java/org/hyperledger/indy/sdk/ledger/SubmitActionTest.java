package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.json.JSONObject;
import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class SubmitActionTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	private String did;

	@Before
	public void createDid() throws Exception {
		DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
		did = result.getDid();
	}

	@Test
	public void testSubmitActionWorksForGetValidatorInfo() throws Exception {
		String getValidatorInfoRequest = Ledger.buildGetValidatorInfoRequest(did).get();
		getValidatorInfoRequest = Ledger.signRequest(wallet, did, getValidatorInfoRequest).get();
		Ledger.submitAction(pool, getValidatorInfoRequest, null, - 1).get();
	}

	@Test
	public void testSubmitActionWorksForPoolRestart() throws Exception {
		String poolRestartRequest = Ledger.buildPoolRestartRequest(did, "cancel", null).get();
		poolRestartRequest = Ledger.signRequest(wallet, did, poolRestartRequest).get();
		Ledger.submitAction(pool, poolRestartRequest, null, - 1).get();
	}

	@Test
	public void testSubmitActionWorksForNodes() throws Exception {
		String nodes = "[\"Node1\",\"Node2\"]";
		String getValidatorInfoRequest = Ledger.buildGetValidatorInfoRequest(did).get();
		getValidatorInfoRequest = Ledger.signRequest(wallet, did, getValidatorInfoRequest).get();
		String responseJson = Ledger.submitAction(pool, getValidatorInfoRequest, nodes, - 1).get();
		JSONObject response = new JSONObject(responseJson);
		assertEquals(2, response.length());
		assertTrue(response.has("Node1"));
		assertTrue(response.has("Node2"));
	}
}
