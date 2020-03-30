package com.evernym.sdk.vcx.proof;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class ProofApi extends VcxJava.API {
    private ProofApi(){}

    private static final Logger logger = LoggerFactory.getLogger("ProofApi");
    private static Callback vcxProofCreateCB = new Callback() {
        public void callback(int commandHandle, int err, int proofHandle){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofHandle = [" + proofHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            Integer result = proofHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofCreate(
            String sourceId,
            String requestedAttrs,
            String requestedPredicates,
            String revocationInterval,
            String name
    ) throws VcxException {
        ParamGuard.notNull(sourceId, "sourceId");
        ParamGuard.notNull(requestedAttrs, "requestedAttrs");
        ParamGuard.notNull(requestedPredicates, "requestedPredicates");
        ParamGuard.notNull(revocationInterval, "revocationInterval");
        ParamGuard.notNull(name, "name");
        logger.debug("proofCreate() called with: sourceId = [" + sourceId + "], requestedAttrs = [" + requestedAttrs + "], requestedPredicates = [" + requestedPredicates + "], revocationInterval = [" + revocationInterval + "], name = [" + name + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        if (requestedPredicates.isEmpty()) requestedPredicates = "[]";
        int result = LibVcx.api.vcx_proof_create(commandHandle, sourceId, requestedAttrs, requestedPredicates, revocationInterval, name, vcxProofCreateCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofSendRequestCB = new Callback() {
        public void callback(int commandHandle, int err){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofSendRequest(
            int proofHandle,
            int connectionHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        ParamGuard.notNull(connectionHandle, "connectionHandle");
        logger.debug("proofSendRequest() called with: proofHandle = [" + proofHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_send_request(commandHandle, proofHandle, connectionHandle, vcxProofSendRequestCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofGetRequestMsgCB = new Callback() {
        public void callback(int commandHandle, int err, String msg){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], msg = [" + msg + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            Integer result = commandHandle;
            future.complete(msg);
        }
    };

    public static CompletableFuture<String> proofGetRequestMsg(
            int proofHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofGetRequestMsg() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_get_request_msg(commandHandle, proofHandle, vcxProofGetRequestMsgCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxGetProofCB = new Callback() {
        public void callback(int commandHandle, int err, int proofState, String responseData){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofState = [" + proofState + "], responseData = [****]");
            CompletableFuture<GetProofResult> future = (CompletableFuture<GetProofResult>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            GetProofResult result = new GetProofResult(proofState,responseData);
            future.complete(result);
        }
    };

    public static CompletableFuture<GetProofResult> getProof(
            int proofHandle,
            int connectionHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        ParamGuard.notNull(connectionHandle, "connectionHandle");
        logger.debug("getProof() called with: proofHandle = [" + proofHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<GetProofResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_proof(commandHandle, proofHandle, connectionHandle, vcxGetProofCB);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<GetProofResult> getProofMsg(
            int proofHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("getProof() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<GetProofResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_proof_msg(commandHandle, proofHandle, vcxGetProofCB);
        checkResult(result);

        return future;
    }


    // vcx_proof_accepted
    public static CompletableFuture<Integer> proofAccepted(
            int proofHandle,
            String responseData
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        ParamGuard.notNull(responseData, "responseData");
        logger.debug("proofAccepted() called with: proofHandle = [" + proofHandle + "], responseData = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();

        int result = LibVcx.api.vcx_proof_accepted(proofHandle, responseData);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofUpdateStateCB = new Callback() {
        public void callback(int commandHandle, int err, int state){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            Integer result = state;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofUpdateState(
            int proofHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofUpdateState() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_update_state(commandHandle, proofHandle, vcxProofUpdateStateCB);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<Integer> proofUpdateStateWithMessage(
            int proofHandle,
            String message
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofUpdateStateWithMessage() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_update_state_with_message(commandHandle, proofHandle, message, vcxProofUpdateStateCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofGetStateCB = new Callback() {
        public void callback(int commandHandle, int err, int state){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            Integer result = state;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofGetState(
            int proofHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofGetState() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_get_state(commandHandle, proofHandle, vcxProofGetStateCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofSerializeCB = new Callback() {
        public void callback(int commandHandle, int err, String proofState){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofState = [" + proofState + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            future.complete(proofState);
        }
    };

    public static CompletableFuture<String> proofSerialize(
            int proofHandle
    ) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofSerialize() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_serialize(commandHandle, proofHandle, vcxProofSerializeCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofDeserializeCB = new Callback() {
        public void callback(int commandHandle, int err, int proofHandle){
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofHandle = [" + proofHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if(!checkCallback(future,err)) return;
            Integer result = proofHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofDeserialize(
            String serializedProof
    ) throws VcxException {
        ParamGuard.notNull(serializedProof, "serializedProof");
        logger.debug("proofDeserialize() called with: serializedProof = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_proof_deserialize(commandHandle, serializedProof, vcxProofDeserializeCB);
        checkResult(result);

        return future;
    }

    public static int proofRelease(int proofHandle) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofRelease() called with: proofHandle = [" + proofHandle + "]");

        int result = LibVcx.api.vcx_proof_release(proofHandle);
        checkResult(result);

        return result;
    }

}
