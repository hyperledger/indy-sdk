package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.*;

public class GetResponseMetadata extends IndyIntegrationTestWithPoolAndSingleWallet {
    @Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
    public void testGetResponseMetadataWorksForNymRequests() throws Exception {
        DidResults.CreateAndStoreMyDidResult trusteeDidResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
        String trusteeDid = trusteeDidResult.getDid();

        DidResults.CreateAndStoreMyDidResult myDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
        String did = myDidResult.getDid();

        String nymRequest = Ledger.buildNymRequest(trusteeDid, did, null, null, null).get();
        String nymResponse = Ledger.signAndSubmitRequest(pool, wallet, trusteeDid, nymRequest).get();

        String metadata = Ledger.getResponseMetadata(nymResponse).get();
        JSONObject metadataObj = new JSONObject(metadata);
        assertTrue(metadataObj.has("seqNo"));
        assertTrue(metadataObj.has("txnTime"));
        assertFalse(metadataObj.has("lastTxnTime"));
        assertFalse(metadataObj.has("lastSeqNo"));

        String getNymRequest = Ledger.buildGetNymRequest(did, did).get();
        String getNymResponse = PoolUtils.ensurePreviousRequestApplied(pool, getNymRequest, response -> {
            JSONObject getSchemaResponseObject = new JSONObject(response);
            return ! getSchemaResponseObject.getJSONObject("result").isNull("seqNo");
        });

        metadata = Ledger.getResponseMetadata(getNymResponse).get();
        metadataObj = new JSONObject(metadata);
        assertTrue(metadataObj.has("seqNo"));
        assertTrue(metadataObj.has("txnTime"));
        assertTrue(metadataObj.has("lastTxnTime"));
        assertFalse(metadataObj.has("lastSeqNo"));
    }
}
