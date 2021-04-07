package org.hyperledger.indy.sdk;

import org.hyperledger.indy.sdk.wallet.Wallet;
import org.junit.After;
import org.junit.Before;


public class IndyIntegrationTestWithSingleWallet extends IndyIntegrationTest {

	public Wallet wallet;

	@Before
	public void createWallet() throws Exception {
		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
		this.wallet = Wallet.openWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}

	@After
	public void deleteWallet() throws Exception {
		wallet.closeWallet().get();
		Wallet.deleteWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}
}
