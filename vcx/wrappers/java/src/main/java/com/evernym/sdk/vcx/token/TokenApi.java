package com.evernym.sdk.vcx.token;


import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class TokenApi extends VcxJava.API {

    private TokenApi() {
    }

    private static final Logger logger = LoggerFactory.getLogger("TokenApi");
    private static Callback vcxTokenCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String tokenInfo) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], tokenInfo = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            future.complete(tokenInfo);
        }
    };

    public static CompletableFuture<String> getTokenInfo(
            int paymentHandle
    ) throws VcxException {
        logger.debug("getTokenInfo() called with: paymentHandle = [" + paymentHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_wallet_get_token_info(commandHandle, paymentHandle, vcxTokenCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxSendTokensCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int error, String receipt) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], error = [" + error + "], receipt = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, error)) {
                return;
            }
            future.complete(receipt);
        }
    };

    public static CompletableFuture<String> sendTokens(
            int paymentHandle,
            String tokens,
            String recipient
    ) throws VcxException {
        logger.debug("sendTokens() called with: paymentHandle = [" + paymentHandle + "], tokens = [" + tokens + "], recipient = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_wallet_send_tokens(commandHandle, paymentHandle, tokens, recipient, vcxSendTokensCB);
        checkResult(result);
        return future;
    }


    private static Callback vcxCreatePaymentAddressCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int error, String address) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], error = [" + error + "], address = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, error)) {
                return;
            }
            future.complete(address);
        }
    };

    public static CompletableFuture<String> createPaymentAddress(
            String seed
    ) throws VcxException {
        logger.debug("createPaymentAddress() called with: seed = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_wallet_create_payment_address(commandHandle, seed, vcxCreatePaymentAddressCB);
        checkResult(result);
        return future;
    }
}
