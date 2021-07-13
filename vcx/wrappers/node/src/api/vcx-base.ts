import * as ffi from 'ffi-napi'
import { VCXInternalError } from '../errors'
import { createFFICallbackPromise, ICbRef } from '../utils/ffi-helpers'
import { GCWatcher } from '../utils/memory-management-helpers'
import { ISerializedData } from './common'

export type IVCXBaseCreateFn = (cb: ICbRef) => number

export abstract class VCXBase<SerializedData> extends GCWatcher {
  protected static async _deserialize<T extends VCXBase<any> = any, P = object> (
    VCXClass: new(sourceId: string, ...args: any[]) => T,
    objData: ISerializedData<{ source_id: string }>,
    constructorParams?: P
  ): Promise<T> {
    try {
      const obj = new VCXClass(objData.data.source_id, constructorParams)
      await obj._initFromData(objData)
      return obj
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  protected abstract _serializeFn: (commandHandle: number, handle: number, cb: ICbRef) => number
  protected abstract _deserializeFn: (commandHandle: number, handle: string, cb: ICbRef) => number
  protected _sourceId: string

  constructor (sourceId: string) {
    super()
    this._sourceId = sourceId
  }

  /**
   *
   * Data returned can be used to recreate an entity by passing it to the deserialize function.
   *
   * Same json object structure that is passed to the deserialize function.
   *
   * Example:
   *
   * ```
   *  data = await object.serialize()
   * ```
   */
  public async serialize (): Promise<ISerializedData<SerializedData>> {
    try {
      const dataStr = await createFFICallbackPromise<string>(
        (resolve, reject, cb) => {
          const rc = this._serializeFn(0, this.handle, cb)
          if (rc) {
            reject(rc)
            return
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'string'],
          (handle: string, err: any, serializedData?: string) => {
            if (err) {
              reject(err)
              return
            }
            if (!serializedData) {
              reject('no data to serialize')
              return
            }
            resolve(serializedData)
          }
        )
      )
      const data: ISerializedData<SerializedData> = JSON.parse(dataStr)
      return data
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /** The source Id assigned by the user for this object */
  get sourceId () {
    return this._sourceId
  }

  protected async _create (createFn: IVCXBaseCreateFn): Promise<void> {
    const handleRes = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = createFn(cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (xHandle: number, err: number, handle: number) => {
            if (err) {
              reject(err)
              return
            }
            resolve(handle)
          })
    )
    this._setHandle(handleRes)
  }

  private async _initFromData (objData: object): Promise<void> {
    const commandHandle = 0
    const objHandle = await createFFICallbackPromise<number>(
        (resolve, reject, cb) => {
          const rc = this._deserializeFn(commandHandle, JSON.stringify(objData), cb)
          if (rc) {
            reject(rc)
          }
        },
        (resolve, reject) => ffi.Callback(
          'void',
          ['uint32', 'uint32', 'uint32'],
          (xHandle: number, err: number, handle: number) => {
            if (err) {
              reject(err)
            }
            resolve(handle)
          })
    )
    this._setHandle(objHandle)
  }

}
