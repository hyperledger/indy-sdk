package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.UnknownPaymentMethodException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class BuildSetTxnFeesRequestTest extends PaymentIntegrationTest {

	@Test
	public void testBuildSetTxnFeesRequestWorksForUnknownPaymentMethod() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownPaymentMethodException.class));

		Payments.buildSetTxnFeesRequest(wallet, DID_TRUSTEE, paymentMethod, fees).get();
	}

	@Test
	public void testBuildSetTxnFeesRequestWorksForInvalidFees() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Payments.buildSetTxnFeesRequest(wallet, DID_TRUSTEE, paymentMethod, "[txnType1:1, txnType2:2]").get();
	}
}
