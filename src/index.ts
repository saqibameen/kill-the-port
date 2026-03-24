import { runBinary } from './binary.js';
import type { KillPortOptions, KillPortRangeOptions, KillResult } from './types.js';

export type { KillPortOptions, KillPortRangeOptions, KillResult };

/**
 * Kill processes running on the specified port(s).
 */
export async function killPort({ port, protocol = 'tcp', graceful = false, dryRun = false }: KillPortOptions): Promise<KillResult[]> {
  const ports = Array.isArray(port) ? port : [port];
  const args = ports.map(String);

  args.push('--protocol', protocol);
  if (graceful) args.push('--graceful');
  if (dryRun) args.push('--dry-run');
  args.push('--json');

  const { stdout } = await runBinary({ args });

  try {
    return JSON.parse(stdout) as KillResult[];
  } catch {
    return ports.map((p) => ({ port: p, pids: [], error: null }));
  }
}

/**
 * Kill processes running on a range of ports.
 */
export async function killPortRange({ from, to, protocol = 'tcp', graceful = false, dryRun = false }: KillPortRangeOptions): Promise<KillResult[]> {
  const args = [`${from}-${to}`];

  args.push('--protocol', protocol);
  if (graceful) args.push('--graceful');
  if (dryRun) args.push('--dry-run');
  args.push('--json');

  const { stdout } = await runBinary({ args });

  try {
    return JSON.parse(stdout) as KillResult[];
  } catch {
    return [];
  }
}
