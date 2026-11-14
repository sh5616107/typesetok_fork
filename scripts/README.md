# TypesetOK Scripts

Utility scripts for development, testing, and deployment.

## Spec Alignment Scripts

### spec-alignment-status.ps1
Check progress of spec alignment project (Phases 1-5).

```powershell
.\scripts\spec-alignment-status.ps1
```

Shows:
- Current branch
- Phase completion (which files exist)
- Test results
- Documentation status
- Quick stats

## Existing Scripts

### bundle-ui.mjs
Bundle the UI packages for Electron.

```bash
node scripts/bundle-ui.mjs
```

### sync-changelog.mjs
Synchronize CHANGELOG.md versions.

```bash
node scripts/sync-changelog.mjs
```

### build-desktop-bundle.mjs
Build complete desktop application bundle.

```bash
node scripts/build-desktop-bundle.mjs
```

### build-installer.mjs
Create Windows installer.

```bash
node scripts/build-installer.mjs
```

### measure-startup.mjs
Measure application startup performance.

```bash
node scripts/measure-startup.mjs
```

### update-contributors.mjs
Update contributor list from git history.

```bash
node scripts/update-contributors.mjs
```

## Development Workflow

### Full Build
```powershell
# 1. Build Rust
cargo build --workspace --release

# 2. Build TypeScript
npm run build

# 3. Run tests
cargo test --workspace
npm test

# 4. Check spec alignment
.\scripts\spec-alignment-status.ps1
```

### Quick Test Cycle
```powershell
# Test specific phase
cargo test --test spec_section_16_1_balance

# Test Rust only
cargo test -p tok-typeset

# Test with output
cargo test -- --nocapture
```

### Release Build
```powershell
# Contributors
npm run contributors

# Build everything
npm run build

# Package desktop app
npm run package:desktop

# Build installer
npm run package:installer
```

## Adding New Scripts

1. Place script in `scripts/` directory
2. Add documentation here
3. Make PowerShell scripts executable:
   ```powershell
   Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
   ```
4. Make Node scripts executable:
   ```bash
   chmod +x scripts/your-script.mjs
   ```

## Script Conventions

- **PowerShell**: Use `.ps1` extension, include help comment
- **Node**: Use `.mjs` extension for ES modules
- **Documentation**: Add summary here and detailed help in script
- **Error Handling**: Use `$ErrorActionPreference = "Stop"` in PowerShell
- **Output**: Use colors for clarity (Green = success, Red = error, Yellow = warning)
