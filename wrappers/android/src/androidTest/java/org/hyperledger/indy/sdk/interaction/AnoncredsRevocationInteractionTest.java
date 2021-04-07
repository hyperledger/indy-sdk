package org.hyperledger.indy.sdk.interaction;

import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.anoncreds.Anoncreds;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults;
import org.hyperledger.indy.sdk.blob_storage.*;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.ledger.Ledger;
import org.hyperledger.indy.sdk.ledger.LedgerResults;
import org.hyperledger.indy.sdk.ledger.LedgerResults.ParseResponseResult;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONException;
import org.json.JSONObject;
import org.json.JSONArray;
import org.junit.*;
import org.junit.rules.Timeout;

import java.util.concurrent.TimeUnit;

import static org.hyperledger.indy.sdk.utils.EnvironmentUtils.getIndyHomePath;

public class AnoncredsRevocationInteractionTest extends IndyIntegrationTestWithPoolAndSingleWallet {
	private final String COMMON_MASTER_SECRET = "common_master_secret_name";

	@Rule
	public Timeout globalTimeout = new Timeout(10	, TimeUnit.MINUTES);
	private Wallet proverWallet;
	private String proverWalletConfig = new JSONObject().put("id", "proverWallet").toString();

	public AnoncredsRevocationInteractionTest() throws JSONException {
	}

	@Before
	public void createProverWallet() throws Exception {
		Wallet.createWallet(proverWalletConfig, WALLET_CREDENTIALS).get();
		proverWallet = Wallet.openWallet(proverWalletConfig, WALLET_CREDENTIALS).get();
	}

	@After
	public void deleteWalletWallet() throws Exception {
		proverWallet.closeWallet().get();
		Wallet.deleteWallet(proverWalletConfig, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testAnoncredsRevocationInteractionIssuanceByDemand() throws Exception {
		// Issuer create DID
		DidResults.CreateAndStoreMyDidResult trusteeDidInfo = Did.createAndStoreMyDid(this.wallet, new JSONObject().put("seed", TRUSTEE_SEED).toString()).get();
		DidResults.CreateAndStoreMyDidResult issuerDidInfo = Did.createAndStoreMyDid(this.wallet, "{}").get();
		String nymRequest = Ledger.buildNymRequest(trusteeDidInfo.getDid(), issuerDidInfo.getDid(),
				issuerDidInfo.getVerkey(), null, "TRUSTEE").get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDidInfo.getDid(), nymRequest).get();

		String issuerDid = issuerDidInfo.getDid();

		// Prover create DID
		DidResults.CreateAndStoreMyDidResult proverDidInfo = Did.createAndStoreMyDid(proverWallet,"{}").get();

		String proverDid = proverDidInfo.getDid();
		String proverVerkey = proverDidInfo.getVerkey();

		// Issuer publish Prover DID
		nymRequest = Ledger.buildNymRequest(issuerDid, proverDid, proverVerkey, null, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, nymRequest).get();

		// ISSUER post to Ledger Schema, CredentialDefinition, RevocationRegistry

		// Issuer creates Schema
		AnoncredsResults.IssuerCreateSchemaResult schemaInfo =
				Anoncreds.issuerCreateSchema(issuerDidInfo.getDid(), GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();

		String schemaJson = schemaInfo.getSchemaJson();

		// Issuer posts Schema to Ledger
		String schemaRequest = Ledger.buildSchemaRequest(issuerDid, schemaJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, schemaRequest).get();

		// Issuer get Schema from Ledger
		String getSchemaRequest = Ledger.buildGetSchemaRequest(issuerDid, schemaInfo.getSchemaId()).get();
		String getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, response -> {
			JSONObject getSchemaResponseObject = new JSONObject(response);
			return ! getSchemaResponseObject.getJSONObject("result").isNull("seqNo");
		});

		// !!IMPORTANT!!
		// It is important to get Schema from Ledger and parse it to get the correct schema JSON and correspondent id in Ledger
		// After that we can create CredentialDefinition for received Schema(not for result of indy_issuer_create_schema)

		ParseResponseResult schemaInfo1 = Ledger.parseGetSchemaResponse(getSchemaResponse).get();
		schemaJson = schemaInfo1.getObjectJson();

		// Issuer creates CredentialDefinition
		AnoncredsResults.IssuerCreateAndStoreCredentialDefResult credDefInfo =
				Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid, schemaJson,
						TAG, null, new JSONObject().put("support_revocation", true).toString()).get();

		String credDefId = credDefInfo.getCredDefId();
		String credDefJson = credDefInfo.getCredDefJson();


		// Issuer post CredentialDefinition to Ledger
		String credDefRequest = Ledger.buildCredDefRequest(issuerDid, credDefJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, credDefRequest).get();

		// Issuer creates RevocationRegistry
		/* FIXME: getIndyHomePath hard coded forward slash "/". It will not work for Windows. */
		String tailsWriterConfig = new JSONObject(String.format("{\"base_dir\":\"%s\", \"uri_pattern\":\"\"}",
				getIndyHomePath("tails")).replace('\\', '/')).toString();
		BlobStorageWriter tailsWriterHandle = BlobStorageWriter.openWriter("default", tailsWriterConfig).get();

		AnoncredsResults.IssuerCreateAndStoreRevocRegResult revRegInfo =
				Anoncreds.issuerCreateAndStoreRevocReg(wallet, issuerDid, null, TAG,
						credDefId,
						new JSONObject().put("max_cred_num", 5).put("issuance_type", "ISSUANCE_ON_DEMAND").toString(),
						tailsWriterHandle).get();

		String revRegId = revRegInfo.getRevRegId();
		String revRegDefJson = revRegInfo.getRevRegDefJson();
		String revRegEntryJson = revRegInfo.getRevRegEntryJson();

		// Issuer posts RevocationRegistryDefinition to Ledger
		String revRegDefRequest = Ledger.buildRevocRegDefRequest(issuerDid, revRegDefJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegDefRequest).get();

		// Issuer posts RevocationRegistryEntry to Ledger
		String revRegEntryRequest = Ledger.buildRevocRegEntryRequest(issuerDid, revRegId,
				REVOC_REG_TYPE, revRegEntryJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegEntryRequest).get();

		// Issuance Credential for Prover

		// Prover creates Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, COMMON_MASTER_SECRET).get();

		// Issuer creates Credential Offer
		String credOfferJson = Anoncreds.issuerCreateCredentialOffer(wallet, credDefId).get();

		// Prover gets CredentialDefinition from Ledger
		JSONObject credOffer = new JSONObject(credOfferJson);
		String getCredDefRequest = Ledger.buildGetCredDefRequest(proverDid, credOffer.getString("cred_def_id")).get();

		String getCredDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getCredDefRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult credDefIdInfo = Ledger.parseGetCredDefResponse(getCredDefResponse).get();

		credDefId = credDefIdInfo.getId();
		credDefJson = credDefIdInfo.getObjectJson();

		// Prover creates Credential Request
		AnoncredsResults.ProverCreateCredentialRequestResult credReqInfo =
				Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, credOfferJson,
						credDefJson, COMMON_MASTER_SECRET).get();

		String credReqJson = credReqInfo.getCredentialRequestJson();
		String credReqMetadataJson = credReqInfo.getCredentialRequestMetadataJson();

		// Issuer creates TailsReader
		BlobStorageReader blobStorageReaderHandle = BlobStorageReader.openReader(TYPE, tailsWriterConfig).get();

		// Issuer creates Credential
		AnoncredsResults.IssuerCreateCredentialResult credRegInfo =
				Anoncreds.issuerCreateCredential(wallet, credOfferJson, credReqJson,
						GVT_CRED_VALUES, revRegId,
						blobStorageReaderHandle.getBlobStorageReaderHandle()).get();

		String credJson = credRegInfo.getCredentialJson();
		String credRevId = credRegInfo.getRevocId();
		String revocRegDeltaJson = credRegInfo.getRevocRegDeltaJson();

		// Issuer posts RevocationRegistryDelta to Ledger
		revRegEntryRequest = Ledger.buildRevocRegEntryRequest(issuerDid, revRegId, REVOC_REG_TYPE, revocRegDeltaJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegEntryRequest).get();

		// Prover gets RevocationRegistryDefinition
		JSONObject credential = new JSONObject(credJson);
		String getRevRegDefRequest = Ledger.buildGetRevocRegDefRequest(proverDid, credential.getString("rev_reg_id")).get();
		String getRevRegDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDefRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		ParseResponseResult revRegInfo1 = Ledger.parseGetRevocRegDefResponse(getRevRegDefResponse).get();
		String revocRegDefJson = revRegInfo1.getObjectJson();

		// Prover store received Credential
		Anoncreds.proverStoreCredential(proverWallet, "credential1_id",
				credReqMetadataJson, credJson, credDefJson, revocRegDefJson).get();

		// Verifying Prover Credential
		Thread.sleep(3000);

		long to = System.currentTimeMillis() / 1000;
		String proofRequest = new JSONObject().
				put("nonce", "123432421212").
				put("name", "proof_req_1").
				put("version", "0.1").
				put("requested_attributes", new JSONObject().
						put("attr1_referent", new JSONObject().
								put("name", "name"))).
				put("requested_predicates", new JSONObject().
						put("predicate1_referent", new JSONObject().
								put("name", "age").put("p_type", ">=").put("p_value", 18))).
				put("non_revoked", new JSONObject().
						put("to", to)).toString();


		// Prover gets Claims for Proof Request
		String credsJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequest).get();

		JSONObject credentials = new JSONObject(credsJson);
		JSONArray credsForReferent = credentials.getJSONObject("attrs").getJSONArray("attr1_referent");
		JSONObject cred_info = credsForReferent.getJSONObject(0).getJSONObject("cred_info");

		// Prover gets RevocationRegistryDelta from Ledger

		String getRevRegDeltaRequest = Ledger.buildGetRevocRegDeltaRequest(proverDid, cred_info.getString("rev_reg_id"), - 1, (int) to).get();
		String getRevRegDeltaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDeltaRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		LedgerResults.ParseRegistryResponseResult revRegInfo2 = Ledger.parseGetRevocRegDeltaResponse(getRevRegDeltaResponse).get();

		revRegId = revRegInfo2.getId();
		revocRegDeltaJson = revRegInfo2.getObjectJson();
		long timestamp = revRegInfo2.getTimestamp();

		// Prover creates RevocationState
		String revStateJson = Anoncreds.createRevocationState(blobStorageReaderHandle.getBlobStorageReaderHandle(),
				revocRegDefJson, revocRegDeltaJson, timestamp, credRevId).get();

		// Prover gets Schema from Ledger
		getSchemaRequest = Ledger.buildGetSchemaRequest(proverDid, cred_info.getString("schema_id")).get();
		getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		ParseResponseResult schemaInfo2 = Ledger.parseGetSchemaResponse(getSchemaResponse).get();
		String schemaId = schemaInfo2.getId();
		schemaJson = schemaInfo2.getObjectJson();

		// Prover creates Proof
		String requestedCredentialsJson = new JSONObject().
				put("self_attested_attributes", new JSONObject()).
				put("requested_attributes", new JSONObject().
						put("attr1_referent", new JSONObject().
								put("cred_id", cred_info.get("referent")).
								put("timestamp", timestamp).
								put("revealed", true))).
				put("requested_predicates", new JSONObject().
						put("predicate1_referent", new JSONObject().
								put("cred_id", cred_info.get("referent")).
								put("timestamp", timestamp))).toString();

		String schemasJson = new JSONObject().put(schemaId, new JSONObject(schemaJson)).toString();
		String credDefsJson = new JSONObject().put(credDefId, new JSONObject(credDefJson)).toString();
		String revStatesJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revStateJson))).toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequest,
				requestedCredentialsJson, COMMON_MASTER_SECRET,
				schemasJson, credDefsJson, revStatesJson).get();

		JSONObject proof = new JSONObject(proofJson);
		JSONObject identifier = proof.getJSONArray("identifiers").getJSONObject(0);

		// Verifier gets Schema from Ledger
		String getSchemaReq = Ledger.buildGetSchemaRequest(DID_MY1, identifier.getString("schema_id")).get();
		String getSchemaResp = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult schemaInfo3 = Ledger.parseGetSchemaResponse(getSchemaResp).get();
		schemaId = schemaInfo3.getId();
		schemaJson = schemaInfo3.getObjectJson();

		// Verifier gets CredDef from Ledger
		String getCredDefReq = Ledger.buildGetCredDefRequest(DID_MY1, identifier.getString("cred_def_id")).get();
		String getCredDefResp = PoolUtils.ensurePreviousRequestApplied(pool, getCredDefReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult credDefInfo3 = Ledger.parseGetCredDefResponse(getCredDefResp).get();
		credDefId = credDefInfo3.getId();
		credDefJson = credDefInfo3.getObjectJson();

		// Verifier gets RevocationRegistryDefinition from Ledger
		String getRevRegDefReq = Ledger.buildGetRevocRegDefRequest(DID_MY1, identifier.getString("rev_reg_id")).get();
		String getRevRegDefResp = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDefReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult revRegDefInfo3 = Ledger.parseGetRevocRegDefResponse(getRevRegDefResp).get();
		String revRegDefId = revRegDefInfo3.getId();
		revRegDefJson = revRegDefInfo3.getObjectJson();

		// Verifier gets RevocationRegistry from Ledger
		String getRevRegReq = Ledger.buildGetRevocRegRequest(DID_MY1, identifier.getString("rev_reg_id"), identifier.getInt("timestamp")).get();
		String getRevRegResp = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult revRegInfo3 = Ledger.parseGetRevocRegResponse(getRevRegResp).get();
		revRegId = revRegInfo3.getId();
		String revRegJson = revRegInfo3.getObjectJson();
		timestamp = revRegInfo3.getTimestamp();

		// Verifier verifies proof
		Assert.assertNotEquals("Alex",
				proof.getJSONObject("requested_proof").
						getJSONObject("revealed_attrs").
						getJSONObject("attr1_referent").toString());

		schemasJson = new JSONObject().put(schemaId, new JSONObject(schemaJson)).toString();
		credDefsJson = new JSONObject().put(credDefId, new JSONObject(credDefJson)).toString();
		String revRegDefsJson = new JSONObject().put(revRegDefId, new JSONObject(revRegDefJson)).toString();
		String revRegsJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revRegJson))).toString();

		Boolean valid = Anoncreds.verifierVerifyProof(proofRequest,
				proofJson,
				schemasJson,
				credDefsJson,
				revRegDefsJson,
				revRegsJson).get();
		Assert.assertTrue(valid);

		// Issuer revokes credential
		String revRegDeltaJson = Anoncreds.issuerRevokeCredential(wallet,
				blobStorageReaderHandle.getBlobStorageReaderHandle(),
				revRegId, credRevId).get();

		// Issuer post RevocationRegistryDelta to Ledger
		revRegEntryRequest = Ledger.buildRevocRegEntryRequest(issuerDid, revRegId, REVOC_REG_TYPE, revRegDeltaJson).get();

		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegEntryRequest).get();

		// Verifying Prover Credential after Revocation
		Thread.sleep(3000);

		long from = to;
		to = System.currentTimeMillis() / 1000;

		// Prover gets RevocationRegistryDelta from Ledger
		getRevRegDeltaRequest = Ledger.buildGetRevocRegDeltaRequest(proverDid, revRegId, (int) from, (int) to).get();
		getRevRegDeltaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDeltaRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult revRegInfo4 = Ledger.parseGetRevocRegDeltaResponse(getRevRegDeltaResponse).get();

		revRegId = revRegInfo4.getId();
		revocRegDeltaJson = revRegInfo4.getObjectJson();
		timestamp = revRegInfo4.getTimestamp();

		// Prover creates RevocationState
		revStateJson = Anoncreds.createRevocationState(blobStorageReaderHandle.getBlobStorageReaderHandle(),
				revocRegDefJson, revocRegDeltaJson, timestamp, credRevId).get();


		requestedCredentialsJson = new JSONObject().
				put("self_attested_attributes", new JSONObject()).
				put("requested_attributes", new JSONObject().
						put("attr1_referent", new JSONObject().
								put("cred_id", cred_info.get("referent")).
								put("timestamp", timestamp).
								put("revealed", true))).
				put("requested_predicates", new JSONObject().
						put("predicate1_referent", new JSONObject().
								put("cred_id", cred_info.get("referent")).
								put("timestamp", timestamp))).toString();


		revStatesJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revStateJson))).toString();

		proofJson = Anoncreds.proverCreateProof(proverWallet,
				proofRequest,
				requestedCredentialsJson,
				COMMON_MASTER_SECRET,
				schemasJson,
				credDefsJson,
				revStatesJson).get();

		proof = new JSONObject(proofJson);
		identifier = proof.getJSONArray("identifiers").getJSONObject(0);

		// Verifier gets RevocationRegistry from Ledger
		getRevRegReq = Ledger.buildGetRevocRegRequest(DID_MY1, identifier.getString("rev_reg_id"), identifier.getInt("timestamp")).get();
		getRevRegResp = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		LedgerResults.ParseRegistryResponseResult revRegInfo5 = Ledger.parseGetRevocRegResponse(getRevRegResp).get();
		revRegId = revRegInfo5.getId();
		revRegJson = revRegInfo5.getObjectJson();
		timestamp = revRegInfo5.getTimestamp();

		revRegsJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revRegJson))).toString();

		valid = Anoncreds.verifierVerifyProof(proofRequest,
				proofJson,
				schemasJson,
				credDefsJson,
				revRegDefsJson,
				revRegsJson).get();
		Assert.assertFalse(valid);
	}

	@Test
	public void testAnoncredsRevocationInteractionIssuanceByDefault() throws Exception {

		// Issuer create DID
		DidResults.CreateAndStoreMyDidResult trusteeDidInfo = Did.createAndStoreMyDid(this.wallet, new JSONObject().put("seed", TRUSTEE_SEED).toString()).get();

		DidResults.CreateAndStoreMyDidResult issuerDidInfo = Did.createAndStoreMyDid(this.wallet, "{}").get();
		String nymRequest = Ledger.buildNymRequest(trusteeDidInfo.getDid(), issuerDidInfo.getDid(),
				issuerDidInfo.getVerkey(), null, "TRUSTEE").get();
		Ledger.signAndSubmitRequest(pool, wallet, trusteeDidInfo.getDid(), nymRequest).get();

		String issuerDid = issuerDidInfo.getDid();

		// Prover create DID
		DidResults.CreateAndStoreMyDidResult proverDidInfo = Did.createAndStoreMyDid(proverWallet, "{}").get();

		String proverDid = proverDidInfo.getDid();
		String proverVerkey = proverDidInfo.getVerkey();

		// Issuer publish Prover DID
		nymRequest = Ledger.buildNymRequest(issuerDid, proverDid, proverVerkey, null, null).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, nymRequest).get();

		// ISSUER post to Ledger Schema, CredentialDefinition, RevocationRegistry

		// Issuer creates Schema
		AnoncredsResults.IssuerCreateSchemaResult schemaInfo =
				Anoncreds.issuerCreateSchema(issuerDidInfo.getDid(), GVT_SCHEMA_NAME,
						SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();

		String schemaJson = schemaInfo.getSchemaJson();

		// Issuer posts Schema to Ledger
		String schemaRequest = Ledger.buildSchemaRequest(issuerDid, schemaJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, schemaRequest).get();

		// Issuer get Schema from Ledger
		String getSchemaRequest = Ledger.buildGetSchemaRequest(issuerDid, schemaInfo.getSchemaId()).get();
		String getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, response -> {
			JSONObject getSchemaResponseObject = new JSONObject(response);
			return ! getSchemaResponseObject.getJSONObject("result").isNull("seqNo");
		});

		// !!IMPORTANT!!
		// It is important to get Schema from Ledger and parse it to get the correct schema JSON and correspondent id in Ledger
		// After that we can create CredentialDefinition for received Schema(not for result of indy_issuer_create_schema)

		ParseResponseResult schemaInfo1 = Ledger.parseGetSchemaResponse(getSchemaResponse).get();
		schemaJson = schemaInfo1.getObjectJson();

		// Issuer creates CredentialDefinition

		AnoncredsResults.IssuerCreateAndStoreCredentialDefResult credDefInfo =
				Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid, schemaJson,
						TAG, null, new JSONObject().put("support_revocation", true).toString()).get();

		String credDefId = credDefInfo.getCredDefId();
		String credDefJson = credDefInfo.getCredDefJson();

		// Issuer post CredentialDefinition to Ledger
		String credDefRequest = Ledger.buildCredDefRequest(issuerDid, credDefJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, credDefRequest).get();

		// Issuer creates RevocationRegistry
		/* FIXME: getIndyHomePath hard coded forward slash "/". It will not work for Windows. */
		String tailsWriterConfig = new JSONObject(String.format("{\"base_dir\":\"%s\", \"uri_pattern\":\"\"}",
				getIndyHomePath("tails")).replace('\\', '/')).toString();
		BlobStorageWriter tailsWriterHandle = BlobStorageWriter.openWriter("default", tailsWriterConfig).get();

		AnoncredsResults.IssuerCreateAndStoreRevocRegResult revRegInfo =
				Anoncreds.issuerCreateAndStoreRevocReg(wallet, issuerDid, null, TAG,
						credDefId,
						new JSONObject().put("max_cred_num", 5).put("issuance_type", "ISSUANCE_BY_DEFAULT").toString(),
						tailsWriterHandle).get();

		String revRegId = revRegInfo.getRevRegId();
		String revRegDefJson = revRegInfo.getRevRegDefJson();
		String revRegEntryJson = revRegInfo.getRevRegEntryJson();

		// Issuer posts RevocationRegistryDefinition to Ledger
		String revRegDefRequest = Ledger.buildRevocRegDefRequest(issuerDid, revRegDefJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegDefRequest).get();

		// Issuer posts RevocationRegistryEntry to Ledger
		String revRegEntryRequest = Ledger.buildRevocRegEntryRequest(issuerDid, revRegId, REVOC_REG_TYPE, revRegEntryJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegEntryRequest).get();

		// Issuance Credential for Prover

		// Prover creates Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, COMMON_MASTER_SECRET).get();

		// Issuer creates Credential Offer
		String credOfferJson = Anoncreds.issuerCreateCredentialOffer(wallet, credDefId).get();

		// Prover gets CredentialDefinition from Ledger
		String getCredDefRequest = Ledger.buildGetCredDefRequest(proverDid, credDefInfo.getCredDefId()).get();

		String getCredDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getCredDefRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult credDefIdInfo = Ledger.parseGetCredDefResponse(getCredDefResponse).get();

		credDefId = credDefIdInfo.getId();
		credDefJson = credDefIdInfo.getObjectJson();

		// Prover creates Credential Request
		AnoncredsResults.ProverCreateCredentialRequestResult credReqInfo =
				Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, credOfferJson,
						credDefJson, COMMON_MASTER_SECRET).get();

		String credReqJson = credReqInfo.getCredentialRequestJson();
		String credReqMetadataJson = credReqInfo.getCredentialRequestMetadataJson();

		// Issuer creates TailsReader
		BlobStorageReader blobStorageReaderHandle = BlobStorageReader.openReader(TYPE, tailsWriterConfig).get();

		// Issuer creates Credential
		// Issuer must not post rev_reg_delta to ledger for ISSUANCE_BY_DEFAULT strategy

		AnoncredsResults.IssuerCreateCredentialResult credRegInfo =
				Anoncreds.issuerCreateCredential(wallet, credOfferJson, credReqJson,
						GVT_CRED_VALUES, revRegId,
						blobStorageReaderHandle.getBlobStorageReaderHandle()).get();

		String credJson = credRegInfo.getCredentialJson();
		String credRevId = credRegInfo.getRevocId();

		// Prover gets RevocationRegistryDefinition
		String getRevRegDefRequest = Ledger.buildGetRevocRegDefRequest(proverDid, revRegId).get();
		String getRevRegDefResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDefRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		ParseResponseResult revRegInfo1 = Ledger.parseGetRevocRegDefResponse(getRevRegDefResponse).get();

		revRegId = revRegInfo1.getId();
		String revocRegDefJson = revRegInfo1.getObjectJson();

		// Prover store received Credential

		Anoncreds.proverStoreCredential(proverWallet, "credential1_id",
				credReqMetadataJson, credJson, credDefJson,
				revocRegDefJson).get();

		// Verifying Prover Credential
		Thread.sleep(3000);

		long to = System.currentTimeMillis() / 1000;
		String proofRequest = new JSONObject().
				put("nonce", "123432421212").
				put("name", "proof_req_1").
				put("version", "0.1").
				put("requested_attributes", new JSONObject().
						put("attr1_referent", new JSONObject().
								put("name", "name"))).
				put("requested_predicates", new JSONObject().
						put("predicate1_referent", new JSONObject().
								put("name", "age").put("p_type", ">=").put("p_value", 18))).
				put("non_revoked", new JSONObject().
						put("to", to)).toString();


		// Prover gets Claims for Proof Request

		String credentialsJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequest).get();

		JSONObject credentials = new JSONObject(credentialsJson);
		JSONArray credentialsForReferent = credentials.getJSONObject("attrs").getJSONArray("attr1_referent");
		JSONObject credential = credentialsForReferent.getJSONObject(0).getJSONObject("cred_info");

		// Prover gets RevocationRegistryDelta from Ledger

        /* FIXME */
		String getRevRegDeltaRequest = Ledger.buildGetRevocRegDeltaRequest(proverDid, revRegId, - 1, (int) to).get();
		String getRevRegDeltaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDeltaRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		LedgerResults.ParseRegistryResponseResult revRegInfo2 = Ledger.parseGetRevocRegDeltaResponse(getRevRegDeltaResponse).get();

		revRegId = revRegInfo2.getId();
		String revocRegDeltaJson = revRegInfo2.getObjectJson();

		// Prover creates RevocationState
		long timestamp = to;

		String revStateJson = Anoncreds.createRevocationState(blobStorageReaderHandle.getBlobStorageReaderHandle(),
				revocRegDefJson, revocRegDeltaJson, (int) timestamp, credRevId).get();

		// Prover gets Schema from Ledger
		getSchemaRequest = Ledger.buildGetSchemaRequest(proverDid, schemaInfo1.getId()).get();
		getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		ParseResponseResult schemaInfo2 = Ledger.parseGetSchemaResponse(getSchemaResponse).get();
		String schemaId = schemaInfo2.getId();
		schemaJson = schemaInfo2.getObjectJson();

		// Prover creates Proof
		String requestedCredentialsJson = new JSONObject().
				put("self_attested_attributes", new JSONObject()).
				put("requested_attributes", new JSONObject().
						put("attr1_referent", new JSONObject().
								put("cred_id", credential.get("referent")).
								put("timestamp", timestamp).
								put("revealed", true))).
				put("requested_predicates", new JSONObject().
						put("predicate1_referent", new JSONObject().
								put("cred_id", credential.get("referent")).
								put("timestamp", timestamp))).toString();

		String schemasJson = new JSONObject().put(schemaId, new JSONObject(schemaJson)).toString();
		String credDefsJson = new JSONObject().put(credDefId, new JSONObject(credDefJson)).toString();
		String revStatesJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revStateJson))).toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequest,
				requestedCredentialsJson, COMMON_MASTER_SECRET,
				schemasJson, credDefsJson, revStatesJson).get();


		JSONObject proof = new JSONObject(proofJson);

		// Verifier gets RevocationRegistry from Ledger

		String getRevRegReq = Ledger.buildGetRevocRegRequest(DID_MY1, revRegId, (int) timestamp).get();
		String getRevRegResp = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult revRegInfo3 = Ledger.parseGetRevocRegResponse(getRevRegResp).get();

		revRegId = revRegInfo3.getId();
		String revRegJson = revRegInfo3.getObjectJson();

		// Verifier verifies proof
		Assert.assertNotEquals("Alex",
				proof.getJSONObject("requested_proof").
						getJSONObject("revealed_attrs").
						getJSONObject("attr1_referent").toString());


		String revRegDefsJson = new JSONObject().put(revRegId, new JSONObject(revocRegDefJson)).toString();
		String revRegsJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revRegJson))).toString();

		Boolean valid = Anoncreds.verifierVerifyProof(proofRequest,
				proofJson,
				schemasJson,
				credDefsJson,
				revRegDefsJson,
				revRegsJson).get();
		Assert.assertTrue(valid);

		// Issuer revokes credential
		String revRegDeltaJson = Anoncreds.issuerRevokeCredential(wallet,
				blobStorageReaderHandle.getBlobStorageReaderHandle(),
				revRegId, credRevId).get();

		// Issuer post RevocationRegistryDelta to Ledger
		revRegEntryRequest = Ledger.buildRevocRegEntryRequest(issuerDid, revRegId, REVOC_REG_TYPE, revRegDeltaJson).get();
		Ledger.signAndSubmitRequest(pool, wallet, issuerDid, revRegEntryRequest).get();

		// Verifying Prover Credential after Revocation
		Thread.sleep(3000);

		long from = to;
		to = System.currentTimeMillis() / 1000;

		// Prover gets RevocationRegistryDelta from Ledger
		getRevRegDeltaRequest = Ledger.buildGetRevocRegDeltaRequest(proverDid, revRegId, (int) from, (int) to).get();
		getRevRegDeltaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegDeltaRequest, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult revRegInfo4 = Ledger.parseGetRevocRegDeltaResponse(getRevRegDeltaResponse).get();

		revRegId = revRegInfo4.getId();
		revocRegDeltaJson = revRegInfo4.getObjectJson();
		timestamp = revRegInfo4.getTimestamp();

		// Prover creates RevocationState
		revStateJson = Anoncreds.createRevocationState(blobStorageReaderHandle.getBlobStorageReaderHandle(),
				revocRegDefJson, revocRegDeltaJson, (int) timestamp, credRevId).get();


		requestedCredentialsJson = new JSONObject().
				put("self_attested_attributes", new JSONObject()).
				put("requested_attributes", new JSONObject().
						put("attr1_referent", new JSONObject().
								put("cred_id", credential.get("referent")).
								put("timestamp", timestamp).
								put("revealed", true))).
				put("requested_predicates", new JSONObject().
						put("predicate1_referent", new JSONObject().
								put("cred_id", credential.get("referent")).
								put("timestamp", timestamp))).toString();


		revStatesJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revStateJson))).toString();

		proofJson = Anoncreds.proverCreateProof(proverWallet,
				proofRequest,
				requestedCredentialsJson,
				COMMON_MASTER_SECRET,
				schemasJson,
				credDefsJson,
				revStatesJson).get();

		// Verifier gets RevocationRegistry from Ledger
		getRevRegReq = Ledger.buildGetRevocRegRequest(DID_MY1, revRegId, (int) timestamp).get();
		getRevRegResp = PoolUtils.ensurePreviousRequestApplied(pool, getRevRegReq, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});

		LedgerResults.ParseRegistryResponseResult revRegInfo5 = Ledger.parseGetRevocRegResponse(getRevRegResp).get();
		revRegId = revRegInfo5.getId();
		revRegJson = revRegInfo5.getObjectJson();
		timestamp = revRegInfo5.getTimestamp();

		revRegsJson = new JSONObject().put(revRegId, new JSONObject().
				put(String.valueOf(timestamp), new JSONObject(revRegJson))).toString();

		valid = Anoncreds.verifierVerifyProof(proofRequest,
				proofJson,
				schemasJson,
				credDefsJson,
				revRegDefsJson,
				revRegsJson).get();
		Assert.assertFalse(valid);
	}
}
