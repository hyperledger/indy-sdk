package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidJSONParameters;
import org.hyperledger.indy.sdk.did.DidResults;
import org.json.JSONArray;
import org.json.JSONObject;
import org.junit.*;

public class NodeRequestsTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	private String dest = "A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y";

	JSONObject data = new JSONObject()
			.put("node_ip", "10.0.0.100")
			.put("client_ip", "10.0.0.100")
			.put("node_port", 910)
			.put("client_port", 911)
			.put("alias", "some")
			.put("blskey", "4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba")
			.put("blskey_pop", "RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1")
			.put("services", new JSONArray().put("VALIDATOR"));

	@Test
	public void testBuildNodeRequestWorks() throws Exception {
		JSONObject expectedResult = new JSONObject()
				.put("identifier", DID)
				.put("operation",
						new JSONObject()
								.put("type", "0")
								.put("dest", dest)
								.put("data", data)
				);

		String nodeRequest = Ledger.buildNodeRequest(DID, dest, data.toString()).get();
		assert (new JSONObject(nodeRequest).toMap().entrySet()
				.containsAll(
						expectedResult.toMap().entrySet()));
	}

	@Test
	@Ignore
	public void testSendNodeRequestWorksForNewSteward() throws Exception {
		DidResults.CreateAndStoreMyDidResult didResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String trusteeDid = didResult.getDid();

		DidResults.CreateAndStoreMyDidResult myDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String myDid = myDidResult.getDid();
		String myVerkey = myDidResult.getVerkey();

		String nymRequest = Ledger.buildNymRequest(trusteeDid, myDid, myVerkey, null, "STEWARD").get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDid, nymRequest).get();

		String nodeRequest = Ledger.buildNodeRequest(myDid, dest, data.toString()).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, nodeRequest).get();
	}
}
