package org.hyperledger.indy.sdk;

/**
 * Exception thrown when the SDK reports than an invalid parameter was passed to it.
 */
public class InvalidParameterException extends IndyException {

	private static final long serialVersionUID = -1802344846222826490L;
	private int parameterIndex;

	/**
	 * Gets the index of the parameter the SDK reported as incorrect.
	 * @param sdkErrorCode
	 * @return
	 */
	private static int getParamIndex(int sdkErrorCode)
	{
		assert(sdkErrorCode >= 100 && sdkErrorCode <= 111);
		return sdkErrorCode - 99;
	}

	/**
	 * Constructs the error message for the exception from the SDK error code.
	 *
	 * @param sdkErrorCode The SDK error code.
	 * @return A message indicating which parameter was incorrect.
	 */
	private static String buildMessage(int sdkErrorCode)
	{
		return String.format("The value passed to parameter %s is not valid.", getParamIndex(sdkErrorCode));
	}

	/**
	 * Initializes a new InvalidParameterException with the SDK error code.
	 *
	 * @param sdkErrorCode The SDK error code.
	 */
    public InvalidParameterException(int sdkErrorCode)
    {
		super(buildMessage(sdkErrorCode), sdkErrorCode);
		parameterIndex = getParamIndex(sdkErrorCode);
	}

	/**
	 * Gets the index of the parameter that was incorrect.
	 *
	 * @return The index of the parameter that was incorrect.
	 */
	public int getParameterIndex() {
		return parameterIndex;
	}
}
