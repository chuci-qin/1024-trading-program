# Devnet部署指南（绕过1024Chain RPC问题）

> **目的**: 在Solana Devnet上测试Trading Program功能  
> **原因**: 1024Chain RPC WebSocket配置问题暂时阻塞部署  
> **状态**: 立即可用

---

## 🚀 快速部署到Devnet

### Step 1: 切换到Devnet

```bash
cd 1024-trading-program

# 配置Devnet RPC
solana config set --url https://api.devnet.solana.com

# 验证配置
solana config get
```

### Step 2: 获取测试SOL

```bash
# 申请测试SOL
solana airdrop 2

# 检查余额
solana balance
```

### Step 3: 部署Program

```bash
# 部署Trading Program
solana program deploy target/deploy/trading_program.so

# 保存返回的Program ID
# 例如: Program Id: HqW7...ABC
```

### Step 4: 保存Program ID

```bash
# 手动保存
echo "HqW7...ABC" > program-id-devnet.txt

# 更新lib.rs
# 手动编辑 src/lib.rs，替换 declare_id!("...")
```

### Step 5: 创建测试USDC

```bash
# 创建USDC Mint (6位精度)
spl-token create-token --decimals 6

# 保存Mint地址
USDC_MINT=<返回的Mint地址>

# 创建Vault USDC Account
spl-token create-account $USDC_MINT
VAULT_USDC=<返回的Account地址>

# 创建Insurance Fund Account
spl-token create-account $USDC_MINT
INSURANCE_FUND=<返回的Account地址>

# 创建Fee Treasury Account  
spl-token create-account $USDC_MINT
FEE_TREASURY=<返回的Account地址>
```

### Step 6: 初始化Trading Vault

```bash
# 设置环境变量
export PROGRAM_ID="<你的Program ID>"
export RPC_URL="https://api.devnet.solana.com"

# 运行初始化（需要修改示例代码中的USDC账户地址）
cargo run --example initialize
```

---

## 🧪 测试Trading Program

### Test 1: 测试LockMargin

```rust
use trading_program_client::TradingProgramClient;
use trading_program::state::{Side, MarginMode};
use solana_sdk::signature::{Keypair, read_keypair_file};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建客户端
    let program_id: Pubkey = "<YOUR_PROGRAM_ID>".parse()?;
    let authority = read_keypair_file("path/to/authority.json")?;
    
    let client = TradingProgramClient::new(
        "https://api.devnet.solana.com".to_string(),
        program_id,
        authority,
        vault_usdc,
        insurance_fund,
        fee_treasury,
    );
    
    // 创建测试用户
    let user = Keypair::new();
    
    // Airdrop SOL
    // ... (使用RpcClient)
    
    // 创建用户USDC账户并mint测试USDC
    // ...
    
    // 测试开仓
    let sig = client.lock_margin(
        &user.pubkey(),
        &user_usdc_account,
        "test_isolated".to_string(),
        "BTC-PERP".to_string(),
        Side::Buy,
        1_000_000,           // 0.001 BTC
        101_885_000_000,     // $101,885
        20,
        MarginMode::Cross,
    ).await?;
    
    println!("✅ Position opened: {}", sig);
    
    // 测试平仓
    let (sig, pnl) = client.unlock_margin(
        &user.pubkey(),
        &user_usdc_account,
        "test_isolated".to_string(),
        "BTC-PERP".to_string(),
        500_000,             // 0.0005 BTC
        102_500_000_000,     // $102,500
    ).await?;
    
    println!("✅ Position closed: signature={}, PnL={}", sig, pnl);
    
    Ok(())
}
```

### Test 2: 验证链上状态

```bash
# 查询Position PDA
POSITION_PDA=$(solana-keygen grind --starts-with pos:1)
solana account $POSITION_PDA --url https://api.devnet.solana.com

# 查询Vault余额
spl-token balance <VAULT_USDC_ACCOUNT> --url https://api.devnet.solana.com

# 查询用户余额变化
spl-token balance <USER_USDC_ACCOUNT> --url https://api.devnet.solana.com
```

---

## 📊 预期结果

### 开仓后

```
用户USDC: 1000 → 994.91 (-5.09 USDC锁定)
Vault USDC: 0 → 5.09
Position PDA: 已创建
  - size: 0.001 BTC
  - locked_usdc: 5.09 USDC
  - entry_price: $101,885
```

### 平仓后（50%）

```
用户USDC: 994.91 → 997.76 (+2.85 USDC返还)
Vault USDC: 5.09 → 2.545
Position PDA: 已更新
  - size: 0.0005 BTC (50%)
  - locked_usdc: 2.545 USDC (50%)
  - realized_pnl: +0.3075 USDC
```

---

## ✅ 验证清单

### Program功能验证

- [ ] InitializeVault成功
- [ ] LockMargin锁定USDC成功
- [ ] Position PDA创建成功
- [ ] UnlockMargin返还USDC成功
- [ ] PnL计算正确
- [ ] Liquidate强平成功（如果触发）

### 数据验证

- [ ] 用户USDC余额 = PostgreSQL余额
- [ ] Vault USDC余额 = 所有Position的locked_usdc总和
- [ ] Event logs正确emit

---

## 🔄 迁移到1024Chain

一旦1024Chain RPC问题解决：

```bash
# 1. 切换回1024Chain
solana config set --url https://testnet-rpc.1024chain.com/rpc/

# 2. 重新部署
solana program deploy target/deploy/trading_program.so

# 3. 更新Program ID配置

# 4. 重新初始化

# 5. 重新测试
```

---

## 📞 获取帮助

**1024Chain RPC问题**:
- 联系1024Chain技术团队
- 说明WebSocket endpoint配置需要修复

**Program测试**:
- 使用Devnet环境（完全兼容）
- 功能验证后再部署到Testnet

---

**创建时间**: 2025-11-13  
**状态**: 准备就绪  
**优先级**: P0（绕过阻塞）

