package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Before;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;


public class GetKeyMetadataTest extends IndyIntegrationTestWithSingleWallet {

	private String key;

	@Before
	public void createKey() throws Exception {
		key = Crypto.createKey(wallet, "{}").get();
	}

	@Test
	public void testGetKeyMetadataWorks() throws Exception {
		Crypto.setKeyMetadata(wallet, key, METADATA).get();
		String receivedMetadata = Crypto.getKeyMetadata(wallet, key).get();
		assertEquals(METADATA, receivedMetadata);
	}

	@Test
	public void testGetKeyMetadataWorksForNoKey() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Crypto.getKeyMetadata(wallet, VERKEY).get();
	}
}