package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.IOException;
import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.junit.Test;

import java.io.File;
import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;


public class ExportWalletTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testExportWalletWorks() throws Exception {
		Did.createAndStoreMyDid(wallet, "{}").get();

		Wallet.exportWallet(wallet, EXPORT_CONFIG_JSON).get();

		assertTrue(new File(EXPORT_PATH).exists());
	}

	@Test
	public void testExportWalletWorksForExistsPath() throws Exception {
		assertTrue(new File(EXPORT_PATH).mkdirs());

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(IOException.class));

		Wallet.exportWallet(wallet, EXPORT_CONFIG_JSON).get();
	}
}
