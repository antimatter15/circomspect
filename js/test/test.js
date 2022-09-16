const { execSync } = require('child_process')
const path = require('path')
const assert = require('assert')
const fs = require('fs')
const binFileUtils = require('@iden3/binfileutils')
const r1csfile = require('r1csfile')

function circomspect(args) {
    const cmd = path.join('../', require('../package.json')['bin']['circomspect'])
    return execSync(cmd + ' ' + args, {
        cwd: __dirname,
    }).toString('utf-8')
}

const tests = []

function test(name, fn) {
    tests.push({ name, fn })
}

async function run() {
    for (let { name, fn } of tests) {
        const filters = process.argv.slice(2).join(' ').trim()
        if (!name.includes(filters)) {
            console.log('⏭️ ', name)
            continue
        }
        try {
            if (!fs.existsSync(__dirname + '/out')) fs.mkdirSync(__dirname + '/out')
            await fn()
            console.log('✅', name)
        } catch (e) {
            console.log('❌', name)
            console.log(e.stack)
        } finally {
            if (fs.existsSync(__dirname + '/out'))
                fs.rmSync(__dirname + '/out', { recursive: true })
        }
    }
}

test('circomspect command executes', () => {
    const stdout = circomspect('--help')
    assert(stdout.includes('circomspect [OPTIONS] [INPUT]...'), 'missing stdout')
})

test('basic test', () => {
    const stdout = circomspect('basic.circom')
    assert(stdout.includes('No issues found'), 'expected no issues')
})

test('advanced test', () => {
    try {
        circomspect('field_elements_func.circom')
        assert(false, 'expected circomspect to exit with non-zero exit code')
    } catch (err) {
        const stdout = err.stdout.toString('utf-8')
        assert(stdout.includes('The value assigned to `y[0]` is not used to compute the return value.'), 'expected warning')        
    }
})

test('sarif test', () => {
    try {
        circomspect('field_elements_func.circom --sarif-file out/out.sarif')
        assert(false, 'expected circomspect to exit with non-zero exit code')
    } catch (err) {
        const data = JSON.parse(fs.readFileSync(__dirname + '/out/out.sarif', 'utf-8'))
        assert(data.runs[0].results.some(k => k.ruleId === 'CS0008'), 'missing CS0008')
    }
})


run()
