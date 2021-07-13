package com.usb.sovrin

import ch.qos.logback.classic.Level
import ch.qos.logback.classic.Logger
import com.evernym.sdk.vcx.ErrorCode
import com.evernym.sdk.vcx.LibVcx
import com.evernym.sdk.vcx.VcxException
import com.evernym.sdk.vcx.connection.ConnectionApi
import com.evernym.sdk.vcx.credentialDef.CredentialDefApi
import com.evernym.sdk.vcx.proof.ProofApi
import com.evernym.sdk.vcx.schema.SchemaApi
import com.evernym.sdk.vcx.vcx.VcxApi
import com.evernym.sdk.vcx.issuer.IssuerApi
import com.google.gson.Gson
import com.sun.jna.Library
import com.sun.jna.Native
import org.slf4j.LoggerFactory
import java.io.File
import java.util.*
import java.util.concurrent.ExecutionException


fun main(args: Array<String>) {



    var defaultLevel: Level = Level.OFF

    if(args.isNotEmpty()  && args.size==2 && args[0] == "loglevel"){
        defaultLevel = Level.toLevel(args[1].toUpperCase())
    }
    val root = LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME) as Logger
    root.setLevel(defaultLevel)

    val wallets = File("${System.getProperty("user.home")}/.indy_client/wallet")

    if(wallets.exists()){

        //Clean existing wallets
        wallets.listFiles().forEach {

            if(it.isDirectory && it.name != "forward_agent_wallet_id"){

                it.deleteRecursively()
            }
        }


    }

    var libvcx = "llib/libvcx.so"

    if(System.getProperty("os.name") == "Mac OS X"){
        libvcx = "lib/libvcx.dylib"
    }

    LibVcx.init(File(libvcx))

    var libnullpay = "lib/libnullpay.so"

    if(System.getProperty("os.name") == "Mac OS X"){
        libnullpay = "lib/libnullpay.dylib"
    }

    /**Initialize Lib Null Pay library for the payment APIs. Currently there is no java wrapper available for libnullpay So we have to manually
     * load the library. This code written base off of LibVCX.init()
     * com.sun.jna is used to bridge the gap between java and OS specific DLL(libraries)
     **/
    Native.loadLibrary(File(libnullpay).absolutePath,PAYMENT_API::class.java)
            .nullpay_init()

    try {

        println("####################################################################")
        println("# 1 Provision an agent and wallet, get back configuration details")
        println("####################################################################")

        //Initialize the vcx with provision agent. This is only needed for local network
        //For Test network use vcxInitWithConfig directly
        var vcxconfig = vcxProvisionAgent("lib/faber-provisionconfig.json"
                ,"Faber","http://robohash.org/234",
                "lib/localpoolconfig.json" )


        println("####################################################################")
        println("# 2 Initialize libvcx with new configuration")
        println("####################################################################")

        //Initialize vcx with the vcxconfig received from the provision agent initialization
        VcxApi.vcxInitWithConfig(vcxconfig).get()
        println("=====Configuration Initialized====")



        val sourceId="schema_123"
        val schemaName = "degree" + UUID.randomUUID()
        val schemaVersion = "0.0.1"
        val schemaData = "[\"name\", \"date\", \"degree\"]"


        println("####################################################################")
        println("# 3 Create a new schema on the ledger")
        println("####################################################################")

        var schemaHandle = SchemaApi.schemaCreate(sourceId,schemaName,schemaVersion,schemaData,0).get()

        println("====Schema Handle is : $schemaHandle====")

        val schemaId =  SchemaApi.schemaGetSchemaId(schemaHandle).get()

        println("====Schema Id is : $schemaId====")

        val attr = SchemaApi.schemaGetAttributes(sourceId,schemaId).get()

        println("====$attr====")


        println("####################################################################")
        println("# 4 Create a new credential definition on the ledger")
        println("####################################################################")

        val credDefHandle = CredentialDefApi.credentialDefCreate("creddef_uuid","degree",schemaId,null,"tag1","{\"support_revocation\":false, \"tails_file\": \"/tmp/tailsfile.txt\", \"max_creds\": 1}",0).get()

        val defId = CredentialDefApi.credentialDefGetCredentialDefId(credDefHandle).get()

        println(defId)


        println("####################################################################")
        println("# 5 Create a connection to alice and print out the invite details")
        println("####################################################################")

        //Create a connection to alice and print out the invite details
        var connCreateHandle = ConnectionApi.vcxConnectionCreate("alice").get()

        var connHandle = ConnectionApi.vcxConnectionConnect(connCreateHandle,"{\"use_public_did\": true}").get()

        var updateHandle = ConnectionApi.vcxConnectionUpdateState(connCreateHandle).get()

        var connDetail = ConnectionApi.connectionInviteDetails(connCreateHandle,0).get()

        println("======Alice Connection Invite Detail========\n${connDetail}")

        var connDetailObj = Gson().fromJson(connDetail,ConnectionDetail::class.java)

        println("####################################################################")
        println("# 6 Poll agency and wait for alice to accept the invitation (start alice.py now)")
        println("####################################################################")

        var connStateHandle = ConnectionApi.connectionGetState(connCreateHandle).get()

        //If connection status = 4 then it's connnected except not
        while (connStateHandle!=IssuerCredentialState.Accepted.code){

            Thread.sleep(20000)
            ConnectionApi.vcxConnectionUpdateState(connCreateHandle).get()
            connStateHandle = ConnectionApi.connectionGetState(connCreateHandle).get()
            println("Connection Status is : ${connStateHandle}")

        }

        println("======Faber is now connected to Alice=======")


        println("####################################################################")
        println("# 12 Create an IssuerCredential object using the schema and credential definition")
        println("####################################################################")

        var credOffer = IssuerApi.issuerCreateCredential("alice_degree",credDefHandle,connDetailObj.senderDetail.DID
                ,Gson().toJson(Degree("alice","04-2019","math")),"alice_degree",0).get()



        println("####################################################################")
        println("# 13 Issue credential offer to alice")
        println("####################################################################")

        IssuerApi.issuerSendcredentialOffer(credOffer,connCreateHandle).get()
        IssuerApi.issuerCredentialUpdateState(credOffer).get()

        println("####################################################################")
        println("# 14 Poll agency and wait for alice to send a credential request")
        println("####################################################################")
        var credReqState = IssuerApi.issuerCredntialGetState (credOffer).get()

        while(credReqState!=IssuerCredentialState.RequestReceived.code){

            Thread.sleep(2000)
            IssuerApi.issuerCredentialUpdateState(credOffer).get()
            credReqState = IssuerApi.issuerCredntialGetState (credOffer).get()


        }

        println("####################################################################")
        println("# 17 Issue credential to alice")
        println("####################################################################")
        IssuerApi.issuerSendCredential(credOffer,connCreateHandle).get()


        println("####################################################################")
        println("# 18 Wait for alice to accept credential")
        println("####################################################################")

        IssuerApi.issuerCredentialUpdateState(credOffer).get()
        credReqState = IssuerApi.issuerCredntialGetState (credOffer).get()
        while(credReqState!=IssuerCredentialState.Accepted.code){

            Thread.sleep(10000)
            IssuerApi.issuerCredentialUpdateState(credOffer).get()
            credReqState = IssuerApi.issuerCredntialGetState (credOffer).get()


        }


        var proof_attrs = "[" +
                "{\"name\": \"name\"},"+
                "{\"name\": \"date\"},"+
                "{\"name\": \"degree\"}"+
                "]"

        println("####################################################################")
        println("# 19 Create a Proof object")
        println("####################################################################")

        var proofReq = ProofApi.proofCreate("proof_uuid",proof_attrs,"","{}","degree_proof").get()


        println("####################################################################")
        println("# 20 Request proof of degree from alice")
        println("####################################################################")

        ProofApi.proofSendRequest(proofReq,connCreateHandle).get()


        println("####################################################################")
        println("# 21 Poll agency and wait for alice to provide proof")
        println("####################################################################")

        var proofReqState = ProofApi.proofGetState (proofReq).get()

        while(proofReqState!=IssuerCredentialState.Accepted.code){

            Thread.sleep(2000)
            ProofApi.proofUpdateState(proofReq).get()
            proofReqState = ProofApi.proofGetState (proofReq).get()

        }

        println("####################################################################")
        println("# 27 Process the proof provided by alice")
        println("####################################################################")

        var proofResult = ProofApi.getProof(proofReq,connCreateHandle).get()

        if(proofResult.proof_state == ProofState.Verified.code){

            println("####################################################################")
            println("proof is verified!!")
            println("####################################################################")

        }else{

            println("####################################################################")
            println("could not verify proof :(")
            println("####################################################################")

        }

    }catch(e: ExecutionException){


        var cause = e.cause
        if(cause !=null && cause is VcxException){

            println("======${ErrorCode.valueOf(cause.sdkErrorCode).name}")

        }
    }finally {

        VcxApi.vcxShutdown(false)

    }
}

public interface PAYMENT_API : Library {

    fun nullpay_init(): Int
}

@Throws(VcxException::class)
fun vcxProvisionAgent(configPath: String, institution_name: String,institution_logo_url: String,genesis_path: String):String {
    val f = File(configPath).readText()
    val result = LibVcx.api.vcx_provision_agent(f)

    println("############${result}##################")

    var provison = Gson().fromJson(result,VCXProvision::class.java)


    provison.institution_name = institution_name
    provison.institution_logo_url=institution_logo_url
    provison.genesis_path=genesis_path
    println("############${provison}##################")


    return Gson().toJson(provison)

}

data class VCXProvision(var agency_did:String,var agency_endpoint:String,var agency_verkey:String,var genesis_path:String
                        ,var institution_did:String,var institution_logo_url:String,var institution_name:String,
                        var institution_verkey:String,var remote_to_sdk_did:String,var remote_to_sdk_verkey:String,
                        var sdk_to_remote_did:String,var sdk_to_remote_verkey:String,var wallet_key:String,
                        var wallet_name:String
)

data class CredentialOffer(var msg_type:String,var version:String,var to_did:String,var from_did:String,var libindy_offer:String,
                           var credential_attrs:Any,var schema_seq_no:Int,var cred_def_id:String,var claim_name:String,
                           var claim_id:String,var msg_ref_id:String?)

data class Degree(val name: String, val date:String, val degree:String)

data class ConnectionDetail(var statusCode:String,var connReqId:String,var senderDetail:SenderDetail
                            , var senderAgencyDetail: SenderAgencyDetail,var targetName:String,var statusMsg:String)

data class SenderDetail(var name:String,var agentKeyDlgProof: AgentKeyDelegateProof,var DID:String,var logoUrl:String,var verKey:String,var publicDID:String)

data class AgentKeyDelegateProof(var agentDID:String,var agentDelegatedKey: String,var signature:String)

data class SenderAgencyDetail(var DID:String,var verKey:String,var endpoint:String)


data class ProofRequest(var type: ProofRequestType )

data class ProofRequestType(var name:String,var version:String )

enum class IssuerCredentialState(val code:Int){

    Undefined(0),
    Initialized(1),
    OfferSent(2),
    RequestReceived(3),
    Accepted(4),
    Unfulfilled(5),
    Expired(6),
    Revoked(7)
}

enum class ProofState(var code:Int){

    Undefined(0),
    Verified(1),
    Invalid(2)
}






