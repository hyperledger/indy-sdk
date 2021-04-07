## Indy SDK for Android

This Android wrapper currently API level 21 or higher.

Pull requests welcome!

### How to install
Android library is noy yet deployed to maven repo. To use it you have to build it yourself as described below.

### How to build

Then run:

    ./gradlew assambleDebug

_Note that this will download libindy from https://repo.sovrin.org/android/libindy/stable._
_Version of libindy is defined in `indy.version` in `gradle.properties` file._

### Example use
For the main workflow examples check test folder: https://github.com/hyperledger/indy-sdk/tree/master/wrappers/android/src/androidTest/java/org/hyperledger/indy/sdk

#### Logging
The Android wrapper uses slf4j as a facade for various logging frameworks, such as java.util.logging, logback and log4j.
