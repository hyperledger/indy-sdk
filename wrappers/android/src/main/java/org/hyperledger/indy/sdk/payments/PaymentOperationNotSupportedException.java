package org.hyperledger.indy.sdk.payments;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

public class PaymentOperationNotSupportedException extends IndyException {
	private static final long serialVersionUID = - 5009466707967765943L;
	private static final String message = "Operation is not supported for payment method";

	/**
	 * Initializes a new {@link PaymentOperationNotSupportedException} with the specified message.
	 */
	public PaymentOperationNotSupportedException() {
		super(message, ErrorCode.PaymentOperationNotSupportedError.value());
	}
}
