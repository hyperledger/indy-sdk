package com.usb.sovrin


import ch.qos.logback.classic.Level
import ch.qos.logback.classic.Logger
import com.evernym.sdk.vcx.ErrorCode
import com.evernym.sdk.vcx.LibVcx
import com.evernym.sdk.vcx.VcxException
import com.evernym.sdk.vcx.connection.ConnectionApi
import com.evernym.sdk.vcx.credential.CredentialApi
import com.evernym.sdk.vcx.proof.DisclosedProofApi
import com.evernym.sdk.vcx.proof.ProofApi
import com.evernym.sdk.vcx.vcx.VcxApi
import com.google.gson.Gson
import com.google.gson.GsonBuilder
import com.google.gson.reflect.TypeToken
import com.sun.jna.Library
import com.sun.jna.Native
import org.slf4j.LoggerFactory
import java.io.File
import java.util.*
import java.util.concurrent.ExecutionException


fun main(args: Array<String>) {




    var defaultLevel: Level = Level.OFF

    if(args.isNotEmpty() && args.size==2 && args[0] == "loglevel"){
        defaultLevel = Level.toLevel(args[1].toUpperCase())
    }
    val root = LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME) as Logger
    root.setLevel(defaultLevel)

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
        println("# 7 Provision an agent and wallet, get back configuration details")
        println("####################################################################")


        //Initialize the vcx with provision agent. This is only needed for local network
        //For Test network use vcxInitWithConfig directly
        var vcxconfig = vcxProvisionAgent("lib/alice-provisionconfig.json"
                ,"alice","http://robohash.org/456",
                "lib/localpoolconfig.json" )


        println("####################################################################")
        println("# 8 Initialize libvcx with new configuration")
        println("####################################################################")

        //Initialize vcx with the vcxconfig received from the provision agent initialization
        VcxApi.vcxInitWithConfig(vcxconfig).get()
        println("=====Configuration Initialized====")



        println("####################################################################")
        println("# 9 Input faber.py invitation details")
        println("####################################################################")

        val connectionDetail = readLine()!!

        println("####################################################################")
        println("# 10 Convert to valid json and string and create a connection to faber")
        println("####################################################################")

        var conn_to_faber = ConnectionApi.vcxCreateConnectionWithInvite("faber",connectionDetail).get()

        ConnectionApi.vcxConnectionConnect(conn_to_faber,"{\"use_public_did\": true}").get()

        ConnectionApi.vcxConnectionUpdateState(conn_to_faber).get()

        println("####################################################################")
        println("# 11 Wait for faber.py to issue a credential offer")
        println("####################################################################")

        Thread.sleep(30000)

        var offer = CredentialApi.credentialGetOffers(conn_to_faber).get()
        println("=====Offer recevied from Faber ${offer}")


        val listType = object : TypeToken<ArrayList<ArrayList<CredentialOffer>>>() {}.getType()
        val offers: List<List<CredentialOffer>> = Gson().fromJson(offer, listType)


        println("=====Creating credential with offer")


        val credOffer = CredentialApi.credentialCreateWithOffer("credential",Gson().toJson(offers.get(0))).get()


        println("####################################################################")
        println("# 15 After receiving credential offer, send credential request")
        println("####################################################################")

        CredentialApi.credentialSendRequest(credOffer,conn_to_faber,0).get()

        println("####################################################################")
        println("# 16 Poll agency and accept credential offer from faber")
        println("####################################################################")
        CredentialApi.credentialUpdateState(credOffer).get()

        var credReqState = CredentialApi.credentialGetState (credOffer).get()


        while(credReqState!=IssuerCredentialState.Accepted.code){

            Thread.sleep(2000)
            CredentialApi.credentialUpdateState(credOffer).get()
            credReqState = CredentialApi.credentialGetState (credOffer).get()


        }
        var cred = CredentialApi.getCredential(credOffer).get()
        println("======CREDENTIAL IS: ${cred}")

        println("####################################################################")
        println("# 22 Poll agency for a proof request")
        println("####################################################################")
        var proofRequests = DisclosedProofApi.proofGetRequests(conn_to_faber).get()


        println(proofRequests)

        val proofType = object : TypeToken<ArrayList<Map<String, Any>>>() {}.getType()
        val proofs: MutableList<MutableMap<String,Any>> = Gson().fromJson(proofRequests, proofType)

        val topic:MutableMap<String,Any> =  proofs.get(0).get("@topic") as MutableMap<String,Any>
        topic.put("mid", (topic.get("mid") as Double).toInt())
        topic.put("tid", (topic.get("tid") as Double).toInt())



        var proofStr = GsonBuilder().serializeNulls()
                .create().toJson(proofs.get(0))

        println("proof requests received ${proofStr}")



        println("####################################################################")
        println("# 23 Create a Disclosed proof object from proof request")
        println("####################################################################")
        val newProof = DisclosedProofApi.proofCreateWithRequest("proof",proofStr).get()

        println("####################################################################")
        println("# 24 Query for credentials in the wallet that satisfy the proof request")
        println("####################################################################")
        val credentialsJson = DisclosedProofApi.proofRetrieveCredentials(newProof).get()

        println("====Credentials in Wallets are ${credentialsJson}")

        val credType = object : TypeToken<Map<String, Any>>() {}.getType()
        val credentials: MutableMap<String,Any> = Gson().fromJson(credentialsJson, credType)

        var proof = credentials.map {


            // println(it.key)
            var attrs = it.value as Map<String,Any>

            var credentialAttrs =attrs.map {

                var credDetail = it.value as List<Any>
                //println(y.get(0))

                Pair(it.key, mapOf(Pair("credential",credDetail.get(0))))
            }.toMap()

            // println (credentialAttrs)
            Pair(it.key,credentialAttrs)
        }.toMap()

        val generatedProof =  Gson().toJson(proof)


        println("####################################################################")
        println("# 25 Generate the proof")
        println("####################################################################")

        DisclosedProofApi.proofGenerate(newProof,generatedProof,"{}").get()

        println("####################################################################")
        println("# 26 Send the proof to faber")
        println("####################################################################")


        DisclosedProofApi.proofSend(newProof,conn_to_faber).get()



    }catch(e: ExecutionException){


        var cause = e.cause
        if(cause !=null && cause is VcxException){

            println("======${ErrorCode.valueOf(cause.sdkErrorCode).name}")

        }
    }finally {

        VcxApi.vcxShutdown(false)

    }
}
