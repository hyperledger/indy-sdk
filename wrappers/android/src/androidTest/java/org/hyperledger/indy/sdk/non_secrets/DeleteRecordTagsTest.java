package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class DeleteRecordTagsTest extends NonSecretsIntegrationTest {

	@Test
	public void testDeleteRecordTagsWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();

		checkRecordField(wallet, type, id, "tags", tags);

		WalletRecord.deleteTags(wallet, type, id, "[\"tagName1\"]").get();

		String expectedTags = "{\"tagName2\": \"5\", \"tagName3\": \"12\"}";
		checkRecordField(wallet, type, id, "tags", expectedTags);
	}

	@Test
	public void testDeleteRecordTagsWorksForNotFoundRecord() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		WalletRecord.deleteTags(wallet, type, id, "[\"tagName1\"]").get();
	}
}