package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.junit.Test;

import static org.junit.Assert.assertEquals;


public class GenerateWalletKeyTest extends IndyIntegrationTest {

	@Test
	public void testGenerateWalletKeyWorks() throws Exception {
		String key = Wallet.generateWalletKey(null).get();

		String credentials = "{ \"key\":\"" + key + "\", \"key_derivation_method\":\"RAW\"}";
		Wallet.createWallet(WALLET_CONFIG, credentials).get();
	}

	@Test
	public void testGenerateWalletKeyWorksForSeed() throws Exception {
		String config = "{ \"seed\":\"" + MY1_SEED + "\"}";
		String key = Wallet.generateWalletKey(config).get();
		assertEquals("CwMHrEQJnwvuE8q9zbR49jyYtVxVBHNTjCPEPk1aV3cP", key);

		String credentials = "{ \"key\":\"" + key + "\", \"key_derivation_method\":\"RAW\"}";
		Wallet.createWallet(WALLET_CONFIG, credentials).get();
	}
}