import * as ffi from 'ffi-napi'
import * as ref from 'ref-napi'
import * as Struct from 'ref-struct-di'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'

export const Logger = Struct({
  flushFn: ffi.Function('void', []),
  logFn: ffi.Function('void', ['int', 'string', 'string', 'string', 'string', 'int'])
})

export function loggerToVoidPtr (_logger: any | Struct): Buffer {
  const _pointer = ref.alloc('void *')
  ref.writePointer(_pointer, 0, _logger.ref())
  return _pointer
}

function voidPtrToLogger (loggerPtr: any): any {
  const loggerPtrType = ref.refType(Logger)
  loggerPtr.type = loggerPtrType
  return loggerPtr.deref().deref()
}

const Ilogger = {
  context: ref.refType(ref.refType('void')),
  file: 'string',
  level: 'uint32',
  line: 'uint32',
  message: 'string',
  module_path: 'string',
  target: 'string'
}

function flushFunction (context: any) {
  const _logger = voidPtrToLogger(context)
  _logger.flushFn()
}

export function loggerFunction (context: any,
                                level: number,
                                target: string,
                                message: string,
                                modulePath: string,
                                file: string,
                                line: number) {
  const _logger = voidPtrToLogger(context)
  _logger.logFn(level, target, message, modulePath, file, line)
}

const loggerFnCb = ffi.Callback('void',
[Ilogger.context, Ilogger.level, Ilogger.target, Ilogger.message, Ilogger.module_path, Ilogger.file, Ilogger.line],
  (_context: any,
   _level: number,
   _target: string,
   _message: string,
   _modulePath: string,
   _file: string,
   _line: number) => {
    loggerFunction(_context, _level, _target, _message, _modulePath, _file, _line)
  })

const flushFnCb = ffi.Callback('void', [Ilogger.context], (_context: any) => { flushFunction(_context) })
// need to keep these in this scope so they are not garbage collected.
const logger = Logger()
let pointer

/**
 *
 * Set the Logger to A Custom Logger
 *
 * Example:
 * ```
 * var logFn = (level: number, target: string, message: string, modulePath: string, file: string, line: number) => {
 *   count = count + 1
 *   console.log('level: ' + level)
 *   console.log('target: ' + target)
 *   console.log('message: ' + message)
 *   console.log('modulePath: ' + modulePath)
 *   console.log('file: ' + file)
 *   console.log('line: ' + line)
 * }
 * setLogger(logFn)
 * ```
 *
 */
/* tslint:disable:no-empty */
export function setLogger (userLogFn: any) {
  logger.logFn = userLogFn
  logger.flushFn = () => {}
  pointer = loggerToVoidPtr(logger)
  try {
    rustAPI().vcx_set_logger(pointer, ref.NULL, loggerFnCb, flushFnCb)
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

/**
 * Sets the Logger to Default
 *
 * Accepts a string indicating what level to log at.
 * Example:
 * ```
 * defaultLogger('info')
 * ```
 *
 */

export function defaultLogger (level: string) {
  try {
    rustAPI().vcx_set_default_logger(level)
  } catch (err) {
    throw new VCXInternalError(err)
  }
}
