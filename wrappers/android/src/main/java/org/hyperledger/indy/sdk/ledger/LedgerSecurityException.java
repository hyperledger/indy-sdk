package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when a transaction cannot be sent to to insufficient privileges.
 */
public class LedgerSecurityException extends IndyException
{
	private static final long serialVersionUID = 1695822815015877550L;
	private final static String message = "The transaction cannot be sent as the privileges for the current pool connection don't allow it.";

	/**
	 * Initializes a new LedgerSecurityException.
	 */
	public LedgerSecurityException()
	{
		super(message, ErrorCode.LedgerSecurityError.value());
	}
}