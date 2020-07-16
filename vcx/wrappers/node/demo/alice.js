const { DisclosedProof } = require('../dist/src/api/disclosed-proof')
const { Connection } = require('../dist/src/api/connection')
const { Credential } = require('../dist/src/api/credential')
const { StateType } = require('../dist/src')
const readlineSync = require('readline-sync')
const sleepPromise = require('sleep-promise')
const demoCommon = require('./common')
const logger = require('./logger')
const url = require('url')
const isPortReachable = require('is-port-reachable')
const { runScript } = require('./script-comon')

const utime = Math.floor(new Date() / 1000)
const optionalWebhook = 'http://localhost:7209/notifications/alice'

// TODO: Alice uses the same seed as faber, will overwrite faber's service record
const provisionConfig = {
  agency_url: 'http://localhost:8080',
  agency_did: 'VsKV7grR1BUE29mG2Fm2kX',
  agency_verkey: 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR',
  wallet_name: `node_vcx_demo_alice_wallet_${utime}`,
  wallet_key: '123',
  payment_method: 'null',
  enterprise_seed: '000000000000000000000000Trustee1'
}

const logLevel = 'error'

async function runAlice (options) {
  await demoCommon.initLibNullPay()

  logger.info('#0 Initialize rust API from NodeJS')
  await demoCommon.initRustApiAndLogger(logLevel)

  if (options.comm === 'aries') {
    provisionConfig.protocol_type = '2.0'
    provisionConfig.communication_method = 'aries'
    logger.info('Running with Aries VCX Enabled! Make sure VCX agency is configured to use protocol_type 2.0')
  }
  if (options.postgresql) {
    await demoCommon.loadPostgresPlugin(provisionConfig)
    provisionConfig.wallet_type = 'postgres_storage'
    provisionConfig.storage_config = '{"url":"localhost:5432"}'
    provisionConfig.storage_credentials = '{"account":"postgres","password":"mysecretpassword","admin_account":"postgres","admin_password":"mysecretpassword"}'
    logger.info(`Running with PostreSQL wallet enabled! Config = ${provisionConfig.storage_config}`)
  } else {
    logger.info('Running with builtin wallet.')
  }
  if (await isPortReachable(url.parse(optionalWebhook).port, { host: url.parse(optionalWebhook).hostname })) { // eslint-disable-line
    provisionConfig.webhook_url = optionalWebhook
    logger.info(`Running with webhook notifications enabled! Webhook url = ${optionalWebhook}`)
  } else {
    logger.info('Webhook url will not be used')
  }

  logger.info('#8 Provision an agent and wallet, get back configuration details')
  const config = await demoCommon.provisionAgentInAgency(provisionConfig)

  logger.info('#9 Initialize libvcx with new configuration')
  await demoCommon.initVcxWithProvisionedAgentConfig(config)

  logger.info('Input faber.py invitation details')
  const details = readlineSync.question('Enter your invite details: ')
  const jdetails = JSON.parse(details)

  logger.info('#10 Convert to valid json and string and create a connection to faber')
  const connectionToFaber = await Connection.createWithInvite({ id: 'faber', invite: JSON.stringify(jdetails) })
  await connectionToFaber.connect({ data: '{"use_public_did": true}' })
  let connectionState = await connectionToFaber.getState()
  while (connectionState !== StateType.Accepted) {
    await sleepPromise(2000)
    await connectionToFaber.updateState()
    connectionState = await connectionToFaber.getState()
  }

  logger.info('#11 Wait for faber.py to issue a credential offer')
  await sleepPromise(10000)
  const offers = await Credential.getOffers(connectionToFaber)
  logger.info(`Alice found ${offers.length} credential offers.`)
  logger.debug(JSON.stringify(offers))

  // Create a credential object from the credential offer
  const credential = await Credential.create({ sourceId: 'credential', offer: JSON.stringify(offers[0]) })

  logger.info('#15 After receiving credential offer, send credential request')
  await credential.sendRequest({ connection: connectionToFaber, payment: 0 })

  logger.info('#16 Poll agency and accept credential offer from faber')
  let credentialState = await credential.getState()
  while (credentialState !== StateType.Accepted) {
    await sleepPromise(2000)
    await credential.updateState()
    credentialState = await credential.getState()
  }

  logger.info('#22 Poll agency for a proof request')
  const requests = await DisclosedProof.getRequests(connectionToFaber)

  logger.info('#23 Create a Disclosed proof object from proof request')
  const proof = await DisclosedProof.create({ sourceId: 'proof', request: JSON.stringify(requests[0]) })

  logger.info('#24 Query for credentials in the wallet that satisfy the proof request')
  const credentials = await proof.getCredentials()

  // Use the first available credentials to satisfy the proof request
  for (let i = 0; i < Object.keys(credentials.attrs).length; i++) {
    const attr = Object.keys(credentials.attrs)[i]
    credentials.attrs[attr] = {
      credential: credentials.attrs[attr][0]
    }
  }

  logger.info('#25 Generate the proof')
  await proof.generateProof({ selectedCreds: credentials, selfAttestedAttrs: {} })

  logger.info('#26 Send the proof to faber')
  await proof.sendProof(connectionToFaber)

  let proofState = await proof.getState()
  while (proofState !== StateType.Accepted) {
    await sleepPromise(2000)
    await proof.updateState()
    proofState = await proof.getState()
  }
  logger.info('Proof is verified.')
  process.exit(0)
}

const optionDefinitions = [
  {
    name: 'help',
    alias: 'h',
    type: Boolean,
    description: 'Display this usage guide.'
  },
  {
    name: 'comm',
    type: String,
    description: 'Communication method. Possible values: aries, legacy. Default is aries.',
    defaultValue: 'aries'
  },
  {
    name: 'postgresql',
    type: Boolean,
    description: 'If specified, postresql wallet will be used.',
    defaultValue: false
  }
]

const usage = [
  {
    header: 'Options',
    optionList: optionDefinitions
  },
  {
    content: 'Project home: {underline https://github.com/Patrik-Stas/indy-wallet-watch}'
  }
]

function areOptionsValid (options) {
  const allowedCommMethods = ['aries', 'legacy']
  if (!(allowedCommMethods.includes(options.comm))) {
    console.error(`Unknown communication method ${options.comm}. Only ${JSON.stringify(allowedCommMethods)} are allowed.`)
    return false
  }
  return true
}

runScript(optionDefinitions, usage, areOptionsValid, runAlice)
