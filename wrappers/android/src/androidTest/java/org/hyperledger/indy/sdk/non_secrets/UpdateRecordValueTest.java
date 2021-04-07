package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class UpdateRecordValueTest extends NonSecretsIntegrationTest {

	@Test
	public void testUpdateRecordValueWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tagsEmpty).get();

		checkRecordField(wallet, type, id, "value", value);

		WalletRecord.updateValue(wallet, type, id, value2).get();

		checkRecordField(wallet, type, id, "value", value2);
	}

	@Test
	public void testUpdateRecordValueWorksForNotFoundRecord() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		WalletRecord.updateValue(wallet, type, id, value).get();
	}
}