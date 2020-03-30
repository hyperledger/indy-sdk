package org.hyperledger.indy.sdk.interaction;

import org.apache.commons.io.FileUtils;
import org.hyperledger.indy.sdk.IndyIntegrationTestWithPoolAndSingleWallet;
import org.hyperledger.indy.sdk.anoncreds.Anoncreds;
import org.hyperledger.indy.sdk.anoncreds.AnoncredsResults;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageReader;
import org.hyperledger.indy.sdk.blob_storage.BlobStorageWriter;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidJSONParameters;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.ledger.Ledger;
import org.hyperledger.indy.sdk.ledger.LedgerResults;
import org.hyperledger.indy.sdk.ledger.LedgerResults.ParseResponseResult;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;
import org.junit.Test;

import java.io.File;
import java.math.BigInteger;
import java.time.Instant;
import java.util.Iterator;

import static org.junit.Assert.assertFalse;

public class AnoncredsVerifyProofAfterCredentialRevokeTest extends IndyIntegrationTestWithPoolAndSingleWallet {
	// This test is a copy of a project attached to IS-1368. We omitted pool and wallet preparation.

	private static String indyClientPath;

	@Test
	public void testAnoncredsVerifyProofAfterCredentialRevoke() throws Exception {

		// create steward did from seed
		String seed = "000000000000000000000000Steward1";
		DidJSONParameters.CreateAndStoreMyDidJSONParameter stewardDIDParameter = new DidJSONParameters.CreateAndStoreMyDidJSONParameter(null, seed, null,
				null);
		DidResults.CreateAndStoreMyDidResult createDidResult = Did.createAndStoreMyDid(wallet, stewardDIDParameter.toString())
				.get();
		String didSteward = createDidResult.getDid();

		// create trust anchor did and write it to the ledger
		createDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String didTrustAnchor = createDidResult.getDid();
		String keyTrustAnchor = createDidResult.getVerkey();
		String request = Ledger.buildNymRequest(didSteward, didTrustAnchor, keyTrustAnchor, null, "TRUST_ANCHOR").get();
		String response = Ledger.signAndSubmitRequest(pool, wallet, didSteward, request).get();


		// trust anchor creates schema and credential definition and writes them to the
		// ledger
		String schemaName = "testschema";
		String version = "1.01";
		JSONArray jsonAttr = new JSONArray();
		jsonAttr.put("licencenumber");
		jsonAttr.put("firstname");
		jsonAttr.put("lastname");
		String schemaAttributes = jsonAttr.toString();
		AnoncredsResults.IssuerCreateSchemaResult schemaResult = Anoncreds
				.issuerCreateSchema(didTrustAnchor, schemaName, version, schemaAttributes).get();
		request = Ledger.buildSchemaRequest(didTrustAnchor, schemaResult.getSchemaJson()).get();
		response = Ledger.signAndSubmitRequest(pool, wallet, didTrustAnchor, request).get();
		System.out.println("Write schema to ledger response:\n" + response + "\n");
		// schema has been written to the ledger

		String schemaId = schemaResult.getSchemaId();



		// Trust Anchor writes a credential def to the ledger. He first get the schemadef and schemaid from the ledger
		String getSchemaRequest = Ledger.buildGetSchemaRequest(didTrustAnchor, schemaId).get();
		String getSchemaResponse = PoolUtils.ensurePreviousRequestApplied(pool, getSchemaRequest, schemaResponse -> {
			JSONObject getSchemaResponseObject = new JSONObject(schemaResponse);
			return !getSchemaResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult schemaDefParseResult = Ledger.parseGetSchemaResponse(getSchemaResponse).get();
		String schemaJson = schemaDefParseResult.getObjectJson();
		String schemaDef = schemaJson.toString();

		AnoncredsResults.IssuerCreateAndStoreCredentialDefResult credentialResult = Anoncreds.issuerCreateAndStoreCredentialDef(wallet,
				didTrustAnchor, schemaDef, "myTag", "CL", "{\"support_revocation\":true}").get();
		request = Ledger.buildCredDefRequest(didTrustAnchor, credentialResult.getCredDefJson()).get();
		response = Ledger.signAndSubmitRequest(pool, wallet, didTrustAnchor, request).get();
		System.out.println("Write credential def to ledger response:\n" + response + "\n");
		// credential def has been written to the ledger

		String credDefId = credentialResult.getCredDefId();
		String credDef = credentialResult.getCredDefJson();


		// now the trust anchor creates a revReg and writes the definition to the ledger
		String revocDir = indyClientPath + "/" + "revoc_dir";
		File revocDirPath = new File(revocDir);
		if (!revocDirPath.exists()) {
			revocDirPath.mkdir();
		}
		JSONObject tailsWriterConfig = new JSONObject();
		tailsWriterConfig.put("base_dir", revocDir);
		tailsWriterConfig.put("uri_pattern", "");
		BlobStorageWriter blobWriter = BlobStorageWriter.openWriter("default", tailsWriterConfig.toString()).get();

		JSONObject revRegConfig = new JSONObject();
		revRegConfig.put("issuance_type", "ISSUANCE_ON_DEMAND"); // the other option is ISSUANCE_ON_DEMAND
		revRegConfig.put("max_cred_num", 100);

		AnoncredsResults.IssuerCreateAndStoreRevocRegResult storeRevocResult = Anoncreds.issuerCreateAndStoreRevocReg(wallet,
				didTrustAnchor, null, "myTagRevoc", credDefId, revRegConfig.toString(), blobWriter)
				.get();


		request = Ledger.buildRevocRegDefRequest(didTrustAnchor, storeRevocResult.getRevRegDefJson()).get();
		System.out.println("Write revocation def to ledger request:\n" + request + "\n");
		response = Ledger.signAndSubmitRequest(pool, wallet, didTrustAnchor, request).get();
		System.out.println("Write revocation def to ledger response:\n" + response + "\n");
		// the revocation def has been written to the ledger

		String revRegDefId = storeRevocResult.getRevRegId();
		String revRegDef = storeRevocResult.getRevRegDefJson();


		// we publish the initial accum value to the ledger
		String intialEntry = storeRevocResult.getRevRegEntryJson();
		String revDefType = "CL_ACCUM";
		request = Ledger.buildRevocRegEntryRequest(didTrustAnchor, revRegDefId, revDefType, intialEntry).get();
		response = Ledger.signAndSubmitRequest(pool, wallet, didTrustAnchor, request).get();
		System.out.println("Write initial accum to ledger response:\n" + response + "\n");


		// read accum from ledger
		long timestampAfterCreatingRevDef = getUnixTimeStamp();
		request = Ledger.buildGetRevocRegRequest(didTrustAnchor, revRegDefId, timestampAfterCreatingRevDef).get();
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult resultAfterCreatingRevDef = Ledger.parseGetRevocRegResponse(response).get();
		System.out.println("Accum Value at (after creating rev def): " + timestampAfterCreatingRevDef + "\n" +  resultAfterCreatingRevDef.getObjectJson() + "\n");
		//



		// trust anchor issues a credential corresponding to the prior created
		// credential definition and issues it to someone
		JSONObject attributesToIssue = new JSONObject();
		attributesToIssue.put("licencenumber", "L2ZKT17Q2");
		attributesToIssue.put("firstname", "MyFirstNamePhilipp");
		attributesToIssue.put("lastname", "MyLastNameMorrison");

		JSONObject credentialDataForIndy = encode(attributesToIssue);
		String credentialOffer = Anoncreds.issuerCreateCredentialOffer(wallet, credentialResult.getCredDefId()).get();

		createDidResult = Did.createAndStoreMyDid(wallet, "{}").get();
		String didProver = createDidResult.getDid();
		String linkSecret = Anoncreds.proverCreateMasterSecret(wallet, null).get();

		AnoncredsResults.ProverCreateCredentialRequestResult proverCredReqResult = Anoncreds.proverCreateCredentialReq(wallet,
				didProver, credentialOffer, credentialResult.getCredDefJson(), linkSecret).get();


		BlobStorageReader blobReader = BlobStorageReader.openReader("default", tailsWriterConfig.toString()).get();
		int blobReaderHandle = blobReader.getBlobStorageReaderHandle();


		AnoncredsResults.IssuerCreateCredentialResult createCredResult = Anoncreds.issuerCreateCredential(wallet, credentialOffer,
				proverCredReqResult.getCredentialRequestJson(), credentialDataForIndy.toString(), revRegDefId, blobReaderHandle).get();

		String issuedCredential = createCredResult.getCredentialJson();
		String issuedCredentialDelta = createCredResult.getRevocRegDeltaJson();
		System.out.println("Created credential:\n" + issuedCredential + "\n");
		System.out.println("Created credential delta:\n" + issuedCredentialDelta + "\n");

		// trust anchor publishes the new delta to the ledger
		request = Ledger.buildRevocRegEntryRequest(didTrustAnchor, revRegDefId, revDefType, issuedCredentialDelta).get();
		response = Ledger.signAndSubmitRequest(pool, wallet, didTrustAnchor, request).get();
		System.out.println("Issuer writes the delta after issueing a credential to the ledger response \n" + response + "\n");




		// read accum from ledger
		long timestampAfterWritingDeltaAfterIssueingCredential = getUnixTimeStamp();
		request = Ledger.buildGetRevocRegRequest(didTrustAnchor, revRegDefId, timestampAfterWritingDeltaAfterIssueingCredential).get();
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult resultAfterCredentialIssueing = Ledger.parseGetRevocRegResponse(response).get();
		System.out.println("Accum Value at (after issueing credential): " + timestampAfterWritingDeltaAfterIssueingCredential + "\n" +  resultAfterCredentialIssueing.getObjectJson() + "\n");
		//




		// the prover stores the created credential in his wallet
		String credentialReferent = Anoncreds
				.proverStoreCredential(wallet, null, proverCredReqResult.getCredentialRequestMetadataJson(),
						createCredResult.getCredentialJson(), credentialResult.getCredDefJson(), revRegDef)
				.get();

		System.out.println("The credential has been stored under the uuid:\n" + credentialReferent + "\n");



		// the issuer revokes the credential and publishes the new delta on the ledger
		String credRevocId = createCredResult.getRevocId();
		System.out.println("The credential Revoc Id is:\n" + credRevocId + "\n");
		String newDeltaAfterRevocation = Anoncreds.issuerRevokeCredential(wallet, blobReaderHandle, revRegDefId, credRevocId).get();
		request = Ledger.buildRevocRegEntryRequest(didTrustAnchor, revRegDefId, revDefType, newDeltaAfterRevocation).get();
		//System.out.println("Request to publish the new delta after Revocatin on the ledger:\n" + request + "\n");
		response = Ledger.signAndSubmitRequest(pool, wallet, didTrustAnchor, request).get();
		System.out.println("The issuer has revoked the credential and published the new accum delta on the ledger\n" + response + "\n");


		Thread.sleep(3*1000); // let the thread sleep, so we definetly get a timestamp which is bigger than the moment we revoked the credential




		// read accum from ledger
		long timestampAfterRevocation = getUnixTimeStamp();
		request = Ledger.buildGetRevocRegRequest(didTrustAnchor, revRegDefId, timestampAfterRevocation).get();
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		LedgerResults.ParseRegistryResponseResult resultAfterRevocation = Ledger.parseGetRevocRegResponse(response).get();
		System.out.println("Accum Value at (after revocation): " + timestampAfterRevocation + "\n" +  resultAfterRevocation.getObjectJson() + "\n");
		//




		/*
		 * The credential has been issued to the prover and he saved it.
		 * Now we want to get a simple proof for licence_number.
		 * We want the prover to reveal this attribute.
		 *
		*/

		// we create a proof request
		JSONObject proofRequest = new JSONObject();
		proofRequest.put("name", "proof_req");
		proofRequest.put("version", "0.1");
		proofRequest.put("nonce", "123432421212");
		JSONObject requested_attributes = new JSONObject();
		JSONObject attribute_info = new JSONObject();
		attribute_info.put("name", "licencenumber");
		JSONObject restrictions = new JSONObject();
		restrictions.put("issuer_did", didTrustAnchor); // the restriction is that the trust anchor issued the credential
		attribute_info.put("restrictions", restrictions);
		requested_attributes.put("attr1_referent", attribute_info);
		proofRequest.put("requested_attributes", requested_attributes);
		proofRequest.put("requested_predicates", new JSONObject());
		long timestamp = getUnixTimeStamp();
		// the credentials must not be revoked in the particular moment = timestamp
		proofRequest.put("non_revoked", new JSONObject().put("from", timestamp).put("to", timestamp));

		System.out.println("Proof-Request has beed created\n" + proofRequest + "\n");



		// build requested credentials which are needed for the actual proof
		String credentials_for_proofRequest = Anoncreds
				.proverGetCredentialsForProofReq(wallet, proofRequest.toString()).get();

		JSONObject requestedCredentials = new JSONObject();
		JSONObject reqAttributes = new JSONObject();
		long proverTimestamp = getUnixTimeStamp(); // this is the timestamp of the moment in which the proover creates the non revocation proof
		reqAttributes.put("attr1_referent", new JSONObject().put("timestamp", timestamp).put("cred_id", credentialReferent).put("revealed", true));
		requestedCredentials.put("self_attested_attributes", new JSONObject());
		requestedCredentials.put("requested_attributes", reqAttributes);
		requestedCredentials.put("requested_predicates", new JSONObject());

		// create schemas which participate in the proof
		JSONObject schemas = new JSONObject();
		schemas.put(schemaResult.getSchemaId(), new JSONObject(schemaResult.getSchemaJson()));

		// create creds which participate in the proof
		JSONObject creds = new JSONObject();
		creds.put(credentialResult.getCredDefId(), new JSONObject(credentialResult.getCredDefJson()));


		// create the revocation states which participate in the proof
		request = Ledger.buildGetRevocRegDeltaRequest(null, revRegDefId, timestamp, timestamp).get(); // read the delta for the interval, which was requested in the proof request
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		System.out.println("Read the delta from the ledger response:\n" + response + "\n");

		LedgerResults.ParseRegistryResponseResult deltaResult = Ledger.parseGetRevocRegDeltaResponse(response).get();
		String delta = deltaResult.getObjectJson();

		// the prover creates the revocaton state in the proverTimestamp moment
		String revState = Anoncreds.createRevocationState(blobReaderHandle, storeRevocResult.getRevRegDefJson(), delta, proverTimestamp, createCredResult.getRevocId()).get();
		JSONObject revStates = new JSONObject();
		revStates.put(revRegDefId, new JSONObject().put(Long.toString(timestamp), new JSONObject(revState)));
		System.out.println("The revocation states have been created\n" + revStates);

		/*
		 * The has created the revocation state at the particular moment which is defined by proverTimestamp.
		 * He created the revocation state for the interval, which was given by the proof request
		 */


		// prover create proof
		String proof = Anoncreds.proverCreateProof(wallet, proofRequest.toString(), requestedCredentials.toString(),
				linkSecret, schemas.toString(), creds.toString(), revStates.toString()).get();



		System.out.println("The prover has created a proof for the given proof request\n" + proof + "\n");


		/*
		 * The verifier verifies the credential. Note that the issuer has revoked the credential
		 * before the prover created the proof. More precisely, the issuer revoked the credential
		 * prior proverTimestamp and published the delta on the ledger.
		 */


		// the prover creates his own revocation definitions.
		JSONObject revocRegDefs = new JSONObject();
		request = Ledger.buildGetRevocRegDefRequest(didTrustAnchor, revRegDefId).get();
		request = Ledger.signRequest(wallet, didTrustAnchor, request).get();
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		ParseResponseResult parseResult = Ledger.parseGetRevocRegDefResponse(response).get();
		String revRegDefReadFromLedgerByVerifier = parseResult.getObjectJson();
		revocRegDefs.put(revRegDefId, new JSONObject(revRegDefReadFromLedgerByVerifier));
		System.out.println("Prover has build his own Revocation Defs:\n" + revocRegDefs + "\n");

		// the prover creates his own revocation states. Herefor he uses the timestamps from the
		// original proofrequest
		JSONObject revocRegs = new JSONObject();
		long from = timestamp;
		long to = timestamp;

		request = Ledger.buildGetRevocRegDeltaRequest(didTrustAnchor, revRegDefId, from, to).get();
		request = Ledger.signRequest(wallet, didTrustAnchor, request).get();
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		System.out.println("Prover has read the revoc delta for interval from: " + from + "to: " + to + " response from ledger \n" + response + "\n");
		LedgerResults.ParseRegistryResponseResult parseRegRespResult = Ledger.parseGetRevocRegDeltaResponse(response).get();
		String proverReadDeltaFromLedger = parseRegRespResult.getObjectJson();
		System.out.println("Prover has read the revoc delta for interval from: " + from + "to: " + to + "\n" + proverReadDeltaFromLedger + "\n");

		/*
		 * revoc delta for current timestamp
		 */
		long time = getUnixTimeStamp();
		request = Ledger.buildGetRevocRegDeltaRequest(didTrustAnchor, revRegDefId, time, time).get();
		request = Ledger.signRequest(wallet, didTrustAnchor, request).get();
		response = PoolUtils.ensurePreviousRequestApplied(pool, request, innerResponse -> {
			JSONObject innerResponseObject = new JSONObject(innerResponse);
			return !innerResponseObject.getJSONObject("result").isNull("seqNo");
		});
		System.out.println("Prover has read the revoc delta for interval from: " + time + "to: " + time + " response from ledger \n" + response + "\n");
		parseRegRespResult = Ledger.parseGetRevocRegDeltaResponse(response).get();
		proverReadDeltaFromLedger = parseRegRespResult.getObjectJson();
		System.out.println("Prover has read the revoc delta for interval from: " + time + "to: " + time + "\n" + proverReadDeltaFromLedger + "\n");
		/*
		 *
		 */


		revocRegs.put(revRegDefId, new JSONObject().put(Long.toString(proverTimestamp), new JSONObject(proverReadDeltaFromLedger)));
		System.out.println("Prover has build his own Revocation States:\n" + revocRegDefs + "\n");


		/*
		request = Ledger.buildGetRevocRegRequest(didProver, revRegDefId, proverTimestamp).get();
		response = Ledger.submitRequest(pool, request).get();
		System.out.println("Read RevocReg from Ledger response:\n" + response + "\n");


		request = Ledger.buildGetRevocRegRequest(didProver, revRegDefId, getUnixTimeStamp()).get();
		response = Ledger.submitRequest(pool, request).get();
		System.out.println("Read RevocReg from Ledger response:\n" + response + "\n");
		*/

		Boolean verifyResult = Anoncreds.verifierVerifyProof(proofRequest.toString(), proof, schemas.toString(),
				creds.toString(), revocRegDefs.toString(), revocRegs.toString()).get();

		assertFalse(verifyResult);

		System.out.println("The proof result ist: " + verifyResult);

		FileUtils.deleteDirectory(revocDirPath);
	}

	private static JSONObject encode(JSONObject attributesToIssue) {
		try {
			JSONObject result = new JSONObject();
			Iterator<String> keyIterator = attributesToIssue.keys();
			while (keyIterator.hasNext()) {
				String key = keyIterator.next();
				String rawValue = attributesToIssue.getString(key);
				String encValue = encStringAsInt(rawValue);
				result.put(key, new JSONObject().put("raw", rawValue).put("encoded", encValue));
			}

			return result;
		} catch (JSONException e) {
			return null;
		}
	}

	private static String encStringAsInt(String string) {
		try {
			Integer.parseInt(string);
			return string;
		} catch (Exception e) {
			BigInteger bigInt = new BigInteger(string.getBytes());
			return bigInt.toString();
		}
	}


	public static long getUnixTimeStamp() {
		return Instant.now().getEpochSecond();
	}
}