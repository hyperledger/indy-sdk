package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when attempting to create a wallet using the same name as a wallet that already exists.
 */
public class WalletExistsException extends IndyException
{
	private static final long serialVersionUID = 1829076830401150667L;
	private final static String message = "A wallet with the specified name already exists.";

	/**
	 * Initializes a new WalletExistsException.
	 */
	public WalletExistsException()
	{
		super(message, ErrorCode.WalletAlreadyExistsError.value());
	}
}