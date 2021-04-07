package org.hyperledger.indy.sdk.payment;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.payments.IncompatiblePaymentException;
import org.hyperledger.indy.sdk.payments.Payments;
import org.hyperledger.indy.sdk.payments.UnknownPaymentMethodException;
import org.junit.Test;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;

public class BuildMintRequestTest extends PaymentIntegrationTest {

	@Test
	public void testBuildMintRequestWorksForUnknownPaymentMethod() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(UnknownPaymentMethodException.class));

		Payments.buildMintRequest(wallet, DID_TRUSTEE, outputs, null).get();
	}

	@Test
	public void testBuildMintRequestWorksForEmptyOutputs() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		Payments.buildMintRequest(wallet, DID_TRUSTEE, emptyArray, null).get();
	}

	@Test
	public void testBuildMintRequestWorksForIncompatiblePaymentMethods() throws Exception {
		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(IncompatiblePaymentException.class));

		Payments.buildMintRequest(wallet, DID_TRUSTEE, incompatibleOutputs, null).get();
	}
}
