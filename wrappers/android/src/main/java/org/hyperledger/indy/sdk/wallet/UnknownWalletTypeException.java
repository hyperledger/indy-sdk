package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when opening a wallet while specifying a wallet type that has not been registered.
 */
public class UnknownWalletTypeException extends IndyException
{
	private static final long serialVersionUID = -6275711661964891560L;
	private final static String message = "The wallet type specified has not been registered.";

	/**
	 * Initializes a new UnknownWalletTypeException.
	 */
	public UnknownWalletTypeException()
	{
		super(message, ErrorCode.WalletUnknownTypeError.value());
	}
}