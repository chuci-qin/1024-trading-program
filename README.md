# 1024 Trading Program

> **Phase 2: 开仓资金托管机制** - 确保 `PostgreSQL总余额 = 链上USDC` 永远相等

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-1.18.26-blueviolet)](https://solana.com/)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org/)

## 📋 项目概述

1024 Trading Program 是一个部署在 Solana/1024Chain 上的链上程序，用于实现永续合约交易的**USDC保证金托管**。

### 核心目标

```
⭐⭐⭐ 关键约束始终成立:

Main Account.balance_e6 + Σ(All Sub Accounts.balance_e6) = 链上USDC总额

无论何时、任何操作（开仓、平仓、盈亏）都严格相等！
```

### 问题背景

**Phase 1 问题**：
- 用户充值后，链上USDC不变
- 交易盈亏只在PostgreSQL中记录
- PostgreSQL总余额 ≠ 链上USDC
- 提现受限，数据一致性依赖数据库

**Phase 2 解决方案**：
- 开仓时：用户USDC → Program托管账户（链上USDC实时减少）
- 平仓时：Program → 计算PnL → 返还USDC（链上USDC实时增减）
- 结果：PostgreSQL总余额 = 链上USDC ✅

---

## 🎯 核心功能

**Trading Program专注于一件事：USDC托管**

### ✅ Lock Margin（开仓锁定）
- 计算所需保证金（IM = notional / leverage）
- SPL Token Transfer: 用户 → Program托管账户
- 创建/更新 UserPosition PDA
- 更新 TradingVault 全局状态

### ✅ Unlock Margin（平仓返还）
- 计算Realized PnL
- 计算返还金额（释放IM + PnL）
- SPL Token Transfer: Program → 用户
- 更新/删除 UserPosition
- 更新 TradingVault

### ✅ Liquidate（强平）
- 验证保证金率 < 100%
- 计算清算损失和费用
- 分配清算费（50%清算人 + 50%Fee Treasury）
- 剩余资金进Insurance Fund

### ✅ Update Position（更新）
- 更新标记价格
- 计算未实现盈亏
- 更新保证金率
- 更新清算状态

---

## 💡 Smart Hedge在哪里？

**Smart Hedge不在trading-program中实现！**

Smart Hedge是**链下业务逻辑**，应该在：
- **1024-core/crates/smart-hedge-engine**

Smart Hedge如何工作：
1. 监控保证金率（链下）
2. 当110%触发时，调用trading-program的**unlock_margin**（部分平仓）
3. 管理保护池（PostgreSQL，链下）
4. 反向建仓时，调用trading-program的**lock_margin**
5. 不需要特殊的instruction！

**trading-program只提供USDC存取的基础能力！**

---

## 📊 数据结构

### TradingVault（全局状态）

```rust
pub struct TradingVault {
    pub authority: Pubkey,              // Program管理员
    pub total_locked_usdc_e6: i64,      // 总锁定USDC
    pub total_positions: u64,           // 总持仓数
    pub insurance_fund_e6: i64,         // 保险基金
    pub fee_treasury_e6: i64,           // 手续费金库
    // ... 统计数据
}
```

**PDA Seeds**: `[b"trading_vault"]`

### UserPosition（用户持仓）

```rust
pub struct UserPosition {
    pub wallet: Pubkey,                 // 用户钱包
    pub account_id: String,             // 账户ID
    pub market: String,                 // 市场（BTC-PERP）
    pub size_e6: i64,                   // 持仓数量
    pub entry_price_e6: i64,            // 开仓均价
    pub leverage: u32,                  // 杠杆倍数
    pub locked_usdc_e6: i64,            // 锁定保证金
    pub unrealized_pnl_e6: i64,         // 未实现盈亏
    // ... 风控字段
}
```

**PDA Seeds**: `[b"position", user.key(), account_id, market]`

**就这两个！简单明了！**

---

## 🏗️ 项目结构

```
1024-trading-program/
├── Cargo.toml                 # Rust配置
├── README.md                  # 本文档
├── LICENSE                    # MIT License
├── rust-toolchain.toml        # Rust 1.75.0
│
├── src/
│   ├── lib.rs                 # Program入口
│   ├── state.rs               # 数据结构定义
│   ├── instruction.rs         # 指令定义
│   ├── processor.rs           # 指令处理逻辑
│   ├── error.rs               # 错误定义
│   └── utils.rs               # 工具函数
│
├── tests/
│   ├── lock_unlock_tests.rs  # 开平仓测试
│   ├── liquidation_tests.rs  # 强平测试
│   └── smart_hedge_tests.rs  # Smart Hedge测试
│
└── docs/
    ├── 1-项目说明和详细规划.md
    ├── 2-测试套件规划.md
    └── 3-开发与测试进度.md
```

---

## 🚀 快速开始

### 环境要求

- Rust 1.75.0+
- Solana CLI 1.18.26
- Anchor Framework（可选）

### 编译

```bash
# 安装依赖
cargo build-sbf

# 运行测试
cargo test-sbf

# 部署到Devnet
solana program deploy target/deploy/trading_program.so
```

### 部署到1024Chain Testnet

```bash
# 设置RPC
solana config set --url https://testnet-rpc.1024chain.com/rpc/

# 部署
solana program deploy target/deploy/trading_program.so

# 查看Program ID
solana program show <PROGRAM_ID>
```

---

## 📐 使用示例

### 1. 初始化Vault（仅一次）

```rust
// Rust客户端
let ix = trading_program::instruction::TradingInstruction::InitializeVault;

let tx = Transaction::new_signed_with_payer(
    &[ix],
    Some(&authority.pubkey()),
    &[&authority],
    recent_blockhash,
);

client.send_and_confirm_transaction(&tx)?;
```

### 2. 开仓（Lock Margin）

```rust
let ix = TradingInstruction::LockMargin {
    account_id: "test_isolated".to_string(),
    market: "BTC-PERP".to_string(),
    side: Side::Buy,
    size_e6: 1_000_000,              // 0.001 BTC
    entry_price_e6: 101_885_000_000, // $101,885
    leverage: 20,
    margin_mode: MarginMode::Cross,
};

// 发送交易...
```

### 3. 平仓（Unlock Margin）

```rust
let ix = TradingInstruction::UnlockMargin {
    account_id: "test_isolated".to_string(),
    market: "BTC-PERP".to_string(),
    close_size_e6: 500_000,          // 0.0005 BTC (50%平仓)
    exit_price_e6: 102_500_000_000,  // $102,500
};

// 发送交易...
```

---

## 🔧 技术规格

### 约束和限制

```
最大杠杆: 100x
最小保证金: 0.001 USDC
最大单仓USDC: 1,000,000 USDC

保证金率阈值:
  - 警告: 150%
  - Smart Hedge: 110%
  - 强平: 100%

手续费:
  - Maker: 0.015% - 0% (VIP分级)
  - Taker: 0.045% - 0.020% (VIP分级)
  - 强平: 0.5%
  - Smart Hedge: 0.1%
```

### Gas成本预估

| 操作 | 预估成本 (SOL) | 预估成本 (USD @ $200/SOL) |
|------|---------------|---------------------------|
| 初始化Vault | 0.005 | $1.00 |
| 开仓 | 0.0005 | $0.10 |
| 平仓 | 0.0003 | $0.06 |
| 强平 | 0.0008 | $0.16 |

---

## 🧪 测试

### 运行所有测试

```bash
cargo test-sbf
```

### 运行特定测试

```bash
cargo test-sbf lock_margin
cargo test-sbf unlock_margin
cargo test-sbf liquidate
```

### 测试覆盖率

```bash
cargo tarpaulin --out Html
```

---

## 📋 开发进度

### Milestone 1: 基础功能（2周）✅ 已完成

- [x] 项目框架搭建
- [x] 数据结构定义
- [x] Lock Margin实现
- [x] Unlock Margin实现
- [x] Liquidate实现
- [x] 单元测试（22个，100%通过）

### Milestone 2: Smart Hedge（1周）🚧 Phase 2.2

- [ ] Partial Close实现
- [ ] Protection Pool管理
- [ ] Reentry Position实现
- [ ] TP/SL执行
- [ ] Smart Hedge测试

### Milestone 3: 部署和集成（3天）✅ 已完成

- [x] Testnet部署 ✅ **已部署！**
- [x] 1024-core集成架构 ✅
- [ ] 前端集成测试 📋
- [ ] 性能测试 📋

---

## 🎉 部署信息

### 1024Chain Testnet部署

```yaml
Program ID: E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw
Network: 1024Chain Testnet
RPC: https://testnet-rpc.1024chain.com/rpc/
Explorer: https://testnet-scan.1024chain.com/

部署日期: 2025-11-13
部署Slot: 12492844
Program大小: 221,264 bytes (216 KB)
```

### 查看部署

**区块浏览器**:
https://testnet-scan.1024chain.com/address/E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw

**部署交易**:
https://testnet-scan.1024chain.com/tx/3yhgKi33Vm5RUkXJoqggJ9ewp42j3ZsJhWodYyUfvckLvH2pg4SzwTruWaXc4PCDsDosTgpdsiy9pmq1mnePZuJS

---

## 🔗 相关链接

- **GitHub**: https://github.com/chuciqin/1024-trading-program
- **文档**: [docs/1-项目说明和详细规划.md](docs/1-项目说明和详细规划.md)
- **1024Chain Explorer**: https://testnet-scan.1024chain.com/
- **1024Chain RPC**: https://testnet-rpc.1024chain.com/rpc/

---

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md)

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

## 👤 作者

**Chuci Qin**
- GitHub: [@chuciqin](https://github.com/chuciqin)
- Email: xavierqinn@gmail.com

---

## 📝 版本历史

- **v2.0.0** (2025-11-13): Phase 2初始版本，实现开仓资金托管
- **v1.0.0** (2025-11-12): 项目规划和文档

---

**🎯 核心价值**:
- 🔐 完全链上USDC托管，可验证
- 📊 PostgreSQL = 链上USDC（永远相等）
- 🚀 Smart Hedge创新风控
- 💰 低成本（每笔~$0.16）
- 🌐 去中心化、开源

---

**让永续合约交易更安全、更透明！** 🎉
