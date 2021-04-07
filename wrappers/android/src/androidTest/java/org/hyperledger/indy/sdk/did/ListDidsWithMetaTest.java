package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.json.JSONArray;
import org.junit.Test;

import static org.junit.Assert.assertEquals;


public class ListDidsWithMetaTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testListDidsWithMetaWorks() throws Exception {
		DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
		String did = result.getDid();
		Did.setDidMetadata(wallet, did, METADATA).get();

		String listDidsWithMetaJson = Did.getListMyDidsWithMeta(wallet).get();
		JSONArray listDidsWithMeta = new JSONArray(listDidsWithMetaJson);

		assertEquals(1, listDidsWithMeta.length());

		assertEquals(did, listDidsWithMeta.getJSONObject(0).getString("did"));
		assertEquals(METADATA, listDidsWithMeta.getJSONObject(0).getString("metadata"));
	}
}