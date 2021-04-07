package org.hyperledger.indy.sdk;

import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.ledger.Ledger;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.After;
import org.junit.Before;

import static org.junit.Assert.assertTrue;


public class IndyIntegrationTestWithPoolAndSingleWallet extends IndyIntegrationTest {

	public Pool pool;
	public Wallet wallet;

	@Before
	public void createPoolAndWallet() throws Exception {
		String poolName = PoolUtils.createPoolLedgerConfig();
		pool = Pool.openPoolLedger(poolName, null).get();

		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
		this.wallet = Wallet.openWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}

	@After
	public void deletePoolAndWallet() throws Exception {
		if (pool != null) {
			pool.closePoolLedger().get();
		}

		if (wallet != null) {
			wallet.closeWallet().get();
			Wallet.deleteWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
		}
	}

	protected void checkResponseType(String response, String expectedType) throws JSONException {
		assertTrue(compareResponseType(response, expectedType));
	}

	protected boolean compareResponseType(String response, String expectedType) throws JSONException {
		JSONObject res = new JSONObject(response);
		return expectedType.equals(res.getString("op"));
	}

	protected String createStoreAndPublishDidFromTrustee() throws Exception {
		DidResults.CreateAndStoreMyDidResult trusteeDidResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String trusteeDid = trusteeDidResult.getDid();

		DidResults.CreateAndStoreMyDidResult myDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String myDid = myDidResult.getDid();
		String myVerkey = myDidResult.getVerkey();

		String nymRequest = Ledger.buildNymRequest(trusteeDid, myDid, myVerkey, null, "TRUSTEE").get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDid, nymRequest).get();

		return myDid;
	}
}
