# CDoc — CC Doctor 完整路线图

> **项目**: CDoc (CC Doctor — Claude Code Doctor)
> **语言**: Rust (stable 1.96+)
> **二进制**: `cdoc` (~3MB)
> **愿景**: Claude Code 用户的"家庭医生"——管理配置、诊断健康、检测遗忘

---

## Phase 1: 配置管理 ✅ DONE

### 已实现功能

| 命令 | 功能 |
|------|------|
| `cdoc stats` | 一页总览：rules/skills/agents/hooks/sessions 数量统计 |
| `cdoc list rules` | 列出 20 类规则目录及文件数、extends 引用 |
| `cdoc list skills` | 列出 16 个 skill、描述 |
| `cdoc list hooks` | 列出 settings.json 和 hooks.json 中的 hooks |
| `cdoc list agents` | 列出 65 个 agent、model、tools |
| `cdoc validate rules` | 校验 rules 的跨文件引用（extends、See skill、markdown link） |
| `cdoc validate hooks` | 校验 hook 语法、引号平衡、脚本存在性 |
| `cdoc doctor` | 全面诊断：目录结构、空规则、损坏 skill/agent、settings 合法性 |

### 发布前 TODOs
- [x] warnings 消到 1 个 (is_tool_error dead_code)
- [ ] 写 README.md
- [ ] 创建 GitHub repo + push
- [ ] CI/CD (build/test/release)

---

## Phase 2: 健康监测 & 遗忘检测 ✅ DONE

### 已实现功能

| 命令 | 功能 |
|------|------|
| `cdoc health latest` | 分析最近一次会话，输出完整健康报告 |
| `cdoc health session <ID>` | 分析指定会话 |
| `cdoc health project <DIR>` | 批量分析项目下所有会话 |
| `cdoc health project <DIR> --limit N` | 限制分析数量 |
| `cdoc health watch` | 实时轮询监控（10s 间隔） |
| `cdoc health report --format json` | JSON 格式报告 |
| `cdoc health report --output file.txt` | 输出到文件 |

### 7 种遗忘信号

| # | 信号 | 检测逻辑 | 权重 | 零配置 |
|:--:|------|------|:--:|:--:|
| 1 | **行为基线一致性** | 自动从前 10 轮提取开头模式/语言比例/Markdown 格式，检测后续偏离 | 25% | ✅ |
| 2 | **上下文压缩频率** | 检测 input_tokens >60% 断崖下降，>2 次告警 | 20% | ✅ |
| 3 | **用户纠正频率** | 13 种中英文纠正短语正则 | 20% | ✅ |
| 4 | **工具错误率** | tool_result.is_error 占比，>5% 警告 | 10% | ✅ |
| 5 | **上下文使用趋势** | peak tokens / 模型上限（自动识别 DeepSeek 128K / Anthropic 200K） | 10% | ✅ |
| 6 | **缓存命中率** | cache_read / (read+write) | 5% | ✅ |
| 7 | **重复指令检测** | Levenshtein 相似度 >85% 判定重复 | 5% | ✅ |

### 关键设计决策（实际实现）

1. **行为基线自校准**: 不硬编码任何模式。从每会话前 10 轮自动提取开头模式、语言比例、Markdown 使用，检测是否在会话中偏离。零用户配置。
2. **模型自适应**: 上下文上限自动检测——`deepseek` → 128K，其他 → 200K。
3. **压缩检测**: 基于 input_tokens 断崖下降（>60% 且 prev>50000）而非 JSONL 标记，兼容所有提供商格式。
4. **可选自定义金丝雀**: 支持 `~/.claude/cdoc.toml` 中 `[[canary]]` 手动配置额外检测模式。

---

## Phase 3: 扩展记忆 & Canary 配置

### 功能

```
cdoc memory init              # 创建 .cdoc.toml 配置文件
cdoc memory add "..."         # 添加记忆条目
cdoc memory list              # 列出所有记忆
cdoc memory remove <INDEX>    # 删除记忆
cdoc memory inject            # 输出格式化的记忆片段（可粘贴到 CLAUDE.md）
cdoc memory inject --format claude-md  # 以 CLAUDE.md 格式输出
```

### .cdoc.toml 完整格式

```toml
[project]
name = "aperture"
description = "Claude Code 项目"

# 金丝雀规则 — 核心遗忘检测
[[canary]]
name = "曼波问候"
pattern = "曼波~"
type = "per_turn"       # per_turn | behavior | presence
severity = "high"        # high | medium | low
max_miss = 3             # 连续失配 N 轮后告警
description = "确保 Claude 没有遗忘 CLAUDE.md 中的问候指令"

[[canary]]
name = "rust-code-only"
pattern = "cargo|Cargo\.toml|\.rs\b|impl |fn main"
type = "behavior"
severity = "medium"
description = "确保代码输出仍然是 Rust"

# 上下文健康阈值
[context]
min_remaining_pct = 20          # 剩余百分比低于此值告警
max_compactions = 3             # 单会话压缩次数上限
alert_on_compression = true     # 每次压缩都记录

# 记忆片段 — 比 CLAUDE.md 更轻量的上下文注入
[memory]
entries = [
    "用户偏好 Rust，讨厌 Java",
    "项目使用 PostgreSQL，不是 MySQL",
    "API 部署在 fly.io，数据库在 Supabase",
]
tags = ["preferences", "infra", "stack"]

# 信号权重（可自定义）
[signals.weights]
canary = 0.25
compaction = 0.20
corrections = 0.20
error_rate = 0.15
context_usage = 0.10
repeated = 0.05
cache_ratio = 0.05
```

### 技术栈
- `toml 0.8` (已有) — 配置文件解析/序列化
- `serde` (已有) — struct ↔ toml 映射

### 文件清单
```
src/memory/
├── model.rs          # EccConfig, Canary, MemoryEntry, SignalWeights
├── init.rs           # 生成 .cdoc.toml 模板
├── add.rs            # 追加记忆/金丝雀
├── list.rs           # 列出记忆/金丝雀
├── remove.rs         # 删除记忆/金丝雀
└── inject.rs         # 格式化输出（到 stdout，可重定向）
```

---

## Phase 4: 实时监控守护进程

### 功能

```
cdoc watch                    # 启动守护进程，监控所有项目
cdoc watch --daemon           # 后台运行
cdoc watch --notify           # 发现问题时桌面通知
cdoc watch --hook             # 作为 Claude Code hook 运行
```

- 监控 `~/.claude/projects/` 下的 JSONL 文件变化
- 当发现金丝雀丢失或健康恶化时，实时告警
- 可集成到 Claude Code 的 `Stop` hook 中，每次回复后自动检查

### 技术栈
- `notify` crate — 文件系统事件监听（inotify/FSEvents/ReadDirectoryChangesW）
- `notify-rust` crate — 桌面通知
- 或保持 Phase 2 的轮询方案（更简单、跨平台一致）

### 参考
- [notify crate](https://docs.rs/notify/latest/notify/)
- [notify-rust](https://docs.rs/notify-rust/latest/notify_rust/)

---

## Phase 5: 社区 & 发布

### GitHub 仓库

```
cdoc/
├── .github/
│   └── workflows/
│       ├── ci.yml           # build + test + clippy + fmt (ubuntu, macos, windows)
│       └── release.yml      # tag push → 编译三平台二进制 → 上传 Release
├── README.md
├── CHANGELOG.md
├── LICENSE (MIT)
└── CONTRIBUTING.md
```

### 发布渠道

| 渠道 | 操作 |
|------|------|
| **GitHub Releases** | CI 自动编译 Linux/macOS/Windows 二进制 |
| **crates.io** | `cargo publish`，支持 `cargo install cdoc` |
| **Homebrew** | 提交 formula 到 homebrew-core |
| **awesome-claude-code** | 提交 PR 到 [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) |
| **Claude Code 社区** | Hacker News Show HN, Reddit r/ClaudeCode, Twitter/X |

### CI 配置 (ci.yml)

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with: { toolchain: stable }
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
```

### Release 配置 (release.yml)

```yaml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin, aarch64-apple-darwin, x86_64-pc-windows-msvc]
    runs-on: ${{ (contains(matrix.target, 'apple') && 'macos-latest') || (contains(matrix.target, 'windows') && 'windows-latest') || 'ubuntu-latest' }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v1
        with:
          files: target/${{ matrix.target }}/release/cdoc*
```

---

## 总览表

| 阶段 | 内容 | 核心价值 | 时间 | 状态 |
|:--:|------|------|:--:|:--:|
| **P1** | 配置管理 (list/validate/doctor) | 看清楚自己有什么 | 2周 | ✅ DONE |
| **P2** | 健康监测 (7 种遗忘信号) | 自动发现上下文问题 | 2-3周 | 🔲 待做 |
| **P3** | 扩展记忆 (.cdoc.toml + canary) | 用户可配置的防遗忘规则 | 1周 | 🔲 待做 |
| **P4** | 实时监控 (watch/notify/hook) | 运行时自动保护 | 1-2周 | 🔲 待做 |
| **P5** | 社区发布 (CI/Release/推广) | 让更多人用上 | 1周 | 🔲 待做 |

## 关键参考汇总

| 资源 | URL | 对应阶段 |
|------|-----|:--:|
| Claude Code 源码 (泄露) | `github.com/oboard/claude-code-rev` | P1-P4 |
| Claude Code System Prompts | `github.com/Piebald-AI/claude-code-system-prompts` | P2 |
| Rust clap 文档 | `docs.rs/clap/latest/clap/` | P1-P4 |
| Rust serde_json 文档 | `docs.rs/serde_json/latest/serde_json/` | P2 |
| Rust similar 文档 | `docs.rs/similar/latest/similar/` | P2 |
| Rust notify 文档 | `docs.rs/notify/latest/notify/` | P4 |
| awesome-claude-code | `github.com/hesreallyhim/awesome-claude-code` | P5 |
| GitHub Actions 文档 | `docs.github.com/en/actions` | P5 |

---

> 今晚先睡，明天从 Phase 1 残存的 6 个 TODO 开始收尾，然后推 GitHub，接着 Phase 2。
