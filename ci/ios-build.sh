#!/bin/sh

set -e
set -x

if [ "$1" = "--help" ] ; then
  echo "Usage: <package> <targets>"
  return
fi

package="$1"

[ -z ${package} ] && exit 1

export PKG_CONFIG_ALLOW_CROSS=1
export POD_FILE_NAME=${package}.tar.gz
export LIBINDY_POD_VERSION=1.8.2

if [ -z "${OPENSSL_DIR}" ]; then
    export OPENSSL_DIR=$(brew --prefix openssl)
fi

echo "Build IOS POD started..."

TYPE="debug"

cd ${package}

if [[ $# -eq 2 ]]; then # build for single platform
  echo "... for target $2 ..."
  cargo lipo --targets $2
elif [[ $# -eq 3 ]]; then # build for two platforms
  echo "... for targets $2,$3 ..."
  TYPE="release"
  cargo lipo --$TYPE --targets $2,$3
else  # build for all platforms
  echo "... for all default targets ..."
  TYPE="release"
  cargo lipo --$TYPE
fi
echo 'Build completed successfully.'

WORK_DIR="out_pod"
echo "Try to create out directory: $WORK_DIR"
mkdir $WORK_DIR

if [[ ! "$WORK_DIR" || ! -d "$WORK_DIR" ]]; then
  echo "Could not create temp dir $WORK_DIR"
  exit 1
fi

echo "Packing..."

PACKAGE="${package}.a"

cp include/*.h $WORK_DIR
cp ../LICENSE $WORK_DIR
cp target/universal/$TYPE/$PACKAGE $WORK_DIR
cd $WORK_DIR
tar -cvzf $POD_FILE_NAME *
cd -
ls -l $WORK_DIR/$POD_FILE_NAME

echo "Packing completed."

echo "Out directory: $WORK_DIR"
