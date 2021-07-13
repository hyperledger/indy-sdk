package com.evernym.sdk.vcx.issuer;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.util.concurrent.CompletableFuture;


public class IssuerApi extends VcxJava.API {

    private static final Logger logger = LoggerFactory.getLogger("IssuerApi");
    private static final Callback issuerCreateCredentialCB = new Callback() {
        // TODO: This callback and jna definition needs to be fixed for this API
        // it should accept connection handle as well
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int credentialHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], credentialHandle = [" + credentialHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = credentialHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> issuerCreateCredential(String sourceId,
                                                                    int credentialDefHandle,
                                                                    String issuerId,
                                                                    String credentialData,
                                                                    String credentialName,
                                                                    long price) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
        ParamGuard.notNullOrWhiteSpace(credentialData, "credentialData");
        ParamGuard.notNullOrWhiteSpace(credentialName, "credentialName");

        logger.debug("issuerCreateCredential() called with: sourceId = [" + sourceId + "], credentialDefHandle = [" + credentialDefHandle + "], issuerId = [" + issuerId + "], credentialData = [" + credentialData + "], credentialName = [" + credentialName + "], price = [" + price + "]");
        //TODO: Check for more mandatory params in vcx to add in PamaGuard
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_create_credential(
                issue,
                sourceId,
                credentialDefHandle,
                issuerId,
                credentialData,
                credentialName,
                String.valueOf(price),
                issuerCreateCredentialCB);
        checkResult(result);
        return future;
    }

    private static Callback issuerSendCredentialOfferCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
            future.complete(err);
        }
    };

    public static CompletableFuture<Integer> issuerSendCredentialOffer(int credentialHandle,
                                                                       int connectionHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        ParamGuard.notNull(connectionHandle, "connectionHandle");
        logger.debug("issuerSendcredentialOffer() called with: credentialOffer = [" + credentialHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_send_credential_offer(
                issue,
                credentialHandle,
                connectionHandle,
                issuerSendCredentialOfferCB
        );
        checkResult(result);
        return future;
    }

    public static CompletableFuture<String> issuerGetCredentialOfferMsg(int credentialHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerSendCredentialOffer() called with: credentialHandle = [****]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_get_credential_offer_msg(
                issue,
                credentialHandle,
                issuerCredentialStringCB
        );
        checkResult(result);
        return future;
    }

    private static Callback issuerCredentialUpdateStateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err,int state) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(state);
        }
    };

    public static CompletableFuture<Integer> issuerCredentialUpdateState(int credentialHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerCredentialUpdateState() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);
        int result = LibVcx.api.vcx_issuer_credential_update_state(issue, credentialHandle, issuerCredentialUpdateStateCB);
        checkResult(result);
        return future;
    }

    public static CompletableFuture<Integer> issuerCredentialUpdateStateWithMessage(int credentialHandle, String message) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        ParamGuard.notNull(message, "message");

        logger.debug("issuerCredentialUpdateStateWithMessage() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);
        int result = LibVcx.api.vcx_issuer_credential_update_state_with_message(issue, credentialHandle, message, issuerCredentialUpdateStateCB);
        checkResult(result);
        return future;
    }

    private static Callback issuerCredentialGetStateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int state) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(state);
        }
    };

    public static CompletableFuture<Integer> issuerCredentialGetState(int credentialHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerCredentialGetState() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);
        int result = LibVcx.api.vcx_issuer_credential_get_state(issue, credentialHandle, issuerCredentialGetStateCB);
        checkResult(result);
        return future;
    }
    private static Callback issuerSendCredentialCB = new Callback() {
        public void callback(int commandHandle, int err, int credentialDefId) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], credentialDefId = [" + credentialDefId + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(credentialDefId);
        }
    };

    public static CompletableFuture<Integer> issuerSendCredential(int credentialHandle,
                                                                 int connectionHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        ParamGuard.notNull(connectionHandle, "connectionHandle");
        logger.debug("issuerSendCredential() called with: credentialHandle = [" + credentialHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_send_credential(
                issue,
                credentialHandle,
                connectionHandle,
                issuerSendCredentialCB);

        checkResult(result);
        return future;
    }

    public static CompletableFuture<String> issuerGetCredentialMsg(int credentialHandle,
                                                                   String myPwDid) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerGetCredentialMsg() called with: credentialHandle = [****]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_get_credential_msg(
                issue,
                credentialHandle,
                myPwDid,
                issuerCredentialStringCB
        );
        checkResult(result);
        return future;
    }

    private static Callback issuerCredentialStringCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String stringData) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], string = [" + stringData + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
            String result = stringData;
            future.complete(result);
        }
    };

    public static CompletableFuture<String> issuerCredentialSerialize(int credentialHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerCredentialSerialize() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_credential_serialize(
                issue,
                credentialHandle,
                issuerCredentialStringCB
        );
        checkResult(result);
        return future;
    }

    private static Callback issuerCredentialDeserializeCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int handle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], handle = [" + handle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
            Integer result = handle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> issuerCredentialDeserialize(String serializedData) throws VcxException {
        ParamGuard.notNull(serializedData, "serializedData");
        logger.debug("issuerCredentialDeserialize() called with: serializedData = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_credential_deserialize(
                issue,
                serializedData,
                issuerCredentialDeserializeCB
        );
        checkResult(result);
        return future;
    }



    public static CompletableFuture<Integer> issuerTerminateCredential(
            int credentialHandle,
            int state,
            String msg
    ) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        ParamGuard.notNull(state, "state");
        ParamGuard.notNullOrWhiteSpace(msg, "msg");
        logger.debug("issuerTerminateCredential() called with: credentialHandle = [" + credentialHandle + "], state = [" + state + "], msg = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int issue = addFuture(future);

        int result = LibVcx.api.vcx_issuer_terminate_credential(
                issue,
                credentialHandle,
                state,
                msg);
        checkResult(result);

        return future;

    }
    public static int issuerCredentialRelease(int credentialHandle) throws VcxException {
        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerCredentialRelease() called with: credentialHandle = [" + credentialHandle + "]");

        int result = LibVcx.api.vcx_issuer_credential_release(credentialHandle);
        checkResult(result);

        return result;
    }

    public static CompletableFuture<Integer> issuerCredentialRequest(
            int credentialHandle,
            String credentialRequest) throws VcxException {

        ParamGuard.notNull(credentialHandle, "credentialHandle");
        ParamGuard.notNull(credentialRequest, "credentialRequest");
        logger.debug("issuercredentialRequest() called with: credentialHandle = [" + credentialHandle + "], credentialRequest = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();

        int result = LibVcx.api.vcx_issuer_get_credential_request(
                credentialHandle,
                credentialRequest);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<Integer> issuerAcceptRequest(
            int credentialHandle) throws VcxException {

        ParamGuard.notNull(credentialHandle, "credentialHandle");
        logger.debug("issuerAcceptRequest() called with: credentialHandle = [" + credentialHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();

        int result = LibVcx.api.vcx_issuer_accept_credential(
                credentialHandle);
        checkResult(result);

        return future;
    }
}
