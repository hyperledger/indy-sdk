#!/usr/bin/env bash

WORKDIR=${PWD}
TARGET_ARCH=$1
TARGET_API=$2
CROSS_COMPILE=$3
NDK_LIB_DIR="lib"

set -e

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    echo "Sample : ./build.nondocker.sh x86 16 i686-linux-android openssl_x86 libsodium_x86 libzmq_x86 libindy"
    exit 1
fi

if [ -z "${TARGET_API}" ]; then
    echo STDERR "Missing TARGET_API argument"
    echo STDERR "e.g. 21"
    echo "Sample : ./build.nondocker.sh x86 16 i686-linux-android openssl_x86 libsodium_x86 libzmq_x86 libindy"
    exit 1
fi

if [ -z "${CROSS_COMPILE}" ]; then
    echo STDERR "Missing CROSS_COMPILE argument"
    echo STDERR "e.g. i686-linux-android"
    echo "Sample : ./build.nondocker.sh x86 16 i686-linux-android openssl_x86 libsodium_x86 libzmq_x86 libindy"
    exit 1
fi

if [ "${TARGET_ARCH}" = "x86_64" ]; then
    NDK_LIB_DIR="lib64"
fi


if [ -z "${OPENSSL_DIR}" ]; then
    OPENSSL_DIR="openssl_${TARGET_ARCH}"
    if [ -d "${OPENSSL_DIR}" ] ; then
        echo "Found ${OPENSSL_DIR}"
    elif [ -z "$4" ]; then
        echo STDERR "Missing OPENSSL_DIR argument and environment variable"
        echo STDERR "e.g. set OPENSSL_DIR=<path> for environment or openssl_${TARGET_ARCH}"
        exit 1
    else
        OPENSSL_DIR=$4
    fi
    export OPENSSL_DIR=${OPENSSL_DIR}
fi

if [ -z "${SODIUM_DIR}" ] ; then
    SODIUM_DIR="libsodium_${TARGET_ARCH}"
    if [ -d "${SODIUM_DIR}" ] ; then
        echo "Found ${SODIUM_DIR}"
    elif [ -z "$5" ]; then
        echo STDERR "Missing SODIUM_DIR argument and environment variable"
        echo STDERR "e.g. set SODIUM_DIR=<path> for environment or libsodium_${TARGET_ARCH}"
        exit 1
    else
        SODIUM_DIR=$5
    fi


fi


if [ -z "${LIBZMQ_DIR}" ] ; then
    LIBZMQ_DIR="libzmq_${TARGET_ARCH}"
    if [ -d "${LIBZMQ_DIR}" ] ; then
        echo "Found ${LIBZMQ_DIR}"
    elif [ -z "$6" ]; then
        echo STDERR "Missing LIBZMQ_DIR argument and environment variable"
        echo STDERR "e.g. set LIBZMQ_DIR=<path> for environment or libzmq_${TARGET_ARCH}"
        exit 1
    else
        LIBZMQ_DIR=$6
    fi

fi


if [ -z "${LIBINDY_DIR}" ] ; then
    LIBINDY_DIR="libindy_${TARGET_ARCH}"
    if [ -d "${LIBINDY_DIR}" ] ; then
        echo "Found ${LIBINDY_DIR}"
    elif [ -z "$7" ] ; then
        echo STDERR "Missing LIBINDY_DIR argument and environment variable"
        echo STDERR "e.g. set LIBINDY_DIR=<path> for environment or libindy_${TARGET_ARCH}"
        exit 1
    else
        LIBINDY_DIR=$7
    fi
    export LIBINDY_DIR=${LIBINDY_DIR}
fi

if [ -d "${LIBINDY_DIR}/lib" ] ; then
    LIBINDY_DIR="${LIBINDY_DIR}/lib"
fi

if [ -z "${LIBNULLPAY_DIR}" ] ; then
    LIBNULLPAY_DIR="libnullpay"
    if [ -d "${LIBNULLPAY_DIR}" ] ; then
        echo "Found ${LIBNULLPAY_DIR}"
    elif [ -z "$8" ] ; then
        echo STDERR "Missing LIBNULLPAY_DIR argument and environment variable"
        echo STDERR "e.g. set LIBNULLPAY_DIR=<path> for environment or libnullpay"
        exit 1
    else
        LIBNULLPAY_DIR=$8
    fi
    if [ -d "${LIBNULLPAY_DIR}/${CROSS_COMPILE}" ] ; then
        LIBNULLPAY_DIR=${LIBNULLPAY_DIR}/${CROSS_COMPILE}
    fi
    export LIBNULLPAY_DIR=${LIBNULLPAY_DIR}
fi
if [ -d "${LIBNULLPAY_DIR}/lib" ] ; then
    LIBNULLPAY_DIR="${LIBNULLPAY_DIR}/lib"
    echo ${LIBNULLPAY_DIR}
fi



if [ "$(uname)" == "Darwin" ]; then
    echo "Downloading NDK for OSX"
    export TOOLCHAIN_PREFIX=${WORKDIR}/toolchains/darwin
    mkdir -p ${TOOLCHAIN_PREFIX}
    pushd $TOOLCHAIN_PREFIX
    if [ ! -d "android-ndk-r16b" ] ; then
        echo "Downloading android-ndk-r16b-darwin-x86_64.zip"
        wget -q https://dl.google.com/android/repository/android-ndk-r16b-darwin-x86_64.zip
        unzip -qq android-ndk-r16b-darwin-x86_64.zip
    else
        echo "Skipping download android-ndk-r16b-linux-x86_64.zip"
    fi
    export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r16b
    popd
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    echo "Downloading NDK for Linux"
    export TOOLCHAIN_PREFIX=${WORKDIR}/toolchains/linux
    mkdir -p ${TOOLCHAIN_PREFIX}
    pushd $TOOLCHAIN_PREFIX
    if [ ! -d "android-ndk-r16b" ] ; then
        echo "Downloading android-ndk-r16b-linux-x86_64.zip"
        wget -q https://dl.google.com/android/repository/android-ndk-r16b-linux-x86_64.zip
        unzip -qq android-ndk-r16b-linux-x86_64.zip
    else
        echo "Skipping download android-ndk-r16b-linux-x86_64.zip"
    fi
    export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r16b
    popd
fi


LIBVCX=../../../
CROSS_COMPILE_DIR=${CROSS_COMPILE}
TARGET_ARCH_DIR=${TARGET_ARCH}
if [ "${TARGET_ARCH}" = "armv7" ]; then
    TARGET_ARCH_DIR="arm"
    CROSS_COMPILE_DIR="arm-linux-androideabi"
fi

export SODIUM_LIB_DIR=${SODIUM_DIR}/lib
export SODIUM_INCLUDE_DIR=${SODIUM_DIR}/include
export LIBZMQ_LIB_DIR=${LIBZMQ_DIR}/lib
export LIBZMQ_INCLUDE_DIR=${LIBZMQ_DIR}/include
export PKG_CONFIG_ALLOW_CROSS=1
export CARGO_INCREMENTAL=1
export RUST_LOG=indy=trace
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=1
export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/${TARGET_ARCH_DIR}
export PATH=${TOOLCHAIN_DIR}/bin:${PATH}
export PKG_CONFIG_ALLOW_CROSS=1
export CC=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE_DIR}-clang
export AR=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE_DIR}-ar
export STRIP=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE_DIR}-strip
export CXX=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE_DIR}-clang++
export CXXLD=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE_DIR}-ld
export RANLIB=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE_DIR}-ranlib
export TARGET=android

printenv

python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py --arch ${TARGET_ARCH_DIR} --api ${TARGET_API} --install-dir ${TOOLCHAIN_DIR}
cat << EOF > ~/.cargo/config
[target.${CROSS_COMPILE}]
ar = "${AR}"
linker = "${CC}"
EOF

rustup target add ${CROSS_COMPILE}

pushd $LIBVCX
export OPENSSL_STATIC=1
#cargo clean
cargo build --release --no-default-features --features "ci" --target=${CROSS_COMPILE}
# TEMPORARY HACK (need to build libvcx without duplicate .o object files):
# There are duplicate .o object files inside the libvcx.a file and these
# lines of logic remove those duplicate .o object files
rm -rf target/${CROSS_COMPILE}/release/tmpobjs
mkdir target/${CROSS_COMPILE}/release/tmpobjs
pushd target/${CROSS_COMPILE}/release/tmpobjs
    ${AR} -x ../libvcx.a
    ls > ../objfiles
    xargs ${AR} cr ../libvcx.a.new < ../objfiles
    ${STRIP} -S -x -o ../libvcx.a.stripped ../libvcx.a.new
    mv ../libvcx.a.stripped ../libvcx.a
popd
popd

LIBVCX_BUILDS=${WORKDIR}/libvcx_${TARGET_ARCH}
mkdir -p ${LIBVCX_BUILDS}
$CC -v -shared -o ${LIBVCX_BUILDS}/libvcx.so -Wl,--whole-archive \
${LIBVCX}/target/${CROSS_COMPILE}/release/libvcx.a \
${TOOLCHAIN_DIR}/sysroot/usr/${NDK_LIB_DIR}/libz.so \
${TOOLCHAIN_DIR}/sysroot/usr/${NDK_LIB_DIR}/libm.a \
${TOOLCHAIN_DIR}/sysroot/usr/${NDK_LIB_DIR}/liblog.so \
${LIBINDY_DIR}/libindy.a \
${LIBNULLPAY_DIR}/libnullpay.a \
${TOOLCHAIN_DIR}/${CROSS_COMPILE_DIR}/${NDK_LIB_DIR}/libgnustl_shared.so \
${OPENSSL_DIR}/lib/libssl.a \
${OPENSSL_DIR}/lib/libcrypto.a \
${SODIUM_LIB_DIR}/libsodium.a \
${LIBZMQ_LIB_DIR}/libzmq.a \
${TOOLCHAIN_DIR}/${CROSS_COMPILE_DIR}/${NDK_LIB_DIR}/libgnustl_shared.so -Wl,--no-whole-archive -z muldefs

${STRIP} -S -x -o ${LIBVCX_BUILDS}/libvcx.so.new ${LIBVCX_BUILDS}/libvcx.so
mv ${LIBVCX_BUILDS}/libvcx.so.new ${LIBVCX_BUILDS}/libvcx.so

cp "${LIBVCX}/target/${CROSS_COMPILE}/release/libvcx.a" ${LIBVCX_BUILDS}/
