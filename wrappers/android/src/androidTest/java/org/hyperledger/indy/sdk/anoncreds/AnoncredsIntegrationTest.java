package org.hyperledger.indy.sdk.anoncreds;

import org.hyperledger.indy.sdk.utils.AndroidInitHelper;
import org.hyperledger.indy.sdk.utils.StorageUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.IssuerCreateAndStoreCredentialDefResult;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults.ProverCreateCredentialRequestResult;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.*;
import org.junit.rules.ExpectedException;
import org.junit.rules.Timeout;

import java.util.concurrent.TimeUnit;

public class AnoncredsIntegrationTest {

	@Rule
	public ExpectedException thrown = ExpectedException.none();

	@Rule
	public Timeout globalTimeout = new Timeout(10, TimeUnit.MINUTES);

	private static Boolean walletOpened = false;

	static Wallet wallet;
	static String gvtSchemaId;
	static String gvtSchema;
	static String xyzSchemaId;
	static String xyzSchema;
	static String issuer1gvtCredDefId;
	static String issuer1gvtCredDef;
	static String issuer1xyzCredDef;
	static String issuer1GvtCredOffer;
	static String issuer2GvtCredOffer;
	static String issuer1GvtCredReq;
	static String issuer1GvtCredReqMetadata;
	String CREDENTIALS = "{\"key\":\"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY\", \"key_derivation_method\":\"RAW\"}";
	String masterSecretId = "master_secret_name";
	String issuerDid = "NcYxiDXkpYi6ov5FcYDi1e";
	String proverDid = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";
	String defaultCredentialDefinitionConfig = "{\"support_revocation\":false}";
	String tag = "tag1";
	String gvtSchemaName = "gvt";
	String schemaVersion = "1.0";
	String gvtSchemaAttributes = "[\"name\", \"age\", \"sex\", \"height\"]";
	String credentialId1 = "id1";
	String credentialId2 = "id2";
    String credentialIdX = "idX";
	// note that encoding is not standardized by Indy except that 32-bit integers are encoded as themselves. IS-786
	String gvtCredentialValuesJson = new JSONObject("{\n" +
			"               \"sex\":{\"raw\":\"male\",\"encoded\":\"5944657099558967239210949258394887428692050081607692519917050011144233115103\"},\n" +
			"               \"name\":{\"raw\":\"Alex\",\"encoded\":\"1139481716457488690172217916278103335\"},\n" +
			"               \"height\":{\"raw\":\"175\",\"encoded\":\"175\"},\n" +
			"               \"age\":{\"raw\":\"28\",\"encoded\":\"28\"}\n" +
			"        }").toString();
	String xyzCredentialValuesJson = new JSONObject("{\n" +
			"               \"status\":{\"raw\":\"partial\",\"encoded\":\"51792877103171595686471452153480627530895\"},\n" +
			"               \"period\":{\"raw\":\"8\",\"encoded\":\"8\"}\n" +
			"        }").toString();
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

	public AnoncredsIntegrationTest() throws JSONException {
	}

	@Before
	public void setUp() throws Exception {
		AndroidInitHelper.init();
		initCommonWallet();
	}

	private void initCommonWallet() throws Exception {

		if (walletOpened) {
			return;
		}

		StorageUtils.cleanupStorage();

		String walletConfig =
				new JSONObject()
						.put("id", "anoncredsCommonWallet")
						.toString();

		Wallet.createWallet(walletConfig, CREDENTIALS).get();
		wallet = Wallet.openWallet(walletConfig, CREDENTIALS).get();

		AnoncredsResults.IssuerCreateSchemaResult createSchemaResult =
				Anoncreds.issuerCreateSchema(issuerDid, gvtSchemaName, schemaVersion, gvtSchemaAttributes).get();
		gvtSchemaId = createSchemaResult.getSchemaId();
		gvtSchema = createSchemaResult.getSchemaJson();

		String xyzSchemaAttributes = "[\"status\", \"period\"]";
		String xyzSchemaName = "xyz";
		createSchemaResult = Anoncreds.issuerCreateSchema(issuerDid, xyzSchemaName, schemaVersion, xyzSchemaAttributes).get();
		xyzSchemaId = createSchemaResult.getSchemaId();
		xyzSchema = createSchemaResult.getSchemaJson();

		//Issue GVT issuer1GvtCredential by Issuer1
		IssuerCreateAndStoreCredentialDefResult issuer1CreateGvtCredDefResult =
				Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid, gvtSchema, tag, null, defaultCredentialDefinitionConfig).get();
		issuer1gvtCredDefId = issuer1CreateGvtCredDefResult.getCredDefId();
		issuer1gvtCredDef = issuer1CreateGvtCredDefResult.getCredDefJson();

		//Issue XYZ issuer1GvtCredential by Issuer1
		IssuerCreateAndStoreCredentialDefResult issuer1CreateXyzCredDefResult =
				Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid, xyzSchema, tag, null, defaultCredentialDefinitionConfig).get();
		String issuer1xyzCredDefId = issuer1CreateXyzCredDefResult.getCredDefId();
		issuer1xyzCredDef = issuer1CreateXyzCredDefResult.getCredDefJson();

		//Issue GVT issuer1GvtCredential by Issuer2
		String issuerDid2 = "VsKV7grR1BUE29mG2Fm2kX";
		IssuerCreateAndStoreCredentialDefResult issuer2CreateGvtCredDefResult =
				Anoncreds.issuerCreateAndStoreCredentialDef(wallet, issuerDid2, gvtSchema, tag, null, defaultCredentialDefinitionConfig).get();
		String issuer2gvtCredDefId = issuer2CreateGvtCredDefResult.getCredDefId();
		String issuer2gvtCredDef = issuer2CreateGvtCredDefResult.getCredDefJson();

		issuer1GvtCredOffer = Anoncreds.issuerCreateCredentialOffer(wallet, issuer1gvtCredDefId).get();
		String issuer1XyzCredOffer = Anoncreds.issuerCreateCredentialOffer(wallet, issuer1xyzCredDefId).get();
		issuer2GvtCredOffer = Anoncreds.issuerCreateCredentialOffer(wallet, issuer2gvtCredDefId).get();

		Anoncreds.proverCreateMasterSecret(wallet, masterSecretId).get();

		ProverCreateCredentialRequestResult createCredReqResult =
				Anoncreds.proverCreateCredentialReq(wallet, proverDid, issuer1GvtCredOffer, issuer1gvtCredDef, masterSecretId).get();
		issuer1GvtCredReq = createCredReqResult.getCredentialRequestJson();
		issuer1GvtCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		AnoncredsResults.IssuerCreateCredentialResult createCredResult =
				Anoncreds.issuerCreateCredential(wallet, issuer1GvtCredOffer, issuer1GvtCredReq, gvtCredentialValuesJson, null, - 1).get();
		String issuer1GvtCredential = createCredResult.getCredentialJson();

		Anoncreds.proverStoreCredential(wallet, credentialId1, issuer1GvtCredReqMetadata, issuer1GvtCredential, issuer1gvtCredDef, null).get();

		createCredReqResult = Anoncreds.proverCreateCredentialReq(wallet, proverDid, issuer1XyzCredOffer, issuer1xyzCredDef, masterSecretId).get();
		String issuer1XyzCredReq = createCredReqResult.getCredentialRequestJson();
		String issuer1XyzCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		createCredResult = Anoncreds.issuerCreateCredential(wallet, issuer1XyzCredOffer, issuer1XyzCredReq, xyzCredentialValuesJson, null, - 1).get();
		String issuer1XyzCredential = createCredResult.getCredentialJson();

		Anoncreds.proverStoreCredential(wallet, credentialId2, issuer1XyzCredReqMetadata, issuer1XyzCredential, issuer1xyzCredDef, null).get();

		createCredReqResult = Anoncreds.proverCreateCredentialReq(wallet, proverDid, issuer2GvtCredOffer, issuer2gvtCredDef, masterSecretId).get();
		String issuer2GvtCredReq = createCredReqResult.getCredentialRequestJson();
		String issuer2GvtCredReqMetadata = createCredReqResult.getCredentialRequestMetadataJson();

		String gvt2CredValues = "{" +
				"           \"sex\":{\"raw\":\"male\",\"encoded\":\"2142657394558967239210949258394838228692050081607692519917028371144233115103\"},\n" +
				"           \"name\":{\"raw\":\"Alexander\",\"encoded\":\"21332817548165488690172217217278169335\"},\n" +
				"           \"height\":{\"raw\":\"170\",\"encoded\":\"170\"},\n" +
				"           \"age\":{\"raw\":\"28\",\"encoded\":\"28\"}\n" +
				"   }";

		createCredResult = Anoncreds.issuerCreateCredential(wallet, issuer2GvtCredOffer, issuer2GvtCredReq, gvt2CredValues, null, - 1).get();
		String issuer2GvtCredential = createCredResult.getCredentialJson();

		String credentialId3 = "id3";
		Anoncreds.proverStoreCredential(wallet, credentialId3, issuer2GvtCredReqMetadata, issuer2GvtCredential, issuer2gvtCredDef, null).get();

		Anoncreds.proverStoreCredential(wallet, credentialIdX, issuer2GvtCredReqMetadata, issuer2GvtCredential, issuer2gvtCredDef, null).get();

		walletOpened = true;
	}
}
