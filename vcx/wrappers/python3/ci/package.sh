#!/bin/bash
set -e
export PATH=${PATH}:$(pwd)/vcx/ci/scripts
export VCX_VERSION=$(toml_utils.py vcx/libvcx/Cargo.toml)
echo "VCX_VERSION: ${VCX_VERSION}"
export DIST=`pwd`/vcx/wrappers/python3/dist/
echo "DIST: ${DIST}"
export PACKAGE_NAME='python3-vcx-wrapper'
echo "PACKAGE_NAME: ${PACKAGE_NAME}"
cd vcx/wrappers/python3
python3 setup.py sdist
PACKAGE=${PACKAGE_NAME}-${VCX_VERSION}.tar.gz
FILELY_PACKAGE=${PACKAGE_NAME}_${VCX_VERSION}.tar.gz
# Added test so that we can confirm the new package name and that it was created.
echo "Listing the Package Directory"
echo "+++++++++++++++++++"
echo
ls -al ${DIST}
echo
echo "==================="
echo
echo "Listing the Package Itself"
echo "+++++++++++++++++++"
echo
ls -al ${DIST}/${PACKAGE}
echo
echo "==================="
echo
if [ ! -e ${DIST}/${PACKAGE} ]; then
    echo "Python Package Not Created"
	echo "+++++++++++++++++++"
	echo "${DIST}/${PACKAGE}"
    echo
    echo "==================="
    echo
    exit 1
fi
cd ../../..
mv ${DIST}/${PACKAGE} output/${FILELY_PACKAGE}
