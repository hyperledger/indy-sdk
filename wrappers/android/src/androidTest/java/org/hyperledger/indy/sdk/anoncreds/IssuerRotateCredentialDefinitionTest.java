package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.JsonObjectSimilar;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertFalse;

public class IssuerRotateCredentialDefinitionTest extends AnoncredsIntegrationTest {

	public IssuerRotateCredentialDefinitionTest() throws JSONException {
	}

	@Test
	public void testIssuerRotateCredentialDefinitionWorks() throws Exception {
		String walletConfig =
				new JSONObject()
						.put("id", "testIssuerRotateCredentialDefinitionWorks")
						.toString();

		Wallet.createWallet(walletConfig, CREDENTIALS).get();
		Wallet localWallet = Wallet.openWallet(walletConfig, CREDENTIALS).get();

		AnoncredsResults.IssuerCreateAndStoreCredentialDefResult credDefRes =
				Anoncreds.issuerCreateAndStoreCredentialDef(localWallet, issuerDid, gvtSchema, tag, null, defaultCredentialDefinitionConfig).get();

		String credDefId = credDefRes.getCredDefId();
		String credDef = credDefRes.getCredDefJson();

		String tempCredDef = Anoncreds.issuerRotateCredentialDefStart(localWallet, credDefId, null).get();

		assertFalse(JsonObjectSimilar.similar(new JSONObject(credDef), new JSONObject(tempCredDef)));

		Anoncreds.issuerRotateCredentialDefApply(localWallet, credDefId).get();

		localWallet.closeWallet();
		Wallet.deleteWallet(walletConfig, CREDENTIALS).get();
	}
}
