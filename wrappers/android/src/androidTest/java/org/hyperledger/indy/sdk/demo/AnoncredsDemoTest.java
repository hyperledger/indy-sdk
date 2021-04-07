package org.hyperledger.indy.sdk.demo;

import org.hyperledger.indy.sdk.IndyIntegrationTest;
import org.hyperledger.indy.sdk.InvalidStructureException;
import org.hyperledger.indy.sdk.anoncreds.Anoncreds;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateAndStoreCredentialDefResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateCredentialResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.ProverCreateCredentialRequestResult;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageReader;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageWriter;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.pool.PoolJSONParameters;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.Timeout;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;

import static org.hamcrest.CoreMatchers.isA;
import static org.hyperledger.indy.sdk.utils.EnvironmentUtils.getIndyHomePath;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.assertNotNull;

public class AnoncredsDemoTest extends IndyIntegrationTest {

	@Rule
	public Timeout globalTimeout = new Timeout(10, TimeUnit.MINUTES);

	private Pool pool;
	private String issuerWalletConfig = new JSONObject().put("id", "issuerWallet").toString();
	private Wallet issuerWallet;
	private String proverWalletConfig = new JSONObject().put("id", "proverWallet").toString();
	private Wallet proverWallet;
	private String masterSecretId = "masterSecretId";
	private String credentialId1 = "id1";
	private String credentialId2 = "id2";
	private String issuerDid = "NcYxiDXkpYi6ov5FcYDi1e";
	private String proverDid = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
	private String gvtCredentialValues = GVT_CRED_VALUES;
	private String xyzCredentialValues = new JSONObject("{\n" +
			"        \"status\":{\"raw\":\"partial\", \"encoded\":\"51792877103171595686471452153480627530895\"},\n" +
			"        \"period\":{\"raw\":\"8\", \"encoded\":\"8\"}\n" +
			"    }").toString();

	public AnoncredsDemoTest() throws JSONException {
	}

	@Before
	public void createWallet() throws Exception {
		// Set protocol version
		Pool.setProtocolVersion(PROTOCOL_VERSION).get();

		// Create and Open Pool
		String poolName = PoolUtils.createPoolLedgerConfig();

		PoolJSONParameters.OpenPoolLedgerJSONParameter config2 = new PoolJSONParameters.OpenPoolLedgerJSONParameter(null, null);
		pool = Pool.openPoolLedger(poolName, config2.toJson()).get();

		// Issuer Create and Open Wallet
		Wallet.createWallet(issuerWalletConfig, WALLET_CREDENTIALS).get();
		issuerWallet = Wallet.openWallet(issuerWalletConfig, WALLET_CREDENTIALS).get();

		// Prover Create and Open Wallet
		Wallet.createWallet(proverWalletConfig, WALLET_CREDENTIALS).get();
		proverWallet = Wallet.openWallet(proverWalletConfig, WALLET_CREDENTIALS).get();
	}

	@After
	public void deleteWallet() throws Exception {
		issuerWallet.closeWallet().get();
		Wallet.deleteWallet(issuerWalletConfig, WALLET_CREDENTIALS).get();

		proverWallet.closeWallet().get();
		Wallet.deleteWallet(proverWalletConfig, WALLET_CREDENTIALS).get();

		pool.closePoolLedger().get();
	}

	@Test
	public void testAnoncredsDemo() throws Exception {

		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();
		String gvtSchemaId = createSchemaResult.getSchemaId();
		String gvtSchema = createSchemaResult.getSchemaJson();

		// Issuer create CredentialDef
		IssuerCreateAndStoreCredentialDefResult createCredDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerWallet, issuerDid, gvtSchema, TAG, null, DEFAULT_CRED_DEF_CONFIG).get();
		String credDefId = createCredDefResult.getCredDefId();
		String credDef = createCredDefResult.getCredDefJson();

		// Prover create Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, masterSecretId).get();

		// Issuer create Credential Offer
		String credOffer = Anoncreds.issuerCreateCredentialOffer(issuerWallet, credDefId).get();

		// Prover create CredentialReq
		ProverCreateCredentialRequestResult createCredReqResult = Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, credOffer, credDef, masterSecretId).get();
		String credReq = createCredReqResult.getCredentialRequestJson();
		String credReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer create Credential
		IssuerCreateCredentialResult createCredentialResult =
				Anoncreds.issuerCreateCredential(issuerWallet, credOffer, credReq, gvtCredentialValues, null, - 1).get();
		String credential = createCredentialResult.getCredentialJson();

		// Prover store Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId1, credReqMetadata, credential, credDef, null).get();

		// Prover gets Credentials for Proof Request
		String proofRequestJson = new JSONObject("{" +
				"                    \"nonce\":\"123432421212\",\n" +
				"                    \"name\":\"proof_req_1\",\n" +
				"                    \"version\":\"0.1\", " +
				"                    \"requested_attributes\": {" +
				"                          \"attr1_referent\":{\"name\":\"name\"}," +
				"                          \"attr2_referent\":{\"name\":\"sex\"}," +
				"                          \"attr3_referent\":{\"name\":\"phone\"}" +
				"                     }," +
				"                    \"requested_predicates\":{" +
				"                         \"predicate1_referent\":{\"name\":\"age\",\"p_type\":\">=\",\"p_value\":18}" +
				"                    }" +
				"                  }").toString();

		String credentialsForProofJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequestJson).get();

		JSONObject credentialsForProof = new JSONObject(credentialsForProofJson);
		JSONArray credentialsForAttribute1 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr1_referent");
		JSONArray credentialsForAttribute2 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr2_referent");
		JSONArray credentialsForAttribute3 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr3_referent");
		JSONArray credentialsForPredicate = credentialsForProof.getJSONObject("predicates").getJSONArray("predicate1_referent");

		assertEquals(credentialsForAttribute1.length(), 1);
		assertEquals(credentialsForAttribute2.length(), 1);
		assertEquals(credentialsForAttribute3.length(), 0);
		assertEquals(credentialsForPredicate.length(), 1);

		String credentialUuid = credentialsForAttribute1.getJSONObject(0).getJSONObject("cred_info").getString("referent");

		// Prover create Proof
		String selfAttestedValue = "8-800-300";
		String requestedCredentialsJson = new JSONObject(String.format("{\n" +
				"                                          \"self_attested_attributes\":{\"attr3_referent\":\"%s\"},\n" +
				"                                          \"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%s\", \"revealed\":true},\n" +
				"                                                                    \"attr2_referent\":{\"cred_id\":\"%s\", \"revealed\":false}},\n" +
				"                                          \"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%s\"}}\n" +
				"                                        }", selfAttestedValue, credentialUuid, credentialUuid, credentialUuid)).toString();

		String schemas = new JSONObject(String.format("{\"%s\":%s}", gvtSchemaId, gvtSchema)).toString();
		String credentialDefs = new JSONObject(String.format("{\"%s\":%s}", credDefId, credDef)).toString();
		String revocStates = new JSONObject("{}").toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequestJson, requestedCredentialsJson,
				masterSecretId, schemas, credentialDefs, revocStates).get();
		JSONObject proof = new JSONObject(proofJson);

		// Verifier verify Proof
		JSONObject revealedAttr1 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr1_referent");
		assertEquals("Alex", revealedAttr1.getString("raw"));

		assertNotNull(proof.getJSONObject("requested_proof").getJSONObject("unrevealed_attrs").getJSONObject("attr2_referent").getInt("sub_proof_index"));

		assertEquals(selfAttestedValue, proof.getJSONObject("requested_proof").getJSONObject("self_attested_attrs").getString("attr3_referent"));

		String revocRegDefs = new JSONObject("{}").toString();
		String revocRegs = new JSONObject("{}").toString();

		Boolean valid = Anoncreds.verifierVerifyProof(proofRequestJson, proofJson, schemas, credentialDefs, revocRegDefs, revocRegs).get();
		assertTrue(valid);
	}

	@Test
	public void testAnoncredsWorksForMultipleIssuerSingleProver() throws Exception {

		Wallet issuerGvtWallet = issuerWallet;

		// Issuer2 Create and Open Wallet
		String issuer2WalletConfig = new JSONObject().put("id", "issuer2Wallet").toString();
		Wallet.createWallet(issuer2WalletConfig, WALLET_CREDENTIALS).get();
		Wallet issuerXyzWallet = Wallet.openWallet(issuer2WalletConfig, WALLET_CREDENTIALS).get();

		// Issuer1 create GVT Schema
		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();
		String gvtSchemaId = createSchemaResult.getSchemaId();
		String gvtSchema = createSchemaResult.getSchemaJson();

		// Issuer1 create CredentialDef
		IssuerCreateAndStoreCredentialDefResult createCredDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerGvtWallet, issuerDid, gvtSchema, TAG, null, DEFAULT_CRED_DEF_CONFIG).get();
		String gvtCredDefId = createCredDefResult.getCredDefId();
		String gvtCredDef = createCredDefResult.getCredDefJson();

		// Issuer2 create XYZ Schema
		String issuerDid2 = "VsKV7grR1BUE29mG2Fm2kX";

		// Issuer2 create XYZ Schema
		createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid2, XYZ_SCHEMA_NAME, SCHEMA_VERSION, XYZ_SCHEMA_ATTRIBUTES).get();
		String xyzSchemaId = createSchemaResult.getSchemaId();
		String xyzSchema = createSchemaResult.getSchemaJson();

		//5. Issuer create CredentialDef
		createCredDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerXyzWallet, issuerDid2, xyzSchema, TAG, null, DEFAULT_CRED_DEF_CONFIG).get();
		String xyzCredDefId = createCredDefResult.getCredDefId();
		String xyzCredDef = createCredDefResult.getCredDefJson();

		// Prover create Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, masterSecretId).get();

		// Issuer1 create Credential Offer
		String gvtCredOffer = Anoncreds.issuerCreateCredentialOffer(issuerGvtWallet, gvtCredDefId).get();

		// Issuer2 create Credential Offer
		String xyzCredOffer = Anoncreds.issuerCreateCredentialOffer(issuerXyzWallet, xyzCredDefId).get();

		// Prover create Credential Request for GVT Credential Offer
		ProverCreateCredentialRequestResult createCredReqResult = Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, gvtCredOffer, gvtCredDef, masterSecretId).get();
		String gvtCredReq = createCredReqResult.getCredentialRequestJson();
		String gvtCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer create Credential
		IssuerCreateCredentialResult gvtCreateCredentialResult =
				Anoncreds.issuerCreateCredential(issuerGvtWallet, gvtCredOffer, gvtCredReq, gvtCredentialValues, null, - 1).get();
		String gvtCredential = gvtCreateCredentialResult.getCredentialJson();

		// Prover store Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId1, gvtCredReqMetadata, gvtCredential, gvtCredDef, null).get();

		// Prover create CredentialReq for GVT Credential Offer
		createCredReqResult = Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, xyzCredOffer, xyzCredDef, masterSecretId).get();
		String xyzCredReq = createCredReqResult.getCredentialRequestJson();
		String xyzCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer create Credential
		IssuerCreateCredentialResult xyzCreateCredentialResult = Anoncreds.issuerCreateCredential(issuerXyzWallet, xyzCredOffer, xyzCredReq, xyzCredentialValues, null, - 1).get();
		String xyzCredential = xyzCreateCredentialResult.getCredentialJson();

		// Prover store Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId2, xyzCredReqMetadata, xyzCredential, xyzCredDef, null).get();

		// Prover gets Credentials for Proof Request
		String proofRequestJson = new JSONObject("{" +
				"                    \"nonce\":\"123432421212\",\n" +
				"                    \"name\":\"proof_req_1\",\n" +
				"                    \"version\":\"0.1\", " +
				"                    \"requested_attributes\": {" +
				"                          \"attr1_referent\":{ \"name\":\"name\"}," +
				"                          \"attr2_referent\":{ \"name\":\"status\"}" +
				"                     }," +
				"                    \"requested_predicates\":{" +
				"                         \"predicate1_referent\":{\"name\":\"age\",\"p_type\":\">=\",\"p_value\":18}," +
				"                          \"predicate2_referent\":{\"name\":\"period\",\"p_type\":\">=\",\"p_value\":5}" +
				"                    }" +
				"                  }").toString();

		String credentialsForProofJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequestJson).get();
		assertNotNull(credentialsForProofJson);

		JSONObject credentialsForProof = new JSONObject(credentialsForProofJson);
		JSONArray credentialsForAttribute1 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr1_referent");
		JSONArray credentialsForAttribute2 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr2_referent");
		JSONArray credentialsForPredicate1 = credentialsForProof.getJSONObject("predicates").getJSONArray("predicate1_referent");
		JSONArray credentialsForPredicate2 = credentialsForProof.getJSONObject("predicates").getJSONArray("predicate2_referent");

		assertEquals(credentialsForAttribute1.length(), 1);
		assertEquals(credentialsForAttribute2.length(), 1);
		assertEquals(credentialsForPredicate1.length(), 1);
		assertEquals(credentialsForPredicate2.length(), 1);

		String credentialUuidForAttr1 = credentialsForAttribute1.getJSONObject(0).getJSONObject("cred_info").getString("referent");
		String credentialUuidForAttr2 = credentialsForAttribute2.getJSONObject(0).getJSONObject("cred_info").getString("referent");
		String credentialUuidForPredicate1 = credentialsForPredicate1.getJSONObject(0).getJSONObject("cred_info").getString("referent");
		String credentialUuidForPredicate2 = credentialsForPredicate2.getJSONObject(0).getJSONObject("cred_info").getString("referent");

		// Prover create Proof
		String requestedCredentialsJson = new JSONObject(String.format("{\n" +
				"                                          \"self_attested_attributes\":{},\n" +
				"                                          \"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%s\", \"revealed\":true},\n" +
				"                                                                    \"attr2_referent\":{\"cred_id\":\"%s\", \"revealed\":true}},\n" +
				"                                          \"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%s\"}," +
				"                                                                    \"predicate2_referent\":{\"cred_id\":\"%s\"}}\n" +
				"                                        }", credentialUuidForAttr1, credentialUuidForAttr2, credentialUuidForPredicate1, credentialUuidForPredicate2)).toString();

		String schemas = new JSONObject(String.format("{\"%s\":%s, \"%s\":%s}", gvtSchemaId, gvtSchema, xyzSchemaId, xyzSchema)).toString();
		String credentialDefs = new JSONObject(String.format("{\"%s\":%s, \"%s\":%s}", gvtCredDefId, gvtCredDef, xyzCredDefId, xyzCredDef)).toString();
		String revocStates = new JSONObject("{}").toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequestJson, requestedCredentialsJson,
				masterSecretId, schemas, credentialDefs, revocStates).get();
		JSONObject proof = new JSONObject(proofJson);

		// Verifier verify Proof
		JSONObject revealedAttr1 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr1_referent");
		assertEquals("Alex", revealedAttr1.getString("raw"));

		JSONObject revealedAttr2 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr2_referent");
		assertEquals("partial", revealedAttr2.getString("raw"));

		String revocRegDefs = new JSONObject("{}").toString();
		String revocRegs = new JSONObject("{}").toString();

		Boolean valid = Anoncreds.verifierVerifyProof(proofRequestJson, proofJson, schemas, credentialDefs, revocRegDefs, revocRegs).get();
		assertTrue(valid);

		// Close and delete Issuer2 Wallet
		issuerXyzWallet.closeWallet().get();
		Wallet.deleteWallet(issuer2WalletConfig, WALLET_CREDENTIALS).get();
	}

	@Test
	public void testAnoncredsWorksForSingleIssuerSingleProverMultipleCredentials() throws Exception {
		// Issuer create GVT Schema
		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();
		String gvtSchemaId = createSchemaResult.getSchemaId();
		String gvtSchema = createSchemaResult.getSchemaJson();

		// Issuer create CredentialDef
		IssuerCreateAndStoreCredentialDefResult createCredDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerWallet, issuerDid, gvtSchema, TAG, null, DEFAULT_CRED_DEF_CONFIG).get();
		String gvtCredDefId = createCredDefResult.getCredDefId();
		String gvtCredDef = createCredDefResult.getCredDefJson();

		// Issuer create XYZ Schema
		createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, XYZ_SCHEMA_NAME, SCHEMA_VERSION, XYZ_SCHEMA_ATTRIBUTES).get();
		String xyzSchemaId = createSchemaResult.getSchemaId();
		String xyzSchema = createSchemaResult.getSchemaJson();

		// Issuer create CredentialDef
		createCredDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerWallet, issuerDid, xyzSchema, TAG, null, DEFAULT_CRED_DEF_CONFIG).get();
		String xyzCredDefId = createCredDefResult.getCredDefId();
		String xyzCredDef = createCredDefResult.getCredDefJson();

		// Prover create Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, masterSecretId).get();

		// Issuer create GVT Credential Offer
		String gvtCredOffer = Anoncreds.issuerCreateCredentialOffer(issuerWallet, gvtCredDefId).get();

		// Issuer create XYZ Credential Offer
		String xyzCredOffer = Anoncreds.issuerCreateCredentialOffer(issuerWallet, xyzCredDefId).get();

		// Prover create CredentialReq for GVT Credential Offer
		ProverCreateCredentialRequestResult createCredReqResult =
				Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, gvtCredOffer, gvtCredDef, masterSecretId).get();
		String gvtCredReq = createCredReqResult.getCredentialRequestJson();
		String gvtCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer create GVT Credential
		IssuerCreateCredentialResult gvtCreateCredentialResult =
				Anoncreds.issuerCreateCredential(issuerWallet, gvtCredOffer, gvtCredReq, gvtCredentialValues, null, - 1).get();
		String gvtCredential = gvtCreateCredentialResult.getCredentialJson();

		// Prover store GVT Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId1, gvtCredReqMetadata, gvtCredential, gvtCredDef, null).get();

		// Prover create CredentialReq for XYZ Credential Offer
		createCredReqResult = Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, xyzCredOffer, xyzCredDef, masterSecretId).get();
		String xyzCredReq = createCredReqResult.getCredentialRequestJson();
		String xyzCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer create XYZ Credential
		IssuerCreateCredentialResult xyzCreateCredentialResult =
				Anoncreds.issuerCreateCredential(issuerWallet, xyzCredOffer, xyzCredReq, xyzCredentialValues, null, - 1).get();
		String xyzCredential = xyzCreateCredentialResult.getCredentialJson();

		// Prover store XYZ Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId2, xyzCredReqMetadata, xyzCredential, xyzCredDef, null).get();

		// Prover gets Credentials for Proof Request
		String proofRequestJson = new JSONObject("{" +
				"                    \"nonce\":\"123432421212\",\n" +
				"                    \"name\":\"proof_req_1\",\n" +
				"                    \"version\":\"0.1\", " +
				"                    \"requested_attributes\": {" +
				"                          \"attr1_referent\":{ \"name\":\"name\"}," +
				"                          \"attr2_referent\":{ \"name\":\"status\"}" +
				"                     }," +
				"                    \"requested_predicates\":{" +
				"                         \"predicate1_referent\":{\"name\":\"age\",\"p_type\":\">=\",\"p_value\":18}," +
				"                          \"predicate2_referent\":{\"name\":\"period\",\"p_type\":\">=\",\"p_value\":5}" +
				"                    }" +
				"                  }").toString();

		String credentialsForProofJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequestJson).get();
		assertNotNull(credentialsForProofJson);

		JSONObject credentialsForProof = new JSONObject(credentialsForProofJson);
		JSONArray credentialsForAttribute1 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr1_referent");
		JSONArray credentialsForAttribute2 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr2_referent");
		JSONArray credentialsForPredicate1 = credentialsForProof.getJSONObject("predicates").getJSONArray("predicate1_referent");
		JSONArray credentialsForPredicate2 = credentialsForProof.getJSONObject("predicates").getJSONArray("predicate2_referent");

		assertEquals(credentialsForAttribute1.length(), 1);
		assertEquals(credentialsForAttribute2.length(), 1);
		assertEquals(credentialsForPredicate1.length(), 1);
		assertEquals(credentialsForPredicate2.length(), 1);

		String credentialUuidForAttr1 = credentialsForAttribute1.getJSONObject(0).getJSONObject("cred_info").getString("referent");
		String credentialUuidForAttr2 = credentialsForAttribute2.getJSONObject(0).getJSONObject("cred_info").getString("referent");
		String credentialUuidForPredicate1 = credentialsForPredicate1.getJSONObject(0).getJSONObject("cred_info").getString("referent");
		String credentialUuidForPredicate2 = credentialsForPredicate2.getJSONObject(0).getJSONObject("cred_info").getString("referent");

		// Prover create Proof
		String requestedCredentialsJson = String.format("{\n" +
				"                                          \"self_attested_attributes\":{},\n" +
				"                                          \"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%s\", \"revealed\":true},\n" +
				"                                                                    \"attr2_referent\":{\"cred_id\":\"%s\", \"revealed\":true}},\n" +
				"                                          \"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%s\"}," +
				"                                                                    \"predicate2_referent\":{\"cred_id\":\"%s\"}}\n" +
				"                                        }", credentialUuidForAttr1, credentialUuidForAttr2, credentialUuidForPredicate1, credentialUuidForPredicate2);

		String schemas = new JSONObject(String.format("{\"%s\":%s, \"%s\":%s}", gvtSchemaId, gvtSchema, xyzSchemaId, xyzSchema)).toString();
		String credentialDefs = new JSONObject(String.format("{\"%s\":%s, \"%s\":%s}", gvtCredDefId, gvtCredDef, xyzCredDefId, xyzCredDef)).toString();
		String revocStates = new JSONObject("{}").toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequestJson, requestedCredentialsJson,
				masterSecretId, schemas, credentialDefs, revocStates).get();
		JSONObject proof = new JSONObject(proofJson);

		// Verifier verify Proof
		JSONObject revealedAttr1 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr1_referent");
		assertEquals("Alex", revealedAttr1.getString("raw"));

		JSONObject revealedAttr2 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr2_referent");
		assertEquals("partial", revealedAttr2.getString("raw"));

		String revocRegDefs = new JSONObject("{}").toString();
		String revocRegs = new JSONObject("{}").toString();

		Boolean valid = Anoncreds.verifierVerifyProof(proofRequestJson, proofJson, schemas, credentialDefs, revocRegDefs, revocRegs).get();
		assertTrue(valid);
	}

	@Test
	public void testAnoncredsWorksForRevocationProof() throws Exception {

		// Issuer create Schema
		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();
		String gvtSchemaId = createSchemaResult.getSchemaId();
		String schemaJson = createSchemaResult.getSchemaJson();

		// Issuer create credential definition
		String revocationCredentialDefConfig = "{\"support_revocation\":true}";
		IssuerCreateAndStoreCredentialDefResult createCredentialDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerWallet, issuerDid, schemaJson, TAG, null, revocationCredentialDefConfig).get();
		String credDefId = createCredentialDefResult.getCredDefId();
		String credDef = createCredentialDefResult.getCredDefJson();

		// Issuer create revocation registry
		String revRegConfig = new JSONObject("{\"issuance_type\":null,\"max_cred_num\":5}").toString();
		String tailsWriterConfig = new JSONObject(String.format("{\"base_dir\":\"%s\", \"uri_pattern\":\"\"}", getIndyHomePath("tails")).replace('\\', '/')).toString();
		BlobStorageWriter tailsWriter = BlobStorageWriter.openWriter("default", tailsWriterConfig).get();

		AnoncredsResults.IssuerCreateAndStoreRevocRegResult createRevRegResult = Anoncreds.issuerCreateAndStoreRevocReg(issuerWallet, issuerDid, null, TAG, credDefId, revRegConfig, tailsWriter).get();
		String revRegId = createRevRegResult.getRevRegId();
		String revRegDef = createRevRegResult.getRevRegDefJson();

		// Prover create Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, masterSecretId).get();

		// Issuer create Credential Offer
		String credOffer = Anoncreds.issuerCreateCredentialOffer(issuerWallet, credDefId).get();

		// Prover create Credential Request
		ProverCreateCredentialRequestResult createCredReqResult =
				Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, credOffer, credDef, masterSecretId).get();
		String credReq = createCredReqResult.getCredentialRequestJson();
		String credReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer open TailsReader
		BlobStorageReader blobStorageReaderCfg = BlobStorageReader.openReader("default", tailsWriterConfig).get();
		int blobStorageReaderHandleCfg = blobStorageReaderCfg.getBlobStorageReaderHandle();

		// Issuer create Credential
		IssuerCreateCredentialResult createCredentialResult =
				Anoncreds.issuerCreateCredential(issuerWallet, credOffer, credReq, gvtCredentialValues, revRegId, blobStorageReaderHandleCfg).get();
		String credential = createCredentialResult.getCredentialJson();
		String revRegDelta = createCredentialResult.getRevocRegDeltaJson();
		String credRevId = createCredentialResult.getRevocId();

		// Prover store received Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId1, credReqMetadata, credential, credDef, revRegDef).get();

		// Prover gets Credentials for Proof Request
		String proofRequest = new JSONObject("{\n" +
				"                   \"nonce\":\"123432421212\",\n" +
				"                   \"name\":\"proof_req_1\",\n" +
				"                   \"version\":\"0.1\", " +
				"                   \"requested_attributes\":{" +
				"                          \"attr1_referent\":{\"name\":\"name\"}" +
				"                    },\n" +
				"                    \"requested_predicates\":{" +
				"                          \"predicate1_referent\":{\"name\":\"age\",\"p_type\":\">=\",\"p_value\":18}" +
				"                    }" +
				"               }").toString();

		String credentialsJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequest).get();
		JSONObject credentials = new JSONObject(credentialsJson);
		JSONArray credentialsForAttr1 = credentials.getJSONObject("attrs").getJSONArray("attr1_referent");

		String credentialUuid = credentialsForAttr1.getJSONObject(0).getJSONObject("cred_info").getString("referent");

		// Prover create RevocationState
		int timestamp = 100;
		String revStateJson = Anoncreds.createRevocationState(blobStorageReaderHandleCfg, revRegDef, revRegDelta, timestamp, credRevId).get();


		// Prover create Proof
		String requestedCredentialsJson = new JSONObject(String.format("{" +
				"\"self_attested_attributes\":{}," +
				"\"requested_attributes\":{\"attr1_referent\":{\"cred_id\":\"%s\", \"revealed\":true, \"timestamp\":%d }}," +
				"\"requested_predicates\":{\"predicate1_referent\":{\"cred_id\":\"%s\", \"timestamp\":%d}}" +
				"}", credentialUuid, timestamp, credentialUuid, timestamp)).toString();

		String schemas = new JSONObject(String.format("{\"%s\":%s}", gvtSchemaId, schemaJson)).toString();
		String credentialDefs = new JSONObject(String.format("{\"%s\":%s}", credDefId, credDef)).toString();
		String revStates = new JSONObject(String.format("{\"%s\": { \"%s\":%s }}", revRegId, timestamp, revStateJson)).toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequest, requestedCredentialsJson, masterSecretId, schemas,
				credentialDefs, revStates).get();
		JSONObject proof = new JSONObject(proofJson);

		// Verifier verify proof
		JSONObject revealedAttr1 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr1_referent");
		assertEquals("Alex", revealedAttr1.getString("raw"));

		String revRegDefs = new JSONObject(String.format("{\"%s\":%s}", revRegId, revRegDef)).toString();
		String revRegs = new JSONObject(String.format("{\"%s\": { \"%s\":%s }}", revRegId, timestamp, revRegDelta)).toString();

		boolean valid = Anoncreds.verifierVerifyProof(proofRequest, proofJson, schemas, credentialDefs, revRegDefs, revRegs).get();
		assertTrue(valid);
	}

	@Test
	public void testVerifyProofWorksForProofDoesNotCorrespondToProofRequest() throws Exception {

		thrown.expect(ExecutionException.class);
		thrown.expectCause(isA(InvalidStructureException.class));

		// Issuer create Schema
		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, GVT_SCHEMA_NAME, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).get();
		String gvtSchemaId = createSchemaResult.getSchemaId();
		String gvtSchema = createSchemaResult.getSchemaJson();

		// Issuer create CredentialDef
		IssuerCreateAndStoreCredentialDefResult createCredDefResult = Anoncreds.issuerCreateAndStoreCredentialDef(issuerWallet, issuerDid, gvtSchema, TAG, null, DEFAULT_CRED_DEF_CONFIG).get();
		String credDefId = createCredDefResult.getCredDefId();
		String credDef = createCredDefResult.getCredDefJson();

		// Prover create Master Secret
		Anoncreds.proverCreateMasterSecret(proverWallet, masterSecretId).get();

		// Issuer create Credential Offer
		String credOffer = Anoncreds.issuerCreateCredentialOffer(issuerWallet, credDefId).get();

		// Prover create CredentialReq
		ProverCreateCredentialRequestResult createCredReqResult =
				Anoncreds.proverCreateCredentialReq(proverWallet, proverDid, credOffer, credDef, masterSecretId).get();
		String credReq = createCredReqResult.getCredentialRequestJson();
		String credReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		// Issuer create Credential
		IssuerCreateCredentialResult createCredentialResult =
				Anoncreds.issuerCreateCredential(issuerWallet, credOffer, credReq, gvtCredentialValues, null, - 1).get();
		String credential = createCredentialResult.getCredentialJson();

		// Prover store Credential
		Anoncreds.proverStoreCredential(proverWallet, credentialId1, credReqMetadata, credential, credDef, null).get();

		// Prover gets Credentials for Proof Request
		String proofRequestJson = new JSONObject(String.format("{" +
				"                    \"nonce\":\"123432421212\",\n" +
				"                    \"name\":\"proof_req_1\",\n" +
				"                    \"version\":\"0.1\", " +
				"                    \"requested_attrs\": {" +
				"                          \"attr1_referent\":{ \"name\":\"name\", \"restrictions\":[{\"schema_id\":\"%s\"}]}," +
				"                          \"attr2_referent\":{ \"name\":\"phone\"}" +
				"                     }," +
				"                    \"requested_predicates\":{}" +
				"                  }", gvtSchemaId)).toString();

		String credentialsForProofJson = Anoncreds.proverGetCredentialsForProofReq(proverWallet, proofRequestJson).get();
		assertNotNull(credentialsForProofJson);

		JSONObject credentialsForProof = new JSONObject(credentialsForProofJson);
		JSONArray credentialsForAttribute1 = credentialsForProof.getJSONObject("attrs").getJSONArray("attr1_referent");

		assertEquals(credentialsForAttribute1.length(), 1);

		String credentialUuid = credentialsForAttribute1.getJSONObject(0).getJSONObject("cred_info").getString("referent");

		// Prover create Proof
		String selfAttestedValue = "8-800-300";
		String requestedCredentialsJson = new JSONObject(String.format("{\n" +
				"                                          \"self_attested_attributes\":{\"attr2_referent\":\"%s\"},\n" +
				"                                          \"requested_attrs\":{\"attr1_referent\":[\"%s\", true]},\n" +
				"                                          \"requested_predicates\":{}\n" +
				"                                        }", selfAttestedValue, credentialUuid)).toString();

		String schemas = new JSONObject(String.format("{\"%s\":%s}", gvtSchemaId, gvtSchema)).toString();
		String credentialDefs = new JSONObject(String.format("{\"%s\":%s}", credDefId, credDef)).toString();
		String revocInfos = new JSONObject("{}").toString();

		String proofJson = Anoncreds.proverCreateProof(proverWallet, proofRequestJson, requestedCredentialsJson,
				masterSecretId, schemas, credentialDefs, revocInfos).get();
		JSONObject proof = new JSONObject(proofJson);

		// Verifier verify Proof
		JSONObject revealedAttr1 = proof.getJSONObject("requested_proof").getJSONObject("revealed_attrs").getJSONObject("attr1_referent");
		assertEquals("Alex", revealedAttr1.getString("raw"));

		assertEquals(selfAttestedValue, proof.getJSONObject("requested_proof").getJSONObject("self_attested_attrs").getString("attr3_referent"));

		String revocRegDefs = new JSONObject("{}").toString();
		String revocRegs = new JSONObject("{}").toString();

		proofRequestJson = new JSONObject(String.format("{" +
				"                    \"nonce\":\"123432421212\",\n" +
				"                    \"name\":\"proof_req_1\",\n" +
				"                    \"version\":\"0.1\", " +
				"                    \"requested_attributes\": {" +
				"                          \"attr1_referent\":{ \"name\":\"name\", \"restrictions\":[{\"schema_id\":%s}]}" +
				"                     }," +
				"                    \"requested_predicates\":{" +
				"                          \"predicate1_referent\":{\"name\":\"age\",\"p_type\":\">=\",\"p_value\":18}" +
				"                    }" +
				"                  }", gvtSchemaId)).toString();


		Anoncreds.verifierVerifyProof(proofRequestJson, proofJson, schemas, credentialDefs, revocRegDefs, revocRegs).get();
	}
}
