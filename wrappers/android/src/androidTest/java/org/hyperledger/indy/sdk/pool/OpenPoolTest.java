package org.hyperledger.indy.sdk.pool;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.pool.PoolJSONParameters.OpenPoolLedgerJSONParameter;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.isA;
import static org.junit.Assert.assertNotNull;

public class OpenPoolTest extends IndyIntegrationTest {

	@Test
	public void testOpenPoolWorksForNullConfig() throws Exception {
		String poolName = PoolUtils.createPoolLedgerConfig();

		Pool pool = Pool.openPoolLedger(poolName, null).get();

		assertNotNull(pool);
		openedPools.add(pool);
	}

	@Test
	public void testOpenPoolWorksForConfig() throws Exception {
		String poolName = PoolUtils.createPoolLedgerConfig();

		OpenPoolLedgerJSONParameter config = new OpenPoolLedgerJSONParameter(20, 80);
		Pool pool = Pool.openPoolLedger(poolName, config.toJson()).get();

		assertNotNull(pool);
		openedPools.add(pool);
	}

	@Test
	public void testOpenPoolWorksForTwice() throws Exception {
		thrown.expectCause(isA(InvalidPoolException.class));

		String poolName = PoolUtils.createPoolLedgerConfig();

		Pool pool = Pool.openPoolLedger(poolName, null).get();
		assertNotNull(pool);
		openedPools.add(pool);

		Pool.openPoolLedger(poolName, null).get();
	}

	@Test
	public void testOpenPoolWorksForIncompatibleProtocolVersion() throws Exception {
		thrown.expectCause(isA(PoolIncompatibleProtocolVersionException.class));

		Pool.setProtocolVersion(1).get();

		String poolName = PoolUtils.createPoolLedgerConfig();

		Pool.openPoolLedger(poolName, null).get();

		Pool.setProtocolVersion(PROTOCOL_VERSION).get();
	}
}
