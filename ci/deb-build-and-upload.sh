#!/bin/bash

set -e
set -x

if [ "$1" = "--help" ] ; then
  echo "Usage: <package> <version> <type> <repo> <host> <key> <package_type> <extra_flags>"
  return
fi

package="$1"
version="$2"
type="$3"
repo="$4"
host="$5"
key="$6"
package_type="$7"
extra_flags="$8"

[ -z $package ] && exit 1
[ -z $version ] && exit 2
[ -z $type ] && exit 3
[ -z $repo ] && exit 5
[ -z $host ] && exit 6
[ -z $key ] && exit 7
[ -z $package_type ] && exit 8

sed -i -E -e "s/provides = \"${package} \(= [(,),0-9,.]+\)\"/provides = \"${package} \(= ${version}\)\"/g" Cargo.toml
sed -i -e "s/RELEASE=\(%RELEASE%\)/RELEASE=$type/" debian/postinst

cargo deb --no-build --deb-version ${version}-${package_type} --variant ${package}-${package_type}

mkdir debs &&  mv target/debian/*.deb ./debs/

./sovrin-packaging/upload_debs.py ./debs $repo $type --distro=$package_type --host $host --ssh-key $key $extra_flags
