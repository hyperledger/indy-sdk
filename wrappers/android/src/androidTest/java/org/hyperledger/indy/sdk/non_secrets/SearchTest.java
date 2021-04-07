package org.hyperledger.indy.sdk.non_secrets;

import org.json.JSONArray;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import org.hyperledger.indy.sdk.JsonObjectSimilar;

public class SearchTest extends NonSecretsIntegrationTest {

	@Test
	public void testWalletSearchWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();

		WalletSearch search = WalletSearch.open(wallet, type, queryEmpty, optionsEmpty).get();

		String searchRecordsJson = search.fetchNextRecords(wallet, 1).get();

		JSONObject searchRecords = new JSONObject(searchRecordsJson);

		JSONArray records = searchRecords.getJSONArray("records");

		assertEquals(1, records.length());

		JSONObject expected = new JSONObject()
				.put("id", id)
				.putOpt("type", JSONObject.NULL)
				.put("value", value)
				.put("tags", JSONObject.NULL);

		assertTrue(JsonObjectSimilar.similar(expected, records.get(0)));

		search.close();
	}

	@Test
	public void testWalletSearchWorksForOptions() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();

		String options = "{" +
				"   \"retrieveRecords\": true,\n" +
				"   \"retrieveTotalCount\": false,\n" +
				"   \"retrieveType\": false,\n" +
				"   \"retrieveValue\": false,\n" +
				"   \"retrieveTags\": false" +
				"}";
		WalletSearch search = WalletSearch.open(wallet, type, queryEmpty, options).get();

		String searchRecordsJson = search.fetchNextRecords(wallet, 1).get();

		JSONObject searchRecords = new JSONObject(searchRecordsJson);

		JSONArray records = searchRecords.getJSONArray("records");

		assertEquals(1, records.length());

		JSONObject expected = new JSONObject()
				.put("id", id)
				.putOpt("type", JSONObject.NULL)
				.put("value", JSONObject.NULL)
				.putOpt("tags", JSONObject.NULL);

		assertTrue(JsonObjectSimilar.similar(expected, records.get(0)));

		search.close();
	}

	@Test
	public void testWalletSearchWorksForQuery() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();
		WalletRecord.add(wallet, type, id2, value2, tags2).get();
		WalletRecord.add(wallet, type, id3, value3, tags3).get();

		String query = "{\"tagName1\":\"str2\"}";
		WalletSearch search = WalletSearch.open(wallet, type, query, optionsEmpty).get();

		String searchRecordsJson = search.fetchNextRecords(wallet, 3).get();

		JSONObject searchRecords = new JSONObject(searchRecordsJson);

		JSONArray records = searchRecords.getJSONArray("records");

		assertEquals(1, records.length());

		JSONObject expected = new JSONObject()
				.put("id", id2)
				.putOpt("type", JSONObject.NULL)
				.put("value", value2)
				.put("tags", JSONObject.NULL);

		assertTrue(JsonObjectSimilar.similar(expected, records.get(0)));

		search.close();
	}
}
