package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.json.JSONException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class IssuerCreateAndStoreCredentialDefinitionTest extends AnoncredsIntegrationTest {

	public IssuerCreateAndStoreCredentialDefinitionTest() throws JSONException {
	}

	@Test
	public void testIssuerCreateAndStoreCredentialDefWorks() throws Exception {
	}

	@Test
	public void testIssuerCreateAndStoreCredentialDefWorksForInvalidSchemaJson() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		String schema = "{" +
				"           \"name\":\"name\"," +
				"           \"version\":1.0," +
				"           \"attr_names\":[\"name\"]" +
				"        }";

		Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid, schema, "InvalidSchema", null, defaultCredentialDefinitionConfig).get();
	}
}
