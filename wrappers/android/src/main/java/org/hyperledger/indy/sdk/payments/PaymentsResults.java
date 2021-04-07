package org.hyperledger.indy.sdk.payments;

public class PaymentsResults {

	/**
	 * Result from calling addRequestFees
	 */
	public static class AddRequestFeesResult {
		private String reqWithFeesJson;
		private String paymentMethod;

		public AddRequestFeesResult(String reqWithFeesJson, String paymentMethod) {
			this.reqWithFeesJson = reqWithFeesJson;
			this.paymentMethod = paymentMethod;
		}

		public String getReqWithFeesJson() {
			return reqWithFeesJson;
		}

		public String getPaymentMethod() {
			return paymentMethod;
		}
	}

	public static class ParseGetPaymentSourcesWithFromResponseResult {
		private String sourcesJson;
		private int next;

		public ParseGetPaymentSourcesWithFromResponseResult(String sourcesJson, int next) {
			this.next = next;
			this.sourcesJson = sourcesJson;
		}

		public String getSourcesJson() {
			return sourcesJson;
		}

		public int getNext() {
			return next;
		}
	}

	/**
	 * Result from calling BuildGetPaymentSourcesRequestResult
	 */
	public static class BuildGetPaymentSourcesRequestResult {
		private String getSourcesTxnJson;
		private String paymentMethod;

		public BuildGetPaymentSourcesRequestResult(String getSourcesTxnJson, String paymentMethod) {
			this.getSourcesTxnJson = getSourcesTxnJson;
			this.paymentMethod = paymentMethod;
		}

		public String getGetSourcesTxnJson() {
			return getSourcesTxnJson;
		}

		public String getPaymentMethod() {
			return paymentMethod;
		}
	}

	/**
	 * Result from calling buildPaymentRequest
	 */
	public static class BuildPaymentReqResult {
		private String paymentReqJson;
		private String paymentMethod;

		public BuildPaymentReqResult(String paymentReqJson, String paymentMethod) {
			this.paymentReqJson = paymentReqJson;
			this.paymentMethod = paymentMethod;
		}

		public String getPaymentReqJson() {
			return paymentReqJson;
		}

		public String getPaymentMethod() {
			return paymentMethod;
		}
	}

	/**
	 * Result from calling buildMintRequest
	 */
	public static class BuildMintReqResult {
		private String mintReqJson;
		private String paymentMethod;

		public BuildMintReqResult(String mintReqJson, String paymentMethod) {
			this.mintReqJson = mintReqJson;
			this.paymentMethod = paymentMethod;
		}

		public String getMintReqJson() {
			return mintReqJson;
		}

		public String getPaymentMethod() {
			return paymentMethod;
		}
	}

	/**
	 * Result from calling BuildVerifyPaymentReqResult
	 */
	public static class BuildVerifyPaymentReqResult {
		private String verifytReqJson;
		private String paymentMethod;

		public BuildVerifyPaymentReqResult(String verifytReqJson, String paymentMethod) {
			this.verifytReqJson = verifytReqJson;
			this.paymentMethod = paymentMethod;
		}

		public String getVerifyReqJson() {
			return verifytReqJson;
		}

		public String getPaymentMethod() {
			return paymentMethod;
		}
	}
}
