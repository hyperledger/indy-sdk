package org.hyperledger.sdk.wrappertest.wrappertest;


import android.support.v7.app.AppCompatActivity;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.widget.Button;

import com.evernym.connectme.wrappertest.R;

import org.hyperledger.indy.sdk.*;
import org.hyperledger.indy.sdk.did.Did;
import org.hyperledger.indy.sdk.did.DidResults;
import org.hyperledger.indy.sdk.wallet.Wallet;

import java.io.BufferedWriter;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStreamWriter;
import java.util.Date;
import java.util.concurrent.ExecutionException;

import static junit.framework.Assert.assertNotNull;

//import java.util.concurrent.ExecutionException;


public class MainActivity extends AppCompatActivity {
    private static final String TAG = "MainActivity";
    public static File pool_config;
    public static final String POOL_CONFIG = "{\"data\":{\"alias\":\"Node1\",\"blskey\":\"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba\",\"client_ip\":\"35.164.240.131\",\"client_port\":9702,\"node_ip\":\"35.164.240.131\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"Th7MpTaRZVRYnPiabds81Y\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}\n"
            + "{\"data\":{\"alias\":\"Node2\",\"blskey\":\"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk\",\"client_ip\":\"35.164.240.131\",\"client_port\":9704,\"node_ip\":\"35.164.240.131\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"EbP4aYNeTHL6q385GuVpRV\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"\n}"
            + "{\"data\":{\"alias\":\"Node3\",\"blskey\":\"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5\",\"client_ip\":\"35.164.240.131\",\"client_port\":9706,\"node_ip\":\"35.164.240.131\",\"node_port\":9705,\"services\":[\"VALIDATOR\"]},\"dest\":\"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\",\"identifier\":\"4cU41vWW82ArfxJxHkzXPG\",\"txnId\":\"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4\",\"type\":\"0\"\n}"
            + "{\"data\":{\"alias\":\"Node4\",\"blskey\":\"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw\",\"client_ip\":\"35.164.240.131\",\"client_port\":9708,\"node_ip\":\"35.164.240.131\",\"node_port\":9707,\"services\":[\"VALIDATOR\"]},\"dest\":\"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\",\"identifier\":\"TWwCRQRZ2ZHMJFn9TzLp7W\",\"txnId\":\"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008\",\"type\":\"0\"}";



    void createConfigFiles() {
        try {

            MainActivity.pool_config = File.createTempFile("pool_config", ".txn", this.getBaseContext().getCacheDir());
            Log.v(TAG, this.getBaseContext().getCacheDir() + "-=> is the dir");
            FileOutputStream fos = new FileOutputStream(pool_config);

            BufferedWriter bw = new BufferedWriter(new OutputStreamWriter(fos));

            bw.write("{\"data\":{\"alias\":\"Node1\",\"blskey\":\"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba\",\"client_ip\":\"35.164.240.131\",\"client_port\":9702,\"node_ip\":\"35.164.240.131\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"Th7MpTaRZVRYnPiabds81Y\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}");
            bw.newLine();
            bw.write("{\"data\":{\"alias\":\"Node2\",\"blskey\":\"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk\",\"client_ip\":\"35.164.240.131\",\"client_port\":9704,\"node_ip\":\"35.164.240.131\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"EbP4aYNeTHL6q385GuVpRV\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"}");
            bw.newLine();
            bw.write("{\"data\":{\"alias\":\"Node3\",\"blskey\":\"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5\",\"client_ip\":\"35.164.240.131\",\"client_port\":9706,\"node_ip\":\"35.164.240.131\",\"node_port\":9705,\"services\":[\"VALIDATOR\"]},\"dest\":\"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\",\"identifier\":\"4cU41vWW82ArfxJxHkzXPG\",\"txnId\":\"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4\",\"type\":\"0\"}");
            bw.newLine();
            bw.write("{\"data\":{\"alias\":\"Node4\",\"blskey\":\"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw\",\"client_ip\":\"35.164.240.131\",\"client_port\":9708,\"node_ip\":\"35.164.240.131\",\"node_port\":9707,\"services\":[\"VALIDATOR\"]},\"dest\":\"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\",\"identifier\":\"TWwCRQRZ2ZHMJFn9TzLp7W\",\"txnId\":\"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008\",\"type\":\"0\"}");
            bw.newLine();
            bw.flush();
            bw.close();
            fos.close();

            Log.d(TAG, "File path " + pool_config.getPath());
        } catch (IOException e) {
            Log.d(TAG, e.getMessage());
        }
    }


    @Override
    protected void onCreate(Bundle savedInstanceState) {

          final String TYPE = "default";
          final String PATH = getApplicationInfo().dataDir;
          final String PATHJSON = "{ \"path\":\" " + PATH + "\"}";
          final String WALLET = "Wallet1" + new Date().toString();
          final String WALLET_CONFIG = "{ \"id\":\"" + WALLET + "\"}";
          final String WALLET_CREDENTIALS = "{\"key\":\"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY\", \"key_derivation_method\":\"RAW\"}";
          final String RUNTIME_CONFIG = "{\"collect_backtrace\": true}";
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        final Button provisionButton = findViewById(R.id.prov);

        final Button provisionAsyncButton = findViewById(R.id.prov_async);
        provisionAsyncButton.setOnClickListener(v -> {


            try {
                Os.setenv("EXTERNAL_STORAGE", getExternalFilesDir(null).getAbsolutePath(), true);
                Os.setenv("RUST_LOG","TRACE",true);
                Log.d(TAG, ">>>> ENV  "+ System.getenv("RUST_LOG"));
            } catch (ErrnoException e) {

                e.printStackTrace();
            }


            System.loadLibrary("indy");
            LibIndy.init();
            LibIndy.setRuntimeConfig(RUNTIME_CONFIG);
            Log.d(TAG, ">>>> "+ WALLET_CONFIG);
            Log.d(TAG, "onClick: Provision Async clicked");
            try{

                Wallet.createWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
                Wallet wallet = Wallet.openWallet(WALLET_CONFIG, WALLET_CREDENTIALS).get();
                DidResults.CreateAndStoreMyDidResult result = Did.createAndStoreMyDid(wallet, "{}").get();
                assertNotNull(wallet);
                Log.d(TAG, "DID>>>> "+ result.getDid());

                wallet.closeWallet().get();
            }catch (IndyException | InterruptedException | ExecutionException e) {
                Log.d(TAG, ">>>> ERROR");
                e.printStackTrace();
            }


        });
        final Button initButton = findViewById(R.id.init);
        initButton.setOnClickListener(v -> {
                Log.d(TAG, "result of init with config json: " );

        });
        final Button initWithConfigFileButton = findViewById(R.id.init_configFile);
        initWithConfigFileButton.setOnClickListener(v -> {
            createConfigFiles();
        });

    }
}
