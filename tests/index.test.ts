import { describe, it, expect } from 'vitest';
import { killPort, killPortRange } from '../src/index.js';
import type { KillResult, KillPortOptions, KillPortRangeOptions } from '../src/types.js';

describe('killPort', () => {
  it('returns empty pids for unused port', async () => {
    const results = await killPort({ port: 19999, dryRun: true });
    expect(results).toHaveLength(1);
    expect(results[0].port).toBe(19999);
    expect(results[0].pids).toEqual([]);
  });

  it('supports multiple ports', async () => {
    const results = await killPort({ port: [19998, 19999], dryRun: true });
    expect(results).toHaveLength(2);
    expect(results[0].port).toBe(19998);
    expect(results[1].port).toBe(19999);
  });

  it('supports udp protocol', async () => {
    const results = await killPort({ port: 19999, protocol: 'udp', dryRun: true });
    expect(results).toHaveLength(1);
    expect(results[0].pids).toEqual([]);
  });
});

describe('killPortRange', () => {
  it('returns results for port range', async () => {
    const results = await killPortRange({ from: 19990, to: 19993, dryRun: true });
    expect(results).toHaveLength(4);
    for (const r of results) {
      expect(r.pids).toEqual([]);
    }
  });
});

describe('types', () => {
  it('KillPortOptions accepts single port', () => {
    const opts: KillPortOptions = { port: 3000 };
    expect(opts.port).toBe(3000);
  });

  it('KillPortOptions accepts port array', () => {
    const opts: KillPortOptions = { port: [3000, 3001] };
    expect(opts.port).toEqual([3000, 3001]);
  });

  it('KillPortRangeOptions has from and to', () => {
    const opts: KillPortRangeOptions = { from: 3000, to: 3010 };
    expect(opts.from).toBe(3000);
    expect(opts.to).toBe(3010);
  });

  it('KillResult shape is correct', () => {
    const result: KillResult = { port: 3000, pids: [1234], error: null };
    expect(result.port).toBe(3000);
    expect(result.pids).toEqual([1234]);
    expect(result.error).toBeNull();
  });
});
