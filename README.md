# kill-the-port

Kill processes on specified ports — fast, native, cross-platform.

A native Rust binary distributed via npm. No `lsof`, no `netstat`, no shell commands. Uses OS-level APIs directly for maximum speed.

## Why

Existing port killers (`kill-port`, `fkill`) shell out to `lsof`/`netstat` which is slow and fragile. `kill-the-port` uses native APIs:

- **Linux**: Reads `/proc/net/tcp` directly
- **macOS**: Uses `libproc` syscalls
- **Windows**: Calls `GetExtendedTcpTable` Win32 API

## Install

```bash
# Use directly
npx kill-the-port 3000

# Or install globally
npm install -g kill-the-port
```

## CLI Usage

```bash
# Kill single port
kill-the-port 3000

# Kill multiple ports
kill-the-port 3000 8080 9090

# Kill port range
kill-the-port 3000-3010

# Comma-separated
kill-the-port 3000,3001,3002

# Mix and match
kill-the-port 3000 4000-4005 5000,5001

# UDP instead of TCP
kill-the-port 3000 --protocol udp

# Graceful kill (SIGTERM instead of SIGKILL, unix only)
kill-the-port 3000 --graceful

# Dry run — see what would be killed
kill-the-port 3000 --dry-run

# JSON output
kill-the-port 3000 --json
```

## Programmatic API

```typescript
import { killPort, killPortRange } from 'kill-the-port';

// Kill single port
await killPort({ port: 3000 });

// Kill multiple ports
await killPort({ port: [3000, 8080, 9090] });

// Kill port range
await killPortRange({ from: 3000, to: 3010 });

// With options
await killPort({
  port: 3000,
  protocol: 'udp',
  graceful: true,
  dryRun: true,
});
```

## Options

| Option | CLI Flag | Default | Description |
|--------|----------|---------|-------------|
| `protocol` | `-p, --protocol` | `tcp` | Protocol to target (`tcp` or `udp`) |
| `graceful` | `--graceful` | `false` | Send SIGTERM instead of SIGKILL (unix only) |
| `dryRun` | `--dry-run` | `false` | Show what would be killed without killing |
| `json` | `-j, --json` | `false` | Output as JSON (CLI only) |

## How It Works

`kill-the-port` ships precompiled Rust binaries for each platform via npm's `optionalDependencies`. When you install it, npm downloads only the binary for your OS/architecture. No Rust toolchain needed.

The binary uses native OS APIs — no subprocess spawning, no `lsof`, no `netstat`. This makes it significantly faster than alternatives that shell out to system commands.

## Supported Platforms

| Platform | Architecture |
|----------|-------------|
| macOS | ARM64 (Apple Silicon) |
| macOS | x64 (Intel) |
| Linux | x64 |
| Linux | ARM64 |
| Windows | x64 |

## License

MIT
