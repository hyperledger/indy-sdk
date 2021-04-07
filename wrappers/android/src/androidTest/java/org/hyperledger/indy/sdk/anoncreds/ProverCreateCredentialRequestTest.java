package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class ProverCreateCredentialRequestTest extends AnoncredsIntegrationTest {

	public ProverCreateCredentialRequestTest() throws JSONException {
	}

	@Test
	public void testProverCreateAndStoreCredentialReqWorks() throws Exception {
	}

	@Test
	public void testProverCreateAndStoreCredentialReqWorksForInvalidCredentialOffer() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		String credentialOffer = String.format("{\"issuer_did\":\"%s\"}", issuerDid);

		Anoncreds.proverCreateCredentialReq(wallet, proverDid, credentialOffer, issuer1gvtCredDef, masterSecretId).get();
	}

	@Test
	public void testProverCreateAndStoreCredentialReqWorksForInvalidMasterSecret() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Anoncreds.proverCreateCredentialReq(wallet, proverDid, issuer1GvtCredOffer, issuer1gvtCredDef, masterSecretId + "a").get();
	}
}
