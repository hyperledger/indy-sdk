package me.connect;

/**
 * Created by abdussami on 08/06/18.
 */


import android.Manifest;
import android.support.test.InstrumentationRegistry;
import android.support.test.filters.SmallTest;
import android.support.test.rule.GrantPermissionRule;
import android.support.test.runner.AndroidJUnit4;
import android.util.Log;

import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.utils.UtilsApi;
import com.evernym.sdk.vcx.vcx.VcxApi;

import junit.framework.Assert;

import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.RuleChain;
import org.junit.runner.RunWith;

import java.util.concurrent.ExecutionException;

import static me.connect.BridgeUtils.writeCACert;

@RunWith(AndroidJUnit4.class)
@SmallTest
public class VcxWrapperTests {
    private GrantPermissionRule readPermissionRule = GrantPermissionRule.grant(Manifest.permission.READ_EXTERNAL_STORAGE);

    private GrantPermissionRule writePermissionRule = GrantPermissionRule.grant(Manifest.permission.WRITE_EXTERNAL_STORAGE);


    @Rule
    public final RuleChain mRuleChain = RuleChain.outerRule(readPermissionRule)
            .around(writePermissionRule);
    private String TAG = "VCX WRAPPER TESTS::";

    @Test
    public void testAgentProvisionAsync(){
        Log.d(TAG, "testAgenctProvisionAsync() called");
        writeCACert(InstrumentationRegistry.getContext());
        String agencyConfig = "{\n" +
                "    \"agency_url\": \"https://cagency.pdev.evernym.com\",\n" +
                "    \"agency_did\": \"dTLdJqRZLwMuWSogcKfBT\",\n" +
                "    \"agency_verkey\": \"LsPQTDHi294TexkFmZK9Q9vW4YGtQRuLV8wuyZi94yH\",\n" +
                "    \"wallet_name\": \"testWallet\",\n" +
                "    \"wallet_key\": \"123\",\n" +
                "    \"agent_seed\": null,\n" +
                "    \"enterprise_seed\": null\n" +
                "}";
        try {
            String res = UtilsApi.vcxAgentProvisionAsync(agencyConfig).get();

            Log.d(TAG, "vcx::APP::async result Prov: " + res);
            Assert.assertTrue(res.contains("dTLdJqRZLwMuWSogcKfBT"));


        } catch (VcxException e) {
            Log.e(TAG, "testAgenctProvisionAsync: ",e );
        } catch (InterruptedException | ExecutionException e) {
            e.printStackTrace();
        }
    }

    @Test
    public void testInitNullPay() {
        Log.d(TAG, "testInitNullPay: called");
        try {
            int result =  VcxApi.initNullPay();
            Assert.assertSame(0,result);
         } catch (VcxException e) {
             e.printStackTrace();
         }
    }

    @Test
    public void testInitWithConfig(){
        Log.d(TAG, "testInitWithConfig: called");
        String config = "{\n" +
                "    \"agency_endpoint\": \"https://cagency.pdev.evernym.com\",\n" +
                "    \"agency_did\": \"dTLdJqRZLwMuWSogcKfBT\",\n" +
                "    \"agency_verkey\": \"LsPQTDHi294TexkFmZK9Q9vW4YGtQRuLV8wuyZi94yH\",\n" +
                "    \"config\": \"{\\\"data\\\":{\\\"alias\\\":\\\"Node1\\\",\\\"blskey\\\":\\\"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba\\\",\\\"client_ip\\\":\\\"35.164.240.131\\\",\\\"client_port\\\":9702,\\\"node_ip\\\":\\\"35.164.240.131\\\",\\\"node_port\\\":9701,\\\"services\\\":[\\\"VALIDATOR\\\"]},\\\"dest\\\":\\\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\\\",\\\"identifier\\\":\\\"Th7MpTaRZVRYnPiabds81Y\\\",\\\"txnId\\\":\\\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\\\",\\\"type\\\":\\\"0\\\"}\\n{\\\"data\\\":{\\\"alias\\\":\\\"Node2\\\",\\\"blskey\\\":\\\"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk\\\",\\\"client_ip\\\":\\\"35.164.240.131\\\",\\\"client_port\\\":9704,\\\"node_ip\\\":\\\"35.164.240.131\\\",\\\"node_port\\\":9703,\\\"services\\\":[\\\"VALIDATOR\\\"]},\\\"dest\\\":\\\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\\\",\\\"identifier\\\":\\\"EbP4aYNeTHL6q385GuVpRV\\\",\\\"txnId\\\":\\\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\\\",\\\"type\\\":\\\"0\\\"}\\n{\\\"data\\\":{\\\"alias\\\":\\\"Node3\\\",\\\"blskey\\\":\\\"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5\\\",\\\"client_ip\\\":\\\"35.164.240.131\\\",\\\"client_port\\\":9706,\\\"node_ip\\\":\\\"35.164.240.131\\\",\\\"node_port\\\":9705,\\\"services\\\":[\\\"VALIDATOR\\\"]},\\\"dest\\\":\\\"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\\\",\\\"identifier\\\":\\\"4cU41vWW82ArfxJxHkzXPG\\\",\\\"txnId\\\":\\\"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4\\\",\\\"type\\\":\\\"0\\\"}\\n{\\\"data\\\":{\\\"alias\\\":\\\"Node4\\\",\\\"blskey\\\":\\\"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw\\\",\\\"client_ip\\\":\\\"35.164.240.131\\\",\\\"client_port\\\":9708,\\\"node_ip\\\":\\\"35.164.240.131\\\",\\\"node_port\\\":9707,\\\"services\\\":[\\\"VALIDATOR\\\"]},\\\"dest\\\":\\\"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\\\",\\\"identifier\\\":\\\"TWwCRQRZ2ZHMJFn9TzLp7W\\\",\\\"txnId\\\":\\\"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008\\\",\\\"type\\\":\\\"0\\\"}\",\n" +
                "    \"pool_name\": \"poolName\",\n" +
                "    \"wallet_name\": \"testWallet\",\n" +
                "    \"wallet_key\": \"123\",\n" +
                "    \"genesis_path\": \"/storage/emulated/0/genesis.txn\",\n" +
                "    \"remote_to_sdk_did\": \"CgLYpEtZ5mkBCRm3mUgbDj\",\n" +
                "    \"remote_to_sdk_verkey\": \"7NAxdt933AEDUGu81VQfsiNPy7vxmb8KUpQMui2hthKa\",\n" +
                "    \"sdk_to_remote_did\": \"CXpWUXJz7bxH7iKE1z4HYv\",\n" +
                "    \"sdk_to_remote_verkey\": \"7HXh3i26DSq4Y3zoR45Q798PJmL91jaf696tdsiYWXQj\",\n" +
                "    \"institution_name\": \"some-random-name\",\n" +
                "    \"institution_logo_url\": \"https://robothash.com/logo.png\"\n" +
                "}";
        try {
           int result =  VcxApi.vcxInitWithConfig(config).get();
           Assert.assertNotSame(0,result);
        } catch (InterruptedException e) {
            e.printStackTrace();
        } catch (ExecutionException e) {
            e.printStackTrace();
        } catch (VcxException e) {
            e.printStackTrace();
        }
    }
}


