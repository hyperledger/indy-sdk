package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.ErrorCode;
import org.hyperledger.indy.sdk.IndyException;

/**
 * Exception thrown when a proof has been rejected.
 */
public class ProofRejectedException extends IndyException
{
	private static final long serialVersionUID = -5100028213117687183L;
	private final static String message = "The proof has been rejected.";

	/**
	 * Initializes a new ProofRejectionException.
	 */
	public ProofRejectedException()
	{
		super(message, ErrorCode.AnoncredsProofRejected.value());
	}
}