package org.hyperledger.indy.sdk.payments;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

public class TransactionNotAllowedException extends IndyException {
    private static final long serialVersionUID = 6397499268992083529L;
    private static final String message = "The transaction is not allowed to a requester";

    /**
     * Initializes a new {@link TransactionNotAllowedException} with the specified message.
     */
    public TransactionNotAllowedException() {
        super(message, ErrorCode.TransactionNotAllowedError.value());
    }
}
