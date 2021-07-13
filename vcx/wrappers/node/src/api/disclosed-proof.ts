import { Callback } from 'ffi-napi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { ISerializedData } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './vcx-base-with-state'

/**
 *    The API represents an Prover side in the credential presentation process.
 *    Assumes that pairwise connection between Verifier and Prover is already established.
 *
 *    # State
 *
 *    The set of object states and transitions depends on communication method is used.
 *    The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.
 *
 *    proprietary:
 *        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.
 *
 *        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.
 *
 *        VcxStateType::VcxStateAccepted - once `vcx_disclosed_proof_send_proof` (send `PROOF_REQ` message) is called.
 *
 *    aries:
 *        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.
 *
 *        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.
 *
 *        VcxStateType::VcxStateOfferSent - once `vcx_disclosed_proof_send_proof` (send `Presentation` message) is called.
 *        VcxStateType::None - once `vcx_disclosed_proof_decline_presentation_request` (send `PresentationReject` or `PresentationProposal` message) is called.
 *
 *        VcxStateType::VcxStateAccepted - once `Ack` messages is received.
 *        VcxStateType::None - once `ProblemReport` messages is received.
 *
 *    # Transitions
 *
 *    proprietary:
 *        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived
 *
 *        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived
 *
 *        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted
 *
 *    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
 *        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived
 *
 *        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived
 *
 *        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted
 *        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_decline_presentation_request` - VcxStateType::None
 *
 *        VcxStateType::VcxStateOfferSent - received `Ack` - VcxStateType::VcxStateAccepted
 *        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None
 *
 *    # Messages
 *
 *    proprietary:
 *        ProofRequest (`PROOF_REQ`)
 *        Proof (`PROOF`)
 *
 *    aries:
 *        PresentationRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#request-presentation
 *        Presentation - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#presentation
 *        PresentationProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
 *        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
 */

export interface IDisclosedProofData {
  source_id: string,
}

export type IDisclosedProofRequest = object

/**
 * @description Interface that represents the parameters for `DisclosedProof.create` function.
 * @interface
 */
export interface IDisclosedProofCreateData {
  // We're going to need it in the future
  connection: Connection,
  // Institution's identification for the proof, should be unique.
  sourceId: string,
  // Proof request received via "getRequests"
  request: string
}

/**
 * @description Interface that represents the parameters for `DisclosedProof.createWithMsgId` function.
 * @interface
 */
export interface IDisclosedProofCreateWithMsgIdData {
  // Connection to query for proof request
  connection: Connection,
  // Id of the message that contains the proof request
  msgId: string,
  // Institution's personal identification for the proof, should be unique.
  sourceId: string
}

/**
 * @description Interface that represents the result of `DisclosedProof.getCredentials` function.
 * @interface
 * example: {'attrs': {'attribute_0': [{'cred_info': {'schema_id': 'id', 'cred_def_id': 'id', 'attrs': {'attr_name': 'attr_value', ...}, 'referent': '914c7e11'}}]}}
 */
export interface IRetrievedCreds {
  attrs: {
    [ index: string ]: ICredData[]
  },
  predicates: any
}

export interface ICredData {
  cred_info: {
    [ index: string ]: any
  },
  interval: any
}

/**
 * @description Interface that represents the parameters for `DisclosedProof.generateProof` function.
 * @interface
 * example: {'attrs': {'attribute_0': {'credential': {'cred_info': {'cred_def_id': 'od', 'schema_id': 'id', 'referent': '0c212108-9433-4199-a21f-336a44164f38', 'attrs': {'attr_name': 'attr_value', ...}}}}}}
 */
export interface IGenerateProofData {
  selectedCreds: {
    [index: string]: ICredData
  },
  selfAttestedAttrs: {
    [index: string]: string
  }
}

/**
 * @description Interface that represents the parameters for `DisclosedProof.declinePresentationRequest` function.
 * @interface
 */
export interface IDeclinePresentationRequestData {
  // Connection to decline presentation request
  connection: Connection,
  // Human-readable string that explain the reason of decline
  reason?: string,
  // The proposed format of presentation request
  proposal?: any
}

export class DisclosedProof extends VCXBaseWithState<IDisclosedProofData> {
  /**
   * Create a proof for fulfilling a corresponding proof request
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProofRequest = {
   * '@topic': {
   *   mid: 9,
   *   tid: 1
   * },
   * '@type': {
   *   name: 'PROOF_REQUEST',
   *   version: '1.0'
   * },
   * 'msg_ref_id': 'abcd',
   * 'proof_request_data': {
   *   name: 'Account Certificate',
   *   nonce: '838186471541979035208225',
   *   requested_attributes: {
   *      business_2: {
   *       name: 'business'
   *     },
   *     email_1: {
   *       name: 'email'
   *     },
   *     name_0: {
   *       name: 'name'
   *     }
   *   },
   *   requested_predicates: {},
   *   version: '0.1'
   * }
   * sourceId = 'testDisclosedProofSourceId'
   * disclosedProof = await DisclosedProof.create({ connection, request: disclosedProofRequest, sourceId: sourceId })
   * ```
   */
  public static async create ({ sourceId, request }: IDisclosedProofCreateData): Promise<DisclosedProof> {
    const newObj = new DisclosedProof(sourceId)
    try {
      await newObj._create((cb) => rustAPI().vcx_disclosed_proof_create_with_request(
        0,
        sourceId,
        request,
        cb
        )
      )
      return newObj
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Create a proof based off of a known message id for a given connection.
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * ```
   */
  public static async createWithMsgId ({ connection, sourceId, msgId }: IDisclosedProofCreateWithMsgIdData):
  Promise<DisclosedProof> {
    try {
      return await createFFICallbackPromise<DisclosedProof>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_create_with_msgid(0, sourceId, connection.handle, msgId, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
            'void',
            ['uint32', 'uint32', 'uint32', 'string'],
            (xHandle: number, err: number, handle: number, proofReq: string) => {

              if (err) {
                reject(err)
                return
              }
              const newObj = new DisclosedProof(sourceId)
              newObj._setHandle(handle)
              newObj._proofReq = proofReq
              resolve(newObj)
            })
      )
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Builds a proof object with defined attributes.
   * Attributes are provided by a previous call to the serialize function.
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * data = await disclosedProof.serialize()
   * ```
   */
  public static async deserialize (data: ISerializedData<IDisclosedProofData>) {
    try {
      const newObj = await super._deserialize<DisclosedProof, {}>(DisclosedProof, data)
      return newObj
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }
  /**
   * Queries agency for all pending proof requests from the given connection.
   *
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * requests = disclosedProof.getRequests(connection)
   * ```
   */
  public static async getRequests (connection: Connection): Promise<IDisclosedProofRequest[]> {
    const requestsStr = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_disclosed_proof_get_requests(0, connection.handle, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32', 'uint32', 'string'],
        (handle: number, err: number, messages: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(messages)
        })
    )
    const requests = JSON.parse(requestsStr)
    return requests
  }

  protected _releaseFn = rustAPI().vcx_disclosed_proof_release
  protected _updateStFn = rustAPI().vcx_disclosed_proof_update_state
  protected _updateStWithMessageFn = rustAPI().vcx_disclosed_proof_update_state_with_message
  protected _getStFn = rustAPI().vcx_disclosed_proof_get_state
  protected _serializeFn = rustAPI().vcx_disclosed_proof_serialize
  protected _deserializeFn = rustAPI().vcx_disclosed_proof_deserialize
  private _proofReq: string = ''

  /**
   * Get credentials from wallet matching to the proof request associated with proof object
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * creds = await disclosedProof.getCredentials()
   * ```
   */
  public async getCredentials (): Promise<IRetrievedCreds> {
    try {
      const credsStr = await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_retrieve_credentials(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xcommandHandle: number, err: number, credsRet: string) => {
              if (err) {
                reject(err)
                return
              }
              resolve(credsRet)
            })
        )
      const creds: IRetrievedCreds = JSON.parse(credsStr)
      return creds
    } catch (err) {
      throw new VCXInternalError(err)
    }
  }

  /**
   * Sends the proof to the Connection
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * { attrs } = await disclosedProof.getCredentials()
   * valSelfAttested = 'testSelfAttestedVal'
   * await disclosedProof.generateProof({
   *    {},
   *    mapValues(attrs, () => valSelfAttested)
   *  })
   * await disclosedProof.sendProof(connection)
   * ```
   */
  public async sendProof (connection?: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const connectionHandle = connection ? connection.handle : 0
            const rc = rustAPI().vcx_disclosed_proof_send_proof(0, this.handle, connectionHandle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
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
   * Sends the proof reject to the Connection
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * await disclosedProof.rejectProof(connection)
   * ```
   */
  public async rejectProof (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_reject_proof(0, this.handle, connection.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
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
   * Generates the proof message for sending.
   *
   * Example:
   * ```
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * { attrs } = await disclosedProof.getCredentials()
   * valSelfAttested = 'testSelfAttestedVal'
   * await disclosedProof.generateProof({
   *    {},
   *    mapValues(attrs, () => valSelfAttested)
   *  })
   * await disclosedProof.getProofMessage(connection)
   * ```
   */
  public async getProofMessage (): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_get_proof_msg(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, message: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!message) {
                reject(`proof ${this.sourceId} returned empty string`)
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
   * Generates the proof reject message for sending.
   *
   * Example:
   * ```
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * await disclosedProof.getRejectMessage(connection)
   * ```
   */
  public async getRejectMessage (): Promise<string> {
    try {
      return await createFFICallbackPromise<string>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_get_reject_msg(0, this.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
            'void',
            ['uint32', 'uint32', 'string'],
            (xHandle: number, err: number, message: string) => {
              if (err) {
                reject(err)
                return
              }
              if (!message) {
                reject(`proof ${this.sourceId} returned empty string`)
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
   * Accept proof request associated with proof object and
   * generates a proof from the selected credentials and self attested attributes
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * { attrs } = await disclosedProof.getCredentials()
   * valSelfAttested = 'testSelfAttestedVal'
   * await disclosedProof.generateProof({
   *    {},
   *    mapValues(attrs, () => valSelfAttested)
   *  })
   * ```
   */
  public async generateProof ({ selectedCreds, selfAttestedAttrs }: IGenerateProofData): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_generate_proof(
              0,
              this.handle,
              JSON.stringify(selectedCreds),
              JSON.stringify(selfAttestedAttrs),
              cb
            )
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
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
   * Declines presentation request.
   * There are two ways of following interaction:
   *     - Prover wants to propose using a different presentation - pass `proposal` parameter.
   *     - Prover doesn't want to continue interaction - pass `reason` parameter.
   * Note that only one of these parameters can be passed.
   *
   * Note that proposing of different presentation is supported for `aries` protocol only.
   *
   * Example:
   * ```
   * connection = await Connection.create({id: 'foobar'})
   * inviteDetails = await connection.connect()
   * disclosedProof = await DisclosedProof.createWithMsgId(connection, 'testDisclousedProofMsgId', 'sourceId')
   * await disclosedProof.declinePresentationRequest({connection, reason: 'some reason', proposal: null})
   * ```
   */
  public async declinePresentationRequest ({ connection, reason, proposal }: IDeclinePresentationRequestData): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_decline_presentation_request(
              0,
              this.handle,
              connection.handle,
              reason,
              proposal ? JSON.stringify(proposal) : null,
              cb
            )
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback(
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

  get proofRequest (): string {
    return this._proofReq
  }
}
