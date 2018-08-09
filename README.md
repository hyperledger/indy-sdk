# Indy SDK
![logo](https://raw.githubusercontent.com/hyperledger/indy-node/master/collateral/logos/indy-logo.png)
This is the official SDK for [Hyperledger Indy](https://www.hyperledger.org/projects),
which provides a distributed-ledger-based foundation for [self-sovereign identity](https://sovrin.org). Indy provides a software ecosystem for private, secure, and powerful identity, and the Indy SDK enables clients for it.
The major artifact of the SDK is a c-callable
library; there are also convenience wrappers for various programming languages and Indy CLI tool.

All bugs, stories, and backlog for this project are managed through [Hyperledger's Jira](https://jira.hyperledger.org/secure/RapidBoard.jspa)
in project IS (note that regular Indy tickets are in the INDY project instead...). Also, make sure to join
us on [Hyperledger's Rocket.Chat](https://chat.hyperledger.org/) at #indy-sdk to discuss. You will need a Linux Foundation login to get access to these channels

## Understanding Hyperledger Indy

If you have just started learning about self-sovereign identity, here are some resources to increase your understanding:

* This extended tutorial introduces Indy, explains how the whole ecosystem works, and how the
functions in the SDK can be used to construct rich clients: [Indy-SDK Getting-Started Guide](doc/getting-started/getting-started.md)

* A recent webinar explaining self-sovereign identity using Hyperledger Indy and Sovrin: [SSI Meetup Webinar](https://youtu.be/RllH91rcFdE?t=4m30s)

* Visit the main resource for all things "Indy" to get acquainted with the code base, helpful resources, and up-to-date information: [Hyperledger Wiki-Indy](https://wiki.hyperledger.org/projects/indy).

* You may also want to look at the [older guide](https://github.com/hyperledger/indy-node/blob/stable/getting-started.md)
that explored the ecosystem via command line. That material is being
rewritten but still contains some useful ideas.

## How-To Tutorials

Short, simple tutorials that demonstrate how to accomplish common tasks
are also available. See the [doc/how-tos](doc/how-tos) folder.

1. [Write a DID and Query Its Verkey](doc/how-tos/write-did-and-query-verkey/README.md)
2. [Rotate a Key](doc/how-tos/rotate-key/README.md)
3. [Save a Schema and Cred Def](doc/how-tos/save-schema-and-cred-def/README.md)
4. [Issue a Credential](doc/how-tos/issue-credential/README.md)
5. [Negotiate a Proof](doc/how-tos/negotiate-proof/README.md)
6. [Send a Secure Message](doc/how-tos/send-secure-msg/README.md)

## Installing the SDK
### Release channels
The Indy SDK release process defines the following release channels:

* `master` - development builds for each push to master branch.
* `rc` - release candidates.
* `stable` - stable releases.

Please refer to our [release workflow](doc/release-workflow.md) for more details.

### Ubuntu based distributions (Ubuntu 16.04)
It is recommended to install the SDK packages with APT:

    sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 68DB5E88
    sudo add-apt-repository "deb https://repo.sovrin.org/sdk/deb xenial {release channel}"
    sudo apt-get update
    sudo apt-get install -y libindy

{release channel} must be replaced with master, rc or stable to define corresponded release channel.
Please See the section "Release channels" above for more details.

### Windows

1. Go to https://repo.sovrin.org/windows/libindy/{release-channel}.
2. Download last version of libindy.
3. Unzip archives to the directory where you want to save working library.
4. After unzip you will get next structure of files:

* `Your working directory`
    * `include`
        * `...`
    * `lib`
        * `indy.dll`
        * `libeay32md.dll`
        * `libsodium.dll`
        * `libzmq.dll`
        * `ssleay32md.dll`

`include` contains c-header files which contains all necessary declarations
that may be need for your applications.

`lib` contains all necessary binaries which contains libindy and all it's dependencies.
 `You must add to PATH environment variable path to lib`. It's necessary for dynamic linkage
 your application with libindy.

{release channel} must be replaced with master, rc or stable to define corresponded release channel.
See section "Release channels" for more details.

### iOS
See [wrapper iOS install documentation](wrappers/ios/README.md "How to install").

### Android

1. Go to `https://repo.sovrin.org/android/libindy/{release-channel}`.
2. 3 architecture are supported as of now arm,arm64 and x86.
3. Download latest version of libindy.
4. Unzip archives to the directory where you want to save the `.so` files.
5. After unzip you will get next structure of files:

* `Your working directory`
    * `include`
        * `...`
    * `lib`
        * `libindy.so`
        * `libindy_shared.so`
        * `libindy.a`

`include` contains c-header files which contains all necessary declarations
that may be need for your applications.

`lib` contains three types of binaries.
 * `libindy.so` - This is a shared library which is statically linked with all the depenedencies. 
 You dont need to sidelaod other dependencies like zmq, sodium and openssl to android app if you use this.
 
 * `libindy_shared.so` - This is pure shared library. It is not dynamically linked to its dependencies. 
 You need to sideload the binaries with its dependencies. You can download the needed pre-built dependencies from [here](https://github.com/evernym/indy-android-dependencies/tree/v1.0.2)
    * Rename this library to `libindy.so` before loading it into the app. This will help you in having the compatibility with existing wrappers.
    
 * `libindy.a` - This is a static library, which is compiled with NDK.
 
 [How to use instructions.](https://github.com/hyperledger/indy-sdk/blob/master/doc/android-build.md#usage)  

{release channel} must be replaced with rc or stable to define corresponded release channel.
See section "Release channels" for more details.

 **Note** :
 
 - [WARNING] This library should be considered as experimental as currently unit tests are *not* executed in the CI phase.
 
 - We are using the [NDK16b](https://dl.google.com/android/repository/android-ndk-r16b-linux-x86_64.zip) because it is the last NDK to have support for `gnustl_shared` stl. 
 gnustl_shared is deprecated in latest NDK. gnustal_shared is needed because the dependencies are compiled using gnustal_shared and you will get build errors if more than one type of stl while compiling.
 
 
### MacOS

Pre-built libraries are not provided for MacOS. Please look [here](doc/mac-build.md)
for details on building from source for MacOS.

 **Note:** After building `libindy`, add the path containing the library the `LD_LIBRARY_PATH` and
`DYLD_LIBRARY_PATH` environment variables. This is necessary for dynamically linking
your application with `libindy`. The dynamic linker will first check for the library in
`LD_LIBRARY_PATH` if the library in your application doesn't include directory names.
If the library in your application does include any directory name, then dynamic
linker will search for the library in `DYLD_LIBRARY_PATH` (not `LD_LIBRARY_PATH`)
so we recommend you set both variables to be safe.

### RHEL-based distributions (Amazon Linux 2017.03)
Pre-built libraries are not provided for RHEL-based distributions. Please look [here](doc/rhel-build.md)
for details on building from source for RHEL-based distributions.

After successfully compiling `libindy`, you will need to add the path containing `libindy.so` to the
`LD_LIBRARY_PATH` environment variable. This is required for your application to link to
`libindy`.

## How to build Indy SDK from source

* [Ubuntu based distributions (Ubuntu 16.04)](doc/ubuntu-build.md)
* [RHEL based distributions (Amazon Linux 2017.03)](doc/rhel-build.md)
* [Windows](doc/windows-build.md)
* [MacOS](doc/mac-build.md)
* [Android](doc/android-build.md)

**Note:**
By default `cargo build` produce debug artifacts with a large amount of run-time checks.
It's good for development, but this build can be in 100+ times slower for some math calculation.
If you would like to analyse CPU performance of libindy for your use case, you have to use release artifacts (`cargo build --release`). 

## How to start local nodes pool with docker
To test the SDK codebase with a virtual Indy node network, you can start a pool of local nodes using docker:

Start the pool of local nodes on your docker host `9701-9708` by running:

```
cd ci
./manage build
./manage start
```

`manage` script supports an optional environment `POOL_IP` param that allows changing ip of pool nodes in generated pool configuration. 

### Docker port mapping on MacOS

If you use some Docker distribution based on Virtual Box you can use Virtual Box's
port forwarding future to map 9701-9709 container ports to local 9701-9709 ports.

If you use VMWare Fusion to run Docker locally, follow the instructions from
[this article](https://medium.com/@tuweizhong/how-to-setup-port-forward-at-vmware-fusion-8-for-os-x-742ad6ca1344)
and add the following lines to _/Library/Preferences/VMware Fusion/vmnet8/nat.conf_:

```
# Use these with care - anyone can enter into your VM through these...
# The format and example are as follows:
#<external port number> = <VM's IP address>:<VM's port number>
#8080 = 172.16.3.128:80
9701 = <your_docker_ip>:9701
9702 = <your_docker_ip>:9702
9703 = <your_docker_ip>:9703
9704 = <your_docker_ip>:9704
9705 = <your_docker_ip>:9705
9706 = <your_docker_ip>:9706
9707 = <your_docker_ip>:9707
9708 = <your_docker_ip>:9708
9709 = <your_docker_ip>:9709
```
where <your_docker_ip> is your Docker host IP.

Docker machine needs to be rebooted after these changes.

## Wrappers documentation

The following wrappers are tested and complete. There is also active work
on a wrapper for Go; visit
[#indy-sdk on Rocket.Chat](https://chat.hyperledger.org/channel/indy-sdk) for
details.

* [.Net](wrappers/dotnet/README.md)
* [Java](wrappers/java/README.md)
* [Python](wrappers/python/README.md)
* [iOS](wrappers/ios/README.md)
* [NodeJS](wrappers/nodejs/README.md)

## Indy CLI documentation
* An explanation of how to install the official command line interface for that provides commands to manage wallets and interactions with the ledger: [Indy CLI](cli/README.md)

## How to migrate
The documents that provide necessary information for Libindy migration. This document is written for developers using Libindy 1.3.0 to provide necessary information and
to simplify their transition to API of Libindy 1.4.0.
* [v1.3.0 → v1.4.0](doc/migration-guide-1.3.0-1.4.0.md)
* [v1.4.0 → v1.5.0](doc/migration-guide-1.4.0-1.5.0.md)
* [v1.5.0 → v1.6.x](doc/migration-guide-1.5.0-1.6.0.md)

## How to Contribute

* We'd love your help; see these [instructions on how to contribute](http://bit.ly/2ugd0bq).
* You may also want to read this info about [maintainers](MAINTAINERS.md) and our process.
* We use developer certificate of origin (DCO) in all hyperledger repositories,
  so to get your pull requests accepted, you must certify your commits by signing off on each commit.
  More information can be found in [Signing Commits](doc/signing-commits.md) article.