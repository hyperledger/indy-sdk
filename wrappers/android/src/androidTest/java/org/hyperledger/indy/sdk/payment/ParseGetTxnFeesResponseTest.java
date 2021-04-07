package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.UnknownPaymentMethodException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class ParseGetTxnFeesResponseTest extends PaymentIntegrationTest {

	@Test
	public void testParseGetTxnFeesResponseResponseTestWorksForUnknownPaymentMethod() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownPaymentMethodException.class));

		Payments.parseGetTxnFeesResponse(paymentMethod, emptyObject).get();
	}
}
