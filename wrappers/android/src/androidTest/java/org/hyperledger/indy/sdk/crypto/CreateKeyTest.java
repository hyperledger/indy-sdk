package org.hyperledger.indy.sdk.crypto;

import org.bitcoinj.core.Base58;
import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;


public class CreateKeyTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testCreateKeyWorksForSeed() throws Exception {
		String senderVk = Crypto.createKey(wallet, MY1_IDENTITY_KEY_JSON).get();
		assertEquals(32, Base58.decode(senderVk).length);
	}

	@Test
	public void testCreateKeyWorksWithoutSeed() throws Exception {
		String senderVk = Crypto.createKey(wallet, "{}").get();
		assertEquals(32, Base58.decode(senderVk).length);
	}

	@Test
	public void testCreateKeyWorksForInvalidSeed() throws Exception {
		String paramJson = new CryptoJSONParameters.CreateKeyJSONParameter("invalidSeedLength", null).toJson();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Crypto.createKey(wallet, paramJson).get();
	}
}