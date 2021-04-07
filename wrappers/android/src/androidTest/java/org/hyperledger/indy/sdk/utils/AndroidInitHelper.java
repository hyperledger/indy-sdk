package org.hyperledger.indy.sdk.utils;

import android.system.ErrnoException;
import android.system.Os;

import org.hyperledger.indy.sdk.LibIndy;

import pl.brightinventions.slf4android.LogLevel;
import pl.brightinventions.slf4android.LoggerConfiguration;

public class AndroidInitHelper {

    public static void init() throws ErrnoException {
        // Trace debugging for testing.
        LoggerConfiguration.configuration()
                .setRootLogLevel(LogLevel.TRACE);

        // TODO should this be set as part of init?
        Os.setenv("EXTERNAL_STORAGE", EnvironmentUtils.getIndyHomePath(), true);
        Os.setenv("TMPDIR", EnvironmentUtils.getTmpPath(), true);

        if (!LibIndy.isInitialized()) {
            LibIndy.init();
        }
    }

}
