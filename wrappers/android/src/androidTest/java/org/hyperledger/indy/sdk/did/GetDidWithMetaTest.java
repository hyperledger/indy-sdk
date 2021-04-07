package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertEquals;


public class GetDidWithMetaTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testGetDidWithMetaWorks() throws Exception {
		DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
		String did = result.getDid();

		Did.setDidMetadata(wallet, did, METADATA).get();
		String didWithMetaJson = Did.getDidWithMeta(wallet, did).get();
		JSONObject didWithMeta = new JSONObject(didWithMetaJson);

		assertEquals(did, didWithMeta.getString("did"));
		assertEquals(METADATA, didWithMeta.getString("metadata"));
	}
}