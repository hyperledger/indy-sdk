import * as ffi from 'ffi'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { ISerializedData, StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './vcx-base-with-state'
import { PaymentManager } from './vcx-payment-txn'

/**
 *    The object of the VCX API representing an Issuer side in the credential issuance process.
 *    Assumes that pairwise connection between Issuer and Holder is already established.
 *
 *    # State
 *
 *    The set of object states and transitions depends on communication method is used.
 *    The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.
 *
 *    proprietary:
 *        VcxStateType::VcxStateInitialized - once `vcx_issuer_create_credential` (create IssuerCredential object) is called.
 *
 *        VcxStateType::VcxStateOfferSent - once `vcx_issuer_send_credential_offer` (send `CRED_OFFER` message) is called.
 *
 *        VcxStateType::VcxStateRequestReceived - once `CRED_REQ` messages is received.
 *                                                use `vcx_issuer_credential_update_state` or `vcx_issuer_credential_update_state_with_message` functions for state updates.
 *        VcxStateType::VcxStateAccepted - once `vcx_issuer_send_credential` (send `CRED` message) is called.
 *
 *    aries:
 *        VcxStateType::VcxStateInitialized - once `vcx_issuer_create_credential` (create IssuerCredential object) is called.
 *
 *        VcxStateType::VcxStateOfferSent - once `vcx_issuer_send_credential_offer` (send `CredentialOffer` message) is called.
 *
 *        VcxStateType::VcxStateRequestReceived - once `CredentialRequest` messages is received.
 *        VcxStateType::None - once `ProblemReport` messages is received.
 *                                                use `vcx_issuer_credential_update_state` or `vcx_issuer_credential_update_state_with_message` functions for state updates.
 *
 *        VcxStateType::VcxStateAccepted - once `vcx_issuer_send_credential` (send `Credential` message) is called.
 *
 *    # Transitions
 *
 *    proprietary:
 *        VcxStateType::None - `vcx_issuer_create_credential` - VcxStateType::VcxStateInitialized
 *
 *        VcxStateType::VcxStateInitialized - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent
 *
 *        VcxStateType::VcxStateOfferSent - received `CRED_REQ` - VcxStateType::VcxStateRequestReceived
 *
 *        VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential` - VcxStateType::VcxStateAccepted
 *
 *    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
 *        VcxStateType::None - `vcx_issuer_create_credential` - VcxStateType::VcxStateInitialized
 *
 *        VcxStateType::VcxStateInitialized - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent
 *
 *        VcxStateType::VcxStateOfferSent - received `CredentialRequest` - VcxStateType::VcxStateRequestReceived
 *        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None
 *
 *        VcxStateType::VcxStateRequestReceived - vcx_issuer_send_credential` - VcxStateType::VcxStateAccepted
 *
 *        VcxStateType::VcxStateAccepted - received `Ack` - VcxStateType::VcxStateAccepted
 *
 *   # Messages
 *
 *    proprietary:
 *        CredentialOffer (`CRED_OFFER`)
 *        CredentialRequest (`CRED_REQ`)
 *        Credential (`CRED`)
 *
 *    aries:
 *        CredentialProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#propose-credential
 *        CredentialOffer - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#offer-credential
 *        CredentialRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#request-credential
 *        Credential - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#issue-credential
 *        ProblemReport - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0035-report-problem#the-problem-report-message-type
 *        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
 */

/**
 * @description Interface that represents the parameters for `IssuerCredential.create` function.
 * @interface
 */
export interface IIssuerCredentialCreateData {
  // Enterprise's personal identification for the user.
  sourceId: string,
  // Handle of the correspondent credential definition object
  credDefHandle: number,
  // Data attributes offered to person in the credential ('{"state":"UT"}')
  attr: {
    [ index: string ]: string
  },
  // Name of the credential - ex. Drivers Licence
  credentialName: string,
  // price of credential
  price: string,
}

export interface IIssuerCredentialVCXAttributes {
  [ index: string ]: string
}

export interface IIssuerCredentialParams {
  credDefHandle: number,
  credentialName: string,
  attr: IIssuerCredentialVCXAttributes,
  price: string
}

/**
 * Interface that represents the attributes of an Issuer credential object.
 * This interface is expected as the type for deserialize's parameter and serialize's return value
 */
export interface IIssuerCredentialData {
  source_id: string
  handle: number
  schema_seq_no: number
  credential_attributes: string
  credential_name: string
  issuer_did: string
  state: StateType
  msg_uid: string
  cred_def_id: string
  cred_def_handle: number
  price: string
  tails_file?: string
  cred_rev_id?: string
  rev_reg_id?: string
}

// tslint:disable max-classes-per-file
export class IssuerCredentialPaymentManager extends PaymentManager {
  protected _getPaymentTxnFn = rustAPI().vcx_issuer_credential_get_payment_txn
}

/**
 * A Credential created by the issuing party (institution)
 */
export class IssuerCredential extends VCXBaseWithState<IIssuerCredentialData> {
  /**
   * Create a Issuer Credential object that provides a credential for an enterprise's user
    * Assumes a credential definition has been already written to the ledger.
   * ```
   * issuerCredential = await IssuerCredential.create({sourceId: "12",
   * credDefId: "credDefId", attr: {key: "value"}, credentialName: "name", price: "0"})
   * ```
   * @returns {Promise<IssuerCredential>} An Issuer credential Object
   */
  public static async create ({ attr, sourceId, credDefHandle,
                         credentialName, price }: IIssuerCredentialCreateData): Promise<IssuerCredential> {
    try {
      const attrsVCX: IIssuerCredentialVCXAttributes = attr
      const credential = new IssuerCredential(sourceId, { credDefHandle, credentialName, attr: attrsVCX, price })
      const attrsStringified = attrsVCX ? JSON.stringify(attrsVCX) : attrsVCX
      const commandHandle = 0
      const issuerDid = null
      await credential._create((cb) => rustAPI().vcx_issuer_create_credential(
        commandHandle,
        sourceId,
        credDefHandle,
        issuerDid,
        attrsStringified,
        credentialName,
        price,
        cb
        )
      )
      return credential
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  static getParams (credentialData: ISerializedData<IIssuerCredentialData>): IIssuerCredentialParams {
    const { data: { credential_name, price, credential_attributes, cred_def_handle }} = credentialData
    const attr: IIssuerCredentialVCXAttributes = JSON.parse(credential_attributes)
    return {
      attr,
      credDefHandle: cred_def_handle,
      credentialName: credential_name,
      price
    }
  }

  /**
   * Builds an Issuer credential object with defined attributes.
   *
   * Attributes are provided by a previous call to the serialize function.
   * ```
   * issuerCredential = await IssuerCredential.create({sourceId: "12",
   * credDefId: "credDefId", attr: {key: "value"}, credentialName: "name", price: 0})
   * data1 = await issuerCredential.serialize()
   * issuerCredential2 = await IssuerCredential.deserialize(data1)
   * ```
   */
  public static async deserialize (credentialData: ISerializedData<IIssuerCredentialData>): Promise<IssuerCredential> {
    try {
      const params: IIssuerCredentialParams = (function () {
        switch (credentialData.version) {
          case "1.0":
            return IssuerCredential.getParams(credentialData)
          case "2.0":
            return { attr: {}, credDefHandle: -1, credentialName: "", price: "0" }
          case "3.0":
            return IssuerCredential.getParams(credentialData)
          default:
            throw Error(`Unsupported version provided in serialized credential data: ${JSON.stringify(credentialData.version)}`)
        }
      })()
     return await super._deserialize<IssuerCredential, IIssuerCredentialParams>(
        IssuerCredential,
        credentialData,
        params
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  public paymentManager!: IssuerCredentialPaymentManager
  protected _releaseFn = rustAPI().vcx_issuer_credential_release
  protected _updateStFn = rustAPI().vcx_issuer_credential_update_state
  protected _updateStWithMessageFn = rustAPI(). vcx_issuer_credential_update_state_with_message
  protected _getStFn = rustAPI().vcx_issuer_credential_get_state
  protected _serializeFn = rustAPI().vcx_issuer_credential_serialize
  protected _deserializeFn = rustAPI().vcx_issuer_credential_deserialize
  private _credDefHandle: number
  private _credentialName: string
  private _attr: IIssuerCredentialVCXAttributes
  private _price: string

  constructor (sourceId: string, { credDefHandle, credentialName, attr, price }: IIssuerCredentialParams) {
    super(sourceId)
    this._credDefHandle = credDefHandle
    this._credentialName = credentialName
    this._attr = attr
    this._price = price
  }

  /**
   * Sends a credential Offer to the end user.
   *
   * A Credential Offer is made up of the data provided in the creation of this object
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * issuerCredential = await IssuerCredential.create({sourceId: "12",
   *   credDefId: "credDefId", attr: {k    ey: "value"}, credentialName: "name", price: 0})
   * await issuerCredential.sendOffer(connection)
   * ```
   */
  public async sendOffer (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_issuer_send_credential_offer(0, this.handle, connection.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32'],
            (xcommandHandle: number, err: number) => {
              if (err) {
                reject(err)
                return
              }
              resolve()
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Gets the credential offer message for sending to connection.
   *
   * ```
   * connection = await connectionCreateConnect()
   * issuerCredential = await issuerCredentialCreate()
   * await issuerCredential.getCredentialOfferMsg()
   * ```
   *
   */
  public async getCredentialOfferMsg (): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_issuer_get_credential_offer_msg(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, message: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!message) {
                reject(`Credential ${this.sourceId} returned empty string`)
                return
              }
              resolve(message)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Sends the credential to the end user.
   *
   * Credential is made up of the data sent during Credential Offer
   * ```
   * connection = await connectionCreateConnect()
   * issuerCredential = await issuerCredentialCreate()
   * await issuerCredential.sendOffer(connection)
   * await issuerCredential.updateState()
   * assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
   * await issuerCredential.sendCredential(connection)
   * ```
   *
   */
  public async sendCredential (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_issuer_send_credential(0, this.handle, connection.handle, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32'],
          (xcommandHandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Gets the credential message for sending to connection.
   *
   * Credential is made up of the data sent during Credential Offer
   * ```
   * connection = await connectionCreateConnect()
   * issuerCredential = await issuerCredentialCreate()
   * await issuerCredential.sendOffer(connection)
   * await issuerCredential.updateState()
   * assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
   * await issuerCredential.getCredentialMsg()
   * ```
   *
   */
  public async getCredentialMsg (myPwDid: string): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_issuer_get_credential_msg(0, this.handle, myPwDid, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, message: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!message) {
                reject(`Credential ${this.sourceId} returned empty string`)
                return
              }
              resolve(message)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Revokes credential.
   *
   * Credential is made up of the data sent during Credential Offer
   * ```
   * connection = await connectionCreateConnect()
   * issuerCredential = await issuerCredentialCreate()
   * await issuerCredential.sendOffer(connection)
   * await issuerCredential.updateState()
   * assert.equal(await issuerCredential.getState(), StateType.RequestReceived)
   * await issuerCredential.sendCredential(connection)
   * await issuerCredential.revokeCredential()
   * ```
   *
   */
  public async revokeCredential (): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_issuer_revoke_credential(0, this.handle, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32'],
          (xcommandHandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve()
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  get credDefHandle () {
    return this._credDefHandle
  }

  get attr () {
    return this._attr
  }

  get credentialName () {
    return this._credentialName
  }

  get price () {
    return this._price
  }

  protected _setHandle (handle: number) {
    super._setHandle(handle)
    this.paymentManager = new IssuerCredentialPaymentManager({ handle })
  }
}
