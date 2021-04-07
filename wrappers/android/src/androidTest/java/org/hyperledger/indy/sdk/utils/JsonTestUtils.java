package org.hyperledger.indy.sdk.utils;

import com.google.gson.Gson;

import org.json.JSONObject;

import java.util.Map;

public class JsonTestUtils {

    private static Gson gson = new Gson();

    public static Map<String, Object> toJsonMap(String json) {
        return gson.fromJson(json, Map.class);
    }

    public static Map<String, Object> toJsonMap(JSONObject json) {
        return gson.fromJson(json.toString(), Map.class);
    }

}
