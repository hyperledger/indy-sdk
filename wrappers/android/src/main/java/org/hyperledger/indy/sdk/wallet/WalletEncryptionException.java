package org.hyperledger.indy.sdk.wallet;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown occurred during encryption-related operations.
 */
public class WalletEncryptionException extends IndyException
{
	private static final long serialVersionUID = 1829076830401150667L;
	private final static String message = "Error during encryption-related operations.";

	/**
	 * Initializes a new WalletEncryptionException.
	 */
	public WalletEncryptionException()
	{
		super(message, ErrorCode.WalletEncryptionError.value());
	}
}