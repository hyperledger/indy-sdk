package org.hyperledger.indy.sdk.wallet;

import static org.hamcrest.CoreMatchers.isA;

import java.util.concurrent.ExecutionException;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.json.JSONObject;
import org.junit.Test;


public class CreateWalletTest extends IndyIntegrationTest {

	@Test
	public void testCreateWalletWorks() throws Exception {
		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testCreateWalletWorksForEmptyType() throws Exception {
		String config = new JSONObject()
				.put("id", WALLET)
				.toString();

		Wallet.createWallet(config, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testCreateWalletWorksForUnknowType() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownWalletTypeException.class));

		String config =
				new JSONObject()
						.put("id", WALLET)
						.put("storage_type", "unknown_type")
						.toString();

		Wallet.createWallet(config, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testCreateWalletWorksForEmptyName() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		String config = new JSONObject()
				.put("id", "")
				.toString();
		Wallet.createWallet(config, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testCreateWalletWorksForDuplicate() throws Exception {
		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletExistsException.class));

		Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
	}
}