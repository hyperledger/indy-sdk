package org.hyperledger.indy.sdk.crypto;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;
import org.hyperledger.indy.sdk.InvalidParameterException;
import org.hyperledger.indy.sdk.wallet.WalletItemNotFoundException;
import org.json.JSONObject;
import org.junit.Test;
import org.json.JSONArray;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.*;

public class PackUnpackMessageTest extends IndyIntegrationTestWithSingleWallet {
	private String message = "hello world";

	@Test
	public void testPackMessageAnoncryptWorks() throws Exception {
		JSONArray receivers = new JSONArray(new String[]{VERKEY_MY1, VERKEY_MY2, VERKEY_TRUSTEE});

		byte[] packedMessage = Crypto.packMessage(wallet, receivers.toString(), null, message.getBytes()).get();

		assertNotNull(packedMessage);
	}

	@Test
	public void testPackMessageAuthcryptWorks() throws Exception {
		JSONArray receivers = new JSONArray(new String[]{VERKEY_MY1});

		String senderVerkey = Crypto.createKey(wallet, MY1_IDENTITY_KEY_JSON).get();

		byte[] packedMessage = Crypto.packMessage(wallet, receivers.toString(), senderVerkey, message.getBytes()).get();

		assertNotNull(packedMessage);
	}

	@Test
	public void testPackErrorsWithUnknownSenderVerkey() throws Exception {
		JSONArray receivers = new JSONArray(new String[]{VERKEY_MY1});

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(WalletItemNotFoundException.class));

		Crypto.packMessage(wallet, receivers.toString(), IndyIntegrationTest.VERKEY_MY2, message.getBytes()).get();

		// this assert should never trigger since packMessage should throw exception
		assertTrue(false);
	}

	@Test
	public void testPackMessageErrorsWithNoReceivers() throws Exception {
		JSONArray receivers = new JSONArray();

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidParameterException.class));

		Crypto.packMessage(wallet, receivers.toString(), null, message.getBytes()).get();

		// this assert should never trigger since unpackMessage should throw exception
		assertTrue(false);
	}

	@Test
	public void testUnpackMessageAnoncryptWorks() throws Exception {
		String receiverVerkey = Crypto.createKey(wallet, "{}").get();

		JSONArray receivers = new JSONArray(new String[]{receiverVerkey, VERKEY_TRUSTEE});

		byte[] packedMessage = Crypto.packMessage(wallet, receivers.toString(), null, message.getBytes()).get();

		byte[] unpackedMessageBytes = Crypto.unpackMessage(wallet, packedMessage).get();

		JSONObject unpackedMessage = new JSONObject(new String(unpackedMessageBytes));

		assertEquals(message, unpackedMessage.getString("message"));
		assertEquals(receiverVerkey, unpackedMessage.getString("recipient_verkey"));
		assertFalse(unpackedMessage.has("sender_verkey"));
	}

	@Test
	public void testUnpackMesssageAuthcryptWorks() throws Exception {
		String receiverVerkey = Crypto.createKey(wallet, "{}").get();
		String senderVerkey = Crypto.createKey(wallet, "{}").get();

		JSONArray receivers = new JSONArray(new String[]{receiverVerkey});

		byte[] packedMessage = Crypto.packMessage(wallet, receivers.toString(), senderVerkey, message.getBytes()).get();
		byte[] unpackedMessageBytes = Crypto.unpackMessage(wallet, packedMessage).get();

		JSONObject unpackedMessage = new JSONObject(new String(unpackedMessageBytes));

		assertEquals(message, unpackedMessage.getString("message"));
		assertEquals(receiverVerkey, unpackedMessage.getString("recipient_verkey"));
		assertEquals(senderVerkey, unpackedMessage.getString("sender_verkey"));
	}
}
