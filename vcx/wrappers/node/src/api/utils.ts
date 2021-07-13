import { Callback } from 'ffi-napi'

import { VCXInternalError } from '../errors'
import { initRustAPI, rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { IInitVCXOptions } from './common'
// import { resolve } from 'url';

export async function provisionAgent (configAgent: string, options: IInitVCXOptions = {}): Promise<string> {
  /**
   * Provision an agent in the agency, populate configuration and wallet for this agent.
   *
   * Example:
   * ```
   * enterpriseConfig = {
   *     'agency_url': 'https://enym-eagency.pdev.evernym.com',
   *     'agency_did': 'YRuVCckY6vfZfX9kcQZe3u',
   *     'agency_verkey': "J8Yct6FwmarXjrE2khZesUXRVVSVczSoa9sFaGe6AD2v",
   *     'wallet_name': 'LIBVCX_SDK_WALLET',
   *     'agent_seed': '00000000000000000000000001234561',
   *     'enterprise_seed': '000000000000000000000000Trustee1',
   *     'wallet_key': '1234'
   *  }
   * vcxConfig = await provisionAgent(JSON.stringify(enterprise_config))
   */
  try {
    initRustAPI(options.libVCXPath)
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_agent_provision_async(0, configAgent, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, config: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(config)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function updateAgentInfo (options: string): Promise<void> {
  /**
   * Update information on the agent (ie, comm method and type)
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_agent_update_info(0, options, cb)
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

export function getVersion (): string {
  return rustAPI().vcx_version()
}

export async function getLedgerFees (): Promise<string> {
  /**
   * Get ledger fees from the sovrin network
   */
  try {
    const ledgerFees = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_ledger_get_fees(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, fees: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(fees)
        })
    )
    return ledgerFees
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function getLedgerAuthorAgreement (): Promise<string> {
  /**
   * Retrieve author agreement set on the sovrin network
   */
  try {
    const agreement = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_get_ledger_author_agreement(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, agreement: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(agreement)
        })
    )
    return agreement
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export function setActiveTxnAuthorAgreementMeta (text: string | null | undefined,
                                                 version: string | null | undefined,
                                                 hash: string | null | undefined,
                                                 acc_mech_type: string,
                                                 time_of_acceptance: number) {
  /**
   * Set some accepted agreement as active.
   * As result of successful call of this function appropriate metadata will be appended to each write request.
   */
  return rustAPI().vcx_set_active_txn_author_agreement_meta(text, version, hash, acc_mech_type, time_of_acceptance)
}

export function shutdownVcx (deleteWallet: boolean): number {
  return rustAPI().vcx_shutdown(deleteWallet)
}

export interface IUpdateWebhookUrl {
  webhookUrl: string,
}

export async function vcxUpdateWebhookUrl ({ webhookUrl }: IUpdateWebhookUrl): Promise<void> {
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_update_webhook_url(0, webhookUrl, cb)
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

export interface IUpdateInstitutionConfigs {
  name: string,
  logoUrl: string
}
export function updateInstitutionConfigs ({ name, logoUrl }: IUpdateInstitutionConfigs): number {
  const rc = rustAPI().vcx_update_institution_info(name, logoUrl)
  if (rc) {
    throw new VCXInternalError(rc)
  }
  return rc
}

export interface IDownloadMessagesConfigs {
  status: string,
  uids: string,
  pairwiseDids: string
}

export async function downloadMessages
({ status, uids, pairwiseDids }: IDownloadMessagesConfigs): Promise<string> {
  /**
   *  Retrieve messages from the agency
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_messages_download(0, status, uids, pairwiseDids, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, messages: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(messages)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export interface IUpdateMessagesConfigs {
  msgJson: string
}

export async function updateMessages ({ msgJson }: IUpdateMessagesConfigs): Promise<number> {
  /**
   * Update the status of messages from the specified connection
   */
  try {
    return await createFFICallbackPromise<number>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_messages_update_status(0, 'MS-106', msgJson, cb)
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
          resolve(err)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export function setPoolHandle (handle: number): void {
  rustAPI().vcx_pool_set_handle(handle)
}

export async function endorseTransaction (transaction: string): Promise<void> {
  /**
   * Endorse transaction to the ledger preserving an original author
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_endorse_transaction(0, transaction, cb)
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
