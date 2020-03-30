package com.evernym.sdk.vcx;

import com.evernym.sdk.vcx.connection.ConnectionApi;
import com.evernym.sdk.vcx.connection.InvalidConnectionHandleException;
import com.evernym.sdk.vcx.proof.InvalidProofHandleException;
import com.evernym.sdk.vcx.proof.DisclosedProofApi;
import com.evernym.sdk.vcx.vcx.VcxApi;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.concurrent.ExecutionException;

public class DisclosedProofApiTest {
    private String sourceId = "123";
    private String name = "proof name";
    private String proofRequest = "{\"@topic\": {\"mid\": 9,\"tid\": 1},\"@type\": {\"name\": \"PROOF_REQUEST\",\"version\":\"1.0\"},\"msg_ref_id\": \"ymy5nth\",\"proof_request_data\": {\"name\": \"Account Certificate\", \"nonce\": \"838186471541979035208225\", \"requested_attributes\": { \"business_2\": { \"name\": \"business\" }, \"email_1\": { \"name\": \"email\" }, \"name_0\": { \"name\": \"name\" } }, \"requested_predicates\": {}, \"version\": \"0.1\" } }";

    @BeforeEach
    void setup() throws Exception {
        System.setProperty(org.slf4j.impl.SimpleLogger.DEFAULT_LOG_LEVEL_KEY, "DEBUG");
        if (!TestHelper.vcxInitialized) {
            TestHelper.getResultFromFuture(VcxApi.vcxInit(TestHelper.VCX_CONFIG_TEST_MODE));
            TestHelper.vcxInitialized = true;
        }
    }

    @Test
    @DisplayName("create a disclosedProof")
    void createDisclosedProof() throws VcxException, ExecutionException, InterruptedException {
        int result = TestHelper.getResultFromFuture(DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest));
        assert (result != 0);
   }

    @Test
    @DisplayName("throw illegal argument exception if invalid arguments are provided")
    void throwIllegalArgumentxException() {
        Assertions.assertThrows(IllegalArgumentException.class, () -> {
            TestHelper.getResultFromFuture(DisclosedProofApi.proofCreate(sourceId, null, "{}", name));
        });
    }

    @Test
    @DisplayName("serialize and deserialize proof")
    void serializeDisclosedProof() throws VcxException, ExecutionException, InterruptedException {
        int proofHandle = TestHelper.getResultFromFuture(DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest));
        assert (proofHandle != 0);
        String serializedProof = TestHelper.getResultFromFuture(DisclosedProofApi.proofSerialize(proofHandle));
        assert (serializedProof.contains(sourceId));
        int handle = TestHelper.getResultFromFuture(DisclosedProofApi.proofDeserialize(serializedProof));
        assert (handle != 0);
    }

    @Test
    @DisplayName("update state of proof")
    void updateState() throws VcxException, ExecutionException, InterruptedException {
        int proofHandle = TestHelper.getResultFromFuture(DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest));
        assert (proofHandle != 0);
        int result = TestHelper.getResultFromFuture(DisclosedProofApi.proofUpdateState(proofHandle));
        System.out.println("result == " + result);
        assert(result==3);
    }

    @Test
    @DisplayName("get proof message")
    void getProofMessage() throws VcxException, ExecutionException, InterruptedException {
        int proofHandle = TestHelper.getResultFromFuture(DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest));
        assert (proofHandle != 0);
        String msg = TestHelper.getResultFromFuture(DisclosedProofApi.getProofMsg(proofHandle));
        assert (msg.length() > 0);
    }

    @Test
    @DisplayName("decline request")
    void declineRequest() throws VcxException, ExecutionException, InterruptedException {
        int proofHandle = TestHelper.getResultFromFuture(DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest));
        Assertions.assertThrows(InvalidConnectionHandleException.class, ()-> {
            TestHelper.getResultFromFuture(DisclosedProofApi.proofDeclineRequest(proofHandle, 0, null, null));
        });
    }

    @Test
    @DisplayName("get reject message")
    void getRejectMessage() throws VcxException, ExecutionException, InterruptedException {
        int proofHandle = TestHelper.getResultFromFuture(DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest));
        assert (proofHandle != 0);
        String msg = TestHelper.getResultFromFuture(DisclosedProofApi.getRejectMsg(proofHandle));
        assert (msg.length() > 0);
    }

}
