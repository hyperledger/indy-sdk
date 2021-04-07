package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertEquals;


public class AppendRequestEndorserTest extends IndyIntegrationTest {
	@Test
	public void testAppendAuthorAgreementAcceptanceToRequestForTextVersion() throws Exception {
		String requestWithEndorser = Ledger.appendRequestEndorser(REQUEST.toString(), DID_TRUSTEE).get();
		String actualRequest = new JSONObject(requestWithEndorser).getString("endorser");
		assertEquals(actualRequest, DID_TRUSTEE);
	}

}