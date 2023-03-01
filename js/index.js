const path = require('path-browserify')
const { WASI } = require('./vendor/wasi')

const baseNow = Math.floor((Date.now() - performance.now()) * 1e-3)

function hrtime() {
    let clocktime = performance.now() * 1e-3
    let seconds = Math.floor(clocktime) + baseNow
    let nanoseconds = Math.floor((clocktime % 1) * 1e9)
    // return BigInt(seconds) * BigInt(1e9) + BigInt(nanoseconds)
    return seconds * 1e9 + nanoseconds
}

function randomFillSync(buf, offset, size) {
    if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
        // Similar to the implementation of `randomfill` on npm
        let uint = new Uint8Array(buf.buffer, offset, size)
        crypto.getRandomValues(uint)
        return buf
    } else {
        try {
            // Try to load webcrypto in node
            let crypto = require('crypto')
            // TODO: Update to webcrypto in nodejs
            return crypto.randomFillSync(buf, offset, size)
        } catch {
            // If an error occurs, fall back to the least secure version
            // TODO: Should we throw instead since this would be a crazy old browser
            //       or nodejs built without crypto APIs
            if (buf instanceof Uint8Array) {
                for (let i = offset; i < offset + size; i++) {
                    buf[i] = Math.floor(Math.random() * 256)
                }
            }
            return buf
        }
    }
}

const defaultBindings = {
    hrtime: hrtime,
    exit: (code) => {
        if (typeof process !== 'undefined') {
            process.exit(code)
        } else {
            throw new WASIExitError(code)
        }
    },
    kill: (signal) => {
        if (typeof process !== 'undefined') {
            process.kill(process.pid, signal)
        } else {
            throw new WASIKillError(signal)
        }
    },
    randomFillSync: randomFillSync,
    isTTY: () => true,
    path: path,
    fs: null,
}

const defaultPreopens = {
    '.': '.',
}

class CircomspectRunner {
    constructor({ args, env, preopens = defaultPreopens, bindings = defaultBindings } = {}) {
        if (!bindings.fs) {
            throw new Error('You must specify an `fs`-compatible API as part of bindings')
        }
        this.wasi = new WASI({
            args: ['circomspect', ...args],
            env,
            preopens,
            bindings,
        })
    }

    async compile(bufOrResponse) {
        // TODO: Handle ArrayBuffer
        if (bufOrResponse.buffer) {
            return WebAssembly.compile(bufOrResponse)
        }

        // Require Response object if not a TypedArray
        const response = await bufOrResponse
        if (!(response instanceof Response)) {
            throw new Error('Expected TypedArray or Response object')
        }

        const contentType = response.headers.get('Content-Type') || ''

        if ('instantiateStreaming' in WebAssembly && contentType.startsWith('application/wasm')) {
            return WebAssembly.compileStreaming(response)
        }

        const buffer = await response.arrayBuffer()
        return WebAssembly.compile(buffer)
    }

    async execute(bufOrResponse) {
        const mod = await this.compile(bufOrResponse)
        const instance = await WebAssembly.instantiate(mod, {
            ...this.wasi.getImports(mod),
        })

        this.wasi.start(instance)

        // Return the instance in case someone wants to access exports or something
        return instance
    }
}

module.exports.CircomspectRunner = CircomspectRunner
module.exports.preopens = defaultPreopens
module.exports.bindings = defaultBindings
