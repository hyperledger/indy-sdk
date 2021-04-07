package org.hyperledger.indy.sdk.anoncreds;

import org.json.JSONException;
import org.junit.Test;

import static org.junit.Assert.assertEquals;


public class ToUnqualifiedTest extends AnoncredsIntegrationTest {

	public ToUnqualifiedTest() throws JSONException {
	}

	@Test
	public void testToUnqualifiedWorks() throws Exception {
		String qualified = "did:sov:NcYxiDXkpYi6ov5FcYDi1e";
		String unqualified = "NcYxiDXkpYi6ov5FcYDi1e";
		assertEquals(unqualified, Anoncreds.toUnqualified(qualified).get());
		assertEquals(unqualified, Anoncreds.toUnqualified(unqualified).get());

	}
}
