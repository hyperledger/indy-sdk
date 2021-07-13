# Build `libindy` for Ubuntu

## Setup Indy SDK build environment for Ubuntu based distro (Ubuntu 16.04)

1. Install Rust and rustup (https://www.rust-lang.org/install.html).
1. Install required native libraries and utilities:

   ```
   apt-get update && \
   apt-get install -y \
      build-essential \
      pkg-config \
      cmake \
      libssl-dev \
      libsqlite3-dev \
      libzmq3-dev \
      libncursesw5-dev
   ```
   
1. `libindy` requires the modern `1.0.14` version of `libsodium` but Ubuntu 16.04 does not support installation it's from `apt` repository.
 Because of this, it requires to build and install `libsodium` from source:
 ```
cd /tmp && \
   curl https://download.libsodium.org/libsodium/releases/old/unsupported/libsodium-1.0.14.tar.gz | tar -xz && \
    cd /tmp/libsodium-1.0.14 && \
    ./configure --disable-shared && \
    make && \
    make install && \
    rm -rf /tmp/libsodium-1.0.14
```

1. Build `libindy`

   ```
   git clone https://github.com/hyperledger/indy-sdk.git
   cd ./indy-sdk/libindy
   cargo build
   cd ..
   ```
   
**Note:** `libindy` debian package, installed from the apt repository, is statically linked with `libsodium`. 
For manually building this can be achieved by passing `--features sodium_static` into `cargo build` command.
   
   
1. Run integration tests:
    [Start local nodes pool with Docker](https://github.com/hyperledger/indy-sdk/blob/master/README.md#how-to-start-local-nodes-pool-with-docker)


     If you use this method then you have to specify the TEST_POOL_IP as specified below  when running the tests.

     It can be useful if we want to launch integration tests inside another container attached to
     the same docker network.

   * Run tests

     ```
     cd libindy
     RUST_TEST_THREADS=1 cargo test
     ```

     It is possible to change ip of test pool by providing of TEST_POOL_IP environment variable:

     ```
     RUST_TEST_THREADS=1 TEST_POOL_IP=10.0.0.2 cargo test
     ```

1. Build `indy-cli` (Optional)

   `indy-cli` is dependent on `libindy` and should be built after it.

   ```
   cd cli/
   RUSTFLAGS=" -L ../libindy/target/debug" cargo build
   ```
   If you have followed the instructions to build libindy above, the default build type will be `debug`

   Make sure to add the libindy to the path. Replace `/path/to` with the actual path to the libindy directory. Using bash:
   ```
   echo "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libindy/target/{BUILD TYPE}" >> ~/.bashrc
   sudo ldconfig
   source ~/.bashrc
   ```
   To run indy-cli, navigate to `cli/target/debug` and run `./indy-cli`

See [libindy/ci/ubuntu.dockerfile](https://github.com/hyperledger/indy-sdk/tree/master/libindy/ci/ubuntu.dockerfile) for example of Ubuntu based environment creation in Docker.

## Build using Docker 

To build `libindy` for Ubuntu using Docker :
1) Go to `indy-sdk/libindy`
2) Run `build-libindy-ubuntu-with-docker.sh`

This script will build the Docker image `build-libindy-ubuntu` which
contains the built lib in `/home/libindy/target/debug/`.

You can recover the lib `libindy.so` in your current directory `.` with this commands :
```bash
docker container create --name extract build-libindy-ubuntu
docker container cp extract:/home/libindy/target/debug/libindy.so .
docker container rm -f extract
```
