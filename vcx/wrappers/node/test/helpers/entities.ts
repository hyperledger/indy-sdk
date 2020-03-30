import '../module-resolver-helper'

import { assert } from 'chai'
import {
  Connection,
  Credential,
  CredentialDef,
  DisclosedProof,
  IConnectionCreateData,
  IConnectOptions,
  ICredentialCreateWithMsgId,
  ICredentialCreateWithOffer,
  ICredentialDefCreateData,
  ICredentialDefPrepareForEndorserData,
  IDisclosedProofCreateData,
  IDisclosedProofCreateWithMsgIdData,
  IIssuerCredentialCreateData,
  IProofCreateData,
  ISchemaCreateData,
  ISchemaLookupData,
  ISchemaPrepareForEndorserData,
  IssuerCredential,
  Proof,
  Schema
} from 'src'

export const dataConnectionCreate = (): IConnectionCreateData => ({
  id: 'testConnectionId'
})

export const connectionCreate = async (data = dataConnectionCreate()) => {
  const connection = await Connection.create(data)
  assert.notEqual(connection.handle, undefined)
  assert.equal(connection.sourceId, data.id)
  return connection
}

export const dataConnectionConnectOptions = (): IConnectOptions => ({
  data: '{"connection_type":"SMS","phone":"123","use_public_did":true}'
})

export const connectionCreateConnect = async (data = dataConnectionCreate()) => {
  const connection = await connectionCreate(data)
  const connectionData = dataConnectionConnectOptions()
  const inviteDetails = await connection.connect(connectionData)
  assert.ok(inviteDetails)
  return connection
}

export const dataCredentialDefCreate = (): ICredentialDefCreateData => ({
  name: 'testCredentialDefName',
  paymentHandle: 0,
  revocationDetails: {
    maxCreds: undefined,
    supportRevocation: false,
    tailsFile: undefined
  },
  schemaId: 'testCredentialDefSchemaId',
  sourceId: 'testCredentialDefSourceId'
})

export const dataCredentialDefPrepareForEndorser = ():  ICredentialDefPrepareForEndorserData => ({
  endorser: 'V4SGRU86Z58d6TV7PBUe6f',
  name: 'testCredentialDefName',
  revocationDetails: {
    maxCreds: undefined,
    supportRevocation: false,
    tailsFile: undefined
  },
  schemaId: 'testCredentialDefSchemaId',
  sourceId: 'testCredentialDefSourceId'
})

export const credentialDefCreate = async (data = dataCredentialDefCreate()) => {
  const credentialDef = await CredentialDef.create(data)
  assert.notEqual(credentialDef.handle, undefined)
  assert.equal(credentialDef.sourceId, data.sourceId)
  assert.equal(credentialDef.schemaId, data.schemaId)
  assert.equal(credentialDef.name, data.name)
  return credentialDef
}

export const credentialDefPrepareForEndorser = async (data = dataCredentialDefPrepareForEndorser()) => {
  const credentialDef = await CredentialDef.prepareForEndorser(data)
  assert.notEqual(credentialDef.handle, undefined)
  assert.equal(credentialDef.sourceId, data.sourceId)
  assert.equal(credentialDef.schemaId, data.schemaId)
  assert.equal(credentialDef.name, data.name)
  assert.ok(credentialDef.credentialDefTransaction)
  return credentialDef
}

export const credentialOffer = [
  {
    claim_id: 'defaultCredentialId',
    claim_name: 'Credential',
    cred_def_id: 'id',
    credential_attrs: {
      address1: ['101 Tela Lane'],
      address2: ['101 Wilson Lane'],
      city: ['SLC'],
      state: ['UT'],
      zip: ['87121']
    },
    from_did: '8XFh8yBzrpJQmNyZzgoTqB',
    libindy_offer: '{}',
    msg_ref_id: '123',
    msg_type: 'CLAIM_OFFER',
    schema_seq_no: 1487,
    to_did: '8XFh8yBzrpJQmNyZzgoTqB',
    version: '0.1'
  },
  {
    payment_addr: 'pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j',
    payment_required: 'one-time',
    price: 5
  }
]

export const dataCredentialCreateWithOffer = async (): Promise<ICredentialCreateWithOffer> => {
  const connection = await connectionCreateConnect()
  return {
    connection,
    offer: JSON.stringify(credentialOffer),
    sourceId: 'testCredentialSourceId'
  }
}

export const credentialCreateWithOffer = async (data?: ICredentialCreateWithOffer) => {
  if (!data) {
    data = await dataCredentialCreateWithOffer()
  }
  const credential = await Credential.create(data)
  assert.notEqual(credential.handle, undefined)
  assert.equal(credential.sourceId, data.sourceId)
  return credential
}

export const dataCredentialCreateWithMsgId = async (): Promise<ICredentialCreateWithMsgId> => {
  const connection = await connectionCreateConnect()
  return {
    connection,
    msgId: 'testCredentialMsgId',
    sourceId: 'testCredentialSourceId'
  }
}

export const credentialCreateWithMsgId = async (data?: ICredentialCreateWithMsgId) => {
  if (!data) {
    data = await dataCredentialCreateWithMsgId()
  }
  const credential = await Credential.createWithMsgId(data)
  assert.notEqual(credential.handle, undefined)
  assert.equal(credential.sourceId, data.sourceId)
  assert.ok(credential.credOffer)
  return credential
}

const disclosedProofRequest = {
  '@topic': {
    mid: 9,
    tid: 1
  },
  '@type': {
    name: 'PROOF_REQUEST',
    version: '1.0'
  },
  'msg_ref_id': 'abcd',
  'proof_request_data': {
    name: 'Account Certificate',
    nonce: '838186471541979035208225',
    requested_attributes: {
      business_2: {
        name: 'business'
      },
      email_1: {
        name: 'email'
      },
      name_0: {
        name: 'name'
      }
    },
    requested_predicates: {},
    version: '0.1'
  }
}

export const dataDisclosedProofCreateWithRequest = async (): Promise<IDisclosedProofCreateData> => {
  const connection = await connectionCreateConnect()
  return {
    connection,
    request: JSON.stringify(disclosedProofRequest),
    sourceId: 'testDisclousedProofSourceId'
  }
}

export const disclosedProofCreateWithRequest = async (data?: IDisclosedProofCreateData) => {
  if (!data) {
    data = await dataDisclosedProofCreateWithRequest()
  }
  const disclousedProof = await DisclosedProof.create(data)
  assert.notEqual(disclousedProof.handle, undefined)
  assert.equal(disclousedProof.sourceId, data.sourceId)
  return disclousedProof
}

export const dataDisclosedProofCreateWithMsgId = async (): Promise<IDisclosedProofCreateWithMsgIdData> => {
  const connection = await connectionCreateConnect()
  return {
    connection,
    msgId: 'testDisclousedProofMsgId',
    sourceId: 'testDisclousedProofSourceId'
  }
}

export const disclosedProofCreateWithMsgId = async (data?: IDisclosedProofCreateWithMsgIdData) => {
  if (!data) {
    data = await dataDisclosedProofCreateWithMsgId()
  }
  const disclousedProof = await DisclosedProof.createWithMsgId(data)
  assert.notEqual(disclousedProof.handle, undefined)
  assert.equal(disclousedProof.sourceId, data.sourceId)
  assert.ok(disclousedProof.proofRequest)
  return disclousedProof
}

export const dataIssuerCredentialCreate = async (): Promise<IIssuerCredentialCreateData> => {
  const credDef = await credentialDefCreate()
  return {
    attr: {
      key1: 'value1',
      key2: 'value2',
      key3: 'value3'
    },
    credDefHandle: Number(credDef.handle),
    credentialName: 'Credential Name',
    price: '1',
    sourceId: 'testCredentialSourceId'
  }
}

// export const dataIssuerCredentialCreate = (): IIssuerCredentialCreateData => ({
//   attr: {
//     key1: 'value1',
//     key2: 'value2',
//     key3: 'value3'
//   },
//   credDefHandle: 1,
//   credentialName: 'Credential Name',
//   price: '1',
//   sourceId: 'testCredentialSourceId'
// })

export const issuerCredentialCreate = async (_data = dataIssuerCredentialCreate()) => {
  const data = await _data
  const issuerCredential = await IssuerCredential.create(data)
  assert.notEqual(issuerCredential.handle, undefined)
  assert.equal(issuerCredential.sourceId, data.sourceId)
  assert.equal(issuerCredential.credDefHandle, data.credDefHandle)
  assert.equal(issuerCredential.credentialName, data.credentialName)
  assert.equal(issuerCredential.price, data.price)
  return issuerCredential
}

export const dataProofCreate = (): IProofCreateData => ({
  attrs: [
    { name: 'attr1' },
    { name: 'attr2' },
    { names: ['attr3', 'attr4'] }
  ],
  name: 'Proof',
  revocationInterval: {
    from: undefined,
    to: undefined
  },
  sourceId: 'testProofSourceId'
})

export const proofCreate = async (data = dataProofCreate()) => {
  const proof = await Proof.create(data)
  assert.notEqual(proof.handle, undefined)
  assert.equal(proof.sourceId, data.sourceId)
  assert.equal(proof.name, data.name)
  assert.equal(proof.proofState, null)
  assert.deepEqual(proof.requestedAttributes, data.attrs)
  return proof
}

export const dataSchemaCreate = (): ISchemaCreateData => ({
  data: {
    attrNames: [
      'attr1',
      'attr2'
    ],
    name: 'Schema',
    version: '1.0.0'
  },
  paymentHandle: 0,
  sourceId: 'testSchemaSourceId'
})

export const dataSchemaPrepareForEndorser = (): ISchemaPrepareForEndorserData => ({
  data: {
    attrNames: [
      'attr1',
      'attr2'
    ],
    name: 'Schema',
    version: '1.0.0'
  },
  endorser: 'V4SGRU86Z58d6TV7PBUe6f',
  sourceId: 'testSchemaSourceId'
})

export const schemaCreate = async (data = dataSchemaCreate()) => {
  const schema = await Schema.create(data)
  assert.notEqual(schema.handle, undefined)
  assert.equal(schema.sourceId, data.sourceId)
  assert.equal(schema.name, data.data.name)
  assert.deepEqual(schema.schemaAttrs, data.data)
  assert.ok(schema.schemaId)
  return schema
}

export const schemaPrepareForEndorser = async (data = dataSchemaPrepareForEndorser()) => {
  const schema = await Schema.prepareForEndorser(data)
  assert.notEqual(schema.handle, undefined)
  assert.equal(schema.sourceId, data.sourceId)
  assert.equal(schema.name, data.data.name)
  assert.deepEqual(schema.schemaAttrs, data.data)
  assert.ok(schema.schemaId)
  assert.ok(schema.schemaTransaction)
  return schema
}

export const dataSchemaLookup = (): ISchemaLookupData => ({
  schemaId: 'testSchemaSchemaId',
  sourceId: 'testSchemaSourceId'
})

export const schemaLookup = async (data = dataSchemaLookup()) => {
  const schema = await Schema.lookup(data)
  assert.notEqual(schema.handle, undefined)
  assert.equal(schema.sourceId, data.sourceId)
  assert.ok(schema.schemaId)
  return schema
}
