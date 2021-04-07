package org.hyperledger.indy.sdk.pairwise;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class CreatePairwiseTest extends PairwiseIntegrationTest {

	@Test
	public void testCreatePairwiseWorks() throws Exception {
		Pairwise.createPairwise(wallet, theirDid, myDid, metadata).get();
	}

	@Test
	public void testCreatePairwiseWorksForNotFoundMyDid() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Pairwise.createPairwise(wallet, theirDid, DID, null).get();
	}
}
