import * as ffi from 'ffi-napi'
import '../module-resolver-helper'

import { assert } from 'chai'
import { Logger, loggerFunction, loggerToVoidPtr, setLogger } from 'src'
import { initRustAPI } from '../../src/rustlib'
import { errorMessage } from '../../src/utils/error-message'

/* tslint:disable:no-console */
describe('Void Pointer: ', () => {
  it('success: A Logger Class can be cast to a C void pointer', () => {
    let count = 0
    const _logFn = (_level: number,
                    _target: string,
                    _message: string,
                    _modulePath: string,
                    _file: string,
                    _line: number) => {
      count = count + 1
      console.log('level: ' + _level)
      console.log('target: ' + _target)
      console.log('message: ' + _message)
      console.log('modulePath: ' + _modulePath)
      console.log('file: ' + _file)
      console.log('line: ' + _line)
    }
    const logFnCb = ffi.Callback('void', ['int', 'string', 'string', 'string', 'string', 'int'], _logFn)
    const logger = new Logger()
    logger.logFn = logFnCb
    const loggerPtr: any = loggerToVoidPtr(logger)

    const level = 123
    const target = 'target'
    const message = 'message'
    const modulePath = 'modulePath'
    const file = 'file'
    const line = 456

    loggerFunction(loggerPtr, level, target, message, modulePath, file, line)
  })
})

describe('Set Logger: ', () => {
  before(() => initRustAPI())

  it('success: sets custom loggger', () => {
    let count = 0
    const _logFn = (level: number, target: string, message: string, modulePath: string, file: string, line: number) => {
      count = count + 1
      console.log('level: ' + level)
      console.log('target: ' + target)
      console.log('message: ' + message)
      console.log('modulePath: ' + modulePath)
      console.log('file: ' + file)
      console.log('line: ' + line)
    }
    assert(count === 0)
    setLogger(_logFn)
    errorMessage(1058)
    assert(count === 2)
  })
})
