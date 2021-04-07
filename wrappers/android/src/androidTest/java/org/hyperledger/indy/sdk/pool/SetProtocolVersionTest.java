package org.hyperledger.indy.sdk.pool;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.isA;

public class SetProtocolVersionTest extends IndyIntegrationTest {

	@Test
	public void testSetProtocolVersionWorks() throws Exception {
		Pool.setProtocolVersion(PROTOCOL_VERSION).get();
	}

	@Test
	public void testSetProtocolVersionWorksForUnsupported() throws Exception {
		thrown.expectCause(isA(PoolIncompatibleProtocolVersionException.class));

		Pool.setProtocolVersion(0).get();
	}
}
