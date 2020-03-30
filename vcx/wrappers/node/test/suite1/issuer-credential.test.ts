import '../module-resolver-helper'

import { assert } from 'chai'
import { validatePaymentTxn } from 'helpers/asserts'
import {
  connectionCreateConnect,
  dataIssuerCredentialCreate,
  issuerCredentialCreate
} from 'helpers/entities'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import {
  Connection,
  Credential,
  IssuerCredential,
  IssuerCredentialPaymentManager,
  StateType,
  VCXCode,
  VCXMock,
  VCXMockMessage
} from 'src'

describe('IssuerCredential:', () => {
  before(() => initVcxTestMode())

  describe('create:', () => {
    it('success', async () => {
      await issuerCredentialCreate()
    })

    it('throws: missing sourceId', async () => {
      const { sourceId, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: invalid credDefHandle', async () => {
      const { credDefHandle, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_DEF_HANDLE)
    })

    it('throws: missing credDefId', async () => {
      const { credDefHandle, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_DEF_HANDLE)
    })

    it('throws: missing attr', async () => {
      const { attr, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing credentialName', async () => {
      const { credentialName, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    // TODO: Enable once https://evernym.atlassian.net/browse/EN-665 is resolved
    it('throws: missing price', async () => {
      const { price, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: invalid attr', async () => {
      const { attr, ...data } = await dataIssuerCredentialCreate()
      const error = await shouldThrow(() => IssuerCredential.create({ attr: null as any, ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })
  })

  describe('serialize:', () => {
    it('success', async () => {
      const issuerCredential = await issuerCredentialCreate()
      const serialized = await issuerCredential.serialize()
      assert.ok(serialized)
      assert.property(serialized, 'version')
      assert.property(serialized, 'data')
      const { data, version } = serialized
      assert.ok(data)
      assert.ok(version)
      assert.equal(data.source_id, issuerCredential.sourceId)
    })

    it('throws: not initialized', async () => {
      const issuerCredential = new IssuerCredential(null as any, {} as any)
      const error = await shouldThrow(() => issuerCredential.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_ISSUER_CREDENTIAL_HANDLE)
    })

  })

  describe('deserialize:', () => {
    it('success', async () => {
      const issuerCredential1 = await issuerCredentialCreate()
      const data1 = await issuerCredential1.serialize()
      const issuerCredential2 = await IssuerCredential.deserialize(data1)
      assert.equal(issuerCredential2.sourceId, issuerCredential1.sourceId)
      const data2 = await issuerCredential2.serialize()
      assert.deepEqual(data1, data2)
    })

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () => IssuerCredential.deserialize({ source_id: 'Invalid' } as any))
      assert.equal(error.vcxCode, VCXCode.UNKNOWN_ERROR)
    })

    it('throws: incomplete data', async () => {
      const error = await shouldThrow(async () => IssuerCredential.deserialize({
        version: '1.0',
        data: {
          cred_def_id: 'Invalid',
          credential_attributes: '{}',
          credential_name: 'Invalid',
          price: 'Invalid',
          source_id: 'Invalid'
        } } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('updateState:', () => {
    it(`returns ${StateType.None}: not initialized`, async () => {
      const issuerCredential = new IssuerCredential(null as any, {} as any)
      await issuerCredential.updateState()
      assert.equal(await issuerCredential.getState(), StateType.None)
    })

    it(`returns ${StateType.Initialized}: created`, async () => {
      const issuerCredential = await issuerCredentialCreate()
      await issuerCredential.updateState()
      assert.equal(await issuerCredential.getState(), StateType.Initialized)
    })
  })

  describe('sendOffer:', () => {
    it('success', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      await issuerCredential.sendOffer(connection)
      assert.equal(await issuerCredential.getState(), StateType.OfferSent)
    })

    it('throws: not initialized', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = new IssuerCredential(null as any, {} as any)
      const error = await shouldThrow(() => issuerCredential.sendOffer(connection))
      assert.equal(error.vcxCode, VCXCode.INVALID_ISSUER_CREDENTIAL_HANDLE)
    })

    it('throws: connection not initialized', async () => {
      const connection = new (Connection as any)()
      const issuerCredential = await issuerCredentialCreate()
      const error = await shouldThrow(() => issuerCredential.sendOffer(connection))
      assert.equal(error.vcxCode, VCXCode.INVALID_CONNECTION_HANDLE)
    })

    it('can generate the offer message', async () => {
      await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      const message = await issuerCredential.getCredentialOfferMsg()
      assert(message.length > 0)
    })
  })

  describe('sendCredential:', () => {
    it('success', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      await issuerCredential.sendOffer(connection)
      VCXMock.setVcxMock(VCXMockMessage.IssuerCredentialReq)
      VCXMock.setVcxMock(VCXMockMessage.UpdateIssuerCredential)
      await issuerCredential.updateState()
      assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
      await issuerCredential.sendCredential(connection)
      assert.equal(await issuerCredential.getState(), StateType.Accepted)
    })

    it('success', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      const offer = await issuerCredential.getCredentialOfferMsg()
      const cred = await Credential.create({ sourceId: 'name', offer, connection })
      const pwDid = await connection.getPwDid()
      const request = await cred.getRequestMessage({ myPwDid: pwDid, payment: 0 })
      await issuerCredential.updateStateWithMessage(request)
      assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
      const credMsg = await issuerCredential.getCredentialMsg(pwDid)
      await cred.updateStateWithMessage(credMsg)
      assert.equal(await cred.getState(), StateType.Accepted)
    })

    it('throws: not initialized', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = new IssuerCredential(null as any, {} as any)
      const error = await shouldThrow(() => issuerCredential.sendCredential(connection))
      assert.equal(error.vcxCode, VCXCode.INVALID_ISSUER_CREDENTIAL_HANDLE)
    })

    it('throws: no offer', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      const error = await shouldThrow(() => issuerCredential.sendCredential(connection))
      assert.equal(error.vcxCode, VCXCode.NOT_READY)
    })

    it('throws: no request', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      await issuerCredential.sendOffer(connection)
      const error = await shouldThrow(() => issuerCredential.sendCredential(connection))
      assert.equal(error.vcxCode, VCXCode.NOT_READY)
    })

    it('can generate the credential message', async () => {
      const connection = await connectionCreateConnect()
      const issuerCredential = await issuerCredentialCreate()
      await issuerCredential.sendOffer(connection)
      VCXMock.setVcxMock(VCXMockMessage.IssuerCredentialReq)
      VCXMock.setVcxMock(VCXMockMessage.UpdateIssuerCredential)
      await issuerCredential.updateState()
      assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
      const message = await issuerCredential.getCredentialMsg('44x8p4HubxzUK1dwxcc5FU')
      assert(message.length > 0)
    })
  })

  describe('paymentManager:', () => {
    it('exists', async () => {
      const issuerCredential = await issuerCredentialCreate()
      assert.instanceOf(issuerCredential.paymentManager, IssuerCredentialPaymentManager)
      assert.equal(issuerCredential.paymentManager.handle, issuerCredential.handle)
    })

    describe('getPaymentTxn:', () => {
      it('success', async () => {
        const connection = await connectionCreateConnect()
        const issuerCredential = await issuerCredentialCreate()
        await issuerCredential.sendOffer(connection)
        VCXMock.setVcxMock(VCXMockMessage.IssuerCredentialReq)
        VCXMock.setVcxMock(VCXMockMessage.UpdateIssuerCredential)
        await issuerCredential.updateState()
        assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
        await issuerCredential.sendCredential(connection)
        assert.equal(await issuerCredential.getState(), StateType.Accepted)
        const paymentTxn = await issuerCredential.paymentManager.getPaymentTxn()
        validatePaymentTxn(paymentTxn)
      })
    })
  })

  describe('revoke:', () => {
    it('throws: invalid revocation details', async () => {
      const issuerCredential = await issuerCredentialCreate()
      const error = await shouldThrow(() => issuerCredential.revokeCredential())
      assert.equal(error.vcxCode, VCXCode.INVALID_REVOCATION_DETAILS)
    })

    it('success', async () => {
      const issuerCredential1 = await issuerCredentialCreate()
      const data = await issuerCredential1.serialize()
      data.data.cred_rev_id = '123'
      data.data.rev_reg_id = '456'
      data.data.tails_file = 'file'
      const issuerCredential2 = await IssuerCredential.deserialize(data)
      await issuerCredential2.revokeCredential()
    })
  })

})
