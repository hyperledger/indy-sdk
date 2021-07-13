#!/usr/bin/env bash


jniLibs=src/main/jniLibs
libindy_version=1.11.0


mkdir -p ${jniLibs}/armeabi-v7a
mkdir -p ${jniLibs}/arm64
mkdir -p ${jniLibs}/x86
mkdir -p aar

download_libgnustl(){
    rm -rf indy-android-dependencies
    git clone https://github.com/sovrin-foundation/indy-android-dependencies
    pushd indy-android-dependencies/prebuilt/libgnustl
        unzip libgnustl_arm64.zip
        unzip libgnustl_armv7.zip
        unzip libgnustl_x86.zip
    popd
    cp indy-android-dependencies/prebuilt/libgnustl/libgnustl_arm64/libgnustl_shared.so ${jniLibs}/arm64/
    cp indy-android-dependencies/prebuilt/libgnustl/libgnustl_armv7/libgnustl_shared.so ${jniLibs}/armeabi-v7a/
    cp indy-android-dependencies/prebuilt/libgnustl/libgnustl_x86/libgnustl_shared.so ${jniLibs}/x86/
}

copy_native_binaries(){
    cp libindy_arm/lib/libindy.so ${jniLibs}/armeabi-v7a/
    cp libindy_arm64/lib/libindy.so ${jniLibs}/arm64/
    cp libindy_x86/lib/libindy.so ${jniLibs}/x86/
}

download_prebuilt_libindy(){
    wget -O libindy_android_arm_${libindy_version}.zip "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_arm_${libindy_version}.zip"
    unzip libindy_android_arm_${libindy_version}.zip

    wget -O libindy_android_arm64_${libindy_version}.zip "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_arm64_${libindy_version}.zip"
    unzip libindy_android_arm64_${libindy_version}.zip
    
    wget -O libindy_android_x86_${libindy_version}.zip "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_x86_${libindy_version}.zip"
    unzip libindy_android_x86_${libindy_version}.zip
}
cleanup(){
    rm libindy_android_arm_${libindy_version}.zip
    rm libindy_android_arm64_${libindy_version}.zip
    rm libindy_android_x86_${libindy_version}.zip
    rm -rf libindy_arm
    rm -rf libindy_arm64
    rm -rf libindy_x86
    rm -rf indy-android-dependencies
    rm -rf $jniLibs
}


build(){

    cp -r ../java/src/main/java src/main/
    cp -r ../java/src/main/resources src/main/

    javaDir=src/main/java
    javaFiles=`find $javaDir -type f -name "*.java"`

    for f in $javaFiles; do sed -i 's/java.util.concurrent.CompletableFuture/java9.util.concurrent.CompletableFuture/g' "$f"; done

    chmod +x gradlew
    ./gradlew clean assemble

    mv build/outputs/aar/*-debug.aar aar/libindy-debug.aar
    mv build/outputs/aar/*-release.aar aar/libindy-release.aar

}
copy_aar_to_wrappertest(){
    rm ../../samples/android/WrapperTest/libindy-debug/libindy-debug.aar
    cp build/outputs/aar/libindy-debug.aar ../../samples/android/WrapperTest/libindy-debug/
}



download_prebuilt_libindy
copy_native_binaries
download_libgnustl
build
cleanup
