package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.isA;

import java.util.concurrent.ExecutionException;


public class ReplaceKeysApplyTest extends IndyIntegrationTestWithSingleWallet {

	private String did;

	@Before
	public void before() throws Exception {
		did = Did.createAndStoreMyDid(this.wallet, "{}").get().getDid();
	}

	@Test
	public void testReplaceKeysApplyWorks() throws Exception {
		Did.replaceKeysStart(wallet, did, "{}").get();
		Did.replaceKeysApply(wallet, did).get();
	}

	@Test
	public void testReplaceKeysApplyWorksWithoutCallingReplaceStart() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Did.replaceKeysApply(wallet, did).get();
	}

	@Test
	public void testReplaceKeysApplyWorksForNotFoundDid() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Did.replaceKeysStart(wallet, did, "{}").get();
		Did.replaceKeysApply(wallet, DID).get();
	}
}
