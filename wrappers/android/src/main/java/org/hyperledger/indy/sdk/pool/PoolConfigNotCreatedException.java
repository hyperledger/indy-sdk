package org.hyperledger.indy.sdk.pool;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when attempting to open a pool using a configuration that does not exist.
 */
public class PoolConfigNotCreatedException extends IndyException
{
	private static final long serialVersionUID = 6945180938262170499L;
	private final static String message = "The requested pool cannot be opened because it does not have an existing configuration.";

	/**
	 * Initializes a new PoolConfigNotCreatedException.
	 */
	public PoolConfigNotCreatedException()
	{
		super(message, ErrorCode.PoolLedgerNotCreatedError.value());
	}
}