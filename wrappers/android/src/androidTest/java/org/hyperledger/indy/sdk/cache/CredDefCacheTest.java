package org.hyperledger.indy.sdk.cache;

import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.junit.Test;

public class CredDefCacheTest extends CacheIntegrationTest {

	@Test(timeout = PoolUtils.TEST_TIMEOUT_FOR_REQUEST_ENSURE)
	public void testGetSchemaWorks() throws Exception {
		postEntities();

		String defaultOptions = "{\"noCache\": false, \"noUpdate\": false, \"noStore\": false, \"minFresh\": -1}";
		String credDef = Cache.getCredDef(pool, wallet, DID, String.valueOf(credDefId), defaultOptions).get();
	}

	@Test
	public void testPurgeCredDefCacheWorks() throws Exception {
	String defaultOptions = "{\"maxAge\": -1}";
		Cache.purgeCredDefCache(wallet, defaultOptions).get();
	}
}
