# 1024 Trading Program - 测试指南

> Phase 2.2: 端到端测试和性能基准测试

---

## 📋 测试套件概览

### 1. 单元测试和集成测试 ✅

**运行所有测试**:
```bash
cargo test
```

**测试覆盖**:
- ✅ 18个单元测试（100%通过）
- ✅ 保证金计算（IM/MM）
- ✅ PnL计算（Long/Short，盈利/亏损）
- ✅ 强平条件判断
- ✅ 强平费用计算
- ✅ 杠杆验证（1-100x）
- ✅ 边界情况测试

**运行特定测试**:
```bash
# 只运行工具函数测试
cargo test --lib

# 只运行lock/unlock测试
cargo test --test lock_unlock_tests

# 只运行强平测试
cargo test --test liquidation_tests
```

---

## 🧪 端到端测试（Phase 2.2）

### 测试脚本: `examples/e2e_test.rs`

**功能**: 在真实的1024Chain Testnet上验证完整的开仓-平仓流程

### 前置要求

1. **准备Testnet环境**:
   - 1024Chain Testnet RPC访问
   - Authority密钥对（已部署Program）
   - 测试用户密钥对
   - 用户USDC余额（用于支付保证金）

2. **创建测试密钥对**:
```bash
# 创建测试用户
solana-keygen new -o test-user.json

# 空投SOL（用于Gas费）
solana airdrop 1 $(solana-keygen pubkey test-user.json) \
  --url https://testnet-rpc.1024chain.com/rpc/

# 创建USDC Token Account（需要先获取Testnet USDC）
```

3. **设置环境变量**:
```bash
export RPC_URL="https://testnet-rpc.1024chain.com/rpc/"
export PROGRAM_ID="E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw"
export AUTHORITY_KEYPAIR="/path/to/authority.json"
export USER_KEYPAIR="/path/to/test-user.json"
export USDC_MINT="<1024Chain Testnet USDC Mint>"
```

### 运行端到端测试

```bash
cargo run --example e2e_test
```

### 测试流程

1. **连接Testnet** - 验证RPC连接
2. **查询初始余额** - 记录用户USDC余额
3. **开仓（Lock Margin）**:
   - 市场: BTC-PERP
   - 数量: 0.001 BTC
   - 价格: $100,000
   - 杠杆: 20x
   - 保证金模式: Cross
4. **查询持仓** - 验证UserPosition PDA创建
5. **平仓（Unlock Margin）**:
   - 平仓价格: $102,000 (+2%盈利)
   - 验证PnL计算
6. **验证持仓删除** - 确认PDA已删除
7. **查询最终余额** - 验证余额变化

### 验证点

- ✅ 链上USDC实时减少（开仓时）
- ✅ 链上USDC实时增加（平仓时）
- ✅ UserPosition PDA正确创建/删除
- ✅ PnL计算准确
- ✅ 延迟 < 2秒

### 故障排查

**问题1**: USDC余额不足
```bash
# 检查余额
spl-token balance --owner test-user.json <USDC_MINT>

# 如果需要，从其他账户转账
spl-token transfer <USDC_MINT> <AMOUNT> <RECIPIENT>
```

**问题2**: Trading Vault未初始化
```bash
# 运行初始化脚本
cargo run --example initialize_vault
```

**问题3**: 网络问题
```bash
# 测试RPC连接
curl -X POST https://testnet-rpc.1024chain.com/rpc/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

---

## 📊 性能基准测试（Phase 2.2）

### 测试脚本: `examples/benchmark.rs`

**功能**: 测试Trading Program的性能指标

### 运行基准测试

**1. 链下计算性能测试**（无需配置）:
```bash
cargo run --example benchmark --release
```

这会测试：
- IM计算性能（100万次迭代）
- PnL计算性能
- 强平判断性能

**2. 链上操作性能测试**（可选）:
```bash
# 设置环境变量
export RPC_URL="https://testnet-rpc.1024chain.com/rpc/"
export PROGRAM_ID="E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw"
export USER_KEYPAIR="/path/to/test-user.json"
export USDC_MINT="<USDC_MINT>"

# 运行（会消耗真实Gas费用！）
cargo run --example benchmark --release
```

### 性能指标

测试报告包含：
- **延迟统计**: 平均、最小、最大、P50、P95、P99
- **吞吐量**: TPS（每秒交易数）
- **成功率**: 成功/失败比例
- **Gas成本**: 每笔交易的SOL成本

### 性能目标

| 指标 | 目标值 | Phase 1 | Phase 2 |
|------|--------|---------|---------|
| 开仓延迟 | < 2秒 | < 100ms | 待测试 |
| 平仓延迟 | < 2秒 | < 100ms | 待测试 |
| 吞吐量 | > 10 TPS | 100+ TPS | 待测试 |
| 成功率 | > 99% | > 99.9% | 待测试 |
| Gas成本 | < $0.20/笔 | $0 | 待测试 |

### 链下计算性能基准

预期结果（Release模式）:
```
IM计算:
  1,000,000 次迭代
  总时间: ~50ms
  平均: ~50ns
  TPS: ~20,000,000

PnL计算:
  1,000,000 次迭代
  总时间: ~60ms
  平均: ~60ns
  TPS: ~16,000,000

强平判断:
  1,000,000 次迭代
  总时间: ~30ms
  平均: ~30ns
  TPS: ~33,000,000
```

---

## 🔍 数据一致性验证

### 验证PostgreSQL vs 链上余额

**目标**: 确保 `PostgreSQL总余额 = 链上USDC` 永远相等

**验证脚本** (TODO):
```bash
# 1. 查询链上总USDC
solana balance <USER_WALLET>

# 2. 查询PostgreSQL总余额
psql -d 1024ex -c "
  SELECT SUM(balance_e6) FROM accounts WHERE wallet = '<USER_WALLET>';
"

# 3. 对比差异
# 应该为0！
```

### 多笔交易一致性测试

**测试场景**:
1. 开仓10笔
2. 部分平仓5笔
3. 完全平仓5笔
4. 验证每步之后的一致性

**验证命令**:
```bash
# TODO: 创建一致性验证脚本
cargo run --example consistency_test
```

---

## 📈 前端集成测试（Phase 2.2）

### 测试范围

**1. UI显示验证**:
- [ ] 余额实时更新
- [ ] 持仓显示正确
- [ ] PnL计算准确
- [ ] 保证金率显示

**2. WebSocket推送测试**:
- [ ] 开仓事件推送
- [ ] 平仓事件推送
- [ ] 余额变化推送
- [ ] 持仓更新推送

**3. 错误处理**:
- [ ] 余额不足提示
- [ ] 网络错误提示
- [ ] 交易失败回滚

### 测试步骤

1. **启动前端**:
```bash
cd 1024-chain-frontend
npm run dev
```

2. **连接Testnet钱包**

3. **执行交易流程**:
   - 开仓
   - 查看持仓
   - 平仓
   - 验证余额变化

4. **检查浏览器控制台**:
   - WebSocket连接状态
   - 事件接收日志
   - API调用日志

---

## 🚨 测试注意事项

### Gas费用

**链上测试会消耗真实Gas费用！**

预估成本（1024Chain Testnet）:
- 开仓: ~0.0005 SOL (~$0.10)
- 平仓: ~0.0003 SOL (~$0.06)
- 强平: ~0.0008 SOL (~$0.16)

**建议**:
1. 使用Testnet而非Mainnet
2. 少量测试（5-10笔）
3. 监控SOL余额

### 数据清理

测试后清理测试数据：
```bash
# TODO: 创建清理脚本
cargo run --example cleanup_test_data
```

---

## 📝 测试报告

### 报告模板

```markdown
# Trading Program 测试报告

## 测试环境
- Network: 1024Chain Testnet
- Program ID: E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw
- 测试日期: YYYY-MM-DD

## 单元测试
- 总数: 18
- 通过: 18
- 失败: 0
- Ignored: 1

## 端到端测试
- 开仓延迟: X.XXs
- 平仓延迟: X.XXs
- 成功率: XX.X%

## 性能测试
- TPS: XX.XX
- P95延迟: X.XXs
- Gas成本: $X.XX

## 数据一致性
- PostgreSQL vs 链上: ✅/❌
- 差异: $X.XX

## 问题和建议
- ...
```

---

## 🎯 Phase 2.2 测试检查清单

### 必须完成（P0）

- [x] 单元测试（18个）✅
- [ ] 端到端测试脚本 📋
- [ ] 性能基准测试脚本 📋
- [ ] 数据一致性验证 📋

### 期望完成（P1）

- [ ] 前端集成测试
- [ ] 压力测试（100+ TPS）
- [ ] 多用户并发测试
- [ ] 完整测试报告

### 可选（P2）

- [ ] 自动化测试CI/CD
- [ ] 测试覆盖率报告
- [ ] 性能回归测试

---

**更新日期**: 2025-11-13  
**负责人**: Chuci Qin

