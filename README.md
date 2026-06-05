# CDoc — CC Doctor

Claude Code 健康诊断工具。管理配置、分析会话、检测遗忘。

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 安装

```bash
cargo install --git https://github.com/Moguifeng-9119/cdoc
```

或从 [Releases](https://github.com/Moguifeng-9119/cdoc/releases) 下载预编译二进制。

## 快速开始

```bash
cdoc stats              # 一页总览：规则/技能/代理/会话数量
cdoc doctor             # 全面诊断：目录结构、损坏文件、配置合法性
cdoc health latest      # 最近会话健康报告（7 种遗忘信号）
cdoc health project .   # 分析当前项目所有历史会话
cdoc health watch       # 实时监控（10s 轮询）
```

## Phase 1 — 配置管理

| 命令 | 功能 |
|------|------|
| `cdoc stats` | 一页总览 rules/skills/agents/hooks/sessions 数量统计 |
| `cdoc list rules` | 列出规则目录、文件数、交叉引用 |
| `cdoc list skills` | 列出已安装技能 |
| `cdoc list hooks` | 列出 settings.json 和 hooks.json 中的钩子 |
| `cdoc list agents` | 列出代理定义、工具、模型 |
| `cdoc validate rules` | 校验规则的交叉引用完整性 |
| `cdoc validate hooks` | 校验钩子语法、脚本存在性 |
| `cdoc doctor` | 全面诊断：空规则、损坏技能/代理、settings 合法性 |

## Phase 2 — 健康监测

自动分析 JSONL 会话日志，7 种遗忘信号**全部零配置**：

| # | 信号 | 检测逻辑 | 权重 |
|:--:|------|------|:--:|
| 1 | **行为基线一致性** | 从前几轮自动提取开头模式/语言比例/Markdown 格式，检测后续偏离 | 25% |
| 2 | **上下文压缩频率** | 检测 input_tokens 断崖式下降（>40%），>2 次告警 | 20% |
| 3 | **用户纠正频率** | 中英文纠正短语：`不对/你忘了/no, I said` | 20% |
| 4 | **工具错误率** | `tool_result.is_error` 占比，>5% 警告 | 10% |
| 5 | **上下文使用趋势** | 自动识别模型上限（支持 Claude/DeepSeek/Qwen/GLM/MiniMax/MiMo/Kimi/GPT） | 10% |
| 6 | **缓存命中率** | cache_read / (read+write) | 5% |
| 7 | **重复指令检测** | 用户消息 Levenshtein 相似度 >85% | 5% |

### 命令

```bash
cdoc health latest                  # 最近会话完整报告
cdoc health session <UUID>          # 指定会话
cdoc health project <DIR>           # 项目批量分析
cdoc health project <DIR> --limit 5 # 限制数量
cdoc health watch                   # 实时轮询监控
cdoc health watch --interval 30     # 自定义轮询间隔
cdoc health report --format json    # JSON 输出
cdoc health report -o report.txt    # 输出到文件
```

### 输出示例

```
🔍  Session Health: 1543aad3-...
   ──────────────────────────────────────────────────
   Duration: 1h 52m  |  Messages: 445  |  Tools: 150  |  Compactions: 1
   Tokens: 502K in / 185K out  |  Peak: 72K  |  Model: deepseek-v4-pro

  Signals
  🟢 行为基线一致性 [1.00] 行为基线正常，未检测到偏离
  🟢 上下文压缩频率 [0.80] 检测到 1 次压缩，属正常范围
  🟢 用户纠正频率 [0.80] 1 次纠正 / 289 轮对话 = 0.3%
  🟡 工具错误率 [0.60] 15/150 (10.0%) 错误率偏高
  🟢 上下文使用趋势 [1.00] 峰值 72K / 1000K (7%) 充足

  🟢 Overall: HEALTHY (0.85)
```

## 适配的模型上下文

自动识别模型名称匹配已知上限（1M / 400K / 256K / 200K / 128K）：

Claude (Opus/Sonnet/Haiku)、DeepSeek (V3/V4/R1)、Qwen (2.5/3)、GLM (4/4.5/4.6/4.7)、MiniMax (M1/M2/M3)、MiMo (V2 Flash/Pro)、Kimi (K2)、GPT (4o/5/5.x)、Gemini、LLaMA、Mistral

## 可选：自定义金丝雀

`~/.claude/cdoc.toml`（不配置也完全可用）：

```toml
[[canary]]
name = "你的指令锚点"
pattern = "某个正则模式"
max_miss = 5
```

## 故障排除

### "Claude config directory not found"

CDoc 需要 `~/.claude/` 目录存在。如果尚未使用过 Claude Code，先运行一次 `claude` 初始化。

### "No sessions found"

会话文件在 `~/.claude/projects/` 下。如果该目录为空，说明还没有过 Claude Code 会话。

### 部分会话分析失败

`cdoc health project` 会跳过损坏的会话文件并显示 `✗`。用 `cdoc health session <ID>` 单独分析可查看具体错误。

### 会话日志损坏

如果看到 "Malformed lines: N" 警告，说明会话 JSONL 文件有损坏行。CDoc 会自动跳过坏行继续分析，通常不影响结果。严重时可删除对应的 `.jsonl` 文件（会丢失该会话的统计）。

### Windows 路径问题

CDoc 使用 `dirs` crate 自动检测系统路径。Windows 下 Claude Code 配置通常在 `C:\Users\<user>\.claude\`，CDoc 会自动定位。

## 技术栈

Rust · clap · serde_json · regex · chrono · walkdir · colored · similar · toml

## License

MIT
