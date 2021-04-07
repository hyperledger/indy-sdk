package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class AddRecordTagsTest extends NonSecretsIntegrationTest {

	@Test
	public void testAddRecordTagsWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tagsEmpty).get();

		checkRecordField(wallet, type, id, "tags", tagsEmpty);

		WalletRecord.addTags(wallet, type, id, tags).get();

		checkRecordField(wallet, type, id, "tags", tags);
	}

	@Test
	public void testAddRecordTagsWorksForNotFoundRecord() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		WalletRecord.addTags(wallet, type, id, tags).get();
	}
}