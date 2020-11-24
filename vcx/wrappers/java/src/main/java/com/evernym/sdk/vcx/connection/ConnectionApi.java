package com.evernym.sdk.vcx.connection;

/**
 * Created by abdussami on 05/06/18.
 */


import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;
import com.sun.jna.Pointer;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

/**
 * Created by abdussami on 03/06/18.
 */

public class ConnectionApi extends VcxJava.API {

	private static final Logger logger = LoggerFactory.getLogger("ConnectionApi");

	private static Callback vcxConnectionCreateCB = new Callback() {
		// TODO: This callback and jna definition needs to be fixed for this API
		// it should accept connection handle as well
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Integer result = connectionHandle;
			future.complete(result);
		}
	};

	public static CompletableFuture<Integer> vcxConnectionCreate(String sourceId) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
		logger.debug("vcxConnectionCreate() called with: sourceId = [ {} ]", sourceId);
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_create(
				commandHandle,
				sourceId,
				vcxConnectionCreateCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxUpdateStateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int s) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], s = [" + s + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Integer result = s;
			future.complete(result);
		}
	};

	public static CompletableFuture<Integer> vcxConnectionUpdateState(int connectionHandle) throws VcxException {
		logger.debug("vcxConnectionUpdateState() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_update_state(
				commandHandle,
				connectionHandle,
				vcxUpdateStateCB
		);
		checkResult(result);
		return future;
	}

	public static CompletableFuture<Integer> vcxConnectionUpdateStateWithMessage(int connectionHandle, String message) throws VcxException {
		logger.debug("vcxConnectionUpdateState() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_update_state_with_message(
				commandHandle,
				connectionHandle,
				message,
				vcxUpdateStateCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxCreateConnectionWithInviteCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			// TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
			Integer result = connectionHandle;
			future.complete(result);
		}
	};

	public static CompletableFuture<Integer> vcxCreateConnectionWithInvite(String invitationId, String inviteDetails) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(invitationId, "invitationId");
		ParamGuard.notNullOrWhiteSpace(inviteDetails, "inviteDetails");
		logger.debug("vcxCreateConnectionWithInvite() called with: invitationId = [" + invitationId + "], inviteDetails = [****]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_create_with_invite(
				commandHandle,
				invitationId,
				inviteDetails,
				vcxCreateConnectionWithInviteCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionConnectCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String inviteDetails) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], inviteDetails = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			// TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
			String result = inviteDetails;
			future.complete(result);
		}
	};

	@Deprecated
	public static CompletableFuture<String> vcxAcceptInvitation(int connectionHandle, String connectionType) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNullOrWhiteSpace(connectionType, "connectionType");
		return vcxConnectionConnect(connectionHandle, connectionType);
	}

	public static CompletableFuture<String> vcxConnectionConnect(int connectionHandle, String connectionType) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNullOrWhiteSpace(connectionType, "connectionType");
		logger.debug("vcxAcceptInvitation() called with: connectionHandle = [" + connectionHandle + "], connectionType = [" + connectionType + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_connect(
				commandHandle,
				connectionHandle,
				connectionType,
				vcxConnectionConnectCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionRedirectCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			future.complete(0);
		}
	};

	public static CompletableFuture<Integer> vcxConnectionRedirect(int connectionHandle, int redirectConnectionHandle) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		ParamGuard.notNull(redirectConnectionHandle, "redirectConnectionHandle");
		logger.debug("vcxConnectionRedirect() called with: connectionHandle = [" + connectionHandle + "], redirectConnectionHandle = [" + redirectConnectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_redirect(
				commandHandle,
				connectionHandle,
				redirectConnectionHandle,
				vcxConnectionRedirectCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionGetRedirectDetailsCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String redirectDetails) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], redirectDetails = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			String result = redirectDetails;
			future.complete(result);
		}
	};

	public static CompletableFuture<String> vcxConnectionGetRedirectDetails(int connectionHandle) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		logger.debug("vcxConnectionGetRedirectDetails() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_get_redirect_details(
				commandHandle,
				connectionHandle,
				vcxConnectionGetRedirectDetailsCB
		);
		checkResult(result);
		return future;
	}


	private static Callback vcxConnectionSerializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String serializedData) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], serializedData = [" + serializedData + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			// TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
			future.complete(serializedData);
		}
	};

	public static CompletableFuture<String> connectionSerialize(int connectionHandle) throws VcxException {
		ParamGuard.notNull(connectionHandle, "connectionHandle");
		logger.debug("connectionSerialize() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_serialize(
				commandHandle,
				connectionHandle,
				vcxConnectionSerializeCB
		);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionDeserializeCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int connectionHandle) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], connectionHandle = [" + connectionHandle + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			// TODO complete with exception if we find error
//            if (err != 0) {
//                future.completeExceptionally();
//            } else {
//
//            }
			future.complete(connectionHandle);
		}
	};

	public static CompletableFuture<Integer> connectionDeserialize(String connectionData) throws VcxException {
		ParamGuard.notNull(connectionData, "connectionData");
		logger.debug("connectionDeserialize() called with: connectionData = [****]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_deserialize(
				commandHandle,
				connectionData,
				vcxConnectionDeserializeCB
		);
		checkResult(result);
		return future;
	}


	private static Callback vcxConnectionDeleteCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(0);
		}
	};

	public static CompletableFuture<Integer> deleteConnection(int connectionHandle) throws VcxException {
		logger.debug("deleteConnection() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_delete_connection(commandHandle, connectionHandle, vcxConnectionDeleteCB);
		checkResult(result);
		return future;
	}

	private static Callback vcxConnectionInviteDetailsCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String details) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], details = [****]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (!checkCallback(future, err)) return;
			future.complete(details);
		}
	};

	public static CompletableFuture<String> connectionInviteDetails(int connectionHandle, int abbreviated) throws VcxException {
		logger.debug("connectionInviteDetails() called with: connectionHandle = [" + connectionHandle + "], abbreviated = [****]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);
		int result = LibVcx.api.vcx_connection_invite_details(commandHandle, connectionHandle, abbreviated, vcxConnectionInviteDetailsCB);
		checkResult(result);
		return future;
	}


	public static int connectionRelease(int handle) throws VcxException {
		logger.debug("connectionRelease() called with: handle = [" + handle + "]");
		ParamGuard.notNull(handle, "handle");
		int result = LibVcx.api.vcx_connection_release(handle);
		checkResult(result);

		return result;
	}

	private static Callback vcxConnectionGetStateCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int state) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], state = [" + state + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(state);
		}
	};

	public static CompletableFuture<Integer> connectionGetState(int connnectionHandle) throws VcxException {
		logger.debug("connectionGetState() called with: connnectionHandle = [" + connnectionHandle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);
		int result = LibVcx.api.vcx_connection_get_state(commandHandle, connnectionHandle, vcxConnectionGetStateCB);
		checkResult(result);
		return future;
	}

	private static Callback voidCb = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
			CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Void result = null;
			future.complete(result);
		}
	};

	public static CompletableFuture<Void> connectionSendPing(
			int connectionHandle,
			String comment
	) throws VcxException {
		logger.debug("sendPing() called with: connectionHandle = [" + connectionHandle + "], comment = [" + comment + "]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_ping(commandHandle, connectionHandle, comment, voidCb);
		checkResult(result);

		return future;
	}

	public static CompletableFuture<Void> connectionSendDiscoveryFeatures(
			int connectionHandle,
			String query,
			String comment
	) throws VcxException {
		logger.debug("connectionSendDiscoveryFeatures() called with: connectionHandle = [" + connectionHandle + "], query = [" + query + "], comment = [" + comment + "]");
		CompletableFuture<Void> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_connection_send_discovery_features(commandHandle, connectionHandle, query, comment, voidCb);
		checkResult(result);

		return future;
	}

    private static Callback vcxConnectionSendMessageCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String msgId) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], msgId = [" + msgId + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(msgId);
        }
    };

    public static CompletableFuture<String> connectionSendMessage(int connectionHandle, String message, String sendMessageOptions) throws VcxException {
        logger.debug("connectionSendMessage() called with: connectionHandle = [" + connectionHandle + "], message = [****], sendMessageOptions = [" + sendMessageOptions + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_send_message(commandHandle, connectionHandle, message, sendMessageOptions, vcxConnectionSendMessageCB);
        checkResult(result);
        return future;
    }


    private static Callback vcxConnectionSignDataCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, Pointer signature_raw, int signature_len) {

            CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
            if (! checkResult(future, err)) return;

            byte[] encryptedMsg = new byte[signature_len];
            signature_raw.read(0, encryptedMsg, 0, signature_len);

            future.complete(encryptedMsg);
        }
    };


    public static CompletableFuture<byte[]> connectionSignData(int connectionHandle, byte[] data, int dataLength) throws VcxException {

        ParamGuard.notNull(data, "data");

        CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_sign_data(commandHandle, connectionHandle, data, dataLength, vcxConnectionSignDataCB);
        checkResult(future, result);

        return future;
    }

    private static Callback vcxConnectionVerifySignatureCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, boolean valid) {

            CompletableFuture<Boolean> future = (CompletableFuture<Boolean>) removeFuture(xcommand_handle);
            if (! checkResult(future, err)) return;

            future.complete(valid);
        }
    };


    public static CompletableFuture<Boolean> connectionVerifySignature(int connectionHandle, byte[] data, int dataLength, byte[] signature, int signatureLength) throws VcxException {

        ParamGuard.notNull(data, "data");
        ParamGuard.notNull(signature, "signature");

        CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_verify_signature(commandHandle, connectionHandle, data, dataLength, signature, signatureLength, vcxConnectionVerifySignatureCB);
        checkResult(future, result);

        return future;
    }

    private static Callback vcxConnectionGetPwDidCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, String pwDid) {

            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
            if (! checkCallback(future, err)) return;

            future.complete(pwDid);
        }
    };

    public static CompletableFuture<String> connectionGetPwDid(int connectionHandle) throws VcxException {

        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_get_pw_did(commandHandle, connectionHandle, vcxConnectionGetPwDidCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxConnectionGetTheirPwDidCB = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, String theirPwDid) {

            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommand_handle);
            if (! checkCallback(future, err)) return;

            future.complete(theirPwDid);
        }
    };

    public static CompletableFuture<String> connectionGetTheirPwDid(int connectionHandle) throws VcxException {

        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_connection_get_their_pw_did(commandHandle, connectionHandle, vcxConnectionGetTheirPwDidCB);
        checkResult(result);

        return future;
    }

	private static Callback vcxConnectionInfoCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, String info) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], info = [" + info + "]");
			CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			future.complete(info);
		}
	};

	public static CompletableFuture<String> connectionInfo(int connectionHandle) throws VcxException {
		logger.debug("connectionInfo() called with: connectionHandle = [" + connectionHandle + "]");
		CompletableFuture<String> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);
		int result = LibVcx.api.vcx_connection_info(commandHandle, connectionHandle, vcxConnectionInfoCB);
		checkResult(result);
		return future;
	}
}
