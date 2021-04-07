package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.junit.Ignore;
import org.junit.Test;

import java.util.Calendar;

import static org.junit.Assert.assertTrue;

public class PoolRestartRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {
    @Test
    public void testBuildPoolRestartRequestWorksForStartAction() throws Exception {
        String expectedResult = String.format("\"identifier\":\"%s\"," +
                "\"operation\":{\"type\":\"118\"," +
                "\"action\":\"start\"," +
                "\"datetime\":\"0\"}", DID);

        String request = Ledger.buildPoolRestartRequest(DID, "start","0").get();

        assertTrue(request.contains(expectedResult));
    }

    @Test
    public void testBuildPoolRestartRequestWorksForCancelAction() throws Exception {
        String expectedResult = String.format("\"identifier\":\"%s\"," +
                "\"operation\":{\"type\":\"118\"," +
                "\"action\":\"cancel\"}", DID);

        String request = Ledger.buildPoolRestartRequest(DID, "cancel",null).get();

        assertTrue(request.contains(expectedResult));
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
