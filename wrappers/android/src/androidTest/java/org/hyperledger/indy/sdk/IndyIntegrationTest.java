package org.hyperledger.indy.sdk;

import org.hyperledger.indy.sdk.crypto.CryptoJSONParameters;
import org.hyperledger.indy.sdk.did.DidJSONParameters;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.utils.AndroidInitHelper;
import org.hyperledger.indy.sdk.utils.StorageUtils;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.ExpectedException;
import org.junit.rules.Timeout;
import org.junit.runner.RunWith;

import java.io.IOException;
import java.util.HashSet;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;

import androidx.test.ext.junit.runners.AndroidJUnit4;

import static org.hyperledger.indy.sdk.utils.EnvironmentUtils.getIndyHomePath;
import static org.hyperledger.indy.sdk.utils.EnvironmentUtils.getTmpPath;

@RunWith(AndroidJUnit4.class)
public class IndyIntegrationTest {

	protected static final String TRUSTEE_SEED = "000000000000000000000000Trustee1";
	protected static final String MY1_SEED = "00000000000000000000000000000My1";
	protected static final String MY2_SEED = "00000000000000000000000000000My2";
	protected static final String VERKEY = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
	protected static final String VERKEY_MY1 = "GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa";
	protected static final String VERKEY_MY2 = "kqa2HyagzfMAq42H5f9u3UMwnSBPQx2QfrSyXbUPxMn";
	protected static final String VERKEY_TRUSTEE = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL";
	protected static final String INVALID_VERKEY = "CnEDk___MnmiHXEV1WFgbV___eYnPqs___TdcZaNhFVW";
	protected static final String DID = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
	protected static final String DID_MY1 = "VsKV7grR1BUE29mG2Fm2kX";
	protected static final String DID_MY2 = "2PRyVHmkXQnQzJQKxHxnXC";
	protected static final String DID_TRUSTEE = "V4SGRU86Z58d6TV7PBUe6f";
	protected static final String INVALID_DID = "invalid_base58string";
	protected static final String IDENTITY_JSON_TEMPLATE = "{\"did\":\"%s\",\"verkey\":\"%s\"}";
	protected static final byte[] MESSAGE = "{\"reqId\":1496822211362017764}".getBytes();
	protected static final String SCHEMA_DATA = "{\"id\":\"1\", \"name\":\"gvt\",\"version\":\"1.0\",\"attrNames\":[\"name\"],\"ver\":\"1.0\"}";
	protected static final String WALLET = "Wallet1";
	protected static final String TYPE = "default";
	protected static final String METADATA = "some metadata";
	protected static final String ENDPOINT = "127.0.0.1:9700";
	protected static final String CRYPTO_TYPE = "ed25519";
	protected byte[] SIGNATURE = {20, - 65, 100, - 43, 101, 12, - 59, - 58, - 53, 49, 89, - 36, - 51, - 64, - 32, - 35, 97, 77, - 36, - 66, 90, 60, - 114, 23, 16, - 16, - 67, - 127, 45, - 108, - 11, 8, 102, 95, 95, - 7, 100, 89, 41, - 29, - 43, 25, 100, 1, - 24, - 68, - 11, - 21, - 70, 21, 52, - 80, - 20, 11, 99, 70, - 101, - 97, 89, - 41, - 59, - 17, - 118, 5};
	protected byte[] ENCRYPTED_MESSAGE = {- 105, 30, 89, 75, 76, 28, - 59, - 45, 105, - 46, 20, 124, - 85, - 13, 109, 29, - 88, - 82, - 8, - 6, - 50, - 84, - 53, - 48, - 49, 56, 124, 114, 82, 126, 74, 99, - 72, - 78, - 117, 96, 60, 119, 50, - 40, 121, 21, 57, - 68, 89};
	protected String DEFAULT_CRED_DEF_CONFIG = "{\"support_revocation\":false}";
	protected String TAG = "tag1";
	protected String GVT_SCHEMA_NAME = "gvt";
	protected String XYZ_SCHEMA_NAME = "xyz";
	protected String SCHEMA_VERSION = "1.0";
	protected String GVT_SCHEMA_ATTRIBUTES = "[\"name\", \"age\", \"sex\", \"height\"]";
	protected String XYZ_SCHEMA_ATTRIBUTES = "[\"status\", \"period\"]";
	protected String REVOC_REG_TYPE = "CL_ACCUM";
	protected String SIGNATURE_TYPE = "CL";
	protected String TAILS_WRITER_CONFIG =
			"{ \"base_dir\":\"" +  getIndyHomePath("tails").replace('\\', '/') + "\", \"uri_pattern\":\"\"}";
	protected String REV_CRED_DEF_CONFIG = "{\"support_revocation\":true}";
	// note that encoding is not standardized by Indy except that 32-bit integers are encoded as themselves. IS-786
	protected String GVT_CRED_VALUES = "{\n" +
			"        \"sex\": {\"raw\": \"male\", \"encoded\": \"5944657099558967239210949258394887428692050081607692519917050\"},\n" +
			"        \"name\": {\"raw\": \"Alex\", \"encoded\": \"1139481716457488690172217916278103335\"},\n" +
			"        \"height\": {\"raw\": \"175\", \"encoded\": \"175\"},\n" +
			"        \"age\": {\"raw\": \"28\", \"encoded\": \"28\"}\n" +
			"    }";
	protected static final String WALLET_CONFIG = "{ \"id\":\"" + WALLET + "\", \"storage_type\":\"" + TYPE + "\"}";

	protected static final String WALLET_CREDENTIALS = "{\"key\":\"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY\", \"key_derivation_method\":\"RAW\"}";

	protected int PROTOCOL_VERSION = 2;


	protected static final String TRUSTEE_IDENTITY_JSON =
			new DidJSONParameters.CreateAndStoreMyDidJSONParameter(null, TRUSTEE_SEED, null, null).toJson();

	protected static final String MY1_IDENTITY_JSON =
			new DidJSONParameters.CreateAndStoreMyDidJSONParameter(null, MY1_SEED, null, null).toJson();

	protected static final String MY1_IDENTITY_KEY_JSON =
			new CryptoJSONParameters.CreateKeyJSONParameter(MY1_SEED, null).toJson();

	private static final String EXPORT_KEY = "export_key";
	protected static final String EXPORT_PATH = getTmpPath("export_wallet");
	protected static final String EXPORT_CONFIG_JSON = "{ \"key\":\"" + EXPORT_KEY + "\", \"path\":\"" + EXPORT_PATH + "\"}";
	protected static final JSONObject REQUEST = new JSONObject();

	static {
		try {
			REQUEST.put("reqId", 149682221)
						.put("identifier", "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL")
						.put("operation", new JSONObject()
								.put("type", "1")
								.put("dest", "VsKV7grR1BUE29mG2Fm2kX")
								.put("verkey", "GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa")
				);
		} catch (JSONException e) {
			e.printStackTrace();
		}
	}

	@Rule
	public ExpectedException thrown = ExpectedException.none();

	@Rule
	public Timeout globalTimeout = new Timeout(10, TimeUnit.MINUTES);

	@Before
	public void setUp() throws Exception {
		StorageUtils.cleanupStorage();

		AndroidInitHelper.init();
		Pool.setProtocolVersion(PROTOCOL_VERSION).get();
	}

	protected HashSet<Pool> openedPools = new HashSet<>();

	@After
	public void tearDown() throws IOException {
		openedPools.forEach(pool -> {
			try {
				pool.closePoolLedger().get();
			} catch (IndyException | InterruptedException | ExecutionException ignore) {
			}
		});
		openedPools.clear();
		StorageUtils.cleanupStorage();
	}

	@Test
	public void testSetRuntimeConfig() throws Exception {
		LibIndy.setRuntimeConfig("{\"crypto_thread_pool_size\": 2}");
	}
}
