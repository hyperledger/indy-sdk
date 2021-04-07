package org.hyperledger.indy.sdk.anoncreds;

import org.json.JSONException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class ProverCreateMasterSecretTest extends AnoncredsIntegrationTest {

	public ProverCreateMasterSecretTest() throws JSONException {
	}

	@Test
	public void testProverCreateMasterSecretWorks() throws Exception {
	}

	@Test
	public void testProverCreateMasterSecretWorksForDuplicate() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(DuplicateMasterSecretNameException.class));

		Anoncreds.proverCreateMasterSecret(wallet, masterSecretId).get();
	}
}
