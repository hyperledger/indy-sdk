#!/bin/bash -xe

if [ "$1" = "--help" ] ; then
  echo "Usage: <version> <release> <key> <repo_pass> <type> <package>"
  return
fi

version="$1"
release="$2"
key="$3"
repo_pass="$4"
type="$5"
package="$6"
dir=$(pwd)
result_dir=$(pwd)/rpms

[ -z $version   ] && exit 1
[ -z $release   ] && exit 2
[ -z $key       ] && exit 3
[ -z $repo_pass ] && exit 4
[ -z $type      ] && exit 5
[ -z $package   ] && exit 6

sed \
	-e "s|@version@|$version|g" \
	-e "s|@dir@|$dir|g" \
	-e "s|@release@|$release|g" \
	-e "s|@result_dir@|$result_dir|g" \
    rpm/${package}.spec.in > ${package}.spec

mkdir ${result_dir}

fakeroot rpmbuild -ba ${package}.spec --nodeps || exit 7

// Legacy repository
cat <<EOF | sftp -v -oStrictHostKeyChecking=no -i $key repo@$SOVRIN_REPO_HOST
mkdir /var/repository/repos/rpm/$package/$type/$version-$release
cd /var/repository/repos/rpm/$package/$type/$version-$release
put -r ${result_dir}/*
ls -l /var/repository/repos/rpm/$package/$type/$version-$release
EOF

// New repository
rpm_name=$(basename ${result_dir}/*.rpm)
curl --fail -u "$SOVRIN_REPO_USER:$repo_pass" --upload-file ${result_dir}/${rpm_name} https://$SOVRIN_REPO_HOST/repository/$package/$type/$rpm_name
