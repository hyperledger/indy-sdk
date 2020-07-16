const { initRustAPI, initVcxWithConfig, provisionAgent } = require('./../dist/src')
const ffi = require('ffi-napi');
const os = require('os')

const extension = { darwin: '.dylib', linux: '.so', win32: '.dll' }
const libPath = { darwin: '/usr/local/lib/', linux: '/usr/lib/', win32: 'c:\\windows\\system32\\' }

function getLibraryPath (libraryName) {
  const platform = os.platform()
  const postfix = extension[platform.toLowerCase()] || extension.linux
  const libDir = libPath[platform.toLowerCase()] || libPath.linux
  return `${libDir}${libraryName}${postfix}`
}

async function loadPostgresPlugin (provisionConfig) {
  const myffi = ffi.Library(getLibraryPath('libindystrgpostgres'), { postgresstorage_init: ['void', []] })
  await myffi.postgresstorage_init()
}

async function initLibNullPay () {
  const myffi = ffi.Library(getLibraryPath('libnullpay'), { nullpay_init: ['void', []] })
  await myffi.nullpay_init()
}

async function initRustApiAndLogger (logLevel) {
  const rustApi = initRustAPI()
  await rustApi.vcx_set_default_logger(logLevel)
}

async function provisionAgentInAgency (config) {
  return JSON.parse(await provisionAgent(JSON.stringify(config)))
}

async function initVcxWithProvisionedAgentConfig (config) {
  config.institution_name = 'faber'
  config.institution_logo_url = 'http://robohash.org/234'
  config.genesis_path = `${__dirname}/docker.txn`
  await initVcxWithConfig(JSON.stringify(config))
}

function getRandomInt (min, max) {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min)) + min
}

module.exports.loadPostgresPlugin = loadPostgresPlugin
module.exports.initLibNullPay = initLibNullPay
module.exports.initRustApiAndLogger = initRustApiAndLogger
module.exports.provisionAgentInAgency = provisionAgentInAgency
module.exports.initVcxWithProvisionedAgentConfig = initVcxWithProvisionedAgentConfig
module.exports.getRandomInt = getRandomInt
