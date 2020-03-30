Name: libvcx
Version: %{getenv:VCXVERSION}
Release: %{getenv:VCXREVISION}
Summary: This is the official SDK for Evernym's VCX
License: Apache License 2.0
Group: System Environment/Libraries
Requires: python3, zeromq, libindy
AutoReq: no

%define _rpmdir target/
%define _rpmfilename %%{NAME}_%%{VERSION}-%%{RELEASE}_%%{ARCH}.rpm
%define _unpackaged_files_terminate_build 0

%post
#!/bin/sh
# postinst script for libvcx
/sbin/ldconfig
ln -sf `ls -rt /usr/lib/libvcx.so.* | tail -n1` /usr/lib/libvcx.so

%postun
#!/bin/sh
# postinst script for libvcx
if [ "$1" -eq 0 ]; then
  rm -f /usr/lib/libvcx.so
fi
/sbin/ldconfig

%description
This is Evernym's SDK for managing Verifiable Credential eXchange against an
Indy network. For specific instructions on building see the README in the
corresponding github repo https://github.com/evernym/sdk

%files
"/usr/lib/libvcx.so.*"
"/usr/share/libvcx/vcx.h"
"/usr/share/libvcx/provision_agent_keys.py"
