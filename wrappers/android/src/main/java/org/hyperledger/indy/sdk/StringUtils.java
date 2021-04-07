package org.hyperledger.indy.sdk;

public class StringUtils {

	public static boolean isNullOrWhiteSpace(String s) {
		if (s == null)
			return true;
		
		for (int i = 0; i < s.length(); i++) {
			if (!Character.isWhitespace(s.charAt(i))) {
				return false;
			}
		}
		
		return true;
	}
	
}
