package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.Arrays;
import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;

public class CryptoSignTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testCryptoSignWorks() throws Exception {

		String paramJson = new CryptoJSONParameters.CreateKeyJSONParameter(TRUSTEE_SEED, null).toJson();
		String key = Crypto.createKey(wallet, paramJson).get();

		byte[] signature = Crypto.cryptoSign(this.wallet, key, MESSAGE).get();

		assertTrue(Arrays.equals(SIGNATURE, signature));
	}

	@Test
	public void testCryptoSignWorksForUnknowSigner() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Crypto.cryptoSign(this.wallet, VERKEY_TRUSTEE, MESSAGE).get();
	}
}
