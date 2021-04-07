package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class DeleteRecordTest extends NonSecretsIntegrationTest {

	@Test
	public void testDeleteRecordWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tags).get();
		WalletRecord.delete(wallet, type, id).get();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		WalletRecord.get(wallet, type, id, optionsEmpty).get();
	}

	@Test
	public void testDeleteRecordWorksForNotFoundRecord() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		WalletRecord.deleteTags(wallet, type, id, "[\"tagName1\"]").get();
	}
}