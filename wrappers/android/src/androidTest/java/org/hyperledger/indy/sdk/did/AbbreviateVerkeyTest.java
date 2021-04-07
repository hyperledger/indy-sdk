package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.did.DidResults.CreateAndStoreMyDidResult;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;

public class AbbreviateVerkeyTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testAbbrVerkeyWorksForAbbrVerkey() throws Exception {
		CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();

		String verkey = Did.AbbreviateVerkey(result.getDid(), result.getVerkey()).get();

		assertNotEquals(result.getVerkey(), verkey);
	}

	@Test
	public void testAbbrVerkeyWorksForNotAbbrVerkey() throws Exception {
		DidJSONParameters.CreateAndStoreMyDidJSONParameter theirDidJson =
				new DidJSONParameters.CreateAndStoreMyDidJSONParameter(DID_TRUSTEE, null, null, null);

		CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, theirDidJson.toJson()).get();

		String verkey = Did.AbbreviateVerkey(result.getDid(), result.getVerkey()).get();

		assertEquals(result.getVerkey(), verkey);
	}
}
