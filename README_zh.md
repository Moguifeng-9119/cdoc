# cdoc

Claude Code 诊断工具。两件事：审计你的配置，告诉你什么时候会话开始跑偏。

它读取 Claude Code 写到磁盘的 JSONL 会话日志，过一遍健康信号，出报告。不联网，不上报，不调 API。

## 解决什么问题

长会话的质量衰减是可预测的。压缩触发后，上下文被挤压，CLAUDE.md 里的指令从回复中消失。通常第一个丢掉的，就是你写在文件最上面的那条行为规则。

大部分人意识到不对劲的时候，已经跟一个"失忆"的模型浪费了半小时。cdoc 让你提前发现。

## 安装

```bash
cargo install --git https://github.com/Moguifeng-9119/cdoc
```

预编译二进制在 [releases](https://github.com/Moguifeng-9119/cdoc/releases)。

## 用法

```bash
# 看看装了多少东西
cdoc stats
cdoc doctor

# 当前会话健康吗？
cdoc health latest

# 最近几个呢？
cdoc health project . --limit 5

# 盯着
cdoc health watch
```

## 命令

### 配置管理

| 命令 | 做什么 |
|---------|-------------|
| `cdoc stats` | 统计规则、技能、代理、钩子、会话，一屏 |
| `cdoc list rules` | 列出规则目录、文件数、交叉引用 |
| `cdoc list skills` | 列出已安装技能及描述 |
| `cdoc list hooks` | 展示 settings.json 和 hooks.json 中的钩子 |
| `cdoc list agents` | 列出代理定义、工具绑定、模型 |
| `cdoc validate rules` | 检查 extends 和 See skill 引用是否可解析 |
| `cdoc validate hooks` | 校验钩子语法和脚本是否存在 |
| `cdoc doctor` | 全面扫描：空规则目录、损坏技能/代理、settings 合法性、会话统计 |

### 健康监测

每个会话跑 7 个信号。不用配置——工具会读你的 CLAUDE.md，从每个会话的前几轮回复中自动校准基线。

| # | 信号 | 检测内容 |
|:--:|--------|----------------|
| 1 | **行为基线一致性** | 开头模式、语言比例、Markdown 用法在会话中是否漂移 |
| 2 | **压缩频率** | Token 断崖下降（>40%），单会话超 2 次告警 |
| 3 | **用户纠正频率** | "不对""你忘了""no, I said" 等 29 种中英文模式 |
| 4 | **工具错误率** | tool_result.is_error 占比 |
| 5 | **上下文压力** | 峰值 token 相对模型已知上限的比例 |
| 6 | **缓存命中率** | 缓存读取 vs 写入吞吐 |
| 7 | **重复指令** | 用户消息 Levenshtein 距离 >85% |

健康命令：

```bash
cdoc health latest                # 最近一次会话完整报告
cdoc health session <UUID>        # 指定会话
cdoc health project <DIR>         # 批量分析项目的会话
cdoc health project <DIR> --limit 5
cdoc health watch                 # 每 10s 轮询，只在状态变化时输出
cdoc health report --format json  # 机器可读输出
```

## 模型支持

从会话日志中的模型名自动判定上下文窗口上限。未知模型默认 200K。

| 1M | 400K | 256K | 200K | 128K |
|-----|------|------|------|------|
| Opus 4.7, Sonnet 4.6, DeepSeek V4, MiniMax M3, Qwen Turbo, MiMo Pro, GPT-5.4+ | GPT-5.1–5.3 | Qwen3-235B, Kimi K2, MiMo Flash, GPT-5 | Claude (通用), GLM-4.7, GPT-4o, MiniMax M2.7 | DeepSeek V3, Qwen, GLM, Kimi, LLaMA, Mistral |

## 故障排除

**"Claude config directory not found"**

cdoc 需要 `~/.claude/`。没用过 Claude Code 的话先跑一次 `claude`。

**"No sessions found"**

会话文件在 `~/.claude/projects/` 下。空目录说明还没录过会话。

**部分会话分析失败**

用 `cdoc health session <UUID>` 单独看错误。损坏的 JSONL 会被跳过并在输出中标记 "Malformed lines"。

**Windows 路径**

用 `dirs` crate 自动检测。Windows 下配置目录在 `C:\Users\<user>\.claude\`。

## 参与贡献

架构说明和开发环境搭建见 [CONTRIBUTING.md](CONTRIBUTING.md)。

问题和功能讨论在 [GitHub Discussions](https://github.com/Moguifeng-9119/cdoc/discussions)。

## License

MIT
