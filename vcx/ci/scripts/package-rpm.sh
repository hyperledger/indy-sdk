#!/bin/bash
set -e
OUTPUTDIR=output
CURDIR=$(pwd)
export PATH=${PATH}:$(pwd)/vcx/ci/scripts
cd vcx/libvcx/
export RUST_FLAG=$1
VERSION=$2
REVISION=$3
echo "Updating Version in Cargo.toml file"
cargo update-version ${VERSION} ${REVISION}
echo "Updating Cargo"
if [ "${RUST_FLAG}" != "--no-test" ]; then
    echo "Testing libvcx.so"
    cargo test --no-default-features --features "ci" -- --test-threads=1
fi
echo "Building libvcx.so"
cargo build --no-default-features --features "ci"
echo "Updating libvcx.so File with Version"
cargo update-so
echo "Creating .rpm file"

mkdir -p ./target/rpmroot/usr/lib/
cp ./target/debug/libvcx.so.* ./target/rpmroot/usr/lib/

mkdir -p ./target/rpmroot/usr/share/libvcx/
cp ./include/vcx.h ./target/rpmroot/usr/share/libvcx/
cp ./scripts/provision_agent_keys.py ./target/rpmroot/usr/share/libvcx/

rpmbuild --buildroot=${PWD}/target/rpmroot -bb --target x86_64 rpm/libvcx.spec

if [ ! -d ${CURDIR}/${OUTPUTDIR} ]; then
  mkdir -p ${CURDIR}/${OUTPUTDIR}
fi

cp target/*.rpm ${CURDIR}/${OUTPUTDIR}
