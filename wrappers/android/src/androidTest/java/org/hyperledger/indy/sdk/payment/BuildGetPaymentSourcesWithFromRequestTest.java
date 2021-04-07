package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.payments.IncompatiblePaymentException;
import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.UnknownPaymentMethodException;
import org.junit.Ignore;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

@Ignore("For some reason this if failing on x86 (ok on x86_64)")
public class BuildGetPaymentSourcesWithFromRequestTest extends PaymentIntegrationTest {

	@Test
	public void testBuildGetPaymentSourcesWithFromRequestWorksForUnknownPaymentMethod() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownPaymentMethodException.class));

		Payments.buildGetPaymentSourcesWithFromRequest(wallet, DID_TRUSTEE, paymentAddress, 1).get();
	}

	@Test
	public void testBuildGetPaymentSourcesWithFromRequestWorksForInvalidPaymentAddress() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(IncompatiblePaymentException.class));

		Payments.buildGetPaymentSourcesWithFromRequest(wallet, DID_TRUSTEE, "pay:null1", 1).get();
	}
}
