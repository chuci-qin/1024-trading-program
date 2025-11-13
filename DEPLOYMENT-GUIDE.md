# 1024 Trading Program - 部署指南

> **版本**: v2.0.0  
> **网络**: 1024Chain Testnet  
> **日期**: 2025-11-13

---

## 📋 部署前准备

### 1. 环境要求

- Solana CLI 1.18.26+
- Rust 1.75.0+
- 足够的SOL余额（至少0.5 SOL用于部署和初始化）

### 2. 安装Solana CLI

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
```

### 3. 配置RPC

```bash
solana config set --url https://testnet-rpc.1024chain.com/rpc/
```

---

## 🚀 部署步骤

### Step 1: 构建Program

```bash
cd 1024-trading-program
cargo build-sbf
```

**输出**: `target/deploy/trading_program.so`

### Step 2: 部署Program

```bash
# 使用部署脚本（推荐）
./scripts/deploy.sh

# 或手动部署
solana program deploy target/deploy/trading_program.so
```

**输出**: Program ID

### Step 3: 保存Program ID

部署成功后，Program ID会自动保存到：
- `program-id.txt`
- `src/lib.rs`（自动更新declare_id!）

---

## 🏗️ 初始化Trading Vault

### 前置条件

1. **创建USDC Token Accounts**:

```bash
# 设置USDC Mint（1024Chain Testnet）
USDC_MINT=<1024CHAIN_USDC_MINT_ADDRESS>

# 创建Vault USDC Account
spl-token create-account $USDC_MINT

# 创建Insurance Fund Account
spl-token create-account $USDC_MINT

# 创建Fee Treasury Account
spl-token create-account $USDC_MINT
```

### 运行初始化

```bash
# 方法1: 使用示例程序
PROGRAM_ID=$(cat program-id.txt) \
AUTHORITY_KEYPAIR=../../1024-chain/settlement-authority.json \
cargo run --example initialize

# 方法2: 使用初始化脚本
./scripts/initialize-vault.sh
```

---

## ✅ 验证部署

### 1. 检查Program

```bash
PROGRAM_ID=$(cat program-id.txt)
solana program show $PROGRAM_ID
```

### 2. 检查Trading Vault

```bash
# 派生Vault PDA
# 手动计算或使用工具

# 查询Vault账户
solana account <VAULT_PDA>
```

### 3. 测试开仓

```rust
// 使用trading-program-client
let client = TradingProgramClient::new(...);

let sig = client.lock_margin(
    &user_pubkey,
    &user_usdc_account,
    "test_isolated".to_string(),
    "BTC-PERP".to_string(),
    Side::Buy,
    1_000_000, // 0.001 BTC
    101_885_000_000, // $101,885
    20,
    MarginMode::Cross,
).await?;

println!("✅ Position opened: {}", sig);
```

---

## 🔧 配置1024-core集成

### 1. 更新环境变量

```bash
# .env

# Trading Program配置
TRADING_PROGRAM_ENABLED=true
TRADING_PROGRAM_ID=<YOUR_PROGRAM_ID>
TRADING_PROGRAM_AUTHORITY=/path/to/authority-keypair.json

# USDC Accounts
VAULT_USDC_ACCOUNT=<VAULT_USDC_ADDRESS>
INSURANCE_FUND_ACCOUNT=<INSURANCE_FUND_ADDRESS>
FEE_TREASURY_ACCOUNT=<FEE_TREASURY_ADDRESS>
```

### 2. 添加依赖

```toml
# 1024-core/crates/account-domain/Cargo.toml

[dependencies]
# ... 现有依赖
trading-program-client = { path = "../trading-program-client" }
```

### 3. 初始化Service

```rust
// 1024-core启动时

use trading_program_client::TradingProgramClient;

let trading_program_client = if env::var("TRADING_PROGRAM_ENABLED")? == "true" {
    let program_id: Pubkey = env::var("TRADING_PROGRAM_ID")?.parse()?;
    let authority = read_keypair_file(env::var("TRADING_PROGRAM_AUTHORITY")?)?;
    let vault_usdc = env::var("VAULT_USDC_ACCOUNT")?.parse()?;
    let insurance_fund = env::var("INSURANCE_FUND_ACCOUNT")?.parse()?;
    let fee_treasury = env::var("FEE_TREASURY_ACCOUNT")?.parse()?;
    
    Some(Arc::new(TradingProgramClient::new(
        rpc_url,
        program_id,
        authority,
        vault_usdc,
        insurance_fund,
        fee_treasury,
    )))
} else {
    None
};

// 传递给AccountDomainService
```

---

## 🧪 测试部署

### 1. 端到端测试

```bash
cd 1024-core

# 启动backend
cargo run --bin node

# 在另一个终端，发送测试订单
curl -X POST http://localhost:8080/api/orders \
  -H "Content-Type: application/json" \
  -d '{
    "account_id": "test_isolated",
    "market": "BTC-PERP",
    "side": "Buy",
    "size": 0.001,
    "price": 101885,
    "type": "Limit"
  }'
```

### 2. 验证链上状态

```bash
# 查询Position PDA
POSITION_PDA=$(solana-keygen grind --starts-with pos:1)
solana account $POSITION_PDA

# 查询Vault余额
spl-token balance <VAULT_USDC_ACCOUNT>
```

### 3. 验证PostgreSQL

```sql
-- 查询持仓
SELECT * FROM positions WHERE market = 'BTC-PERP';

-- 查询账户余额
SELECT * FROM accounts WHERE id = 'test_isolated';

-- 验证一致性
SELECT 
    SUM(balance_e6) as pg_total,
    -- 链上USDC需要手动查询
FROM accounts WHERE wallet = '<USER_WALLET>';
```

---

## 📊 监控和维护

### 日志查看

```bash
# Program日志
solana logs <PROGRAM_ID>

# 1024-core日志
tail -f /path/to/logs/trading.log
```

### 关键指标

- Lock margin成功率
- Unlock margin成功率
- 平均交易延迟
- Gas成本统计
- 数据一致性检查

---

## ⚠️ 故障排查

### 问题1: 部署失败

**错误**: "Insufficient funds"

**解决**:
```bash
# 检查余额
solana balance

# 申请测试SOL
solana airdrop 1
```

### 问题2: 初始化失败

**错误**: "Account already exists"

**解决**: Vault已初始化，跳过此步骤

### 问题3: Lock margin失败

**错误**: "Insufficient token balance"

**解决**:
```bash
# 给用户铸造测试USDC
spl-token mint $USDC_MINT 1000 $USER_USDC_ACCOUNT
```

---

## 📝 部署清单

### 部署前检查

- [ ] Solana CLI已安装
- [ ] RPC URL正确配置
- [ ] Authority keypair准备好
- [ ] 有足够SOL余额（>0.5 SOL）
- [ ] Program编译成功

### 部署步骤

- [ ] 运行`cargo build-sbf`
- [ ] 运行`./scripts/deploy.sh`
- [ ] 保存Program ID
- [ ] 创建USDC Token Accounts
- [ ] 运行初始化
- [ ] 验证Vault创建成功

### 部署后验证

- [ ] Program可查询
- [ ] Vault PDA创建成功
- [ ] USDC Accounts创建成功
- [ ] 测试开仓成功
- [ ] 测试平仓成功
- [ ] 数据一致性验证通过

---

## 🔗 相关链接

- **1024Chain Explorer**: https://testnet-scan.1024chain.com/
- **1024Chain RPC**: https://testnet-rpc.1024chain.com/rpc/
- **GitHub Repo**: https://github.com/chuciqin/1024-trading-program

---

## 📞 支持

遇到问题？

- GitHub Issues: https://github.com/chuciqin/1024-trading-program/issues
- Email: xavierqinn@gmail.com

---

**最后更新**: 2025-11-13  
**文档版本**: v1.0

