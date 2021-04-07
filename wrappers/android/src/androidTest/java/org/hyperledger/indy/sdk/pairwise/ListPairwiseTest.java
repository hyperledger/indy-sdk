package org.hyperledger.indy.sdk.pairwise;

import org.json.JSONArray;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class ListPairwiseTest extends PairwiseIntegrationTest {

	@Test
	public void testListPairwiseWorks() throws Exception {
		Pairwise.createPairwise(wallet, theirDid, myDid, null).get();

		String listPairwise = Pairwise.listPairwise(wallet).get();
		JSONArray listPairwiseArray = new JSONArray(listPairwise);

		assertEquals(1, listPairwiseArray.length());
		assertEquals(listPairwiseArray.getString(0), String.format(PAIR_TEMPLATE, myDid, theirDid));
	}

	@Test
	public void testListPairwiseWorksForEmptyResult() throws Exception {
		String listPairwise = Pairwise.listPairwise(wallet).get();
		JSONArray listPairwiseArray = new JSONArray(listPairwise);
		assertEquals(0, listPairwiseArray.length());
	}
}
