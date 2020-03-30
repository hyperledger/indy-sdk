#!/bin/sh

source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

mkdir -p $WORK_DIR/libsqlite3-android
cd $WORK_DIR/libsqlite3-android
#if [ ! -f sqlite-android-3240000.aar ]; then
if [ ! -d sqlite3-android ]; then
    rm -rf $WORK_DIR/libsqlite3-android/*
    #wget https://www.sqlite.org/2018/sqlite-android-3240000.aar
    #wget https://www.sqlite.org/2018/sqlite-autoconf-3240000.tar.gz
    git clone https://github.com/ehedenst/sqlite3-android.git
    cd sqlite3-android
else
    cd sqlite3-android
    git checkout .
    git checkout master
    git clean -f
    git clean -fd
    git pull
fi

#SQLITE_DIR=$WORK_DIR/libsqlite3-android/sqlite-android-3240000
#SQLITE_DIR=$WORK_DIR/libsqlite3-android/sqlite-autoconf-3240000
#if [ ! -d $SQLITE_DIR ]; then
    #mkdir -p $SQLITE_DIR
    #cd $SQLITE_DIR
    #unzip ../sqlite-android-3240000.aar
    #mv jni/* ..
    #tar zxf sqlite-autoconf-3240000.tar.gz
#fi

export PATH=$ANDROID_NDK_HOME:$PATH

sed -i .bak 's/3160100/3240000/' Makefile
sed -i .bak 's/2017/2018/' Makefile
sed -i .bak 's/stlport_shared/c++_shared/' jni/Application.mk

sed -i .bak 's/armeabi/armeabi-v7a/' jni/Application.mk
make clean
make

sed -i .bak 's/armeabi-v7a/arm64-v8a/' jni/Application.mk
make

sed -i .bak 's/arm64-v8a/x86/' jni/Application.mk
make

sed -i .bak 's/x86/x86_64/' jni/Application.mk
make
