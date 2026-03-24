export interface KillPortOptions {
  /** Single port or array of ports to kill */
  port: number | number[];
  /** Protocol to target (default: "tcp") */
  protocol?: 'tcp' | 'udp';
  /** Send SIGTERM instead of SIGKILL (default: false, unix only) */
  graceful?: boolean;
  /** Only show what would be killed (default: false) */
  dryRun?: boolean;
}

export interface KillPortRangeOptions {
  /** Start of port range (inclusive) */
  from: number;
  /** End of port range (inclusive) */
  to: number;
  /** Protocol to target (default: "tcp") */
  protocol?: 'tcp' | 'udp';
  /** Send SIGTERM instead of SIGKILL (default: false, unix only) */
  graceful?: boolean;
  /** Only show what would be killed (default: false) */
  dryRun?: boolean;
}

export interface KillResult {
  port: number;
  pids: number[];
  error: string | null;
}
