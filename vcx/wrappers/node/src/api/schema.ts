
import * as ffi from 'ffi'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { ISerializedData } from './common'
import { VCXBase } from './vcx-base'
import { PaymentManager } from './vcx-payment-txn'

/**
 * @interface Interface that represents the parameters for `Schema.create` function.
 * @description
 */
export interface ISchemaCreateData {
  // Enterprise's personal identification for the user.
  sourceId: string,
  // list of attributes that will make up the schema (the number of attributes should be less or equal than 125)
  data: ISchemaAttrs,
  // future use (currently uses any address in the wallet)
  paymentHandle: number
}

/**
 * @interface Interface that represents the parameters for `Schema.prepareForEndorser` function.
 * @description
 */
export interface ISchemaPrepareForEndorserData {
  // Enterprise's personal identification for the user.
  sourceId: string,
  // list of attributes that will make up the schema (the number of attributes should be less or equal than 125)
  data: ISchemaAttrs,
  // DID of the Endorser that will submit the transaction.
  endorser: string
}

/**
 * @interface
 * @description
 * name: name of schema
 * version: version of the scheme
 * attrNames: a list of named attribtes inteded to be added to the schema
 * (the number of attributes should be less or equal than 125)
 */
export interface ISchemaAttrs {
  name: string,
  version: string,
  attrNames: string[]
}

export interface ISchemaSerializedData {
  source_id: string,
  handle: string,
  name: string,
  version: string,
  data: string[],
  schema_id: string,
}

export interface ISchemaTxn {
  sequence_num?: number,
  sponsor?: string,
  txn_timestamp?: number,
  txn_type?: string,
  data?: {
    name: string,
    version: string,
    attr_names: string[]
  }
}

export interface ISchemaParams {
  schemaId: string,
  name: string,
  schemaAttrs: ISchemaAttrs,
}

export interface ISchemaLookupData {
  sourceId: string,
  schemaId: string
}

export enum SchemaState {
  Built = 0,
  Published = 1,
}

// tslint:disable max-classes-per-file
export class SchemaPaymentManager extends PaymentManager {
  protected _getPaymentTxnFn = rustAPI().vcx_schema_get_payment_txn
}

export class Schema extends VCXBase<ISchemaSerializedData> {
  /**
   * Creates a new Schema object that is written to the ledger
   *
   * Example:
   * ```
   * data: {
   *     attrNames: [
   *       'attr1',
   *       'attr2'
   *     ],
   *     name: 'Schema',
   *     version: '1.0.0'
   *   },
   *   paymentHandle: 0,
   *   sourceId: 'testSchemaSourceId'
   * }
   * schema1 = await Schema.create(data)
   * ```
   */
  public static async create ({ paymentHandle, data, sourceId }: ISchemaCreateData): Promise<Schema> {
    try {
      const schema = new Schema(sourceId, { name: data.name, schemaId: '', schemaAttrs: data })
      const commandHandle = 0
      await schema._create((cb) => rustAPI().vcx_schema_create(
        commandHandle,
        schema.sourceId,
        schema._name,
        data.version,
        JSON.stringify(data.attrNames),
        paymentHandle,
        cb
      ))
      await schema.getSchemaId()
      return schema
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Builds a new Schema object that will be published by Endorser later.
   *
   * Example:
   * ```
   * data: {
   *     attrNames: [
   *       'attr1',
   *       'attr2'
   *     ],
   *     name: 'Schema',
   *     version: '1.0.0'
   *   },
   *   endorser: 'V4SGRU86Z58d6TV7PBUe6f',
   *   sourceId: 'testSchemaSourceId'
   * }
   * schema1 = await Schema.prepareForEndorser(data)
   * ```
   */
  public static async prepareForEndorser ({ endorser, data, sourceId }: ISchemaPrepareForEndorserData):
    Promise<Schema> {
    try {
      const schema = new Schema(sourceId, { name: data.name, schemaId: '', schemaAttrs: data })

      const schemaForEndorser = await
      createFFICallbackPromise<{ transaction: string, handle: number }>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_schema_prepare_for_endorser(0, sourceId, schema._name, data.version,
              JSON.stringify(data.attrNames), endorser, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32', 'string'],
            (handle: number, err: number, _schemaHandle: number, _transaction: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!_transaction) {
                reject('no schema transaction')
                return
              }
              resolve({ transaction: _transaction, handle: _schemaHandle })
            })
      )
      schema._setHandle(schemaForEndorser.handle)
      schema._transaction = schemaForEndorser.transaction
      await schema.getSchemaId()
      return schema
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Builds Schema object with defined attributes.
   * Attributes are provided by a previous call to the serialize function.
   *
   * Example:
   * ```
   * sourceId = 'lookupTest'
   * data: {
   *     attrNames: [
   *       'attr1',
   *       'attr2'
   *     ],
   *     name: 'Schema',
   *     version: '1.0.0'
   *   },
   *   paymentHandle: 0,
   *   sourceId: sourceId
   * }
   * schema1 = await Schema.create(data)
   * data1 = await schema1.serialize()
   * schema2 = Schema.deserialize(data1)
   */
  public static async deserialize (schema: ISerializedData<ISchemaSerializedData>) {
    const { data: { name, schema_id, version, data } } = schema
    const schemaParams = {
      name,
      schemaAttrs: { name, version, attrNames: data },
      schemaId: schema_id
    }
    return super._deserialize<Schema, ISchemaParams>(Schema, schema, schemaParams)
  }

  /**
   * Looks up the attributes of an already created Schema.
   *
   * Example:
   * ```
   * sourceId = 'lookupTest'
   * data: {
   *     attrNames: [
   *       'attr1',
   *       'attr2'
   *     ],
   *     name: 'Schema',
   *     version: '1.0.0'
   *   },
   *   paymentHandle: 0,
   *   sourceId: sourceId
   * }
   * schema1 = await Schema.create(data)
   * schemaId1 = await schema1.getSchemaId()
   * data = await Schema.lookup(sourceId, schemaId1)
   * ```
   */
  public static async lookup ({ sourceId, schemaId }: ISchemaLookupData): Promise<Schema> {
    try {
      const schemaLookupData = await
      createFFICallbackPromise<{ data: string, handle: number }>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_schema_get_attributes(0, sourceId, schemaId, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'uint32', 'string'],
            (handle: number, err: number, _schemaHandle: number, _schemaData: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!_schemaData) {
                reject('no schema attrs')
                return
              }
              resolve({ data: _schemaData, handle: _schemaHandle })
            })
      )
      const { name, version, data }: ISchemaSerializedData = JSON.parse(schemaLookupData.data)
      const schemaParams = {
        name,
        schemaAttrs: {
          attrNames: data,
          name,
          version
        },
        schemaId
      }
      const newSchema = new Schema(sourceId, schemaParams)
      newSchema._setHandle(schemaLookupData.handle)
      return newSchema
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  public paymentManager!: SchemaPaymentManager
  protected _releaseFn = rustAPI().vcx_schema_release
  protected _serializeFn = rustAPI().vcx_schema_serialize
  protected _deserializeFn = rustAPI().vcx_schema_deserialize
  protected _name: string
  protected _schemaId: string
  protected _schemaAttrs: ISchemaAttrs
  private _transaction: string = ''

  constructor (sourceId: string, { name, schemaId, schemaAttrs }: ISchemaParams) {
    super(sourceId)
    this._name = name
    this._schemaId = schemaId
    this._schemaAttrs = schemaAttrs
  }

  get schemaAttrs (): ISchemaAttrs {
    return this._schemaAttrs
  }

  get schemaId () {
    return this._schemaId
  }

  get name () {
    return this._name
  }

  get schemaTransaction (): string {
    return this._transaction
  }

  /**
   * Get the ledger ID of the object
   *
   * Example:
   * ```
   * data: {
   *     attrNames: [
   *       'attr1',
   *       'attr2'
   *     ],
   *     name: 'Schema',
   *     version: '1.0.0'
   *   },
   *   paymentHandle: 0,
   *   sourceId: 'testSchemaSourceId'
   * }
   * schema1 = await Schema.create(data)
   * id1 = await schema1.getSchemaId()
   * ```
   */
  protected async getSchemaId (): Promise<string> {
    try {
      const schemaId = await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_schema_get_schema_id(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, schemaIdVal: string) => {
              if (err) {
                reject(err)
                return
              }
              this._schemaId = schemaIdVal
              resolve(schemaIdVal)
            })
        )
      return schemaId
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   *
   * Checks if schema is published on the Ledger and updates the state
   *
   * Example:
   * ```
   * await schema.updateState()
   * ```
   * @returns {Promise<void>}
   */
  public async updateState (): Promise<void> {
    try {
      await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
            const rc = rustAPI().vcx_schema_update_state(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (handle: number, err: any, state: SchemaState) => {
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
   * Get the current state of the schema object
   *
   * Example:
   * ```
   * state = await schema.getState()
   * ```
   * @returns {Promise<SchemaState>}
   */
  public async getState (): Promise<SchemaState> {
    try {
      const stateRes = await createFFICallbackPromise<SchemaState>(
        (resolve, reject, cb) => {
            const rc = rustAPI().vcx_schema_get_state(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (handle: number, err: number, state: SchemaState) => {
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

  protected _setHandle (handle: number) {
    super._setHandle(handle)
    this.paymentManager = new SchemaPaymentManager({ handle })
  }
}
