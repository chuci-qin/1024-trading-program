# 1024 Trading Program - 快速开始

> **5分钟快速了解和部署**

---

## 🎯 这是什么？

**1024 Trading Program** 是一个 Solana/1024Chain 上的智能合约，用于实现永续合约交易的**链上USDC保证金托管**。

### 核心价值

```
解决问题:
  ❌ Phase 1: 链上USDC不变，盈亏只在数据库

实现方案:
  ✅ Phase 2: 开仓锁定USDC → Program托管
  ✅ 平仓返还USDC + 盈亏
  ✅ PostgreSQL总额 = 链上USDC (永远相等！)
```

---

## 🚀 快速部署

### 1. 构建

```bash
cd 1024-trading-program
cargo build-sbf
```

### 2. 部署

```bash
./scripts/deploy.sh
```

### 3. 初始化

```bash
cargo run --example initialize
```

---

## 💻 快速使用

### Rust客户端

```rust
use trading_program_client::TradingProgramClient;
use trading_program::state::Side;

// 创建客户端
let client = TradingProgramClient::new(...);

// 开仓
let sig = client.lock_margin(
    &user,
    &user_usdc,
    "test_isolated".to_string(),
    "BTC-PERP".to_string(),
    Side::Buy,
    1_000_000,           // 0.001 BTC
    101_885_000_000,     // $101,885
    20,                  // 20x
    MarginMode::Cross,
).await?;

// 平仓
let (sig, pnl) = client.unlock_margin(
    &user,
    &user_usdc,
    "test_isolated".to_string(),
    "BTC-PERP".to_string(),
    500_000,             // 0.0005 BTC
    102_500_000_000,     // $102,500
).await?;
```

---

## 🧪 快速测试

```bash
# 运行所有测试
cargo test

# 预期结果: 22 passed ✅
```

---

## 📚 文档导航

### 开发者

- [README.md](README.md) - 完整项目说明
- [DEPLOYMENT-GUIDE.md](DEPLOYMENT-GUIDE.md) - 部署指南
- [docs/1024-CORE-INTEGRATION.md](docs/1024-CORE-INTEGRATION.md) - 集成指南

### 业务和规划

- [1-项目说明和详细规划.md](../docs-by-features/orders-and-trades-and-close/开仓资金托管/1-项目说明和详细规划.md)
- [2-测试套件规划.md](../docs-by-features/orders-and-trades-and-close/开仓资金托管/2-测试套件规划.md)
- [3-开发与测试进度.md](../docs-by-features/orders-and-trades-and-close/开仓资金托管/3-开发与测试进度.md)

### 完成报告

- [PHASE-2-1-COMPLETE.md](PHASE-2-1-COMPLETE.md) - 阶段完成报告
- [COMPLETION-SUMMARY.md](COMPLETION-SUMMARY.md) - 完成总结
- [PROJECT-STATUS.md](PROJECT-STATUS.md) - 项目状态

---

## 🎯 关键数字

- **代码**: 1,712行
- **测试**: 22个（100%通过）
- **文档**: 11个
- **完成度**: 85%
- **工期**: 1天（MVP）

---

## 📞 获取帮助

- **文档**: 查看 [README.md](README.md)
- **Issues**: GitHub Issues
- **Email**: xavierqinn@gmail.com

---

**快速开始完毕！查看完整文档了解更多。** 🎉

