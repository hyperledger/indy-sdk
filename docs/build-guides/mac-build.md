# MacOS build guide

Automated build: clone the repo and run `mac.build.sh` in the `libindy` folder.

## Manual steps

1. Install Rust and rustup (https://www.rust-lang.org/install.html).
2. Install required native libraries and utilities
   ```
   brew install pkg-config
   brew install libsodium   
   brew install automake 
   brew install autoconf
   brew install cmake
   brew install openssl
   brew install zeromq
   brew install zmq
   ```
3. Setup environment variables:
   ```
   export PKG_CONFIG_ALLOW_CROSS=1
   export CARGO_INCREMENTAL=1
   export RUST_LOG=indy=trace
   export RUST_TEST_THREADS=1
   ```
4. Setup OPENSSL_DIR variable: path to installed openssl library
   ```
   for version in `ls -t /usr/local/Cellar/openssl/`; do
        export OPENSSL_DIR=/usr/local/Cellar/openssl/$version
        break
   done
   ```
5. Checkout and build the library:
   ```
   git clone https://github.com/hyperledger/indy-sdk.git
   cd ./indy-sdk/libindy
   cargo build
   ```
6. To compile the CLI, libnullpay, or other items that depend on libindy:
   ```
   export LIBRARY_PATH=/path/to/sdk/libindy/target/<config>
   cd ../cli
   cargo build
   ```
7. Set your `DYLD_LIBRARY_PATH` and `LD_LIBRARY_PATH` environment variables to the path of `indy-sdk/libindy/target/debug`. You may want to put these in your `.bash_profile` to persist them.
Else try 
`cp target/debug/libindy.dylib /usr/local/lib && npm install indy-sdk`
*Setting LD_LIBRARY_PATH no longer works due to SIP: `https://github.com/oracle/node-oracledb/issues/231`

## Note on running local nodes

In order to run local nodes on MacOS, it may be necessary to set up port mapping between the Docker container
and local host. Follow the instructions in [Indy SDK README](https://github.com/hyperledger/indy-sdk#how-to-start-local-nodes-pool-with-docker)

## IOError while running of whole set of tests on MacOS

There is a possible case when some tests are failed if whole set of tests is run (`cargo test`).
But failed tests will be successful in case of separate runs.
If an error message like `IOError` `Too many open files` is present in logs when fails can be fixed by changing default limit.

`ulimit -n <new limit value>`

https://jira.hyperledger.org/browse/IS-1038
