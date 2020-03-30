import * as ffi from 'ffi'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { ISerializedData } from './common'
import { VCXBase } from './vcx-base'
import { PaymentManager } from './vcx-payment-txn'

/**
 * @interface Interface that represents the parameters for `CredentialDef.create` function.
 * @description
 * SourceId: Enterprise's personal identification for the user.
 * name: Name of credential definition
 * schemaId: The schema id given during the creation of the schema
 * revocation: type-specific configuration of credential definition revocation
 *     TODO: Currently supports ISSUANCE BY DEFAULT, support for ISSUANCE ON DEMAND will be added as part of ticket: IS-1074
 *     support_revocation: true|false - Optional, by default its false
 *     tails_file: path to tails file - Optional if support_revocation is false
 *     max_creds: size of tails file - Optional if support_revocation is false
 */
export interface ICredentialDefCreateData {
  sourceId: string,
  name: string,
  schemaId: string,
  revocationDetails: IRevocationDetails,
  paymentHandle: number
}

/**
 * @interface Interface that represents the parameters for `CredentialDef.prepareForEndorser` function.
 * @description
 * SourceId: Enterprise's personal identification for the user.
 * name: Name of credential definition
 * schemaId: The schema id given during the creation of the schema
 * revocation: type-specific configuration of credential definition revocation
 *     TODO: Currently supports ISSUANCE BY DEFAULT, support for ISSUANCE ON DEMAND will be added as part of ticket: IS-1074
 *     support_revocation: true|false - Optional, by default its false
 *     tails_file: path to tails file - Optional if support_revocation is false
 *     max_creds: size of tails file - Optional if support_revocation is false
 * endorser: DID of the Endorser that will submit the transaction.
 */
export interface ICredentialDefPrepareForEndorserData {
  sourceId: string,
  name: string,
  schemaId: string,
  revocationDetails: IRevocationDetails,
  endorser: string
}

export interface ICredentialDefData {
  source_id: string,
  handle: number
  name: string
  credential_def: ICredentialDefDataObj
}

export interface ICredentialDefDataObj {
  ref: number,
  origin: string,
  signature_type: string,
  data: any,
}

export interface ICredentialDefParams {
  schemaId?: string,
  name?: string,
  credDefId?: string,
  tailsFile?: string
}

export interface IRevocationDetails {
  maxCreds?: number,
  supportRevocation?: boolean,
  tailsFile?: string,
}

export enum CredentialDefState {
  Built = 0,
  Published = 1,
}

// tslint:disable max-classes-per-file
export class CredentialDefPaymentManager extends PaymentManager {
  protected _getPaymentTxnFn = rustAPI().vcx_credentialdef_get_payment_txn
}

/**
 * @class Class representing a credential Definition
 */
export class CredentialDef extends VCXBase<ICredentialDefData> {
  /**
   * Creates a new CredentialDef object that is written to the ledger
   *
   * Example:
   * ```
   * data = {
   *   name: 'testCredentialDefName',
   *   paymentHandle: 0,
   *   revocation: false,
   *   schemaId: 'testCredentialDefSchemaId',
   *   sourceId: 'testCredentialDefSourceId'
   * }
   * credentialDef = await CredentialDef.create(data)
   * ```
   */
  public static async create ({
    name,
    paymentHandle,
    revocationDetails,
    schemaId,
    sourceId
  }: ICredentialDefCreateData): Promise<CredentialDef> {
    // Todo: need to add params for tag and config
    const tailsFile = revocationDetails.tailsFile
    const credentialDef = new CredentialDef(sourceId, { name, schemaId, tailsFile })
    const commandHandle = 0
    const issuerDid = null
    const revocation = {
      max_creds: revocationDetails.maxCreds,
      support_revocation: revocationDetails.supportRevocation,
      tails_file: revocationDetails.tailsFile
    }

    try {
      await credentialDef._create((cb) => rustAPI().vcx_credentialdef_create(
        commandHandle,
        sourceId,
        name,
        schemaId,
        issuerDid,
        'tag1',
        JSON.stringify(revocation),
        paymentHandle,
      cb
      ))
      return credentialDef
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Create a new CredentialDef object that will be published by Endorser later.
   *
   * Note that CredentialDef can't be used for credential issuing until it will be published on the ledger.
   *
   * Example:
   * ```
   * data = {
   *   name: 'testCredentialDefName',
   *   endorser: 'V4SGRU86Z58d6TV7PBUe6f',
   *   revocation: false,
   *   schemaId: 'testCredentialDefSchemaId',
   *   sourceId: 'testCredentialDefSourceId'
   * }
   * credentialDef = await CredentialDef.prepareForEndorser(data)
   * ```
   */
  public static async prepareForEndorser ({
    name,
    endorser,
    revocationDetails,
    schemaId,
    sourceId
  }: ICredentialDefPrepareForEndorserData): Promise<CredentialDef> {
    // Todo: need to add params for tag and config
    try {
      const tailsFile = revocationDetails.tailsFile
      const credentialDef = new CredentialDef(sourceId, { name, schemaId, tailsFile })
      const issuerDid = null
      const revocation = {
        max_creds: revocationDetails.maxCreds,
        support_revocation: revocationDetails.supportRevocation,
        tails_file: revocationDetails.tailsFile
      }

      const credDefForEndorser = await
      createFFICallbackPromise<{ credDefTxn: string, revocRegDefTxn: string, revocRegEntryTxn: string, handle: number }>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_credentialdef_prepare_for_endorser(0,
                                                                        sourceId,
                                                                        name,
                                                                        schemaId,
                                                                        issuerDid,
                                                                        'tag1',
                                                                        JSON.stringify(revocation),
                                                                        endorser,
                                                                      cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32', 'string', 'string', 'string'],
            (handle: number, err: number, _handle: number, _credDefTxn: string, _revocRegDefTxn: string,
             _revocRegEntryTxn: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!_credDefTxn) {
                reject('no credential definition transaction')
                return
              }
              resolve({ credDefTxn: _credDefTxn, handle: _handle, revocRegDefTxn: _revocRegDefTxn,
                revocRegEntryTxn: _revocRegEntryTxn })
            })
      )
      credentialDef._setHandle(credDefForEndorser.handle)
      credentialDef._credDefTransaction = credDefForEndorser.credDefTxn
      credentialDef._revocRegDefTransaction = credDefForEndorser.revocRegDefTxn
      credentialDef._revocRegEntryTransaction = credDefForEndorser.revocRegEntryTxn
      return credentialDef
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Builds a credentialDef object with defined attributes.
   * Attributes are provided by a previous call to the serialize function.
   * Example:
   * ```
   * data = {
   *   name: 'testCredentialDefName',
   *   paymentHandle: 0,
   *   revocation: false,
   *   schemaId: 'testCredentialDefSchemaId',
   *   sourceId: 'testCredentialDefSourceId'
   * }
   * credentialDef = await CredentialDef.create(data)
   * data1 = await credentialDef.serialize()
   * credentialDef2 = await CredentialDef.deserialzie(data1)
   * ```
   */
  public static async deserialize (credentialDef: ISerializedData<ICredentialDefData>) {
    // Todo: update the ICredentialDefObj
    const { data: { name } } = credentialDef
    const credentialDefParams = {
      name,
      schemaId: null
    }
    return super._deserialize(CredentialDef, credentialDef, credentialDefParams)
  }

  public paymentManager!: CredentialDefPaymentManager
  protected _releaseFn = rustAPI().vcx_credentialdef_release
  protected _serializeFn = rustAPI().vcx_credentialdef_serialize
  protected _deserializeFn = rustAPI().vcx_credentialdef_deserialize
  private _name: string | undefined
  private _schemaId: string | undefined
  private _credDefId: string | undefined
  private _tailsFile: string | undefined
  private _credDefTransaction: string | null
  private _revocRegDefTransaction: string | null
  private _revocRegEntryTransaction: string | null

  constructor (sourceId: string, { name, schemaId, credDefId, tailsFile }: ICredentialDefParams) {
    super(sourceId)
    this._name = name
    this._schemaId = schemaId
    this._credDefId = credDefId
    this._tailsFile = tailsFile
    this._credDefTransaction = null
    this._revocRegDefTransaction = null
    this._revocRegEntryTransaction = null
  }

  /**
   * Retrieves the credential definition id associated with the created cred def.
   * Example:
   * ```
   * data = {
   *   name: 'testCredentialDefName',
   *   paymentHandle: 0,
   *   revocation: false,
   *   schemaId: 'testCredentialDefSchemaId',
   *   sourceId: 'testCredentialDefSourceId'
   * }
   * credentialDef = await CredentialDef.create(data)
   * id = await credentialDef.getCredDefId()
   * ```
   */
  public async getCredDefId (): Promise<string> {
    try {
      const credDefId = await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_credentialdef_get_cred_def_id(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, credDefIdVal: string) => {
              if (err) {
                reject(err)
                return
              }
              this._credDefId = credDefIdVal
              resolve(credDefIdVal)
            })
        )
      return credDefId
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   *
   * Checks if credential definition is published on the Ledger and updates the state
   *
   * Example:
   * ```
   * await credentialDef.updateState()
   * ```
   * @returns {Promise<void>}
   */
  public async updateState (): Promise<void> {
    try {
      await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
            const rc = rustAPI().vcx_credentialdef_update_state(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (handle: number, err: any, state: CredentialDefState) => {
            if (err) {
              reject(err)
            }
            resolve(state)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Get the current state of the credential definition object
   *
   * Example:
   * ```
   * state = await credentialdef.getState()
   * ```
   * @returns {Promise<CredentialDefState>}
   */
  public async getState (): Promise<CredentialDefState> {
    try {
      const stateRes = await createFFICallbackPromise<CredentialDefState>(
        (resolve, reject, cb) => {
            const rc = rustAPI().vcx_credentialdef_get_state(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (handle: number, err: number, state: CredentialDefState) => {
            if (err) {
              reject(err)
            }
            resolve(state)
          })
      )
      return stateRes
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  get name () {
    return this._name
  }

  get schemaId () {
    return this._schemaId
  }

  get credDefId () {
    return this._credDefId
  }

  get tailsFile () {
    return this._tailsFile
  }

  protected _setHandle (handle: number) {
    super._setHandle(handle)
    this.paymentManager = new CredentialDefPaymentManager({ handle })
  }

  get credentialDefTransaction (): string | null {
    return this._credDefTransaction
  }

  get revocRegDefTransaction (): string | null {
    return this._revocRegDefTransaction
  }

  get revocRegEntryTransaction (): string | null {
    return this._revocRegEntryTransaction
  }
}
