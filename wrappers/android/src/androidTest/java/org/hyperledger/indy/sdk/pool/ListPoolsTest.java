package org.hyperledger.indy.sdk.pool;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONArray;
import org.junit.Test;

import java.io.File;

import static org.junit.Assert.assertEquals;

public class ListPoolsTest extends IndyIntegrationTest {

    @Test
    public void testListPoolsWorks() throws Exception {
        String testPoolName = "testListPoolsWorks";
        File genesisTxnFile = PoolUtils.createGenesisTxnFile(testPoolName);
        PoolJSONParameters.CreatePoolLedgerConfigJSONParameter createPoolLedgerConfigJSONParameter
                = new PoolJSONParameters.CreatePoolLedgerConfigJSONParameter(genesisTxnFile.getAbsolutePath());

        Pool.createPoolLedgerConfig(testPoolName, createPoolLedgerConfigJSONParameter.toJson()).get();
        String listPoolsJson = Pool.listPools().get();

        JSONArray listPools = new JSONArray(listPoolsJson);

        assertEquals(1, listPools.length());
        assertEquals(testPoolName, listPools.getJSONObject(0).getString("pool"));
    }

}
