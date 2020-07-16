import { Callback } from 'ffi-napi'
import * as ref from 'ref-napi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { IUTXO } from './common'
import { voidPtrToUint8Array } from './connection'

export type PaymentAddress = string
export type PaymentAmount = number
export type PaymentHandle = number
/**
 * @interface An interface representing a record that can be added to the wallet
 */
export interface IRecord {
  type_: string,
  id: string,
  value: any,
  tags: any,
}

export interface IRecordUpdate {
  type_: string,
  id: string,
  value: any
}

export interface ISendTokens {
  payment: PaymentHandle,
  tokens: PaymentAmount,
  recipient: PaymentAddress
}

export interface IDeleteRecordTagsOptions {
  tagList: string[]
}

export interface IDeleteRecordData {
  type: string,
  id: string
}

export interface IGetRecordOptions {
  retrieveType: boolean,
  retrieveValue: boolean,
  retrieveTags: boolean
}

export interface IGerRecordData {
  type: string,
  id: string,
  options: IGetRecordOptions
}

export interface IOpenSearchData {
  type: string,
  queryJson: string,
  options: string
}

export interface ISearchNextRecordsOptions {
  count: number
}

export interface IPaymentAddress {
  address: string,
  balance: number,
  utxo: IUTXO[]
}

export interface IWalletTokenInfo {
  balance: number,
  addresses: IPaymentAddress[]
}

export interface IPaymentAddressSeed {
  seed?: string
}

/**
 * @class Class representing a Wallet
 */
export class Wallet {

  /**
   * Gets wallet token info
   *
   * Example:
   * ```
   * info = await Wallet.getTokenInfo()
   * ```
   */
  public static async getTokenInfo (handle?: PaymentHandle): Promise<IWalletTokenInfo> {
    try {
      const walletInfoStr = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_get_token_info(0, handle, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32','string'],
          (xhandle: number, err: number, info: string) => {
            if (err) {
              reject(err)
              return
            }
            resolve(info)
          })
      )
      const walletInfo = JSON.parse(walletInfoStr)
      return walletInfo
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Creates payment address inside wallet
   *
   * Example:
   * ```
   * address = await Wallet.createPaymentAddress('00000000000000000000000001234567')
   * ```
   */
  public static async createPaymentAddress (seed: IPaymentAddressSeed): Promise<string> {
    const cSeed = seed.seed ? seed.seed : null
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_create_payment_address(0, cSeed, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32','string'],
          (xhandle: number, err: number, info: string) => {
            if (err) {
              reject(err)
              return
            }
            resolve(info)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Validates Payment Address
   *
   * Example:
   * ```
   * address = await Wallet.createPaymentAddress('00000000000000000000000001234567')
   * await Wallet.validatePaymentAddress(address)
   * ```
   */
  public static async validatePaymentAddress (paymentAddress: string): Promise<void> {
    try {
      return await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_validate_payment_address(0, paymentAddress, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['int32','uint32'],
          (xhandle: number, err: number) => {
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
   * Sign with Address
   *
   * Example:
   * ```
   * address = await Wallet.signWithAddress('pay:null:addr', bufferOfMsg)
   * await Wallet.signWithAddress('pay:null:addr', bufferOfMsg)
   * ```
   */
  public static async signWithAddress (paymentAddress: string, message: Buffer): Promise<Buffer> {
    try {
      return await createFFICallbackPromise<Buffer>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_sign_with_address(0, paymentAddress,
            ref.address(message), message.length, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32', 'uint32', 'pointer', 'uint32'],
          (xHandle: number, err: number, details: any, length: number) => {
            if (err) {
              reject(err)
              return
            }
            if (!details) {
              reject(`Empty buffer returned`)
              return
            }
            const newBuffer = voidPtrToUint8Array(details, length)
            resolve(newBuffer)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Verify with address
   *
   * Example:
   * ```
   * valid = await connection.verifyWithAddress("pay:null:addr", bufferWithMsg, bufferWithSig)
   * ```
   * @returns {Promise<boolean>}
   */
  public static async verifyWithAddress (paymentAddress: string, message: Buffer, signature: Buffer): Promise<boolean> {
    try {
      return await createFFICallbackPromise<boolean>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_wallet_verify_with_address(0, paymentAddress,
              ref.address(message), message.length,
              ref.address(signature), signature.length, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
            'void',
            ['uint32', 'uint32', 'bool'],
            (xHandle: number, err: number, valid: boolean) => {
              if (err) {
                reject(err)
                return
              }
              resolve(valid)
            })
        )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Sends token to a specified address
   *
   * Example:
   * ```
   * address = await Wallet.createPaymentAddress('00000000000000000000000001234567')
   * await Wallet.sendTokens({
   *     payment: 0,
   *     recipient: address,
   *     tokens: 1
   * })
   */
  public static async sendTokens ({ payment, tokens, recipient }: ISendTokens): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_send_tokens(0, payment, tokens.toString(), recipient, cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32','string'],
          (xhandle: number, err: number, receipt: string) => {
            if (err) {
              reject(err)
              return
            }
            resolve(receipt)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Adds a record to the wallet for storage
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {},
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   * ```
   * @async
   * @param {Record} record
   * @returns {Promise<void>}
   */
  public static async addRecord (record: IRecord): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_add_record(commandHandle,
            record.type_,
            record.id, record.value,
            JSON.stringify(record.tags),
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'], (xhandle: number, err: number) => {
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
   * Updates a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {},
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   * await Wallet.updateRecordValue({
   *   id: 'RecordId',
   *   type_: 'TestType',
   *   value: 'RecordValueNew'
   * })
   * ```
   */
  public static async updateRecordValue (record: IRecordUpdate): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_update_record_value(commandHandle,
            record.type_,
            record.id, record.value,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Updates a record's tags already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *     id: 'RecordId',
   *     tags: {},
   *     type_: 'TestType',
   *     value: 'RecordValue'
   * })
   *
   * updateRecordTags({
   *     id: 'RecordId',
   *     tags: {},
   *     type_: 'TestType',
   *     value: ''
   * })
   * ```
   */
  public static async updateRecordTags (record: IRecord): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_update_record_tags(commandHandle,
            record.type_,
            record.id,
            JSON.stringify(record.tags),
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Adds tags to a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *     id: 'RecordId',
   *     tags: {},
   *     type_: 'TestType',
   *     value: 'RecordValue'
   * })
   *
   * addRecordTags({  id: 'RecordId',
   *     tags: {
   *          "tagName1": "tag value 1",
   *          "~tagName2": "tag value 2 unencrypted",
   *           "tagName3", 1
   *     },
   *     type_: 'TestType',
   *     value: ''
   * })
   * ```
   */
  public static async addRecordTags (record: IRecord): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_add_record_tags(commandHandle,
            record.type_,
            record.id,
            JSON.stringify(record.tags),
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Tags to delete from a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {
   *        "foo": "bar",
   *        "~fizz": "buzz",
   *        "unencyrptedStringTag": "tag value 1",
   *        "~encryptedStringTag": "tag value 2 unencrypted",
   *        "unencyrptedIntTag": 1
   *    },
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   *
   * deleteRecordTags({
   *     id: 'RecordId',
   *     tags: { tagList: [ "foo", "buzz", "~encryptedStringTag" ] }
   *     type_: 'TestType',
   *     value: ''
   * })
   * ```
   */
  public static async deleteRecordTags (record: IRecord, { tagList }: IDeleteRecordTagsOptions): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_delete_record_tags(commandHandle,
            record.type_,
            record.id,
            JSON.stringify(tagList),
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Delete a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {
   *        "foo": "bar",
   *        "~fizz": "buzz",
   *        "unencyrptedStringTag": "tag value 1",
   *        "~encryptedStringTag": "tag value 2 unencrypted",
   *        "unencyrptedIntTag": 1
   *    },
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   *
   * await Wallet.deleteRecord({
   *    id: 'RecordId',
   *    type_: 'TestType'
   * })
   * ```
   */
  public static async deleteRecord ({ type, id }: IDeleteRecordData): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_delete_record(commandHandle,
            type,
            id,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Retrieve a record already in the wallet
   *
   * Example:
   * ```
   * await Wallet.addRecord({
   *    id: 'RecordId',
   *    tags: {
   *        "foo": "bar",
   *        "~fizz": "buzz",
   *        "unencyrptedStringTag": "tag value 1",
   *        "~encryptedStringTag": "tag value 2 unencrypted",
   *        "unencyrptedIntTag": 1
   *    },
   *    type_: 'TestType',
   *    value: 'RecordValue'
   * })
   *
   * record = await Wallet.getReocrd({ type: 'TestType', id: 'RecordId'})
   * ```
   */
  public static async getRecord ({ type, id, options }: IGerRecordData): Promise<string> {
    const commandHandle = 0
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_get_record(commandHandle,
            type,
            id,
            JSON.stringify(options),
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32','string'],
          (xhandle: number, err: number, info: string) => {
            if (err) {
              reject(err)
              return
            }
            resolve(info)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

   /**
    * Open a search handle
    *
    * Example:
    * ```
    * searchHandle = await openSearch({type: 'TestType'})
    * ```
    */
  public static async openSearch ({ type, queryJson, options }: IOpenSearchData): Promise<number> {
    const commandHandle = 0
    try {
      return await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_open_search(commandHandle,
            type,
            queryJson,
            options,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32','uint32'],
          (xhandle: number, err: number, handle: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve(handle)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Close a search handle
   *
   * Example:
   * ```
   * searchHandle = await Wallet.openSearch({type: 'TestType'})
   * await Wallet.closeSearch(searchHandle)
   * ```
   */
  public static async closeSearch (handle: number): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_close_search(commandHandle,
            handle,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve(handle)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Initiate or continue a search
   *
   * Example:
   * ```
   * searchHandle = await Wallet.openSearch({type: 'TestType'})
   * records = await Wallet.searchNextRecords(searchHandle, {count:5})
   * await Wallet.closeSearch(searchHandle)
   * ```
   */
  public static async searchNextRecords (handle: number, { count }: ISearchNextRecordsOptions): Promise<string> {
    const commandHandle = 0
    try {
      return await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_search_next_records(commandHandle,
            handle,
            count,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32','string'],
          (xhandle: number, err: number, info: string) => {
            if (err) {
              reject(err)
              return
            }
            resolve(info)
          })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Imports wallet from file with given key.
   * Cannot be used if wallet is already opened (Especially if vcx_init has already been used).
   *
   * Example:
   * ```
   * config = {
   *     "wallet_name":"",
   *     "wallet_key":"",
   *     "exported_wallet_path":"",
   *     "backup_key":""
   * }
   * await Wallet.import(JSON.stringify(config))
   * ```
   */
  public static async import (config: string): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_import(commandHandle,
            config,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Export a file to a wallet, backup key used for decrypting the file.
   *
   * Example:
   * ```
   * await Wallet.export('/tmp/foobar.wallet', 'key_for_wallet')
   * ```
   */
  public static async export (path: string, backupKey: string): Promise<void> {
    const commandHandle = 0
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_wallet_export(commandHandle,
            path,
            backupKey,
            cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => Callback(
          'void',
          ['uint32','uint32'],
          (xhandle: number, err: number) => {
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
   * Set the wallet handle for libvcx to use, called before vcxInitPostIndy
   *
   * Example:
   * ```
   * Wallet.setHandle(1)
   * setPoolHandle(1)
   * vcxInitPostIndy(config)
   */
  public static setHandle (handle: number): void {
    rustAPI().vcx_wallet_set_handle(handle)
  }
}
