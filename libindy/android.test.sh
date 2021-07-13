#!/usr/bin/env bash
set -x

WORKDIR=${PWD}
LIBINDY_WORKDIR=${WORKDIR}
CI_DIR="${LIBINDY_WORKDIR}/ci"
BUILD_TYPE="--release"
export ANDROID_BUILD_FOLDER="/tmp/android_build"

TARGET_ARCH=$1

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi

if [ -z "${TEST_POOL_IP}" ]; then
    echo "Missing TEST_POOL_IP value, setting default value of 10.0.0.2"
    export TEST_POOL_IP="10.0.0.2"
fi

set -e

source ${CI_DIR}/setup.android.env.sh
generate_arch_flags ${TARGET_ARCH}

echo ">> in runner script"
WORKDIR=${PWD}
declare -a EXE_ARRAY

build_test_artifacts(){
    pushd ${WORKDIR}

        set -e

        cargo clean

        # TODO empty for full testing SET_OF_TESTS=''
        SET_OF_TESTS='--test interaction'

        # TODO move RUSTFLAGS to cargo config and do not duplicate it here
        # build - separate step to see origin build output
        RUSTFLAGS="-lc -lz -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -lsodium -lzmq -lc++_shared" \
            cargo build ${BUILD_TYPE} --target=${TRIPLET}

        # This is needed to get the correct message if test are not built. Next call will just reuse old results and parse the response.
        RUSTFLAGS="-lc -lz -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -lsodium -lzmq -lc++_shared" \
            cargo test ${BUILD_TYPE} --target=${TRIPLET} ${SET_OF_TESTS} --no-run

        # collect items to execute tests, uses resulting files from previous step
        EXE_ARRAY=($(
            RUSTFLAGS="-lc -lz -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -lsodium -lzmq -lc++_shared" \
                cargo test ${BUILD_TYPE} --target=${TRIPLET} ${SET_OF_TESTS} --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"))
    popd
}

create_cargo_config(){
mkdir -p ${LIBINDY_WORKDIR}/.cargo
cat << EOF > ${LIBINDY_WORKDIR}/.cargo/config
[target.${TRIPLET}]
ar = "$(realpath ${AR})"
linker = "$(realpath ${CC})"
EOF
}

execute_on_device(){

    set -x

    adb -e push \
    "${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/${TRIPLET}/libc++_shared.so" "/data/local/tmp/libc++_shared.so"

    adb -e push \
    "${SODIUM_LIB_DIR}/libsodium.so" "/data/local/tmp/libsodium.so"

    adb -e push \
    "${LIBZMQ_LIB_DIR}/libzmq.so" "/data/local/tmp/libzmq.so"

    adb -e push \
    "${LIBINDY_WORKDIR}/target/${TRIPLET}/release/libindy.so" "/data/local/tmp/libindy.so"

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
        time adb -e shell "TEST_POOL_IP=$TEST_POOL_IP LD_LIBRARY_PATH=/data/local/tmp RUST_TEST_THREADS=1 RUST_BACKTRACE=1 RUST_LOG=debug /data/local/tmp/$EXE_NAME && echo $MARK" 2>&1 | tee $OUT
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
