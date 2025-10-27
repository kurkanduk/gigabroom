# 🧹 Gigabroom

**Sweep away gigabytes of build artifacts** - the ultimate disk space cleaner for developers.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Gigabroom is a fast, interactive CLI tool that finds and removes build artifacts, dependency caches, and temporary files from your development projects. Reclaim tens of gigabytes of disk space with just a few keystrokes!

```
   ____ _             _
  / ___(_) __ _  __ _| |__  _ __ ___   ___  _ __ ___
 | |  _| |/ _` |/ _` | '_ \| '__/ _ \ / _ \| '_ ` _ \
 | |_| | | (_| | (_| | |_) | | | (_) | (_) | | | | | |
  \____|_|\__, |\__,_|_.__/|_|  \___/ \___/|_| |_| |_|
          |___/
                                            v0.1.0
  🧹 Sweep away gigabytes of build artifacts
```

## ✨ Features

- 🚀 **Blazing Fast** - Uses parallel scanning and OS-level indexing (Spotlight on macOS)
- 🎨 **Beautiful TUI** - Interactive menus with visual progress bars and color-coded output
- 🎯 **Smart Detection** - Automatically finds artifacts from 15+ languages and tools
- 📊 **Detailed Reports** - See exactly what will be deleted before confirming
- 🔍 **Powerful Filtering** - Filter by category, size, name, or use custom searches
- 💾 **Smart Caching** - Scan results are cached for instant re-scanning
- ⚡ **Batch Operations** - Select all, select by category, or cherry-pick items
- 🔐 **Safe by Default** - Clear warnings for dangerous operations
- 📦 **Zero Config** - Works out of the box, no setup required

## 🎯 What It Cleans

| Category | Examples | Safe? |
|----------|----------|-------|
| 🦀 **Rust** | \`target/\` | ✅ Safe |
| 📦 **Node.js** | \`node_modules/\` | ✅ Safe |
| 🐍 **Python** | \`__pycache__/\`, \`.venv/\` | ✅ Safe |
| ☕ **Java** | Maven \`target/\`, Gradle \`build/\` | ✅ Safe |
| 🐘 **PHP** | \`vendor/\` | ✅ Safe |
| 💎 **Ruby** | \`vendor/bundle/\` | ✅ Safe |
| 🐹 **Go** | \`vendor/\` | ✅ Safe |
| ⚙️ **C/C++** | Build artifacts | ✅ Safe |
| 🔷 **.NET** | \`bin/\`, \`obj/\`, \`packages/\` | ✅ Safe |
| 🦢 **Swift** | \`.build/\`, \`DerivedData/\` | ✅ Safe |
| 💡 **IDE** | \`.idea/\`, \`.vscode/\`, \`.vs/\` | ✅ Safe |
| 🗑️ **OS Junk** | \`.DS_Store\`, \`Thumbs.db\` | ✅ Safe |
| 📝 **Temp Files** | \`*.log\`, \`*.tmp\` | ✅ Safe |
| 📁 **Build** | \`build/\`, \`dist/\`, \`out/\` | ✅ Safe |
| ⚠️ **Package Caches** | npm, pip, Maven global caches | ⚠️ Dangerous |

## 📦 Installation

### Via Cargo (Recommended)

\`\`\`bash
cargo install gigabroom
\`\`\`

### From Source

\`\`\`bash
git clone https://github.com/kurkanduk/gigabroom
cd gigabroom
cargo install --path .
\`\`\`

### Binary Releases

Download pre-built binaries from the [releases page](https://github.com/kurkanduk/gigabroom/releases).

## 🚀 Quick Start

### Interactive Mode (Recommended)

Simply run gigabroom to launch the interactive menu:

\`\`\`bash
gigabroom
\`\`\`

Navigate with arrow keys, select with Space, confirm with Enter!

### Command-Line Mode

Scan the current directory:

\`\`\`bash
gigabroom scan
\`\`\`

Scan a specific directory with custom depth:

\`\`\`bash
gigabroom scan ~/projects --max-depth 5
\`\`\`

Clean specific categories:

\`\`\`bash
gigabroom clean --category rust node python
\`\`\`

Clean everything without confirmation (be careful!):

\`\`\`bash
gigabroom clean --all --yes
\`\`\`

Dry run (preview without deleting):

\`\`\`bash
gigabroom clean --all --dry-run
\`\`\`

## ⌨️ Keyboard Shortcuts

In interactive mode:

- \`↑\` / \`↓\` - Navigate items
- \`Space\` - Select/deselect current item
- \`Enter\` - Confirm selection
- \`Esc\` - Cancel/go back
- \`PgUp\` / \`PgDn\` - Quick navigation

## 🔧 Advanced Usage

### Cache Management

Cache is stored at \`~/.gigabroom-cache.json\` (valid for 5 minutes)

Clear cache:
\`\`\`bash
gigabroom cache clear
\`\`\`

View cache info:
\`\`\`bash
gigabroom cache info
\`\`\`

### Performance Tips

1. **Use Spotlight on macOS** - Much faster than filesystem walk:
   \`\`\`bash
   gigabroom scan --index
   \`\`\`

2. **Limit scan depth** - Faster scans for shallow projects:
   \`\`\`bash
   gigabroom scan --max-depth 3
   \`\`\`

3. **Use size filters** - Skip small files:
   \`\`\`bash
   gigabroom scan --min-size 10MB
   \`\`\`

### JSON Output

For scripting and automation:

\`\`\`bash
gigabroom scan --json > results.json
\`\`\`

## 🛡️ Safety Features

### What's Safe to Delete?

✅ **Always Safe:**
- Project build outputs (\`target/\`, \`build/\`, \`dist/\`)
- Project dependencies (\`node_modules/\`, \`vendor/\`)
- IDE caches (\`.idea/\`, \`.vscode/\`)
- Temporary files (\`*.log\`, \`*.tmp\`)

⚠️ **Use Caution:**
- Package manager global caches (requires re-download for all projects)

### Built-in Protections

1. **Confirmation Prompts** - Always asks before deleting
2. **Dry Run Mode** - Preview deletions with \`--dry-run\`
3. **Detailed Summaries** - See exactly what will be removed
4. **Warning Labels** - Dangerous operations are clearly marked
5. **No System Files** - Only targets known build artifacts

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [dialoguer](https://github.com/console-rs/dialoguer) - Interactive prompts
- [colored](https://github.com/colored-rs/colored) - Terminal colors
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars
- [walkdir](https://github.com/BurntSushi/walkdir) - Filesystem traversal
- [rayon](https://github.com/rayon-rs/rayon) - Parallel processing

---

**Made with ❤️ and 🦀 Rust**

Star ⭐ this repo if gigabroom helped you reclaim disk space!
