package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.json.JSONException;
import org.junit.*;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class IssuerCreateCredentialTest extends AnoncredsIntegrationTest {

	public IssuerCreateCredentialTest() throws JSONException {
	}

	@Test
	public void testIssuerCreateCredentialWorks() throws Exception {}

	@Test
	public void testIssuerCreateCredentialWorksForInvalidCredentialValues() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		String credValues = "{" +
				"        \"sex\":\"male\",\n" +
				"        \"age\":\"28\"" +
				"       }";

		Anoncreds.issuerCreateCredential(wallet, issuer1GvtCredOffer, issuer1GvtCredReq, credValues, null, - 1).get();
	}
}
