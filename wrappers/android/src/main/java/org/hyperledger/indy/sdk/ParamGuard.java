package org.hyperledger.indy.sdk;

public class ParamGuard {

	public static void notNull(Object param, String paramName) {
		if(param == null)
			throw new IllegalArgumentException("A value must be provided for the '" + paramName + "' parameter.");
	}
	
	public static void notNullOrWhiteSpace(String param, String paramName) {
		if(StringUtils.isNullOrWhiteSpace(param))
			throw new IllegalArgumentException("A non-empty string must be provided for the '" + paramName + "' parameter.");
	}
}
