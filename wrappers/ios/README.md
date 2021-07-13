# How to install
A wrapper is a private pod, so private podspec must be set. Put at the top of the Podfile: 
    
    source 'https://github.com/hyperledger/indy-sdk.git'
    
Cocoapod will search for spec files in the root Specs folder.
           
# Binaries

* There are 2 types of automatically built archives containing libindy core library which can be used for your iOS application.
    * library built for aarch64-apple-ios target.
    * universal libindy built for all architectures.
    
    Add pod to target:
    
        pod 'libindy'
       
    These archives are located at `https://repo.sovrin.org/ios/libindy/{release-channel}/libindy-core/`.
    
* There is manually built and published archive containing libindy objective C wrapper.
   This archive is located at `https://repo.sovrin.org/ios/libindy/{release-channel}/indy-objc/`.

    Add pod to target:
        
        pod 'libindy-objc'    
  
{release channel} must be replaced with master, rc or stable to define corresponded release channel.

# How to manually build

## Core library

*Note: Rust 1.46 forces building cdylib crate that causes linking errors. See https://github.com/rust-lang/rust/issues/79373 for details. It will be fixed in 1.49, but until it is out we recommed to use 1.45*

1. Install Rust and rustup (https://www.rust-lang.org/install.html).
1. Install toolchains using command:

   ```
   rustup target add aarch64-apple-ios x86_64-apple-ios
   ```
1. Install cargo-lipo:
   
   ```
   cargo install cargo-lipo
   ```
1. Install required native libraries and utilities:
   
   ```
   brew install libsodium
   brew install zeromq
   brew install openssl (1.0.2q can be any fresh version)
   ```
1. Setup environment variables:
   
   ```
   export PKG_CONFIG_ALLOW_CROSS=1
   export CARGO_INCREMENTAL=1
   ```
1. Edit script ci/ios-build.sh: set the following variables to fit your environment:
   
   ```
   export LIBINDY_POD_VERSION=0.0.1
   ```
   LIBINDY_POD_VERSION - version of libindy-core pod to be built
1. Run the script Validate the output that all goes well. 
  `ci/ios-build.sh {package} [{targets}]`
   
   Parameters:
   * package - target package to build.
        * libindy
        * libnullpay
   * targets - target architectures.
        * one of aarch64-apple-ios x86_64-apple-ios
        * leave empty to build for all architectures.
1. Go to `Specs/libindy` dir.
1. Create directory with name defined in LIBINDY_POD_VERSION:
   
   ```
   mkdir LIBINDY_POD_VERSION
   ```
1. Copy libindy.podspec.json to that new directory from some previous version.
1. Edit this json -> change version field to LIBINDY_POD_VERSION.
1. Add new directory and file inside to git repository.
1. Commit to master branch.
1. for all projects which using libindy do not forget to make:

   ```
   pod repo update
   pod install
   ```
   
   
## Wrapper Cocoapod

Run Archive process for `Indy` target. Custom post-action shell script `universal_framework.sh` will be triggered and you get universal framework. Then put it to folder: `libindy-objc/Indy.framework` and upload to repo.

### Troubleshooting archive post script

Archive post scripts are not integrated into Xcode log outputs. Therefore you will not see output of that script in Xcode. The sctipt will write logs into ${TMPDIR}/Indy_archive.log. 
Script is started after archive is finished in Xcode, so while Xcode indicates that everyrhing is done, the script may still be running in the background. 

# Wrapper usage 

Import header starting from 0.1.3:

```
#import <Indy/Indy.h> 
```
For 0.1.1 and 0.1.2 versions:

```
#import <libindy/libindy.h>
```

All wrapper types and classes have prefix `Indy`.

## Troubleshooting
* Enable Logging - Use `setLogger` to pass a callback that will be used by Libindy to log a record.
* [IS-1058](https://jira.hyperledger.org/browse/IS-1058) 
    * OpenSSL cp: file.tgz: No such file or directory - 
    ```
    sudo gem uninstall cocoapods-downloader
    sudo gem install cocoapods-downloader -v 1.2.0
    ```
    * Multiple commands produce `*/Debug-iphonesimulator/Indy-demo.app/PlugIns/Indy-demoTests.xctest/Info.plist` - remove **Info.plist** from there: Solution -> Target -> Build phases -> **Copy Bundle Resources** 