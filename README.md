<div align="center">

<img src="https://img.shields.io/github/actions/workflow/status/Moguifeng-9119/cdoc/ci.yml?branch=master&style=flat-square&label=CI" alt="CI">
<img src="https://img.shields.io/github/v/release/Moguifeng-9119/cdoc?style=flat-square&color=orange" alt="Release">
<img src="https://img.shields.io/github/license/Moguifeng-9119/cdoc?style=flat-square&color=blue" alt="License">
<img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square" alt="Rust">
<img src="https://img.shields.io/github/stars/Moguifeng-9119/cdoc?style=flat-square" alt="Stars">

</div>

# CDoc — CC Doctor

**Claude Code 的家庭医生。** 一键诊断配置、分析会话、检测遗忘。

```bash
cdoc health latest    # 最近会话健康报告
cdoc doctor           # 全面配置诊断
cdoc health watch     # 实时遗忘监控
```

---

## 为什么需要 CDoc？

Claude Code 会话一长就容易"失忆"——压缩上下文后忘了 CLAUDE.md 里的规则，开始胡言乱语。CDoc 自动分析你的会话日志，在问题发生之前告诉你。

| 痛点 | CDoc 怎么解决 |
|------|---------------|
| 上下文太长，Claude 开始忘事 | **7 种遗忘信号**自动检测行为异常 |
| 不知道装了多少规则/技能/代理 | `cdoc stats` 一页总览 |
| 规则交叉引用断裂了不知道 | `cdoc validate rules` 扫描断链 |
| 会话损坏/日志写坏了 | 解析时自动跳过坏行并报告 |

---

## 快速开始

### 安装

```bash
# Cargo
cargo install --git https://github.com/Moguifeng-9119/cdoc

# 或下载预编译二进制
# https://github.com/Moguifeng-9119/cdoc/releases
```

### 一分钟上手

```bash
cdoc stats                        # 看看你装了多少东西
cdoc doctor                       # 全面体检
cdoc health latest                # 最近一次会话健康吗？
cdoc health project . --limit 5   # 最近 5 个会话的批量分析
```

---

## 功能

### Phase 1 — 配置管理

| 命令 | 功能 |
|------|------|
| `cdoc stats` | 一页概览：规则/技能/代理/钩子/会话/历史 |
| `cdoc list rules` | 列出所有规则目录、文件数、交叉引用 |
| `cdoc list skills` | 列出已安装技能及描述 |
| `cdoc list hooks` | 列出 settings.json / hooks.json 中的钩子 |
| `cdoc list agents` | 列出代理定义、工具绑定、模型 |
| `cdoc validate rules` | 校验规则间的 extends / skill 引用完整性 |
| `cdoc validate hooks` | 校验钩子语法、脚本存在性（默认开启） |
| `cdoc doctor` | 全面诊断：空规则、损坏技能、settings 合法性、会话文件统计 |

### Phase 2 — 健康监测

**全部零配置，开箱即用。**

| # | 信号 | 检测逻辑 | 权重 |
|:--:|------|------|:--:|
| 1 | **行为基线一致性** | 自动提取开头模式/语言比例/Markdown 格式，检测会话中偏离 | 25% |
| 2 | **上下文压缩频率** | 检测 input_tokens 断崖下降（>40%） | 20% |
| 3 | **用户纠正频率** | 29 种中英文纠正短语（不对/你忘了/no, I said...） | 20% |
| 4 | **工具错误率** | tool_result.is_error 占比 | 10% |
| 5 | **上下文使用趋势** | peak / 模型上限（自动识别 20+ 模型族） | 10% |
| 6 | **缓存命中率** | cache_read / (read+write) | 5% |
| 7 | **重复指令检测** | Levenshtein 相似度 >85% | 5% |

```bash
cdoc health latest                # 最近会话完整报告
cdoc health session <UUID>        # 指定会话
cdoc health project <DIR>         # 批量分析
cdoc health project <DIR> --limit 5
cdoc health watch                 # 实时监控（仅状态变化时输出）
cdoc health report --format json  # JSON 输出
```

### 示例输出

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

---

## 适配模型族

自动识别模型名称匹配已知上下文上限：

| 1M | 400K | 256K | 200K | 128K |
|-----|------|------|------|------|
| Opus 4.7, Sonnet 4.6, DeepSeek V4, MiniMax M3, Qwen Turbo, MiMo Pro, GPT-5.4 | GPT-5.1-5.3 | Qwen3-235B, Kimi K2, MiMo Flash, GPT-5 | Claude, GLM-4.7, GPT-4o, MiniMax M2.7 | DeepSeek V3, Qwen, GLM, Kimi, LLaMA, Mistral |

---

## 故障排除

<details>
<summary>"Claude config directory not found"</summary>
需要 ~/.claude/ 目录存在。没用过 Claude Code 的话先运行一次 `claude`。
</details>
<details>
<summary>"No sessions found"</summary>
会话文件在 ~/.claude/projects/ 下。目录为空说明还没有过会话。
</details>
<details>
<summary>部分会话分析失败</summary>
用 `cdoc health session <ID>` 单独分析查看错误。坏行会被跳过并显示 "Malformed lines"。
</details>
<details>
<summary>Windows 路径问题</summary>
CDoc 用 `dirs` crate 自动检测。Windows 下配置在 `C:\Users\<user>\.claude\`。
</details>

---

## 技术栈

Rust · clap · serde_json · regex · chrono · walkdir · colored · similar · toml

## 贡献

[CONTRIBUTING.md](CONTRIBUTING.md) — 架构说明 + 开发指南。

[Discussions](https://github.com/Moguifeng-9119/cdoc/discussions) — 用法问题、功能建议。

## License

MIT
