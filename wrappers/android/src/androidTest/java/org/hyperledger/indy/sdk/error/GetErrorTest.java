package org.hyperledger.indy.sdk.error;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.InvalidParameterException;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.crypto.Crypto;
import org.hyperledger.indy.sdk.crypto.CryptoJSONParameters;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;

public class GetErrorTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testErrors() throws Exception {
		try {
			String paramJson = new CryptoJSONParameters.CreateKeyJSONParameter("invalidSeedLength", null).toJson();
			Crypto.createKey(this.wallet, paramJson).get();
		} catch (ExecutionException e) {
			InvalidStructureException ex = (InvalidStructureException) e.getCause();
			assertEquals(ex.getSdkErrorCode(), ErrorCode.CommonInvalidStructure.value());
			assertFalse(ex.getMessage().isEmpty());
		}

		try {
			byte[] message = {};
			Crypto.cryptoSign(this.wallet, VERKEY, message).get();
		} catch (ExecutionException e) {
			InvalidParameterException ex = (InvalidParameterException) e.getCause();
			assertEquals(ex.getSdkErrorCode(), ErrorCode.CommonInvalidParam5.value());
			assertFalse(ex.getMessage().isEmpty());
		}
	}
}
