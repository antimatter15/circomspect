#!/usr/bin/env node

const { CircomspectRunner, bindings } = require('./index')
const fs = require('fs')
const path = require('path')

async function main() {
    const args = process.argv
        .slice(2)
        .map((k) => (k.startsWith('-') ? k : path.relative(process.cwd(), k)))
    if (args.length === 0) args.push('--help')
    const circom = new CircomspectRunner({
        args,
        env: process.env,
        preopens: preopensFull(),
        bindings: {
            ...bindings,
            fs,
        },
    })
    const wasm_bytes = fs.readFileSync(require.resolve('./circomspect.wasm'))
    await circom.execute(wasm_bytes)
}

// Enumerate all possible relative parent paths for the preopens.
function preopensFull() {
    const preopens = {}
    let cwd = process.cwd()
    while (1) {
        const seg = path.relative(process.cwd(), cwd) || '.'
        preopens[seg] = seg
        const next = path.dirname(cwd)
        if (next === cwd) break
        cwd = next
    }
    return preopens
}

main().catch((err) => {
    console.error(err)
    process.exit(1)
})
