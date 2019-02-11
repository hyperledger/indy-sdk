package me.connect;

import android.content.Context;
import android.os.Environment;
import android.util.Log;

import com.facebook.react.bridge.Promise;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import android.content.ContextWrapper;

/**
 * Created by abdussami on 23/05/18.
 */

public class BridgeUtils {
    private static  String TAG ="BRIDGEUTILS::";
    public static void resolveIfValid(Promise promise, Object result) {
        // Add more conditions here if you want to check if the result is valid
        //e.g Like if result is null return false
        //e.g If result is empty string return false
        if (result != null) {
            promise.resolve(result);
        } else {
            promise.reject("NULL VALUE", "Null value was received as result from wrapper");
        }

    }


    public static void writeCACert(Context context) {
        Log.d(TAG, "writeCACert() called with: context = [" + context + "]");
        ContextWrapper cw = new ContextWrapper(context);
        File cert_file = new File(cw.getFilesDir().toString() + "/cacert.pem");
        if (!cert_file.exists()) {
            try {
                FileWriter fw = new FileWriter(cert_file);
                fw.write(generateCaCertContents());
                fw.flush();
                fw.close();
            } catch (IOException e) {
                Log.e(TAG, "writeCACert: ",e );
            }
        }
    }

    static String generateCaCertContents() {
        File folder = new File("/system/etc/security/cacerts");
        File[] listOfFiles = folder.listFiles();
        StringBuilder sb = new StringBuilder(99999);
        try {
            for (File certFile : listOfFiles) {
                if (certFile.isFile()) {

                    sb.append(System.lineSeparator());
                    sb.append("-----BEGIN CERTIFICATE-----");
                    sb.append(System.lineSeparator()).append(
                            getBetweenStrings(getFileContents(certFile),
                            "-----BEGIN CERTIFICATE-----",
                            "-----END CERTIFICATE-----"));
                    sb.append("-----END CERTIFICATE-----");
                    sb.append(System.lineSeparator());
                    sb.append(System.lineSeparator());
                }
            }
           return sb.toString();

        } catch (FileNotFoundException e) {
            e.printStackTrace();
            return "";
        } catch (IOException e) {
            e.printStackTrace();
        }
        return "";
    }
     static String getFileContents( File file ) throws IOException {
        StringBuffer text = new StringBuffer(99999);
        FileInputStream fileStream = new FileInputStream( file );
        BufferedReader br = new BufferedReader( new InputStreamReader( fileStream ) );
        for ( String line; (line = br.readLine()) != null; )
            text.append( line + System.lineSeparator() );
        return text.toString();
    }
    private static String getBetweenStrings(
            String text,
            String textFrom,
            String textTo) {

        String result = "";

        // Cut the beginning of the text to not occasionally meet a
        // 'textTo' value in it:
        result =
                text.substring(
                        text.indexOf(textFrom) + textFrom.length(),
                        text.length());

        // Cut the excessive ending of the text:
        result =
                result.substring(
                        0,
                        result.indexOf(textTo));

        return result;
    }
}
