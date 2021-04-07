package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class SetEndpointForDidTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	@Test
	public void testSetEndpointForDidWorks() throws Exception {
		Did.setEndpointForDid(wallet, DID, ENDPOINT, VERKEY).get();
	}

	@Test
	public void testSetEndpointForDidWorksForInvalidTransportKey() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Did.setEndpointForDid(wallet, DID, ENDPOINT, INVALID_VERKEY).get();
	}
}