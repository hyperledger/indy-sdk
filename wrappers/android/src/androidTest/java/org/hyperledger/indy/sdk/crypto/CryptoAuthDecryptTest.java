package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.Arrays;
import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import org.hyperledger.indy.sdk.crypto.CryptoResults.AuthDecryptResult;


public class CryptoAuthDecryptTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testAuthDecryptWorks() throws Exception {
		String theirVk = Crypto.createKey(wallet, MY1_IDENTITY_KEY_JSON).get();

		String paramJson = new CryptoJSONParameters.CreateKeyJSONParameter(MY2_SEED, null).toJson();
		String myVk = Crypto.createKey(wallet, paramJson).get();

		byte[] encryptedMsg = Crypto.authCrypt(wallet, theirVk, myVk, MESSAGE).get();
		AuthDecryptResult decryptResult = Crypto.authDecrypt(wallet, myVk, encryptedMsg).get();
		assertEquals(theirVk, decryptResult.getVerkey());
		assertTrue(Arrays.equals(MESSAGE, decryptResult.getDecryptedMessage()));
	}

	@Test
	public void testAuthDecryptWorksForUnknownTheirVk() throws Exception {
		String theirVk = Crypto.createKey(wallet, MY1_IDENTITY_KEY_JSON).get();

		byte[] encryptedMsg = Crypto.authCrypt(wallet, theirVk, VERKEY, MESSAGE).get();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Crypto.authDecrypt(wallet, VERKEY, encryptedMsg).get();
	}
}