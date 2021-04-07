package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONObject;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import org.hyperledger.indy.sdk.JsonObjectSimilar;

public class GetRecordTest extends NonSecretsIntegrationTest {

	@Test
	public void testGetRecordWorksForDefaultOptions() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();

		String recordJson = WalletRecord.get(wallet, type, id, optionsEmpty).get();

		JSONObject actual = new JSONObject(recordJson);

		JSONObject expected = new JSONObject()
				.put("id", id)
				.putOpt("type", JSONObject.NULL)
				.put("value", value)
				.put("tags", JSONObject.NULL);

		assertTrue(JsonObjectSimilar.similar(expected, actual));

	}

	@Test
	public void testGetRecordWorksForFullData() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();

		String optionsJson = "{" +
				"   \"retrieveType\": true,\n" +
				"   \"retrieveValue\": true,\n" +
				"   \"retrieveTags\": true" +
				"}";
		String recordJson = WalletRecord.get(wallet, type, id, optionsJson).get();

		JSONObject record = new JSONObject(recordJson);
		assertEquals(id, record.getString("id"));
		assertEquals(type, record.getString("type"));
		assertEquals(value, record.getString("value"));
		assertTrue(JsonObjectSimilar.similar(new JSONObject(tags), record.getJSONObject("tags")));
	}

	@Test
	public void testGetRecordWorksForNotFoundRecord() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		WalletRecord.get(wallet, type, id, optionsEmpty).get();
	}
}
