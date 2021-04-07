package org.hyperledger.indy.sdk.payment;


import org.hyperledger.indy.sdk.IndyIntegrationTestWithSingleWallet;

public class PaymentIntegrationTest extends IndyIntegrationTestWithSingleWallet {

	static final String paymentMethod = "null";
	static final String paymentAddress = "pay:null:test";
	static final String emptyObject = "{}";
	static final String emptyArray = "[]";
	static final String inputs = "[\"pay:null:1\", \"pay:null:2\"]";
	static final String outputs = "[{\"recipient\": \"pay:null:1\", \"amount\":1}, {\"recipient\": \"pay:null:2\", \"amount\":2}]";
	static final String invalidInputs = "pay:null:1";
	static final String incompatibleInputs = "[\"pay:PAYMENT_METHOD_1:1\", \"pay:PAYMENT_METHOD_2:1\"]";
	static final String incompatibleOutputs = "[{\"recipient\": \"pay:PAYMENT_METHOD_1:1\", \"amount\":1}, {\"recipient\": \"pay:PAYMENT_METHOD_2:1\", \"amount\":1}]";
	static final String fees = "{\"txnType1\":1, \"txnType2\":2}";
	static final String receipt = "pay:null:0_PqVjwJC42sxCTJp";

}
