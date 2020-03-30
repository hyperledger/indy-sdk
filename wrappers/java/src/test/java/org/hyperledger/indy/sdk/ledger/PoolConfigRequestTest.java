package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.json.JSONObject;
import org.junit.Test;

public class PoolConfigRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	@Test
	public void testBuildPoolConfigRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "111")
								.put("writes", false)
								.put("force", false)
				);


		String request = Ledger.buildPoolConfigRequest(DID, false, false).get();
		assert (new JSONObject(request).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	public void testPoolConfigRequestWorks() throws Exception {
		DidResults.CreateAndStoreMyDidResult didResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String did = didResult.getDid();

		String request = Ledger.buildPoolConfigRequest(did, false, false).get();
		Ledger.signAndSubmitRequest(pool, wallet, did, request).get();

		request = Ledger.buildPoolConfigRequest(did, true, false).get();
		Ledger.signAndSubmitRequest(pool, wallet, did, request).get();
	}
}
