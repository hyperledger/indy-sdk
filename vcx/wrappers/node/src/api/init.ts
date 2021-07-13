import { Callback } from 'ffi-napi'

import { VCXInternalError } from '../errors'
import { initRustAPI, rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { IInitVCXOptions } from './common'

/**
 * Initializes VCX with config file.
 * An example config file is at libvcx/sample_config/config.json
 * The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
 *
 * Example:
 * ```
 * await initVcx('/home/username/vcxconfig.json')
 * ```
 */
export async function initVcx (configPath: string, options: IInitVCXOptions = {}): Promise<void> {
  initRustAPI(options.libVCXPath)
  let rc = null
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        rc = rustAPI().vcx_init(0, configPath, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32', 'uint32'],
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
 * Initializes VCX with config file.
 * The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
 *
 * Example:
 * ```
 * config = {
 *   "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
 *   "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
 *   "agency_endpoint": "http://localhost:8080",
 *   "genesis_path":"/var/lib/indy/verity-staging/pool_transactions_genesis",
 *   "institution_name": "institution",
 *   "institution_logo_url": "http://robohash.org/234",
 *   "institution_did": "EwsFhWVoc3Fwqzrwe998aQ",
 *   "institution_verkey": "8brs38hPDkw5yhtzyk2tz7zkp8ijTyWnER165zDQbpK6",
 *   "remote_to_sdk_did": "EtfeMFytvYTKnWwqTScp9D",
 *   "remote_to_sdk_verkey": "8a7hZDyJK1nNCizRCKMr4H4QbDm8Gg2vcbDRab8SVfsi",
 *   "sdk_to_remote_did": "KacwZ2ndG6396KXJ9NDDw6",
 *   "sdk_to_remote_verkey": "B8LgZGxEPcpTJfZkeqXuKNLihM1Awm8yidqsNwYi5QGc"
 *  }
 * await initVcxWithConfig('/home/username/vcxconfig.json')
 * ```
 */
export async function initVcxWithConfig (config: string, options: IInitVCXOptions = {}): Promise<void> {
  initRustAPI(options.libVCXPath)
  let rc = null
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        rc = rustAPI().vcx_init_with_config(0, config, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32', 'uint32'],
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

export function initMinimal (config: string): number {
  return rustAPI().vcx_init_minimal(config)
}
