import * as ffi from 'ffi-napi'
import * as os from 'os'

import { FFIConfiguration, IFFIEntryPoint } from './rustlib'

export interface IVCXRuntimeConfig {
  basepath?: string
}

// VCXRuntime is the object that interfaces with the vcx sdk functions
// FFIConfiguration will contain all the sdk api functions
// VCXRuntimeConfg is a class that currently only contains a chosen basepath for the .so file
// I made it a class just in case we think of more needed configs

const extension = { darwin: '.dylib', linux: '.so', win32: '.dll' }
const libPath = { darwin: '/usr/local/lib/', linux: '/usr/lib/', win32: 'c:\\windows\\system32\\' }

export class VCXRuntime {
  public readonly ffi: IFFIEntryPoint
  private _config: IVCXRuntimeConfig

  constructor (config: IVCXRuntimeConfig = {}) {
    this._config = config
    // initialize FFI
    const libraryPath = this._initializeBasepath()
    this.ffi = ffi.Library(libraryPath, FFIConfiguration)
  }

  private _initializeBasepath = (): string => {
    const platform = os.platform()
    // @ts-ignore
    const postfix = extension[platform.toLowerCase()] || extension.linux
    // @ts-ignore
    const libDir = libPath[platform.toLowerCase()] || libPath.linux
    const library = `libvcx${postfix}`
    const customPath = process.env.LIBVCX_PATH ? process.env.LIBVCX_PATH + library : undefined
    return customPath || this._config.basepath || `${libDir}${library}`
  }
}
