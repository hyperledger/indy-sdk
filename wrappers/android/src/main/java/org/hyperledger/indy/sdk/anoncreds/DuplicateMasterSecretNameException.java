package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when attempting to create a master secret name that already exists.
 */
public class DuplicateMasterSecretNameException extends IndyException
{
	private static final long serialVersionUID = 7180454759216991453L;
	private final static String message = "Another master-secret with the specified name already exists.";

	/**
	 * Initializes a new DuplicateMasterSecretNameException.
	 */
	public DuplicateMasterSecretNameException()
	{
		super(message, ErrorCode.AnoncredsMasterSecretDuplicateNameError.value());
	}
}