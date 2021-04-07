package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults.CreateAndStoreMyDidResult;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;

public class MultiSignRequestTest extends IndyIntegrationTestWithSingleWallet {

	@Test
	public void testMultiSignWorks() throws Exception {
		CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String did1 = result.getDid();

		result = Did.createAndStoreMyDid(wallet, MY1_IDENTITY_JSON).get();
		String did2 = result.getDid();

		String msg = String.format("{\n" +
				"                \"reqId\":1496822211362017764,\n" +
				"                \"identifier\":\"%s\",\n" +
				"                \"operation\":{\n" +
				"                    \"type\":\"1\",\n" +
				"                    \"dest\":\"VsKV7grR1BUE29mG2Fm2kX\",\n" +
				"                    \"verkey\":\"GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa\"\n" +
				"                }\n" +
				"            }", did1);

		String signedMessageJson = Ledger.multiSignRequest(wallet, did1, msg).get();
		signedMessageJson = Ledger.multiSignRequest(wallet, did2, signedMessageJson).get();

		JSONObject signedMessage = new JSONObject(signedMessageJson);

		assertEquals("3YnLxoUd4utFLzeXUkeGefAqAdHUD7rBprpSx2CJeH7gRYnyjkgJi7tCnFgUiMo62k6M2AyUDtJrkUSgHfcq3vua",
				signedMessage.getJSONObject("signatures").getString(did1));
		assertEquals("4EyvSFPoeQCJLziGVqjuMxrbuoWjAWUGPd6LdxeZuG9w3Bcbt7cSvhjrv8SX5e8mGf8jrf3K6xd9kEhXsQLqUg45",
				signedMessage.getJSONObject("signatures").getString(did2));
	}

	@Test
	public void testMultiSignWorksForUnknownDid() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		String msg = "{\"reqId\":1496822211362017764}";
		Ledger.multiSignRequest(wallet, DID, msg).get();
	}

	@Test
	public void testMultiSignWorksForInvalidMessageFormat() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
		String did = result.getDid();

		String msg = "\"reqId\":1496822211362017764";
		Ledger.multiSignRequest(wallet, did, msg).get();
	}

}
