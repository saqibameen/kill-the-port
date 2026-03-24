import { runBinary } from './binary.js';
import type { KillPortOptions, KillPortRangeOptions, KillResult } from './types.js';

export type { KillPortOptions, KillPortRangeOptions, KillResult };

function buildArgs({ protocol = 'tcp', graceful = false, dryRun = false }: {
  protocol?: string;
  graceful?: boolean;
  dryRun?: boolean;
}): string[] {
  const flags = ['--protocol', protocol, '--json'];
  if (graceful) flags.push('--graceful');
  if (dryRun) flags.push('--dry-run');
  return flags;
}

export async function killPort({ port, protocol, graceful, dryRun }: KillPortOptions): Promise<KillResult[]> {
  const ports = Array.isArray(port) ? port : [port];
  const args = [...ports.map(String), ...buildArgs({ protocol, graceful, dryRun })];
  const { stdout } = await runBinary({ args });

  try {
    return JSON.parse(stdout) as KillResult[];
  } catch {
    return ports.map((p) => ({ port: p, pids: [], error: null }));
  }
}

export async function killPortRange({ from, to, protocol, graceful, dryRun }: KillPortRangeOptions): Promise<KillResult[]> {
  const args = [`${from}-${to}`, ...buildArgs({ protocol, graceful, dryRun })];
  const { stdout } = await runBinary({ args });

  try {
    return JSON.parse(stdout) as KillResult[];
  } catch {
    return [];
  }
}
