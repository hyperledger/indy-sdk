#!/usr/bin/env bash

mkdir -p android-sdk/sdk
mkdir -p android-sdk/android
mkdir -p android-sdk/gradle
mkdir -p aar

cd android-sdk

docker pull thyrlian/android-sdk

# copy the pre-downloaded SDK to the mounted 'sdk' directory
docker run -it --rm -v $(pwd)/sdk:/sdk thyrlian/android-sdk \
bash -c 'export DEBIAN_FRONTEND=noninteractive && RUNLEVEL=1 apt-get update && apt-get install -y wget unzip;cp -a $ANDROID_HOME/. /sdk'

# Update SDK
docker run -it --rm -v $(pwd)/sdk:/opt/android-sdk thyrlian/android-sdk \
bash -c '/opt/android-sdk/tools/bin/sdkmanager --update'

# Download required SDK packages
docker run -it --rm -v $(pwd)/sdk:/opt/android-sdk thyrlian/android-sdk \
bash -c '/opt/android-sdk/tools/bin/sdkmanager "platform-tools" "platforms;android-29" "emulator"'


# wrappers folder in indy-sdk
export WRAPPERS_FOLDER="$(pwd)/../.."

docker run -it \
-v $(pwd)/sdk:/opt/android-sdk:rw \
-v $(pwd)/android:/root/.android:rw \
-v $(pwd)/gradle:/root/.gradle:rw \
-v $WRAPPERS_FOLDER:/wrappers \
-v $WRAPPERS_FOLDER/android/aar:/wrappers/android/aar \
--workdir /wrappers/android \
thyrlian/android-sdk /bin/bash -c 'cd /wrappers/android && ./build.sh'