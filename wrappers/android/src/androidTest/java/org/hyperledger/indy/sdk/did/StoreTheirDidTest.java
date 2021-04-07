package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.isA;

import java.util.concurrent.ExecutionException;

public class StoreTheirDidTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testStoreTheirDidWorks() throws Exception {
		Did.storeTheirDid(this.wallet, String.format("{\"did\":\"%s\"}", DID)).get();
	}

	@Test
	public void testCreateMyDidWorksForInvalidIdentityJson() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Did.storeTheirDid(this.wallet, "{\"field\":\"value\"}").get();
	}
}
