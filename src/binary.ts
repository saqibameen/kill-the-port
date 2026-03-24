import { execFile } from 'node:child_process';
import { join, dirname } from 'node:path';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const PLATFORM_MAP: Record<string, string> = {
  'darwin-arm64': '@kill-the-port/darwin-arm64',
  'darwin-x64': '@kill-the-port/darwin-x64',
  'linux-x64': '@kill-the-port/linux-x64',
  'linux-arm64': '@kill-the-port/linux-arm64',
  'win32-x64': '@kill-the-port/win32-x64',
};

function getBinaryPath(): string {
  const platform = process.platform;
  const arch = process.arch;
  const key = `${platform}-${arch}`;
  const binName = platform === 'win32' ? 'kill-the-port.exe' : 'kill-the-port';

  // 1. Check for local dev binary (npm/<platform>/bin/)
  const projectRoot = join(__dirname, '..');
  const localBinPath = join(projectRoot, 'npm', key, 'bin', binName);
  if (existsSync(localBinPath)) {
    return localBinPath;
  }

  // 2. Check for installed platform package
  const pkg = PLATFORM_MAP[key];
  if (!pkg) {
    throw new Error(
      `kill-the-port does not support ${platform}-${arch}.\n` +
        `Supported platforms: ${Object.keys(PLATFORM_MAP).join(', ')}`,
    );
  }

  try {
    const pkgDir = join(require.resolve(`${pkg}/package.json`), '..');
    const binPath = join(pkgDir, 'bin', binName);

    if (!existsSync(binPath)) {
      throw new Error(`Binary not found at ${binPath}`);
    }

    return binPath;
  } catch {
    throw new Error(
      `Failed to find the kill-the-port binary for ${platform}-${arch}.\n` +
        `Make sure ${pkg} is installed. Run: npm install ${pkg}`,
    );
  }
}

export function runBinary({ args }: { args: string[] }): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  const binPath = getBinaryPath();

  return new Promise((resolve) => {
    execFile(binPath, args, { timeout: 10000 }, (error, stdout, stderr) => {
      const exitCode = error && 'code' in error ? (error.code as number) ?? 1 : 0;
      resolve({ stdout: stdout.toString(), stderr: stderr.toString(), exitCode });
    });
  });
}
