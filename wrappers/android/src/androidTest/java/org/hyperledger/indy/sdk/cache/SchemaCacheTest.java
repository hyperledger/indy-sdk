package org.hyperledger.indy.sdk.cache;

import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.*;

import java.util.concurrent.ExecutionException;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertTrue;

public class SchemaCacheTest extends CacheIntegrationTest {

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testGetSchemaWorks() throws Exception {
		postEntities();

		String defaultOptions = "{\"noCache\": false, \"noUpdate\": false, \"noStore\": false, \"minFresh\": -1}";

		String schema = Cache.getSchema(pool, wallet, DID, String.valueOf(schemaId), defaultOptions).get();
	}

	@Test
	public void testPurgeSchemaCacheWorks() throws Exception {
	    String defaultOptions = "{\"maxAge\": -1}";
		Cache.purgeSchemaCache(wallet, defaultOptions).get();
	}
}
