package org.hyperledger.indy.sdk.non_secrets;

import org.hyperledger.indy.sdk.wallet.WalletItemAlreadyExistsException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;


public class AddRecordTest extends NonSecretsIntegrationTest {

	@Test
	public void testAddRecordWorks() throws Exception {
		WalletRecord.add(wallet, type, id, value, tagsEmpty).get();
	}

	@Test
	public void testAddRecordWorksForDuplicate() throws Exception {
		WalletRecord.add(wallet, type, id, value, tagsEmpty).get();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemAlreadyExistsException.class));

		WalletRecord.add(wallet, type, id, value, tagsEmpty).get();
	}
}