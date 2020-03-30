#!/bin/bash

set -e
vcx_version() {
    export PATH=${PATH}:$(pwd)/vcx/ci/scripts
    VCX_VERSION=$(toml_utils.py vcx/libvcx/Cargo.toml)
    echo "VCX_VERSION: ${VCX_VERSION}"
    eval "$1='${VCX_VERSION}'"
}

setup_env() {
    export SCRIPTS_PATH="vcx/libvcx/build_scripts/ios/mac"
    export BASE_DIR="../../../../.."
    export WRAPPER_BASE="vcx/wrappers/ios/vcx"
    export WRAPPER_LIBS="vcx/wrappers/ios/vcx/lib"

    export INDY_BRANCH=$1
    export INDY_VERSION=$2
    export NULL_BRANCH=$3
    export NULL_VERSION=$4
    export SOVTOKEN_VER=$5
    export SOVTOKEN_ZIP=$6
    export RUST_VERSION=$7
    export VCX_VERSION=$8

    check_params

    cd ${SCRIPTS_PATH}

    ./mac.01.libindy.setup.sh ${RUST_VERSION}
    ./mac.02.libindy.env.sh
    ./mac.03.libindy.build.sh
    #./mac.04.libvcx.setup.sh
    ./mac.05.libvcx.env.sh

    cp -rf ~/OpenSSL-for-iPhone ${BASE_DIR}/.macosbuild
    cp -rf ~/libzmq-ios ${BASE_DIR}/.macosbuild
    cp -rf ~/combine-libs ${BASE_DIR}/.macosbuild
}

check_params() {
    if [ -z ${INDY_BRANCH} ] || [ -z ${INDY_VERSION} ] || [ -z ${NULL_BRANCH} ] || [ -z ${NULL_VERSION} ] \
    || [ -z ${SOVTOKEN_VER} ] || [ -z ${SOVTOKEN_ZIP} ] || [ -z ${RUST_VERSION} ] || [ -z ${VCX_VERSION} ]; then
        echo "missing parameters. Expected (INDY_BRANCH, INDY_VERSION, NULL_BRANCH, NULL_VERSION, SOVTOKEN_VER, SOVTOKEN_ZIP,
        RUST_VERSION, VCX_VERSION)"
        exit 1
    fi
}
set_ios_platforms() {
   export IOS_ARCHS="$1"
   export IOS_TARGETS="$2"
}

clear_previous_builds() {
    # clear previous builds from jenkins machine
    if [ ! -z "$(ls -A /Users/jenkins/IOSBuilds/libvcxpartial/)" ]; then
       echo "deleting old libvcxpartial builds"
       rm /Users/jenkins/IOSBuilds/libvcxpartial/*
    fi
    if [ ! -z "$(ls -A /Users/jenkins/IOSBuilds/libvcxall/)" ]; then
       echo "deleting old libvcxall builds"
       rm /Users/jenkins/IOSBuilds/libvcxall/*
    fi
}

build_vcx() {
    IOS_TARGETS=$1

    ./mac.06.libvcx.build.sh nodebug cleanbuild "${IOS_TARGETS}"
}

build_cocoapod() {
    COMBINED_LIB=$1
    IOS_ARCHS=$2
    VCX_VERSION=$3

    ./mac.11.copy.static.libs.to.app.sh
    ./mac.12.combine.static.libs.sh ${COMBINED_LIB} delete nodebug "${IOS_ARCHS}"
    ./mac.13.build.cocoapod.sh ${COMBINED_LIB} "${IOS_ARCHS}" "${VCX_VERSION}"
}

VCX_VERSION=''
vcx_version VCX_VERSION
set_ios_platforms "arm64,armv7,i386,x86_64" "aarch64-apple-ios,armv7-apple-ios,i386-apple-ios,x86_64-apple-ios"
setup_env $@
clear_previous_builds
build_vcx ${IOS_TARGETS}
build_cocoapod libvcxall ${IOS_ARCHS} ${VCX_VERSION}

set_ios_platforms "arm64,armv7" "aarch64-apple-ios,armv7-apple-ios"
build_cocoapod libvcxpartial ${IOS_ARCHS} ${VCX_VERSION}
