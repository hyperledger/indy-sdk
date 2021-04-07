package org.hyperledger.indy.sdk.did;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.junit.Before;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class SetDidMetadataTest extends IndyIntegrationTestWithSingleWallet {

	private String did;

	@Before
	public void createDid() throws Exception {
		DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
		did = result.getDid();
	}

	@Test
	public void testSetDidMetadataWorks() throws Exception {
		Did.setDidMetadata(wallet, did, METADATA).get();
	}

	@Test
	public void testSetDidMetadataWorksForInvalidDid() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Did.setDidMetadata(wallet, INVALID_DID, METADATA).get();
	}
}