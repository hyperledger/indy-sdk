package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.Arrays;
import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;


public class CryptoAnonDecryptTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testAnonDecryptWorks() throws Exception {
		String paramJson = new CryptoJSONParameters.CreateKeyJSONParameter(MY2_SEED, null).toJson();
		String theirVk = Crypto.createKey(wallet, paramJson).get();

		byte[] encryptedMsg = Crypto.anonCrypt(theirVk, MESSAGE).get();
		byte[] decryptedMsg = Crypto.anonDecrypt(wallet, theirVk, encryptedMsg).get();
		assertTrue(Arrays.equals(MESSAGE, decryptedMsg));
	}

	@Test
	public void testParseMsgWorksForUnknownRecipientVk() throws Exception {
		byte[] encryptedMsg = Crypto.anonCrypt(VERKEY, MESSAGE).get();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Crypto.anonDecrypt(wallet, VERKEY, encryptedMsg).get();
	}
}