# Payment Interface

This design proposes the list of commands to Indy CLI to handle payments.

## Goals and ideas

* Indy CLI should provide ability to perform the main payments operations:
  * Creation of payment address
  * Listing of payment addresses
  * Getting list of sources for payment address
  * Sending payment transaction
  * Adding fees to transactions
  * Getting transactions fees amount
* Abstraction level should correspond to Indy SDK. For example, don't hide source abstraction. In the future we can add
  new commands to increase abstractions level.

## New CLI commands

### Create payment address

Create payment address for specific payment method in the wallet

```indy-cli
indy> payment-address create payment_method=<payment-method> seed=[<seed>]
```

Returns:

* Success or error message

### List payment addresses

List payment addresses in the wallet

```indy-cli
indy> payment-address list
```

Returns:

* Table with columns: Payment Address, Payment Method

### Send GET_PAYMENT_SOURCES request

Send request to get list of sources for specified payment addresses

```indy-cli
indy> ledger get-payment-sources payment_address=<payment-address>
```

Returns:

* Table with columns: Source, Payment Address, Amount, Extra

### Send PAYMENT transaction

Send payment transaction

```indy-cli
indy> ledger payment inputs=<source-1>,..,<source-n> outputs=(<recipient-0>,<amount>),..,(<recipient-n>,<amount>) [extra=<extra>]
```

Returns:

* Table with columns: Receipt, Recipient Payment Address, Amount, Extra

Note that "source-n" is identifier presented in "Source" column of ```ledger get-sources``` command output

### Send GET_FEES request

Send request to get fees amount for ledger transactions

```indy-cli
indy> ledger get-fees payment_method=<payment_method>
```

Returns:

* Table with columns: Transaction, Amount

### Prepare MINT transaction

Prepare MINT transaction as json.

```indy-cli
indy> ledger mint-prepare outputs=(<recipient-0>,<amount-0>),..,(<recipient-n>,<amount-n>) [extra=<extra>]
```

Returns:

* MINT transaction json

Sending MINT process is the following:

* Steward 1 calls ```ledger mint-prepare```
* Signs it by calling ```ledger sign-multi```
* Sends the request json to Steward 2 (now we have 1 signature)
* Second Steward signs it by calling ```ledger sign-multi```
* Sends the request json to Steward 3 (now we have 2 signature)
* All Stewards sign the request
* The latest Steward calls ```ledger send-custom``` to send request signed by all Stewards

### Prepare SET_FEES transaction

Prepare SET_FEES transaction as json.

```indy-cli
indy> ledger set-fees-prepare payment_method=<payment_method> fees=<txn-type-1>:<amount-1>,...,<txn-type-n>:<amount-n>
```

Returns:

* SET_FEES transaction json

Sending SET_FEES process is the following:

* Steward 1 calls ```ledger set-fees-prepare```
* Signs it by calling ```ledger sign-multi```
* Sends the request json to Steward 2 (now we have 1 signature)
* Second Steward signs it by calling ```ledger sign-multi```
* Sends the request json to Steward 3 (now we have 2 signature)
* All Stewards sign the request
* The latest Steward calls ```ledger send-custom``` to send request signed by all Stewards


#### Send VERIFY_PAYMENT_RECEIPT request

Send request to get information to verify the payment receipt
```
ledger verify-payment-receipt <receipts>
```

Returns:

* Receipt info as json

### Sign the transaction (for multi-sign case)

Add signature (for multi-sign case) by current DID to transaction json.

```indy-cli
indy> ledger sign-multi txn=<txn-json>
```

Returns:

* Transaction json with added signature

## Existing commands update

All commands to send domain transactions require new optional params to add transactions fees:

```indy-cli
[fees_inputs=<source-1>,..,<source-n>] [fees_outputs=(<recipient-0>,<amount>),..,(<recipient-n>,<amount>)] [extra=<extra>]
```

Note that "source-n" is identifier presented in "Source" column of ```ledger get-sources``` command output

### Sign with a payment address

Many cryptocurrencies provide mechanisms to prove control of a given payment address.
The following command signs messages using the private key that corresponds to the specified payment address.

```indy-cli
indy> payment-address sign address=<payment-address> input=<input>
```

"address" is the payment address generated from calling `create`.

"input is the message to be signed.

The output is a hex encoded signature that can be passed to verify

### Verify signature from a payment address

Verifies a proof of payment address control by verifying a signature.

```indy-cli
indy> payment-address verify address=<payment-address> input=<input> signature=<signature>
```

"address" is the payment address generated from calling `create`. This should be the same address used to create "signature"

"input is the message that was signed. This must be the same message used to create the "signature" otherwise verify will return false.

"signature" is the proof of payment address generated from using sign.
