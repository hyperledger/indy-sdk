#!/usr/bin/env bash

set -e
publish() {
    VCX_VERSION=$1
    if [ -z ${VCX_VERSION} ]; then
        echo "please provide the vcx version that corresponds to linux builds (python, node, deb)"
        exit 1
    fi

    AAR_FOLDER=vcx/wrappers/java/artifacts/aar
    echo "Uploading .aar with version number ==> ${VCX_VERSION}"
    cp -v settings.xml ${AAR_FOLDER}
    pushd ${AAR_FOLDER}

        mv com.evernym-vcx_*-release.aar com.evernym-vcx_${VCX_VERSION}_x86-armv7-release.aar

        mvn -e deploy:deploy-file \
            -Durl="https://evernym.mycloudrepo.io/repositories/libvcx-android" \
            -DrepositoryId="io.cloudrepo" \
            -Dversion=${VCX_VERSION} \
            -Dfile="com.evernym-vcx_${VCX_VERSION}_x86-armv7-release.aar" \
            -DartifactId="vcx" \
            -Dpackaging="aar" \
            -DgroupId="com.evernym" \
            --settings settings.xml
    popd
}


publish $1