import * as ffi from 'ffi-napi'
import { VCXInternalError } from '../errors'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { IPaymentOutput } from './common'

export interface IPaymentTxn {
  amount: number,
  credit: boolean,
  inputs: string[],
  outputs: IPaymentOutput[]
}
export interface IPamentManagerConstructorData {
  handle: number
}
export abstract class PaymentManager {
  public readonly handle: number
  protected abstract _getPaymentTxnFn: (commandId: number, handle: number, cb: any) => number
  constructor ({ handle }: IPamentManagerConstructorData) {
    this.handle = handle
  }
  public async getPaymentTxn (): Promise<IPaymentTxn> {
    try {
      const paymentTxnStr = await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = this._getPaymentTxnFn(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => ffi.Callback('void', ['uint32', 'uint32', 'string'],
          (xcommandHandle: number, err: number, info: any) => {
            if (err) {
              reject(err)
              return
            }
            resolve(info)
          })
        )
      const paymentTxn = JSON.parse(paymentTxnStr)
      return paymentTxn
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
}
