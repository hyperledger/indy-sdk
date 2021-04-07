package org.hyperledger.indy.sdk;

/**
 * Exception thrown when the SDK reports that it is in an invalid state.
 */
public class InvalidStateException extends IndyException
{
	private static final long serialVersionUID = -1741244553102207886L;
	private final static String message = "The SDK library experienced an unexpected internal error.";

	/**
	 * Initializes a new InvalidStateException.
	 */
	public InvalidStateException()
	{
		super(message, ErrorCode.CommonInvalidState.value());
	}
}
