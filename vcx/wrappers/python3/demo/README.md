# Content

There are 2 versions of this demo, the original version (faber.py and alice.py), and a slightly modified 
version (faber-pg.py and alice-pg.py) that supports postgres wallet storage, and illustrated how the vcx 
object serialization/deserialization can be used.

# Running the original demo

This directory contains the following files:

* faber.py - a script that acts as an institution/enterprise by sending a connection 
request, writing a schema/cred_def to the ledger, sending a credential offer, and 
requesting proof.  

* alice.py - a script that acts as an individual by accepting a connection offer, 
requesting a credential and offering proof.

* pool-local.txn - genesis files for connecting to an indy pool on localhost (127.0.0.1) 
(existing file connects to libindy/sovtoken ledger). In Docker environment, the genesis file
is specified for Python scripts (`faber.py` and `alice.py`) by environment variable 
`POOL_TXN_FILE=sandbox/pool_transactions_genesis` where `sandbox` is a Docker volume shared by
`vcx-demo-indy-pool` container, containing the `pool_transactions_genesis` file generated in the Docker 
image `vcx-demo-indy-pool`.

You can run on local (dev) environment, or in Docker environment.

## Run on local (dev) environment

To run these follow the next steps:
 1) Install the latest vcx python package
 2) Install a payment plugin
 3) Start Dummy Cloud Agent according to instruction: https://github.com/hyperledger/indy-sdk/tree/master/vcx/dummy-cloud-agent/README.md
 4) Execute the "faber.py" script first with `python3.5 faber.py`. This script will 
    explain what it is doing and output invite details between the line 
    `**invite details**` and the line `******************`. 
 5) Copy this invite details
 6) Start the "alice.py" script with `python3.5 alice.py`. This script will ask for 
    invite details which can be pasted (from the output of the faber.py script)
 7) Once the connection is established the faber.py script will send a credential offer, credential and proof request automatically.
    The alice.py script will request a credential, store it and offer proof when asked.
    Once they have interacted they will both exit.

## Run on Docker environment

### Prerequisites

The current directory must contains the Indy lib file `libindy.so`. To generate this file
from current sources :
1) Go to `indy-sdk/libindy` directory
2) Follow "Build using Docker" in the file [ubuntu-build](../../../../doc/ubuntu-build.md)
3) Recover `libindy.so` as explained in [ubuntu-build](../../../../doc/ubuntu-build.md)
4) Move `libindy.so` to current directory (i.e. `vcx/wrappers/python3/demo`)

### Instructions

 1) Run `make docker-build` to build necessary images (to do once only)
 2) Run `make docker-start` to start all containers
 3) In a second terminal, run `make docker-exec` and `python3 faber.py`. This script will 
    explain what it is doing and output invite details between the line 
    `**invite details**` and the line `******************`. 
 4) Copy this invite details
 5) In a third terminal, run `make docker-exec` and `python3 alice.py`. This script will ask for 
    invite details which can be pasted (from the output of the faber.py script)
 6) Once the connection is established the faber.py script will send a credential offer, credential and proof request automatically.
    The alice.py script will request a credential, store it and offer proof when asked.
    Once they have interacted they will both exit.
    
 You can clean the Docker environment with the command `make docker-clean`.
 
# Slightly Modified Demo

This demo is run using the following files:

* faber-pg.py : same as faber.py, however once a connection is established, this script provides a menu:
  * 1 = send credential to Alice
  * 2 = send proof request to Alice
  * 3 = poll the Alice connection to see if Alice has sent any messages
  * x = stop and exit

* alice-pg.py : same as alice.py, however once a connection is established, this script provides a menu:
  * y = poll the connection for messages (credential offers or proof requests)
  * n = stop and exit

To create the alice/faber wallets using postgres storage, just add the "--postgres" option when running the script.

Internally, the scripts serialize and deserialize the vcx objects between operations.  
In "real life", these serialized objects could be stored to the database or to wallet non-secrets storage.
