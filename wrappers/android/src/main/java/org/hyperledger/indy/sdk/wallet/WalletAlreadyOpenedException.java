package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when attempting to open a wallet that has already been opened.
 */
public class WalletAlreadyOpenedException extends IndyException
{
	private static final long serialVersionUID = 3294831240096535507L;
	private final static String message = "The wallet is already open.";

	/**
	 * Initializes a new WalletAlreadyOpenedException.
	 */
	public WalletAlreadyOpenedException()
	{
		super(message, ErrorCode.WalletAlreadyOpenedError.value());
	}
}