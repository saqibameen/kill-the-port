import { Command } from 'commander';
import pc from 'picocolors';
import ora from 'ora';
import { createRequire } from 'node:module';
import { runBinary } from './binary.js';

const require = createRequire(import.meta.url);
const { version } = require('../package.json') as { version: string };

const program = new Command();

program
  .name('kill-the-port')
  .description('Kill processes on specified ports — fast, native, cross-platform')
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

    const args = [...ports, '--protocol', opts.protocol];
    if (opts.graceful) args.push('--graceful');
    if (opts.dryRun) args.push('--dry-run');
    if (opts.json) args.push('--json');

    const spinner = opts.json ? null : ora('Finding and killing processes…').start();

    const { stdout, stderr, exitCode } = await runBinary({ args });

    if (spinner) spinner.stop();

    if (opts.json) {
      console.log(stdout.trim());
    } else {
      // Parse the output from the binary and colorize it
      const lines = (stdout + stderr).trim().split('\n').filter(Boolean);
      for (const line of lines) {
        if (line.includes('killed PID') || line.includes('would kill PID')) {
          const portMatch = line.match(/Port (\d+)/);
          const pidMatch = line.match(/PID (.+)$/);
          const action = opts.dryRun ? 'would kill' : 'killed';
          if (portMatch && pidMatch) {
            console.log(`${pc.green('✔')} Port ${pc.bold(portMatch[1])}: ${action} PID ${pc.dim(pidMatch[1])}`);
          } else {
            console.log(`${pc.green('✔')} ${line}`);
          }
        } else if (line.includes('no process found')) {
          const portMatch = line.match(/Port (\d+)/);
          if (portMatch) {
            console.log(`${pc.yellow('⚠')} Port ${pc.bold(portMatch[1])}: no process found`);
          } else {
            console.log(`${pc.yellow('⚠')} ${line}`);
          }
        } else if (line.includes('error')) {
          console.log(`${pc.red('✖')} ${line}`);
        } else {
          console.log(line);
        }
      }
    }

    process.exit(exitCode);
  });

program.parse();
