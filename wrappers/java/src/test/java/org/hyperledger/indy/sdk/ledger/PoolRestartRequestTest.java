package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.json.JSONObject;
import org.junit.Ignore;
import org.junit.Test;

import java.util.Calendar;

import static org.junit.Assert.assertTrue;

public class PoolRestartRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {
    @Test
    public void testBuildPoolRestartRequestWorksForStartAction() throws Exception {
        JSONObject expectedResult = new JSONObject()
                .put("identifier", DID)
                .put("operation",
                        new JSONObject()
                                .put("type", "118")
                                .put("action", "start")
                                .put("datetime", "0")
                );

        String request = Ledger.buildPoolRestartRequest(DID, "start","0").get();
        assert (new JSONObject(request).toMap().entrySet()
                .containsAll(
                        expectedResult.toMap().entrySet()));
    }

    @Test
    public void testBuildPoolRestartRequestWorksForCancelAction() throws Exception {
        JSONObject expectedResult = new JSONObject()
                .put("identifier", DID)
                .put("operation",
                        new JSONObject()
                                .put("type", "118")
                                .put("action", "cancel")
                );

        String request = Ledger.buildPoolRestartRequest(DID, "cancel",null).get();
        assert (new JSONObject(request).toMap().entrySet()
                .containsAll(
                        expectedResult.toMap().entrySet()));
    }

    @Test
    @Ignore
    public void testPoolRestartRequestWorks() throws Exception {
        int nextYear = Calendar.getInstance().get(Calendar.YEAR) + 1;

        DidResults.CreateAndStoreMyDidResult didResult = Did.createAndStoreMyDid(wallet, TRUSTEE_IDENTITY_JSON).get();
        String did = didResult.getDid();

        //start
        String datetime = String.format("\"%s-01-25T12:49:05.258870+00:00\"", nextYear);
        String request = Ledger.buildPoolRestartRequest(did, "start", datetime).get();
        Ledger.signAndSubmitRequest(pool, wallet, did, request).get();

        //cancel
        request = Ledger.buildPoolRestartRequest(did, "cancel", null).get();
        Ledger.signAndSubmitRequest(pool, wallet, did, request).get();
    }
}
