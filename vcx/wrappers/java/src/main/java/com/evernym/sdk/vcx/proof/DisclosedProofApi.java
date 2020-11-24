package com.evernym.sdk.vcx.proof;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class DisclosedProofApi extends VcxJava.API {

    private DisclosedProofApi() {
    }

    private static final Logger logger = LoggerFactory.getLogger("DisclosedProofApi");

    private static Callback vcxProofCreateWithMsgIdCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int proofHandle, String proofRequest) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofHandle = [" + proofHandle + "], proofRequest = [****]");
            CompletableFuture<CreateProofMsgIdResult> future = (CompletableFuture<CreateProofMsgIdResult>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            CreateProofMsgIdResult result = new CreateProofMsgIdResult(proofHandle, proofRequest);
            future.complete(result);
        }
    };

    public static CompletableFuture<CreateProofMsgIdResult> proofCreateWithMsgId(
            String sourceId,
            int connectionHandle,
            String msgId
    ) throws VcxException {
        ParamGuard.notNull(sourceId, "sourceId");
        ParamGuard.notNull(msgId, "msgId");
        logger.debug("proofCreateWithMsgId() called with: sourceId = [" + sourceId + "], connectionHandle = [" + connectionHandle + "], msgId = [" + msgId + "]");
        CompletableFuture<CreateProofMsgIdResult> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_create_with_msgid(commandHandle, sourceId, connectionHandle, msgId, vcxProofCreateWithMsgIdCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofUpdateStateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int state) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(state);
        }
    };

    public static CompletableFuture<Integer> proofUpdateState(
            int proofHandle
    ) throws VcxException {
        logger.debug("proofUpdateState() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_update_state(commandHandle, proofHandle, vcxProofUpdateStateCB);
        checkResult(result);

        return future;
    }

    public static CompletableFuture<Integer> proofUpdateStateWithMessage(
            int proofHandle,
            String message
    ) throws VcxException {
        logger.debug("proofUpdateStateWithMessage() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_update_state_with_message(commandHandle, proofHandle, message, vcxProofUpdateStateCB);
        checkResult(result);

        return future;
    }

    private static Callback proofGetRequestsCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String proofRequests) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofRequests = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(proofRequests);
        }
    };

    public static CompletableFuture<String> proofGetRequests(
            int connectionHandle
    ) throws VcxException {
        logger.debug("proofGetRequests() called with: connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_get_requests(commandHandle, connectionHandle, proofGetRequestsCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofGetStateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int proofHandle, int state) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], proofHandle = [" + proofHandle + "], state = [" + state + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(state);
        }
    };

    public static CompletableFuture<Integer> proofGetState(
            int proofHandle
    ) throws VcxException {
        logger.debug("proofGetState() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_get_state(commandHandle, proofHandle, vcxProofGetStateCB);
        checkResult(result);

        return future;
    }


    public static int proofRelease(int proofHandle) throws VcxException {
        ParamGuard.notNull(proofHandle, "proofHandle");
        logger.debug("proofRelease() called with: proofHandle = [" + proofHandle + "]");

        int result = LibVcx.api.vcx_disclosed_proof_release(proofHandle);
        checkResult(result);

        return result;
    }

    private static Callback vcxProofRetrieveCredentialsCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String matchingCredentials) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], matchingCredentials = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = matchingCredentials;
            future.complete(result);
        }
    };

    public static CompletableFuture<String> proofRetrieveCredentials(
            int proofHandle
    ) throws VcxException {
        logger.debug("proofRetrieveCredentials() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_retrieve_credentials(commandHandle, proofHandle, vcxProofRetrieveCredentialsCB);
        checkResult(result);

        return future;
    }


    private static Callback vcxProofGenerateCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // resolving with no error
            Integer result = 0;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofGenerate(
            int proofHandle,
            String selectedCredentials,
            String selfAttestedAttributes
    ) throws VcxException {
        logger.debug("proofGenerate() called with: proofHandle = [" + proofHandle + "], selectedCredentials = [****], selfAttestedAttributes = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_generate_proof(commandHandle, proofHandle, selectedCredentials, selfAttestedAttributes, vcxProofGenerateCB);
        checkResult(result);

        return future;
    }


    private static Callback vcxProofSendCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // resolving with no error
            Integer result = 0;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofSend(
            int proofHandle,
            int connectionHandle
    ) throws VcxException {
        logger.debug("proofSend() called with: proofHandle = [" + proofHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_send_proof(commandHandle, proofHandle, connectionHandle, vcxProofSendCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofRejectCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // resolving with no error
            Integer result = 0;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofReject(
            int proofHandle,
            int connectionHandle
    ) throws VcxException {
        logger.debug("proofReject() called with: proofHandle = [" + proofHandle + "], connectionHandle = [" + connectionHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_reject_proof(commandHandle, proofHandle, connectionHandle, vcxProofSendCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofGetMsgCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String msg) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], msg = [" + msg + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(msg);
        }
    };

    public static CompletableFuture<String> getProofMsg(
            int proofHandle
    ) throws VcxException {
        logger.debug("getProofMsg() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_get_proof_msg(commandHandle, proofHandle, vcxProofGetMsgCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofGetRejectMsgCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String msg) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], msg = [" + msg + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(msg);
        }
    };

    public static CompletableFuture<String> getRejectMsg(
            int proofHandle
    ) throws VcxException {
        logger.debug("getRejectMsg() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_get_reject_msg(commandHandle, proofHandle, vcxProofGetMsgCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofCreateWithRequestCB = new Callback() {
        public void callback(int command_handle, int err, int proofHandle) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], proofHandle = [" + proofHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(command_handle);
            if(!checkCallback(future, err)) return;
            // resolving with no error
            Integer result = proofHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> proofCreateWithRequest(
            String sourceId,
            String proofRequest
    ) throws VcxException {
        ParamGuard.notNull(sourceId, "sourceId");
        ParamGuard.notNull(proofRequest, "proofRequest");
        logger.debug("proofCreateWithRequest() called with: sourceId = [" + sourceId + "], proofRequest = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_create_with_request(commandHandle, sourceId, proofRequest, vcxProofCreateWithRequestCB);
        checkResult(result);

        return future;
    }


    private static Callback vcxProofSerializeCB = new Callback() {
        public void callback(int command_handle, int err, String serializedProof) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], serializedProof = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(command_handle);
            if(!checkCallback(future, err)) return;

            future.complete(serializedProof);
        }
    };

    public static CompletableFuture<String> proofSerialize(
            int proofHandle
    ) throws VcxException {
        logger.debug("proofSerialize() called with: proofHandle = [" + proofHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_serialize(commandHandle, proofHandle, vcxProofSerializeCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxProofDeserializeCB = new Callback() {
        public void callback(int command_handle, int err, int proofHandle) {
            logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], proofHandle = [" + proofHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(command_handle);
            if(!checkCallback(future, err)) return;

            future.complete(proofHandle);
        }
    };

    public static CompletableFuture<Integer> proofDeserialize(
            String serializedProof
    ) throws VcxException {
        ParamGuard.notNull(serializedProof, "serializedProof");
        logger.debug("proofDeserialize() called with: serializedProof = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_disclosed_proof_deserialize(commandHandle, serializedProof, vcxProofDeserializeCB);
        checkResult(result);

        return future;
    }

	private static Callback vcxDeclinePresentationRequestCB = new Callback() {
		public void callback(int command_handle, int err) {
			logger.debug("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "]");
			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(command_handle);
			if (! checkCallback(future, err)) return;

			future.complete(null);
		}
	};

	public static CompletableFuture<Void> proofDeclineRequest(
			int proofHandle,
			int connectionHandle,
			String reason,
			String proposal
	) throws VcxException {
		logger.debug("declinePresentationRequest() called with: proofHandle = [" + proofHandle + "], connectionHandle = [" + connectionHandle + "], " +
				"reason = [" + reason + "], proposal = [" + proposal + "]");
		CompletableFuture<Void> future = new CompletableFuture<Void>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_disclosed_proof_decline_presentation_request(commandHandle, proofHandle, connectionHandle, reason, proposal, vcxDeclinePresentationRequestCB);
		checkResult(result);

		return future;
	}
}
