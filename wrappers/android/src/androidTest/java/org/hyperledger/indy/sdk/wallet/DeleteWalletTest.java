package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.IndyIntegrationTest;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertNotNull;

import org.hyperledger.indy.sdk.InvalidStateException;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;


public class DeleteWalletTest extends IndyIntegrationTest {

	@Test
	public void testDeleteWalletWorks() throws Exception {
		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
		Wallet.deleteWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
		Wallet.deleteWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testDeleteWalletWorksForOpened() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStateException.class));

		String config = new JSONObject()
				.put("id", "deleteWalletWorksForOpened")
				.toString();

		Wallet.createWallet(config, WALLET_CREDENTIALS).get();
		Wallet.openWallet(config, WALLET_CREDENTIALS).get();
		Wallet.deleteWallet(config, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testDeleteWalletWorksForNotCreated() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletNotFoundException.class));

		Wallet.deleteWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}
}
