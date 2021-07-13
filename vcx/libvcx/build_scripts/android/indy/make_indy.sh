#!/bin/bash

curl https://sh.rustup.rs -sSf > rustup-init.sh
chmod a+x rustup-init.sh
./rustup-init.sh -y
source .cargo/env
python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py --arch ${TARGET_ARCH} --api ${TARGET_API} --install-dir ${TOOLCHAIN_DIR}
cat << EOF > .cargo/config
[target.${CROSS_COMPILE}]
ar = "${AR}"
linker = "${CC}"
EOF

rustup target add ${CROSS_COMPILE}

cd "${HOME}/indy-sdk/libindy"
if [ "${FINAL}" == "1" ] ; then
    export OPENSSL_STATIC=1
    export LIBSODIUM_STATIC=1
    cargo build --release --target=${CROSS_COMPILE}
    $CC -shared -o ${HOME}/libindy.so -Wl,--whole-archive ${TOOLCHAIN_DIR}/sysroot/usr/lib/${CROSS_COMPILE}/libz.a ${TOOLCHAIN_DIR}/sysroot/usr/lib/${CROSS_COMPILE}/libm.a ${HOME}/indy-sdk/libindy/target/${CROSS_COMPILE}/release/libindy.a ${OPENSSL_DIR}/lib/libssl.a ${OPENSSL_DIR}/lib/libcrypto.a ${SODIUM_LIB_DIR}/libsodium.a ${LIBZMQ_LIB_DIR}/libzmq.a ${TOOLCHAIN_DIR}/sysroot/usr/lib/${CROSS_COMPILE}/libstdc++.a -Wl,--no-whole-archive -z muldefs
else
    cargo build --release --target=${CROSS_COMPILE}
    cp "${HOME}/indy-sdk/libindy/target/${CROSS_COMPILE}/release/libindy.so" ${HOME}/
fi
cp "${HOME}/indy-sdk/libindy/target/${CROSS_COMPILE}/release/libindy.a" ${HOME}/
