package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.payments.Payments;
import org.json.JSONArray;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class ListPaymentAddressesTest extends PaymentIntegrationTest {

	@Test
	public void testListPaymentAddressesWorks() throws Exception {
		String paymentAddressJson = Payments.listPaymentAddresses(wallet).get();
		JSONArray paymentAddresses = new JSONArray(paymentAddressJson);
		assertEquals(0, paymentAddresses.length());
	}
}
