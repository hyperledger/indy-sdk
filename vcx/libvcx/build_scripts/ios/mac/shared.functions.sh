#!/bin/sh

indyBranch=${INDY_BRANCH:?"Indy branch needs to be set i.e (stbale or master)"}
indyVersion=${INDY_VERSION:?"Indy version needs to be set"}
nullBranch=${NULL_BRANCH:?"Nullpay branch needs to be set i.e (stbale or master)"}
nullVersion=${NULL_VERSION:?"Nullpay version needs to be set"}

export LIBINDY_IOS_BUILD_URL="https://repo.sovrin.org/ios/libindy/${indyBranch}/libindy-core/${indyVersion}/libindy.tar.gz"
export LIBNULLPAY_IOS_BUILD_URL="https://repo.sovrin.org/ios/libnullpay/${nullBranch}/libnullpay-core/${nullVersion}/libnullpay.tar.gz"

export LIBINDY_FILE=$(basename ${LIBINDY_IOS_BUILD_URL})
export LIBINDY_VERSION=$(basename $(dirname ${LIBINDY_IOS_BUILD_URL}))
export LIBNULLPAY_FILE=$(basename ${LIBNULLPAY_IOS_BUILD_URL})
export LIBNULLPAY_VERSION=$(basename $(dirname ${LIBNULLPAY_IOS_BUILD_URL}))

export BUILD_CACHE=~/.build_libvxc/ioscache
mkdir -p ${BUILD_CACHE}

function abspath() {
    # generate absolute path from relative path
    # $1     : relative filename
    # return : absolute path
    if [ -d "$1" ]; then
        # dir
        (cd "$1"; pwd)
    elif [ -f "$1" ]; then
        # file
        if [[ $1 = /* ]]; then
            echo "$1"
        elif [[ $1 == */* ]]; then
            echo "$(cd "${1%/*}"; pwd)/${1##*/}"
        else
            echo "$(pwd)/$1"
        fi
    fi
}
