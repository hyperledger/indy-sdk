package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.payments.IncompatiblePaymentException;
import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.UnknownPaymentMethodException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class BuildGetPaymentSourcesRequestTest extends PaymentIntegrationTest {

	@Test
	public void testBuildGetPaymentSourcesRequestWorksForUnknownPaymentMethod() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownPaymentMethodException.class));

		Payments.buildGetPaymentSourcesRequest(wallet, DID_TRUSTEE, paymentAddress).get();
	}

	@Test
	public void testBuildGetPaymentSourcesRequestWorksForInvalidPaymentAddress() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(IncompatiblePaymentException.class));

		Payments.buildGetPaymentSourcesRequest(wallet, DID_TRUSTEE, "pay:null1").get();
	}
}
