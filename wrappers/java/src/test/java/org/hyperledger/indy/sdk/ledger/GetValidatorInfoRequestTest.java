package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Assert;
import org.junit.Test;

public class GetValidatorInfoRequestTest extends IndyIntegrationTestWithPoolAndSingleWallet {
    @Test
    public void testBuildGetValidatorInfoRequestWorks() throws Exception {
        JSONObject expectedResult = new JSONObject()
                .put("identifier", DID)
                .put("operation",
                        new JSONObject()
                                .put("type", "119")
                );

        String getValidatorInfoRequest = Ledger.buildGetValidatorInfoRequest(DID).get();
        assert (new JSONObject(getValidatorInfoRequest).toMap().entrySet()
                .containsAll(
                        expectedResult.toMap().entrySet()));
    }

    @Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
    public void testGetValidatorInfoRequestWorks() throws Exception {
        String did = Did.createAndStoreMyDid(this.wallet, new JSONObject().put("seed", TRUSTEE_SEED).toString()).get().getDid();

        String getValidatorInfoRequest = Ledger.buildGetValidatorInfoRequest(did).get();
        String getValidatorInfoResponse = Ledger.signAndSubmitRequest(pool, wallet, did, getValidatorInfoRequest).get();

        JSONObject getValidatorInfoObj = new JSONObject(getValidatorInfoResponse);

        for (int i = 1; i <= 4; i++) {
            Assert.assertFalse(new JSONObject(getValidatorInfoObj.getString(String.format("Node%s", i))).getJSONObject("result").isNull("data"));
        }
    }
}
