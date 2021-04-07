package org.hyperledger.indy.sdk.ledger;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.anoncreds.Anoncreds;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageWriter;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.json.JSONObject;
import org.junit.Rule;
import org.junit.rules.ExpectedException;
import org.junit.rules.Timeout;

import java.util.concurrent.TimeUnit;

public class LedgerIntegrationTest extends IndyIntegrationTestWithPoolAndSingleWallet {

	@Rule
	public ExpectedException thrown = ExpectedException.none();

	@Rule
	public Timeout globalTimeout = new Timeout(10, TimeUnit.MINUTES);

	private static Boolean entitiesPosted = false;

	static String schemaId = "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
	static String credDefId = "NcYxiDXkpYi6ov5FcYDi1e:3:CL:1";
	static String revRegDefId = "NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:CL_ACCUM:TAG_1";

	void postEntities() throws Exception {

		if (entitiesPosted) {
			return;
		}

		String myDid = createStoreAndPublishDidFromTrustee();

		// create and post credential schema
		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(myDid, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();
		String schema = createSchemaResult.getSchemaJson();
		schemaId = createSchemaResult.getSchemaId();

		String schemaRequest = Ledger.buildSchemaRequest(myDid, schema).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, schemaRequest).get();

		String getSchemaRequest = Ledger.buildGetSchemaRequest(myDid, schemaId).get();
		String getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, response -> {
			JSONObject getSchemaResponseObject = new JSONObject(response);
			return !getSchemaResponseObject.getJSONObject("result").isNull("seqNo");
		});

		LedgerResults.ParseResponseResult parseSchemaResult = Ledger.parseGetSchemaResponse(getSchemaResponse).get();

		// create and post credential definition
		AnoncredsResults.IssuerCreateAndStoreCredentialDefResult createCredDefResult =
				Anoncreds.issuerCreateAndStoreCredentialDef(wallet, myDid, parseSchemaResult.getObjectJson(), TAG, null, REV_CRED_DEF_CONFIG).get();
		String credDefJson = createCredDefResult.getCredDefJson();
		credDefId = createCredDefResult.getCredDefId();

		String credDefRequest = Ledger.buildCredDefRequest(myDid, credDefJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, credDefRequest).get();

		// create and post revocation registry
		BlobStorageWriter tailsWriter = BlobStorageWriter.openWriter("default", TAILS_WRITER_CONFIG).get();
		String revRegConfig = "{\"issuance_type\":null,\"max_cred_num\":5}";
		AnoncredsResults.IssuerCreateAndStoreRevocRegResult createRevRegResult = Anoncreds.issuerCreateAndStoreRevocReg(wallet, myDid, null, TAG, credDefId, revRegConfig, tailsWriter).get();
		revRegDefId = createRevRegResult.getRevRegId();
		String revRegDef = createRevRegResult.getRevRegDefJson();
		String revRegEntry = createRevRegResult.getRevRegEntryJson();

		String revRegDefRequest = Ledger.buildRevocRegDefRequest(myDid, revRegDef).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, revRegDefRequest).get();

		String revRegEntryRequest = Ledger.buildRevocRegEntryRequest(myDid, revRegDefId, "CL_ACCUM", revRegEntry).get();
		Ledger.signAndSubmitRequest(pool, wallet, myDid, revRegEntryRequest).get();

		entitiesPosted = true;
	}
}
