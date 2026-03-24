import { Command } from 'commander';
import pc from 'picocolors';
import ora from 'ora';
import { createRequire } from 'node:module';
import { runBinary } from './binary.js';
import type { KillResult } from './types.js';

const require = createRequire(import.meta.url);
const { version } = require('../package.json') as { version: string };

const program = new Command();

program
  .name('kill-the-port')
  .description('Kill processes on specified ports - fast, native, cross-platform')
  .version(version, '-v, --version')
  .argument('<ports...>', 'ports to kill (e.g. 3000 8080 3000-3010 3000,3001)')
  .option('-p, --protocol <protocol>', 'protocol to target', 'tcp')
  .option('--graceful', 'send SIGTERM instead of SIGKILL (unix only)')
  .option('--dry-run', 'show what would be killed without killing')
  .option('-j, --json', 'output results as JSON')
  .action(async (ports: string[], opts: { protocol: string; graceful?: boolean; dryRun?: boolean; json?: boolean }) => {
    if (!opts.json) {
      console.log(`\n  ${pc.bold('kill-the-port')} ${pc.dim(`v${version}`)}\n`);
    }

    const args = [...ports, '--protocol', opts.protocol, '--json'];
    if (opts.graceful) args.push('--graceful');
    if (opts.dryRun) args.push('--dry-run');

    const spinner = opts.json ? null : ora('Finding and killing processes…').start();

    const { stdout, exitCode } = await runBinary({ args });

    if (spinner) spinner.stop();

    if (opts.json) {
      console.log(stdout.trim());
    } else {
      let results: KillResult[] = [];
      try {
        results = JSON.parse(stdout) as KillResult[];
      } catch {
        console.log(pc.red('✖') + ' Failed to parse binary output');
        process.exit(1);
      }

      for (const r of results) {
        if (r.error) {
          console.log(`${pc.red('✖')} Port ${pc.bold(String(r.port))}: ${r.error}`);
        } else if (r.pids.length === 0) {
          console.log(`${pc.yellow('⚠')} Port ${pc.bold(String(r.port))}: no process found`);
        } else {
          const action = opts.dryRun ? 'would kill' : 'killed';
          const pidStr = r.pids.join(', ');
          console.log(`${pc.green('✔')} Port ${pc.bold(String(r.port))}: ${action} PID ${pc.dim(pidStr)}`);
        }
      }
    }

    process.exit(exitCode);
  });

program.parse();
