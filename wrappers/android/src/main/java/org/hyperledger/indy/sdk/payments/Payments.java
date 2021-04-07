package org.hyperledger.indy.sdk.payments;

import com.sun.jna.Callback;
import com.sun.jna.Pointer;

import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import org.hyperledger.indy.sdk.ParamGuard;
import org.hyperledger.indy.sdk.payments.PaymentsResults.AddRequestFeesResult;
import org.hyperledger.indy.sdk.payments.PaymentsResults.BuildGetPaymentSourcesRequestResult;
import org.hyperledger.indy.sdk.payments.PaymentsResults.BuildMintReqResult;
import org.hyperledger.indy.sdk.payments.PaymentsResults.BuildPaymentReqResult;
import org.hyperledger.indy.sdk.payments.PaymentsResults.BuildVerifyPaymentReqResult;
import org.hyperledger.indy.sdk.payments.PaymentsResults.ParseGetPaymentSourcesWithFromResponseResult;
import org.hyperledger.indy.sdk.wallet.Wallet;

import java9.util.concurrent.CompletableFuture;

import static org.hyperledger.indy.sdk.Callbacks.boolCallback;

public class Payments extends IndyJava.API {

    private Payments() {

    }

    /*
     * STATIC CALLBACKS
     */

    /**
     * Callback usedwhen method with string result completes
     */
    private static Callback stringCompleteCb = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String paymentAddress) {
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            future.complete(paymentAddress);
        }
    };

    private static Callback parsePaymentResponseWithFromCompleteCb = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String paymentAddress, int num) {
            CompletableFuture<ParseGetPaymentSourcesWithFromResponseResult> future = (CompletableFuture<ParseGetPaymentSourcesWithFromResponseResult>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            ParseGetPaymentSourcesWithFromResponseResult parsePaymentResponseWithFromResponseResult =
                    new ParseGetPaymentSourcesWithFromResponseResult(paymentAddress, num);

            future.complete(parsePaymentResponseWithFromResponseResult);
        }
    };

    /**
     * Callback used when addRequestFees completes.
     */
    private static Callback addRequestFeesCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String reqWithFeesJson, String paymentMethod) {
            CompletableFuture<AddRequestFeesResult> future = (CompletableFuture<AddRequestFeesResult>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            AddRequestFeesResult addRequestFeesResult = new AddRequestFeesResult(reqWithFeesJson, paymentMethod);

            future.complete(addRequestFeesResult);
        }
    };

    /**
     * Callback used when buildGetPaymentSourcesRequest completes.
     */
    private static Callback BuildGetPaymentSourcesRequestCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String sourcesJson, String paymentMethod) {
            CompletableFuture<BuildGetPaymentSourcesRequestResult> future = (CompletableFuture<BuildGetPaymentSourcesRequestResult>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            BuildGetPaymentSourcesRequestResult addRequestFeesResult = new BuildGetPaymentSourcesRequestResult(sourcesJson, paymentMethod);

            future.complete(addRequestFeesResult);
        }
    };

    /**
     * Callback used when buildGetPaymentSourcesRequest completes.
     */
    private static Callback buildGetPaymentSourcesWithFromRequestCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String sourcesJson, String paymentMethod) {
            CompletableFuture<BuildGetPaymentSourcesRequestResult> future = (CompletableFuture<BuildGetPaymentSourcesRequestResult>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            BuildGetPaymentSourcesRequestResult addRequestFeesResult = new BuildGetPaymentSourcesRequestResult(sourcesJson, paymentMethod);

            future.complete(addRequestFeesResult);
        }
    };


    /**
     * Callback used when buildPaymentRequest completes.
     */
    private static Callback buildPaymentReqCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String paymentReqJson, String paymentMethod) {
            CompletableFuture<BuildPaymentReqResult> future = (CompletableFuture<BuildPaymentReqResult>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            BuildPaymentReqResult addRequestFeesResult = new BuildPaymentReqResult(paymentReqJson, paymentMethod);

            future.complete(addRequestFeesResult);
        }
    };

    /**
     * Callback used when buildMintRequest completes.
     */
    private static Callback buildMintReqCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String mintReqJson, String paymentMethod) {
            CompletableFuture<BuildMintReqResult> future = (CompletableFuture<BuildMintReqResult>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            BuildMintReqResult addRequestFeesResult = new BuildMintReqResult(mintReqJson, paymentMethod);

            future.complete(addRequestFeesResult);
        }
    };

	/**
	 * Callback used when buildVerifyPaymentRequest completes.
	 */
	private static Callback buildVerifyPaymentReqCb = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int xcommandHandle, int err, String verifyReqJson, String paymentMethod) {
			CompletableFuture<BuildVerifyPaymentReqResult> future = (CompletableFuture<BuildVerifyPaymentReqResult>) removeFuture(xcommandHandle);
			if (!checkResult(future, err)) return;

			BuildVerifyPaymentReqResult verifyRequestResult = new BuildVerifyPaymentReqResult(verifyReqJson, paymentMethod);

			future.complete(verifyRequestResult);
		}
	};

    /**
     * Callback used when bytesCb completes.
     */
    private static Callback bytesCb = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, Pointer arr_raw, int arr_len) {

            CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
            if (! checkResult(future, err)) return;

            byte[] result = new byte[arr_len];
            arr_raw.read(0, result, 0, arr_len);
            future.complete(result);
        }
    };

    /*
     * STATIC METHODS
     */

    /**
     * Create the payment address for specified payment method
     *
     * This method generates private part of payment address
     * and stores it in a secure place. Ideally it should be
     * secret in libindy wallet (see crypto module).
     *
     * Note that payment method should be able to resolve this
     * secret by fully resolvable payment address format.
     *
     * @param wallet The wallet.
     * @param paymentMethod Payment method to use (for example, 'sov')
     * @param config payment address config as json:
     *               {
     *                  seed: "str", // allows deterministic creation of payment address
     *               }
     * @return public identifier of payment address in fully resolvable payment address format
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> createPaymentAddress(
            Wallet wallet,
            String paymentMethod,
            String config
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentMethod, "paymentMethod");
        ParamGuard.notNullOrWhiteSpace(config, "config");

        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_create_payment_address(
                commandHandle,
                walletHandle,
                paymentMethod,
                config,
                stringCompleteCb
        );

        checkResult(future, result);

        return future;
    }

    /**
     * Lists all payment addresses that are stored in the wallet
     *
     * @param wallet The wallet.
     * @return json array of string with json addresses
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> listPaymentAddresses(
            Wallet wallet
    ) throws IndyException {
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_list_payment_addresses(
                commandHandle,
                walletHandle,
                stringCompleteCb
        );

        checkResult(future, result);

        return future;
    }

    /**
     * Modifies Indy request by adding information how to pay fees for this transaction
     * according to this payment method.
     *
     * This method consumes set of inputs and outputs. The difference between inputs balance
     * and outputs balance is the fee for this transaction.
     *
     * Not that this method also produces correct fee signatures.
     *
     * Format of inputs is specific for payment method. Usually it should reference payment transaction
     * with at least one output that corresponds to payment address that user owns.
     *
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param reqJson initial transaction request as json
     * @param inputsJson The list of payment sources as json array:
     *   ["source1", ...]
     *     - each input should reference paymentAddress
     *     - this param will be used to determine payment_method
     * @param outputsJson The list of outputs as json array:
     *   [{
     *     recipient: "str", // payment address of recipient
     *     amount: int, // amount
     *   }]
     * @param extra Optional information for payment operation
     *
     * @return modified Indy request with added fees info
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<AddRequestFeesResult> addRequestFees(
            Wallet wallet,
            String submitterDid,
            String reqJson,
            String inputsJson,
            String outputsJson,
            String extra
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(reqJson, "reqJson");
        ParamGuard.notNullOrWhiteSpace(inputsJson, "inputsJson");
        ParamGuard.notNullOrWhiteSpace(outputsJson, "outputsJson");

        CompletableFuture<AddRequestFeesResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_add_request_fees(
                commandHandle,
                walletHandle,
                submitterDid,
                reqJson,
                inputsJson,
                outputsJson,
                extra,
                addRequestFeesCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Parses response for Indy request with fees.
     *
     * @param paymentMethod Payment method to use
     * @param respJson response for Indy request with fees
     * @return receiptsJson - parsed (payment method and node version agnostic) receipts info as json:
     *   [{
     *      receipt: "str", // receipt that can be used for payment referencing and verification
     *      recipient: "str", //payment address of recipient
     *      amount: int, // amount
     *      extra: "str", // optional data from payment transaction
     *   }]
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> parseResponseWithFees(
            String paymentMethod,
            String respJson
    ) throws IndyException {
        return parseResponse(paymentMethod, respJson, LibIndy.api::indy_parse_response_with_fees);
    }

    /**
     * Builds Indy request for getting sources list for payment address
     * according to this payment method.
     * 
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param paymentAddress target payment address
     * @return Indy request for getting sources list for payment address
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    @Deprecated
    public static CompletableFuture<BuildGetPaymentSourcesRequestResult> buildGetPaymentSourcesRequest(
            Wallet wallet,
            String submitterDid,
            String paymentAddress
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentAddress, "paymentAddress");

        CompletableFuture<BuildGetPaymentSourcesRequestResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_get_payment_sources_request(
                commandHandle,
                walletHandle,
                submitterDid,
                paymentAddress,
                BuildGetPaymentSourcesRequestCB);

        checkResult(future, result);

        return future;
    }

    /**
     * Builds Indy request for getting sources list for payment address
     * according to this payment method.
     *
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param paymentAddress target payment address
     * @param from shift to the next slice of payment sources
     * @return Indy request for getting sources list for payment address
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<BuildGetPaymentSourcesRequestResult> buildGetPaymentSourcesWithFromRequest(
            Wallet wallet,
            String submitterDid,
            String paymentAddress,
            int from
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentAddress, "paymentAddress");

        CompletableFuture<BuildGetPaymentSourcesRequestResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_get_payment_sources_with_from_request(
                commandHandle,
                walletHandle,
                submitterDid,
                paymentAddress,
                from,
                buildGetPaymentSourcesWithFromRequestCB);

        checkResult(future, result);

        return future;
    }

    public static CompletableFuture<BuildGetPaymentSourcesRequestResult> buildGetPaymentSourcesWithFromRequest(
            Wallet wallet,
            String submitterDid,
            String paymentAddress
    ) throws IndyException {
        return buildGetPaymentSourcesWithFromRequest(wallet, submitterDid, paymentAddress, -1);
    }

    /**
     * Parses response for Indy request for getting sources list.
     * 
     * @param paymentMethod payment method to use.
     * @param respJson response for Indy request for getting sources list
     * @return parsed (payment method and node version agnostic) sources info as json:
     *   [{
     *      source: "str", // source input
     *      paymentAddress: "str", //payment address for this source
     *      amount: int, // amount
     *      extra: "str", // optional data from payment transaction
     *   }]
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    @Deprecated
    public static CompletableFuture<String> parseGetPaymentSourcesResponse(
            String paymentMethod,
            String respJson
    ) throws IndyException {
        return parseResponse(paymentMethod, respJson, LibIndy.api::indy_parse_get_payment_sources_response);
    }

    /**
     * Parses response for Indy request for getting sources list.
     *
     * @param paymentMethod payment method to use.
     * @param respJson response for Indy request for getting sources list
     * @return parsed (payment method and node version agnostic) sources info as json:
     *   [{
     *      source: "str", // source input
     *      paymentAddress: "str", //payment address for this source
     *      amount: int, // amount
     *      extra: "str", // optional data from payment transaction
     *   }],
     *   next -- pointer to the next slice of payment sources
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> parseGetPaymentSourcesWithFromResponse(
            String paymentMethod,
            String respJson
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentMethod, "paymentMethod");
        ParamGuard.notNullOrWhiteSpace(respJson, "respJson");

        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibIndy.api.indy_parse_get_payment_sources_with_from_response(
                commandHandle, paymentMethod, respJson, parsePaymentResponseWithFromCompleteCb
        );

        checkResult(future, result);

        return future;
    }

    /**
     * Builds Indy request for doing payment
     * according to this payment method.
     *
     * This method consumes set of inputs and outputs.
     *
     * Format of inputs is specific for payment method. Usually it should reference payment transaction
     * with at least one output that corresponds to payment address that user owns.
     * 
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param inputsJson The list of payment sources as json array:
     *   ["source1", ...]
     *   Note that each source should reference payment address
     * @param outputsJson The list of outputs as json array:
     *   [{
     *     recipient: "str", // payment address of recipient
     *     amount: int, // amount
     *   }]
     * @param extra: Optional information for payment operation
     *
     * @return Indy request for doing payment
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<BuildPaymentReqResult> buildPaymentRequest(
            Wallet wallet,
            String submitterDid,
            String inputsJson,
            String outputsJson,
            String extra
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(inputsJson, "inputsJson");
        ParamGuard.notNullOrWhiteSpace(outputsJson, "outputsJson");

        CompletableFuture<BuildPaymentReqResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_payment_req(
                commandHandle,
                walletHandle,
                submitterDid,
                inputsJson,
                outputsJson,
                extra,
                buildPaymentReqCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Parses response for Indy request for payment txn.
     *
     * @param paymentMethod payment method to use
     * @param respJson response for Indy request for payment txn
     * @return parsed (payment method and node version agnostic) receipts info as json:
     *   [{
     *      receipt: "str", // receipt that can be used for payment referencing and verification
     *      recipient: "str", // payment address of recipient
     *      amount: int, // amount
     *      extra: "str", // optional data from payment transaction
     *   }]
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> parsePaymentResponse (
            String paymentMethod,
            String respJson
    ) throws IndyException {
        return parseResponse(paymentMethod, respJson, LibIndy.api::indy_parse_payment_response);
    }

    /**
     * Append payment extra JSON with TAA acceptance data
     *
     * EXPERIMENTAL
     *
     * This function may calculate digest by itself or consume it as a parameter.
     * If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
     *
     * @param extraJson - (Optional) original extra json.
     * @param text - (Optional) raw data about TAA from ledger.
     * @param version - (Optional) raw version about TAA from ledger.
     *     `text` and `version` parameters should be passed together.
     *     `text` and `version` parameters are required if taaDigest parameter is omitted.
     * @param taaDigest - (Optional) digest on text and version.
     *     Digest is sha256 hash calculated on concatenated strings: version || text.
     *     This parameter is required if text and version parameters are omitted.
     * @param mechanism - mechanism how user has accepted the TAA
     * @param time - UTC timestamp when user has accepted the TAA
     *
     * @return A future resolving to an updated extra result as json.
     * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<String> preparePaymentExtraWithAcceptanceData(
            String extraJson,
            String text,
            String version,
            String taaDigest,
            String mechanism,
            long time) throws IndyException {

        ParamGuard.notNull(mechanism, "mechanism");

        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibIndy.api.indy_prepare_payment_extra_with_acceptance_data(
                commandHandle,
                extraJson,
                text,
                version,
                taaDigest,
                mechanism,
                time,
                stringCompleteCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Builds Indy request for doing minting
     * according to this payment method.
     *
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param outputsJson The list of outputs as json array:
     *   [{
     *     recipient: "str", // payment address of recipient
     *     amount: int, // amount
     *     extra: "str", // optional data
     *   }]
     * @param extra Optional information for payment operation
     *
     * @return Indy request for doing minting
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<BuildMintReqResult> buildMintRequest(
            Wallet wallet,
            String submitterDid,
            String outputsJson,
            String extra
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(outputsJson, "outputsJson");

        CompletableFuture<BuildMintReqResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_mint_req(
                commandHandle,
                walletHandle,
                submitterDid,
                outputsJson,
                extra,
                buildMintReqCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Builds Indy request for setting fees for transactions in the ledger
     *
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param paymentMethod payment method to use
     * @param feesJson {
     *   txnType1: amount1,
     *   txnType2: amount2,
     *   .................
     *   txnTypeN: amountN,
     * }
     * @return Indy request for setting fees for transactions in the ledger
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> buildSetTxnFeesRequest(
            Wallet wallet,
            String submitterDid,
            String paymentMethod,
            String feesJson
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentMethod, "paymentMethod");
        ParamGuard.notNullOrWhiteSpace(feesJson, "feesJson");

        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_set_txn_fees_req(
                commandHandle,
                walletHandle,
                submitterDid,
                paymentMethod,
                feesJson,
                stringCompleteCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Builds Indy get request for getting fees for transactions in the ledger
     *
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param paymentMethod payment method to use
     * @return Indy request for getting fees for transactions in the ledger
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> buildGetTxnFeesRequest(
            Wallet wallet,
            String submitterDid,
            String paymentMethod
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentMethod, "paymentMethod");

        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_get_txn_fees_req(
                commandHandle,
                walletHandle,
                submitterDid,
                paymentMethod,
                stringCompleteCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Parses response for Indy request for getting fees
     *
     * @param paymentMethod payment method to use
     * @param respJson response for Indy request for getting fees
     * @return fees_json {
     *   txnType1: amount1,
     *   txnType2: amount2,
     *   .................
     *   txnTypeN: amountN,
     * }
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> parseGetTxnFeesResponse(
            String paymentMethod,
            String respJson
    ) throws IndyException {
        return parseResponse(paymentMethod, respJson, LibIndy.api::indy_parse_get_txn_fees_response);
    }


    /**
     * Builds Indy request for information to verify the payment receipt
     *
     * @param wallet The wallet.
     * @param submitterDid (Option) DID of request sender
     * @param receipt payment receipt to verify
     * @return Indy request for doing verification
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<BuildVerifyPaymentReqResult> buildVerifyPaymentRequest(
            Wallet wallet,
            String submitterDid,
            String receipt
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(receipt, "receipt");

        CompletableFuture<BuildVerifyPaymentReqResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_build_verify_payment_req(
                commandHandle,
                walletHandle,
                submitterDid,
		        receipt,
                buildVerifyPaymentReqCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Parses Indy response with information to verify receipt
     *
     * @param paymentMethod payment method to use
     * @param respJson response of the ledger for verify txn
     * @return parsed (payment method and node version agnostic) receipt verification info as json:
     *   {
     *     sources: ["str", ]
     *     receipts: [ {
     *         recipient: "str", // payment address of recipient
     *         receipt: "str", // receipt that can be used for payment referencing and verification
     *         amount: int, // amount
     *     } ],
     *     extra: "str", //optional data
     * }
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> parseVerifyPaymentResponse(
            String paymentMethod,
            String respJson
    ) throws IndyException {
        return parseResponse(paymentMethod, respJson, LibIndy.api::indy_parse_verify_payment_response);
    }

    private static CompletableFuture<String> parseResponse(
            String paymentMethod,
            String respJson,
            QuadFunction<Integer, String, String, Callback, Integer> method
    ) throws IndyException {
        ParamGuard.notNullOrWhiteSpace(paymentMethod, "paymentMethod");
        ParamGuard.notNullOrWhiteSpace(respJson, "respJson");

        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = method.apply(commandHandle, paymentMethod, respJson, stringCompleteCb);

        checkResult(future, result);

        return future;
    }
    
    /**
     * Gets request requirements (with minimal price) correspondent to specific auth rule
     * in case the requester can perform this action.
     *
     * EXPERIMENTAL
     *
     * If the requester does not match to the request constraints `TransactionNotAllowed` error will be thrown.   
     * 
     * @param getAuthRuleResponseJson response on GET_AUTH_RULE request returning action constraints set on the ledger.
     * @param requesterInfoJson {
     *     "role": string (optional) - role of a user which can sign a transaction.
     *     "sig_count": u64 - number of signers.
     *     "is_owner": bool (optional) - if user is an owner of transaction (false by default).
     *     "is_off_ledger_signature": bool (optional) - if user did is unknow for ledger (false by default).
     * }
     * @param feesJson fees set on the ledger (result of `parseGetTxnFeesResponse`).
     *                 
     * @return requestInfoJson: request info if a requester match to the action constraints.
     * {
     *     "price": u64 - fee required for the action performing,
     *     "requirements": [{
     *         "role": string (optional) - role of users who should sign,
     *         "sig_count": u64 - number of signers,
     *         "need_to_be_owner": bool - if requester need to be owner
     *         "off_ledger_signature": bool - allow signature of unknow for ledger did (false by default).
     *     }]
     * }
     * 
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> getRequestInfo(
            String getAuthRuleResponseJson,
            String requesterInfoJson,
            String feesJson
    ) throws IndyException {
        ParamGuard.notNull(getAuthRuleResponseJson, "getAuthRuleResponseJson");
        ParamGuard.notNull(requesterInfoJson, "requesterInfoJson");
        ParamGuard.notNull(feesJson, "feesJson");

        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        
        int result = LibIndy.api.indy_get_request_info(
                commandHandle,
                getAuthRuleResponseJson,
                requesterInfoJson,
                feesJson,
                stringCompleteCb);

        checkResult(future, result);

        return future;
    }


    /**
     * Signs a message with a payment address.
     *
     * @param wallet    The wallet.
     * @param address:  Payment address of message signer. The key must be created by calling indy_create_address
     * @param message   The message to be signed
     *
     * @return A future that resolves to a signature string.
     * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<byte[]> sigWithAddress(
            Wallet wallet,
            String address,
            byte[] message) throws IndyException {

        ParamGuard.notNull(wallet, "wallet");
        ParamGuard.notNullOrWhiteSpace(address, "address");
        ParamGuard.notNull(message, "message");

        CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
        int commandHandle = addFuture(future);

        int walletHandle = wallet.getWalletHandle();

        int result = LibIndy.api.indy_sign_with_address(
                commandHandle,
                walletHandle,
                address,
                message,
                message.length,
                bytesCb);

        checkResult(future, result);

        return future;
    }

    /**
     * Verify a signature with a payment address.
     *
     * @param address   Payment address of the message signer
     * @param message   Message that has been signed
     * @param signature A signature to be verified
     * @return A future that resolves to true if signature is valid, otherwise false.
     * @throws IndyException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Boolean> verifyWithAddress(
            String address,
            byte[] message,
            byte[] signature) throws IndyException {

        ParamGuard.notNullOrWhiteSpace(address, "address");
        ParamGuard.notNull(message, "message");
        ParamGuard.notNull(signature, "signature");

        CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
        int commandHandle = addFuture(future);

        int result = LibIndy.api.indy_verify_with_address(
                commandHandle,
                address,
                message,
                message.length,
                signature,
                signature.length,
                boolCallback);

        checkResult(future, result);

        return future;
    }

    @FunctionalInterface
    interface QuadFunction<Arg1, Arg2, Arg3, Arg4, Res> {
        Res apply(Arg1 arg1, Arg2 arg2, Arg3 arg3, Arg4 arg4);
    }
}
