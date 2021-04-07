package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.IOException;
import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;


public class ImportWalletTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testImportWalletWorks() throws Exception {
		String did = Did.createAndStoreMyDid(wallet, "{}").get().getDid();
		Did.setDidMetadata(wallet, did, METADATA).get();

		String didWithMetaBefore = Did.getDidWithMeta(wallet, did).get();

		Wallet.exportWallet(wallet, EXPORT_CONFIG_JSON).get();

		wallet.closeWallet().get();
		Wallet.deleteWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();

		Wallet.importWallet(WALLET_CONFIG, WALLET_CREDENTIALS, EXPORT_CONFIG_JSON).get();

		wallet = Wallet.openWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();

		String didWithMetaAfter = Did.getDidWithMeta(wallet, did).get();

		assertEquals(didWithMetaBefore, didWithMetaAfter);
	}

	@Test
	public void testImportWalletWorksForNotExists() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(IOException.class));

		Wallet.importWallet(WALLET_CONFIG, WALLET_CREDENTIALS, EXPORT_CONFIG_JSON).get();
	}
}