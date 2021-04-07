package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.payments.IncompatiblePaymentException;
import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.UnknownPaymentMethodException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class AddRequestFeesTest extends PaymentIntegrationTest {

	@Test
	public void testAddRequestFeesWorksForUnknownPaymentMethod() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownPaymentMethodException.class));

		Payments.addRequestFees(wallet, DID_TRUSTEE, emptyObject, inputs, outputs, null).get();
	}

	@Test
	public void testAddRequestFeesWorksForEmptyInputs() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Payments.addRequestFees(wallet, DID_TRUSTEE, emptyObject, emptyArray, outputs, null).get();
	}

	@Test
	public void testAddRequestFeesWorksForSeveralMethods() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(IncompatiblePaymentException.class));

		Payments.addRequestFees(wallet, DID_TRUSTEE, emptyObject, incompatibleInputs, emptyObject, null).get();
	}
}
