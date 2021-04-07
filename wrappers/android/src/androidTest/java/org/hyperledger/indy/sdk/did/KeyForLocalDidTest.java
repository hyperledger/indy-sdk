package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;


public class KeyForLocalDidTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testKeyForLocalDidWorksForMyDid() throws Exception {
		DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
		String did = result.getDid();
		String key = result.getVerkey();

		String receivedKey = Did.keyForLocalDid(wallet, did).get();
		assertEquals(key, receivedKey);
	}

	@Test
	public void testKeyForDidWorksForNoKey() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Did.keyForLocalDid(wallet, DID_MY2).get();
	}
}