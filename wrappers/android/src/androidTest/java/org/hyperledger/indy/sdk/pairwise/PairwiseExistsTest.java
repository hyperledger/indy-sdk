package org.hyperledger.indy.sdk.pairwise;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class PairwiseExistsTest extends PairwiseIntegrationTest {

	@Test
	public void testPairwiseExistsWorks() throws Exception {
		Pairwise.createPairwise(wallet, theirDid, myDid, null).get();
		assertTrue(Pairwise.isPairwiseExists(wallet, theirDid).get());
	}

	@Test
	public void testPairwiseExistsWorksForNotCreated() throws Exception {
		assertFalse(Pairwise.isPairwiseExists(wallet, theirDid).get());
	}
}
