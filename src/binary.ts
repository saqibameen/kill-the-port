import { execFile } from 'node:child_process';
import { join, dirname } from 'node:path';
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

type SupportedPlatform = 'darwin-arm64' | 'darwin-x64' | 'linux-x64' | 'linux-arm64' | 'win32-x64';

const PLATFORM_MAP: Record<SupportedPlatform, string> = {
  'darwin-arm64': 'kill-the-port-darwin-arm64',
  'darwin-x64': 'kill-the-port-darwin-x64',
  'linux-x64': 'kill-the-port-linux-x64',
  'linux-arm64': 'kill-the-port-linux-arm64',
  'win32-x64': 'kill-the-port-win32-x64',
};

function getBinaryPath(): string {
  const platform = process.platform;
  const arch = process.arch;
  const key = `${platform}-${arch}`;
  const binName = platform === 'win32' ? 'kill-the-port.exe' : 'kill-the-port';

  const pkg = PLATFORM_MAP[key as SupportedPlatform];
  if (!pkg) {
    throw new Error(
      `kill-the-port does not support ${platform}-${arch}.\n` +
        `Supported platforms: ${Object.keys(PLATFORM_MAP).join(', ')}`,
    );
  }

  // 1. Try require.resolve (works when platform pkg is installed as dependency)
  try {
    const pkgDir = join(require.resolve(`${pkg}/package.json`), '..');
    const binPath = join(pkgDir, 'bin', binName);
    if (existsSync(binPath)) {
      return binPath;
    }
  } catch {
    // Not found via require, try other paths
  }

  // 2. Check local dev binary (npm/<platform>/bin/) for development
  const projectRoot = join(__dirname, '..');
  const localBinPath = join(projectRoot, 'npm', key, 'bin', binName);
  if (existsSync(localBinPath)) {
    return localBinPath;
  }

  // 3. Walk up node_modules tree (handles global installs, hoisted deps)
  let dir = __dirname;
  for (let i = 0; i < 5; i++) {
    const candidate = join(dir, 'node_modules', ...pkg.split('/'), 'bin', binName);
    if (existsSync(candidate)) {
      return candidate;
    }
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }

  throw new Error(
    `Failed to find the kill-the-port binary for ${platform}-${arch}.\n` +
      `Make sure ${pkg} is installed. Run: npm install ${pkg}`,
  );
}

let cachedBinaryPath: string | undefined;

export function runBinary({ args }: { args: string[] }): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  if (!cachedBinaryPath) {
    cachedBinaryPath = getBinaryPath();
  }

  return new Promise((resolve) => {
    execFile(cachedBinaryPath!, args, { timeout: 10000 }, (error, stdout, stderr) => {
      const exitCode = error ? (error as NodeJS.ErrnoException & { status?: number }).status ?? 1 : 0;
      resolve({ stdout: stdout.toString(), stderr: stderr.toString(), exitCode });
    });
  });
}
