#!/usr/bin/env bash

set -e
JAR_FOLDER=vcx/wrappers/java/artifacts/jar
JAR_VERSION=$(find ${JAR_FOLDER} -type f -name 'com.evernym-vcx-*.jar'| perl -nle 'print $& if m{(?<=vcx-)(.*)(?=.jar)}' | head -1 | awk '{print $1}')
echo "Uploading .jar with version number ==> ${JAR_VERSION}"
cp -v settings.xml ${JAR_FOLDER}
pushd ${JAR_FOLDER}

    mvn -e deploy:deploy-file \
        -Durl="https://repo.evernym.com/artifactory/libindy-maven-local" \
        -DrepositoryId="artifactory-evernym" \
        -Dversion=${JAR_VERSION} \
        -Dname="vcx" \
        -Dfile="com.evernym-vcx-${JAR_VERSION}.jar" \
        -DartifactId="vcx" \
        -Dpackaging="jar" \
        -DgroupId="com.evernym" \
        $MAVEN_ADD_OPTIONS \
        --settings settings.xml
popd
