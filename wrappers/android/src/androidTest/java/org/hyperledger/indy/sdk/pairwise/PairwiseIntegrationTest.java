package org.hyperledger.indy.sdk.pairwise;


import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.junit.Before;

public class PairwiseIntegrationTest extends IndyIntegrationTestWithSingleWallet {

	protected String myDid;
	String theirDid;
	static final String metadata = "some metadata";
	static final String PAIR_TEMPLATE = "{\"my_did\":\"%s\",\"their_did\":\"%s\"}";


	@Before
	public void createDids() throws Exception {
		DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
		myDid = result.getDid();

		result = Did.createAndStoreMyDid(wallet, "{}").get();
		theirDid = result.getDid();
		String theirVerkey = result.getVerkey();

		Did.storeTheirDid(wallet, String.format(IDENTITY_JSON_TEMPLATE, theirDid, theirVerkey)).get();
	}
}
