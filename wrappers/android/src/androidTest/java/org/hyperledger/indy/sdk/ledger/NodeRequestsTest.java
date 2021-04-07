package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidJSONParameters;
import org.hyperledger.indy.sdk.did.DidResults;
import org.junit.*;

import static org.junit.Assert.assertTrue;

public class NodeRequestsTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	private String dest = "A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y";
	private String data = "{\"node_ip\":\"10.0.0.100\"," +
			"\"node_port\":910," +
			"\"client_ip\":\"10.0.0.100\"," +
			"\"client_port\":911," +
			"\"alias\":\"some\"," +
			"\"services\":[\"VALIDATOR\"]," +
			"\"blskey\":\"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba\"," +
			"\"blskey_pop\":\"RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1\"" +
			"}";

	private DidJSONParameters.CreateAndStoreMyDidJSONParameter stewardDidJson =
			new DidJSONParameters.CreateAndStoreMyDidJSONParameter(null, "000000000000000000000000Steward1", null, null);

	@Test
	public void testBuildNodeRequestWorks() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{" +
				"\"type\":\"0\"," +
				"\"dest\":\"%s\"," +
				"\"data\":%s" +
				"}", DID, dest, data);

		String nodeRequest = Ledger.buildNodeRequest(DID, dest, data).get();

		assertTrue(nodeRequest.replace("\\", "").contains(expectedResult));
	}

	@Test
	public void testSendNodeRequestWorksForNewSteward() throws Exception {
		DidResults.CreateAndStoreMyDidResult didResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String trusteeDid = didResult.getDid();

		DidResults.CreateAndStoreMyDidResult myDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String myDid = myDidResult.getDid();
		String myVerkey = myDidResult.getVerkey();

		String nymRequest = Ledger.buildNymRequest(trusteeDid, myDid, myVerkey, null, "STEWARD").get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDid, nymRequest).get();

		String nodeRequest = Ledger.buildNodeRequest(myDid, dest, data).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, nodeRequest).get();
	}
}
