#!/usr/bin/env node
/**
 * Post-install script: downloads the correct pre-built binary for the current platform.
 */
const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const REPO = 'The-AgenticFlow/AgentFlow';
const BIN_DIR = path.join(__dirname, '..', 'bin');

function detectPlatform() {
    const platform = os.platform();
    const arch = os.arch();

    let osPart, archPart;

    switch (platform) {
        case 'darwin':
            osPart = 'apple-darwin';
            break;
        case 'linux':
            // Check for musl
            try {
                const ldd = execSync('ldd --version 2>&1', { encoding: 'utf8' });
                if (ldd.includes('musl')) {
                    osPart = 'unknown-linux-musl';
                } else {
                    osPart = 'unknown-linux-gnu';
                }
            } catch {
                osPart = 'unknown-linux-gnu';
            }
            break;
        default:
            console.error(`Unsupported platform: ${platform}`);
            process.exit(1);
    }

    switch (arch) {
        case 'x64':
            archPart = 'x86_64';
            break;
        case 'arm64':
            archPart = 'aarch64';
            break;
        default:
            console.error(`Unsupported architecture: ${arch}`);
            process.exit(1);
    }

    return `${archPart}-${osPart}`;
}

function download(url, dest) {
    return new Promise((resolve, reject) => {
        const parsed = new URL(url);
        const client = parsed.protocol === 'https:' ? https : http;
        const file = fs.createWriteStream(dest);

        client.get(url, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                download(response.headers.location, dest).then(resolve).catch(reject);
                return;
            }
            if (response.statusCode !== 200) {
                reject(new Error(`HTTP ${response.statusCode}: ${response.statusMessage}`));
                return;
            }
            response.pipe(file);
            file.on('finish', () => {
                file.close();
                resolve();
            });
        }).on('error', (err) => {
            fs.unlink(dest, () => {});
            reject(err);
        });
    });
}

function extractTarGz(tarPath, destDir) {
    return new Promise((resolve, reject) => {
        const tar = require('child_process').spawn('tar', ['-xzf', tarPath, '-C', destDir, '--strip-components=1']);
        tar.on('close', (code) => {
            if (code === 0) resolve();
            else reject(new Error(`tar exited with code ${code}`));
        });
    });
}

async function main() {
    const platform = detectPlatform();
    console.log(`[@the-agenticflow/openflows] Downloading binary for ${platform}...`);

    // Ensure bin directory exists
    if (!fs.existsSync(BIN_DIR)) {
        fs.mkdirSync(BIN_DIR, { recursive: true });
    }

    // Get latest release tag
    const tag = await new Promise((resolve, reject) => {
        https.get(`https://api.github.com/repos/${REPO}/releases/latest`, {
            headers: { 'User-Agent': 'openflows-npm-installer' }
        }, (res) => {
            let data = '';
            res.on('data', (chunk) => data += chunk);
            res.on('end', () => {
                try {
                    const json = JSON.parse(data);
                    resolve(json.tag_name);
                } catch {
                    reject(new Error('Failed to parse release info'));
                }
            });
        }).on('error', reject);
    });

    const archiveName = `openflows-${tag}-${platform}.tar.gz`;
    const downloadUrl = `https://github.com/${REPO}/releases/download/${tag}/${archiveName}`;
    const tmpFile = path.join(os.tmpdir(), archiveName);

    try {
        await download(downloadUrl, tmpFile);
        await extractTarGz(tmpFile, BIN_DIR);

        // Rename binaries to match expected names
        const binaries = ['agentflow', 'agentflow-setup', 'agentflow-dashboard', 'agentflow-doctor'];
        for (const bin of binaries) {
            const src = path.join(BIN_DIR, bin);
            const dst = path.join(BIN_DIR, `${bin}-bin`);
            if (fs.existsSync(src)) {
                fs.renameSync(src, dst);
                fs.chmodSync(dst, 0o755);
            }
        }

        fs.unlinkSync(tmpFile);
        console.log(`[openflows] Installation complete!`);
    } catch (err) {
        console.error(`[openflows] Failed to download binary: ${err.message}`);
        console.error('[openflows] Falling back to building from source...');

        // Fallback: build from source
        try {
            execSync('cargo build --release -p openflows', { stdio: 'inherit', cwd: path.join(__dirname, '..') });
            const releaseDir = path.join(__dirname, '..', 'target', 'release');
            const binaries = ['openflows', 'openflows-setup', 'openflows-dashboard', 'openflows-doctor'];
            for (const bin of binaries) {
                const src = path.join(releaseDir, bin);
                const dst = path.join(BIN_DIR, `${bin}-bin`);
                if (fs.existsSync(src)) {
                    fs.copyFileSync(src, dst);
                    fs.chmodSync(dst, 0o755);
                }
            }
            console.log('[openflows] Built from source successfully!');
        } catch (buildErr) {
            console.error(`[openflows] Build failed: ${buildErr.message}`);
            console.error('[openflows] Please ensure Rust is installed: https://rustup.rs/');
            process.exit(1);
        }
    }
}

main();
