# Running the VCX Java Demo

### Alice/Faber demo
The alice/faber demo is widely used in the indy-sdk demo. The description of the VCX node demo explains it well, 
including the operation of the cloud agent. 
[Description here](https://github.com/hyperledger/indy-sdk/tree/master/vcx/wrappers/node#run-demo).

### Pre-requirements
#### Libraries
Before you'll be able to run demo, you need to make sure you've compiled 
- [`libindy`](https://github.com/hyperledger/indy-sdk/tree/master/libindy)
- [`libvcx`](https://github.com/hyperledger/indy-sdk/tree/master/vcx)
- [`libnullpay`](https://github.com/hyperledger/indy-sdk/tree/master/libnullpay)
- Optionally [`libindystrgpostgres`](https://github.com/hyperledger/indy-sdk/tree/master/experimental/plugins/postgres_storage) if you want to run demo
with postgres wallet.

Library binaries must be located `/usr/local/lib` on OSX, `/usr/lib` on Linux. 

#### Java wrapper for VCX library
By default, the local LibVCX wrapper is used. Make sure that `com.evernym-vcx-*.jar` is presented in `../build/libs`
[How to build](https://github.com/hyperledger/indy-sdk/blob/master/vcx/wrappers/java/README.md#jar).

Or you can use the pre-built LibVCX wrapper from maven repository. See dependencies section in `build.gradle`.

#### Indy pool
You'll also have to run pool of Indy nodes on your machine. You can achieve by simply running a docker container
which encapsulates multiple interconnected Indy nodes. 
[Instructions here](https://github.com/hyperledger/indy-sdk#how-to-start-local-nodes-pool-with-docker).

### Steps to run demo
- Start [Dummy Cloud Agent](https://github.com/hyperledger/indy-sdk/tree/master/vcx/dummy-cloud-agent)
- Run Faber agent, representing an institution
```
./gradlew faber
```
- Give it a few seconds, then run Alice's agent which will connect with Faber's agent
```
./gradlew alice
```

### Demo with Posgres wallet
You can also run demo in mode where both Faber and Alice are using Postgres wallets. Follow 
[instructions](https://github.com/hyperledger/indy-sdk/tree/master/experimental/plugins/postgres_storage) to 
compile postgres wallet plugin and startup local postgres docker container. 

Once you have that ready, use these commands to start demo in postgres mode.
```
./gradlew faber_pg
```
```
./gradlew alice_pg
```
