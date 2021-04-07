package org.hyperledger.indy.sdk.demo;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.crypto.Crypto;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults.CreateAndStoreMyDidResult;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.pool.PoolJSONParameters;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONObject;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.Timeout;

import java.util.concurrent.TimeUnit;

import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

public class CryptoDemoTest extends IndyIntegrationTest {

	@Rule
	public Timeout globalTimeout = new Timeout(10, TimeUnit.MINUTES);

	@Test
	public void testCryptoDemo() throws Exception {
		//1. Create and Open Pool
		String poolName = PoolUtils.createPoolLedgerConfig();

		PoolJSONParameters.OpenPoolLedgerJSONParameter config2 = new PoolJSONParameters.OpenPoolLedgerJSONParameter(null, null);
		Pool pool = Pool.openPoolLedger(poolName, config2.toJson()).get();

		//2. Create and Open My Wallet
		String myWalletConfig = new JSONObject().put("id", "myWallet").toString();
		Wallet.createWallet(myWalletConfig, WALLET_CREDENTIALS).get();
		Wallet myWallet = Wallet.openWallet(myWalletConfig, WALLET_CREDENTIALS).get();

		//3. Create and Open Their Wallet
		String theirWalletConfig = new JSONObject().put("id", "theirWallet").toString();
		Wallet.createWallet(theirWalletConfig, WALLET_CREDENTIALS).get();
		Wallet theirWallet = Wallet.openWallet(theirWalletConfig, WALLET_CREDENTIALS).get();

		//4. Create My Did
		CreateAndStoreMyDidResult createMyDidResult = Did.createAndStoreMyDid(myWallet, "{}").get();
		assertNotNull(createMyDidResult);

		//5. Create Their Did
		CreateAndStoreMyDidResult createTheirDidResult = Did.createAndStoreMyDid(theirWallet, "{}").get();
		assertNotNull(createTheirDidResult);
		String theirDid = createTheirDidResult.getDid();
		String theirVerkey = createTheirDidResult.getVerkey();

		// 6. Store Their DID
		String identityJson = String.format("{\"did\":\"%s\", \"verkey\":\"%s\"}", theirDid, theirVerkey);
		Did.storeTheirDid(myWallet, identityJson).get();

		// 7. Their sign message
		String msg = "{\n" +
				"        \"reqId\":1495034346617224651,\n" +
				"        \"identifier\":\"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL\",\n" +
				"        \"operation\":{\n" +
				"            \"type\":\"1\",\n" +
				"            \"dest\":\"4efZu2SXufS556yss7W5k6Po37jt4371RM4whbPKBKdB\"\n" +
				"        }\n" +
				"    }";

		byte[] signature = Crypto.cryptoSign(theirWallet, theirVerkey, msg.getBytes()).get();

		// 8. I verify message
		Boolean valid = Crypto.cryptoVerify(theirVerkey, msg.getBytes(), signature).get();
		assertTrue(valid);

		// 9. Close and delete My Wallet
		myWallet.closeWallet().get();
		Wallet.deleteWallet(myWalletConfig, WALLET_CREDENTIALS).get();

		// 10. Close and delete Their Wallet
		theirWallet.closeWallet().get();
		Wallet.deleteWallet(theirWalletConfig, WALLET_CREDENTIALS).get();

		//11. Close Pool
		pool.closePoolLedger().get();
	}
}
