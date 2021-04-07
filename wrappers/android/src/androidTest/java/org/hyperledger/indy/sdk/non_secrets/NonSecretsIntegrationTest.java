package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONObject;
import org.junit.Rule;
import org.junit.rules.ExpectedException;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import org.hyperledger.indy.sdk.JsonObjectSimilar;

public class NonSecretsIntegrationTest extends IndyIntegrationTestWithSingleWallet {

	@Rule
	public ExpectedException thrown = ExpectedException.none();

	String type = "TestType";
	String id = "RecordId";
	String id2 = "RecordId2";
	String id3 = "RecordId3";
	String value = "RecordValue";
	String value2 = "RecordValue2";
	String value3 = "RecordValue3";
	String tagsEmpty = "{}";
	String queryEmpty = "{}";
	String optionsEmpty = "{}";
	String tags =  "{\"tagName1\":\"str1\",\"tagName2\":\"5\",\"tagName3\":\"12\"}";
	String tags2 = "{\"tagName1\":\"str2\",\"tagName2\":\"pre_str3\",\"tagName3\":\"2\"}";
	String tags3 = "{\"tagName1\":\"str1\",\"tagName2\":\"str2\",\"tagName3\":\"str3\"}";

	void checkRecordField(Wallet wallet, String type, String id, String field, String expectedValue) throws Exception {
		String optionsFull = "{\"retrieveType\":true, \"retrieveValue\":true, \"retrieveTags\":true}";
		String recordJson = WalletRecord.get(wallet, type, id, optionsFull).get();
		JSONObject record = new JSONObject(recordJson);

		switch (field) {
			case "value":
				assertEquals(expectedValue, record.getString("value"));
				break;
			case "tags":
				JSONObject expected = new JSONObject(expectedValue);
				assertTrue(JsonObjectSimilar.similar(expected, record.getJSONObject("tags")));
				break;
			default:
				assertTrue(false);
		}

	}
}
