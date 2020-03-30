#!/usr/bin/env bash



WORKDIR="$( cd "$(dirname "$0")" ; pwd -P )"
LIBINDY_WORKDIR="$(realpath ${WORKDIR}/../../libindy)"
LIBNULLPAY_WORKDIR="$(realpath ${WORKDIR}/../../libnullpay)"
LIBVCX_WORKDIR=${WORKDIR}
CI_DIR="${LIBINDY_WORKDIR}/ci"
BUILD_TYPE="--release"
export ANDROID_BUILD_FOLDER="/tmp/android_build"

TARGET_ARCH=$1

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi

source ${CI_DIR}/setup.android.env.sh
generate_arch_flags ${TARGET_ARCH}

if [ -z "${INDY_DIR}" ] ; then
        INDY_DIR="libindy_${TARGET_ARCH}"
        if [ -d "${INDY_DIR}" ] ; then
            echo "Found ${INDY_DIR}"
        elif [ -n "$2" ] ; then
            INDY_DIR=$2
        elif [ -d "${LIBINDY_WORKDIR}/target/${TRIPLET}/release/" ] ; then
            INDY_DIR="${LIBINDY_WORKDIR}/target/${TRIPLET}/release/"
            echo "Found local INDY_DIR=${INDY_DIR}"
        else
            echo STDERR "Missing INDY_DIR argument and environment variable"
            echo STDERR "e.g. set INDY_DIR=<path> for environment or libindy_${TARGET_ARCH}"
            exit 1
        fi
        if [ -d "${INDY_DIR}/lib" ] ; then
            INDY_DIR="${INDY_DIR}/lib"
        fi
fi

echo ">> in runner script"
declare -a EXE_ARRAY

build_test_artifacts(){
    pushd ${WORKDIR}

        set -e
        # The libc.so in the standalone toolchains does not have FORTIFIED_SOURCE compatible symbols.
        # We need to copy the libc.so from platforms folder into the standalone toolchain.
        #DEPS_TARGET_API_LEVEL=21 #FIXME remove it, should be same with TARGET_API. Probably deps (sodium and/or zmq) should be rebuilt
        #cp "${ANDROID_NDK_ROOT}/platforms/android-${DEPS_TARGET_API_LEVEL}/arch-${TARGET_ARCH}/usr/lib/libc.so" "${TOOLCHAIN_DIR}/sysroot/usr/${TOOLCHAIN_SYSROOT_LIB}"

        cargo clean

        SET_OF_TESTS=''

        # TODO move RUSTFLAGS to cargo config and do not duplicate it here
        # build - separate step to see origin build output
        RUSTFLAGS="-L${TOOLCHAIN_DIR}/sysroot/usr/${TOOLCHAIN_SYSROOT_LIB} -lc -lz -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -L${INDY_DIR} -lsodium -lzmq -lc++_shared -lindy" \
        LIBINDY_DIR=${INDY_DIR} \
            cargo build ${BUILD_TYPE} --target=${TRIPLET}

        # This is needed to get the correct message if test are not built. Next call will just reuse old results and parse the response.
        RUSTFLAGS="-L${TOOLCHAIN_DIR}/sysroot/usr/${TOOLCHAIN_SYSROOT_LIB} -lc -lz -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -L${INDY_DIR} -lsodium -lzmq -lc++_shared -lindy" \
        LIBINDY_DIR=${INDY_DIR} \
            cargo test ${BUILD_TYPE} --target=${TRIPLET} ${SET_OF_TESTS} --no-run

        # collect items to execute tests, uses resulting files from previous step
        EXE_ARRAY=($(
            RUSTFLAGS="-L${TOOLCHAIN_DIR}/sysroot/usr/${TOOLCHAIN_SYSROOT_LIB} -lc -lz -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -L${INDY_DIR} -lsodium -lzmq -lc++_shared -lindy" \
            LIBINDY_DIR=${INDY_DIR} \
                cargo test ${BUILD_TYPE} --target=${TRIPLET} ${SET_OF_TESTS} --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"))

    popd
}

create_cargo_config(){
mkdir -p ${LIBVCX_WORKDIR}/.cargo
cat << EOF > ${LIBVCX_WORKDIR}/.cargo/config
[target.${TRIPLET}]
ar = "$(realpath ${AR})"
linker = "$(realpath ${CC})"
EOF
}

execute_on_device(){

    set -x

    adb -e push \
    "${TOOLCHAIN_DIR}/sysroot/usr/lib/${TRIPLET}/libc++_shared.so" "/data/local/tmp/libc++_shared.so"

    adb -e push \
    "${SODIUM_LIB_DIR}/libsodium.so" "/data/local/tmp/libsodium.so"

    adb -e push \
    "${LIBZMQ_LIB_DIR}/libzmq.so" "/data/local/tmp/libzmq.so"

    adb -e push \
    "${LIBINDY_WORKDIR}/target/${TRIPLET}/release/libindy.so" "/data/local/tmp/libindy.so"

    adb -e push \
    "${LIBNULLPAY_WORKDIR}/target/${TRIPLET}/release/libnullpay.so" "/data/local/tmp/libnullpay.so"

    adb -e logcat | grep indy &

    for i in "${EXE_ARRAY[@]}"
    do
       :
        EXE="${i}"
        EXE_NAME=`basename ${EXE}`


        adb -e push "$EXE" "/data/local/tmp/$EXE_NAME"
        adb -e shell "chmod 755 /data/local/tmp/$EXE_NAME"
        OUT="$(mktemp)"
        MARK="ADB_SUCCESS!"
        time adb -e shell "TEST_POOL_IP=10.0.0.2 LD_LIBRARY_PATH=/data/local/tmp RUST_TEST_THREADS=1 RUST_BACKTRACE=1 RUST_LOG=debug /data/local/tmp/$EXE_NAME && echo $MARK" 2>&1 | tee $OUT
        grep $MARK $OUT
    done

}



recreate_avd
setup_dependencies_env_vars ${ABSOLUTE_ARCH}
set_env_vars
create_standalone_toolchain_and_rust_target
create_cargo_config
build_test_artifacts &&
check_if_emulator_is_running &&
execute_on_device
kill_avd
