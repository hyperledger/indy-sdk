package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.junit.Before;
import org.junit.Test;


public class SetKeyMetadataTest extends IndyIntegrationTestWithSingleWallet {

	private String key;

	@Before
	public void createKey() throws Exception {
		key = Crypto.createKey(wallet, "{}").get();
	}

	@Test
	public void testSetKeyMetadataWorks() throws Exception {
		Crypto.setKeyMetadata(wallet, key, METADATA).get();
	}

	@Test
	public void testSetKeyMetadataWorksForNoKey() throws Exception {
		Crypto.setKeyMetadata(wallet, VERKEY, METADATA).get();
	}
}