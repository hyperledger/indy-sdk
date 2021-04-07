package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.junit.Test;

import java.util.Calendar;

import static org.junit.Assert.assertTrue;

public class PoolUpgradeRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	@Test
	public void testBuildPoolUpgradeRequestWorksForStartAction() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{\"type\":\"109\"," +
				"\"name\":\"upgrade-java\"," +
				"\"version\":\"2.0.0\"," +
				"\"action\":\"start\"," +
				"\"sha256\":\"f284b\"," +
				"\"schedule\":{}," +
				"\"reinstall\":false," +
				"\"force\":false}", DID);

		String request = Ledger.buildPoolUpgradeRequest(DID, "upgrade-java", "2.0.0", "start", "f284b", - 1,
				"{}", null, false, false, null).get();

		assertTrue(request.contains(expectedResult));
	}

	@Test
	public void testBuildPoolUpgradeRequestWorksForPackage() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{\"type\":\"109\"," +
				"\"name\":\"upgrade-java\"," +
				"\"version\":\"2.0.0\"," +
				"\"action\":\"start\"," +
				"\"sha256\":\"f284b\"," +
				"\"schedule\":{}," +
				"\"reinstall\":false," +
				"\"force\":false," +
				"\"package\":\"some_package\"}", DID);

		String request = Ledger.buildPoolUpgradeRequest(DID, "upgrade-java", "2.0.0", "start", "f284b", - 1,
				"{}", null, false, false, "some_package").get();

		assertTrue(request.contains(expectedResult));
	}

	@Test
	public void testBuildPoolUpgradeRequestWorksForCancelAction() throws Exception {
		String expectedResult = String.format("\"identifier\":\"%s\"," +
				"\"operation\":{\"type\":\"109\"," +
				"\"name\":\"upgrade-java\"," +
				"\"version\":\"2.0.0\"," +
				"\"action\":\"cancel\"," +
				"\"sha256\":\"f284b\"," +
				"\"reinstall\":false," +
				"\"force\":false}", DID);

		String request = Ledger.buildPoolUpgradeRequest(DID, "upgrade-java", "2.0.0", "cancel", "f284b", - 1,
				null, null, false, false, null).get();

		assertTrue(request.contains(expectedResult));
	}

	@Test
	public void testPoolUpgradeRequestWorks() throws Exception {
		int nextYear = Calendar.getInstance().get(Calendar.YEAR) + 1;

		DidResults.CreateAndStoreMyDidResult didResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String did = didResult.getDid();

		//start
		String schedule = String.format("{\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\":\"%s-01-25T12:49:05.258870+00:00\",\n" +
						"                   \"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\":\"%s-01-25T13:49:05.258870+00:00\",\n" +
						"                   \"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\":\"%s-01-25T14:49:05.258870+00:00\",\n" +
						"                   \"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\":\"%s-01-25T15:49:05.258870+00:00\"}",
				nextYear, nextYear, nextYear, nextYear);
		String request = Ledger.buildPoolUpgradeRequest(did, "upgrade-java", "2.0.0", "start",
				"f284bdc3c1c9e24a494e285cb387c69510f28de51c15bb93179d9c7f28705398", - 1, schedule, null, false, false, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, did, request).get();

		//cancel
		request = Ledger.buildPoolUpgradeRequest(did, "upgrade-java", "2.0.0", "cancel",
				"ac3eb2cc3ac9e24a494e285cb387c69510f28de51c15bb93179d9c7f28705398", - 1, null, null, false, false, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, did, request).get();
	}
}
