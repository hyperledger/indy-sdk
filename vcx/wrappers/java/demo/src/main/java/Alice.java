import com.evernym.sdk.vcx.connection.ConnectionApi;
import com.evernym.sdk.vcx.credential.CredentialApi;
import com.evernym.sdk.vcx.proof.DisclosedProofApi;
import com.evernym.sdk.vcx.utils.UtilsApi;
import com.evernym.sdk.vcx.vcx.VcxApi;

import com.jayway.jsonpath.JsonPath;

import org.apache.commons.cli.CommandLine;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Scanner;
import java.util.logging.Logger;

import utils.Common;
import static utils.Common.prettyJson;
import static utils.State.StateType;

public class Alice {
    // get logger for demo - INFO configured
    static final Logger logger = Common.getDemoLogger();

    public static void main(String[] args) throws Exception {
        // Library logger setup - ERROR|WARN|INFO|DEBUG|TRACE
        Common.setLibraryLogger("ERROR");

        CommandLine options = Common.getCommandLine(args);
        if (options == null) System.exit(0);

        logger.info("#0 Initialize");
        Common.loadNullPayPlugin();

        // static configuration
        long utime = System.currentTimeMillis() / 1000;
        String provisionConfig = JsonPath.parse("{" +
                "  agency_url: 'http://localhost:8080'," +
                "  agency_did: 'VsKV7grR1BUE29mG2Fm2kX'," +
                "  agency_verkey: 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR'," +
                "  wallet_name: 'node_vcx_demo_alice_wallet_" + utime + "'," +
                "  wallet_key: '123'," +
                "  payment_method: 'null'," +
                "  enterprise_seed: '000000000000000000000000Trustee1'" +
                "}").jsonString();

        // Communication method. aries.
        provisionConfig = JsonPath.parse(provisionConfig).put("$", "protocol_type", "4.0").jsonString();
        logger.info("Running with Aries VCX Enabled! Make sure VCX agency is configured to use protocol_type 4.0");

        if (options.hasOption("postgres")) {
            Common.loadPostgresPlugin();
            provisionConfig = JsonPath.parse(provisionConfig).put("$", "wallet_type", "postgres_storage")
                    .put("$", "storage_config", "{\"url\":\"localhost:5432\"}")
                    .put("$", "storage_credentials", "{\"account\":\"postgres\",\"password\":\"mysecretpassword\"," +
                            "\"admin_account\":\"postgres\",\"admin_password\":\"mysecretpassword\"}").jsonString();
            logger.info("Running with PostreSQL wallet enabled! Config = " + JsonPath.read(provisionConfig, "$.storage_config"));
        } else {
            logger.info("Running with builtin wallet.");
        }

        logger.info("#8 Provision an agent and wallet, get back configuration details: \n" + prettyJson(provisionConfig));
        String vcxConfig = UtilsApi.vcxProvisionAgent(provisionConfig);

        vcxConfig = JsonPath.parse(vcxConfig).put("$", "institution_name", "alice")
                .put("$", "institution_logo_url", "http://robohash.org/345")
                .put("$", "protocol_version", "2")
                .put("$", "genesis_path", System.getProperty("user.dir") + "/genesis.txn").jsonString();
        logger.info("#9 Initialize libvcx with new configuration\n" + prettyJson(vcxConfig));
        VcxApi.vcxInitWithConfig(vcxConfig).get();

        logger.info("Input faber invitation details\nEnter your invite details:");
        Scanner sc = new Scanner(System.in);
        String details = sc.nextLine();

        logger.info("#10 Convert to valid json and string and create a connection to faber");
        int connectionHandle = ConnectionApi.vcxCreateConnectionWithInvite("faber", details).get();
        ConnectionApi.vcxConnectionConnect(connectionHandle, "{\"use_public_did\": true}").get();
        int connectionState = ConnectionApi.vcxConnectionUpdateState(connectionHandle).get();
        while (connectionState != StateType.Accepted) {
            Thread.sleep(2000);
            connectionState = ConnectionApi.vcxConnectionUpdateState(connectionHandle).get();
        }

        logger.info("#11 Wait for faber to issue a credential offer");
        Thread.sleep(5000);
        String offers = CredentialApi.credentialGetOffers(connectionHandle).get();
        logger.info("Alice found " + JsonPath.read(offers, "$.length()") + " credential offers.");
        String credentialOffer = JsonPath.parse((LinkedHashMap)JsonPath.read(offers, "$.[0]")).jsonString();
        logger.info("credential offer:\n" + prettyJson(credentialOffer));

        // Create a credential object from the credential offer
        int credentialHandle = CredentialApi.credentialCreateWithOffer("credential", credentialOffer).get();

        logger.info("#15 After receiving credential offer, send credential request");
        CredentialApi.credentialSendRequest(credentialHandle, connectionHandle, 0).get();

        logger.info("#16 Poll agency and accept credential from faber");
        int credentialState = CredentialApi.credentialUpdateState(credentialHandle).get();
        while (credentialState != StateType.Accepted) {
            Thread.sleep(2000);
            credentialState = CredentialApi.credentialUpdateState(credentialHandle).get();
        }

        logger.info("#22 Poll agency for a proof request");
        String requests = DisclosedProofApi.proofGetRequests(connectionHandle).get();
        while (JsonPath.read(requests, "$.length()").equals("0")) {
            Thread.sleep(2000);
            requests = DisclosedProofApi.proofGetRequests(connectionHandle).get();
        }
        logger.info("Alice found " + JsonPath.read(requests, "$.length()") + " proof requests.");
        String proofRequest = JsonPath.parse((LinkedHashMap)JsonPath.read(requests, "$.[0]")).jsonString();
        logger.info("proof request:\n" + prettyJson(proofRequest));

        logger.info("#23 Create a Disclosed proof object from proof request");
        int proofHandle = DisclosedProofApi.proofCreateWithRequest("proof", proofRequest).get();

        logger.info("#24 Query for credentials in the wallet that satisfy the proof request");
        String credentials = DisclosedProofApi.proofRetrieveCredentials(proofHandle).get();

        LinkedHashMap<String, Object> attrs = JsonPath.read(credentials, "$.attrs");
        for(String key : attrs.keySet()){
            String attr = JsonPath.parse((LinkedHashMap)JsonPath.read(credentials, "$.attrs." + key + ".[0]")).jsonString();
            credentials = JsonPath.parse(credentials).set("$.attrs." + key, JsonPath.parse("{\"credential\":"+ attr + "}").json()).jsonString();
        }

        logger.info("#25 Generate the proof");
        DisclosedProofApi.proofGenerate(proofHandle, credentials, "{}").get();

        logger.info("#26 Send the proof to faber");
        DisclosedProofApi.proofSend(proofHandle, connectionHandle).get();

        logger.info("#27 Wait for Faber to receive the proof");
        int proofState = DisclosedProofApi.proofUpdateState(proofHandle).get();
        while (proofState != StateType.Accepted) {
            Thread.sleep(2000);
            proofState = DisclosedProofApi.proofUpdateState(proofHandle).get();
        }
        logger.info("Faber received the proof");

        System.exit(0);
    }
}
