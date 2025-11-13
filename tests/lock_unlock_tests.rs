//! Lock Margin和Unlock Margin集成测试

use trading_program::utils::*;
use trading_program::state::Side;

#[test]
fn test_calculate_initial_margin() {
    // 测试保证金计算
    // 场景: 0.001 BTC @ $101,885, 20x杠杆
    
    let size_e6 = 1_000_000; // 0.001 BTC
    let price_e6 = 101_885_000_000; // $101,885
    let leverage = 20;
    
    let im = calculate_initial_margin(size_e6, price_e6, leverage).unwrap();
    
    // IM = (size × price) / leverage
    // = (0.001 × 101,885) / 20
    // = 101.885 / 20
    // = $5.09425
    
    assert_eq!(im, 5_094_250_000); // $5,094.25 (e6)
    println!("✅ IM计算正确: {} USDC", im as f64 / 1_000_000.0);
}

#[test]
fn test_calculate_realized_pnl_long_profit() {
    // 测试Long盈利PnL计算
    // 场景: Long 0.0005 BTC, Entry $101,885, Exit $102,500
    
    let entry = 101_885_000_000;
    let exit = 102_500_000_000;
    let size = 500_000; // 0.0005 BTC
    
    let pnl = calculate_realized_pnl(true, entry, exit, size).unwrap();
    
    // PnL = (exit - entry) × size (e6格式)
    // = (615_000_000 × 500_000) / 1_000_000
    // = $307.5 (e6)
    
    assert_eq!(pnl, 307_500_000); // $307.5 (e6)
    println!("✅ Long盈利PnL计算正确: {} USDC", pnl as f64 / 1_000_000.0);
}

#[test]
fn test_calculate_realized_pnl_long_loss() {
    // 测试Long亏损PnL计算
    // 场景: Long 0.0005 BTC, Entry $101,885, Exit $100,000
    
    let entry = 101_885_000_000;
    let exit = 100_000_000_000;
    let size = 500_000; // 0.0005 BTC
    
    let pnl = calculate_realized_pnl(true, entry, exit, size).unwrap();
    
    // PnL = (exit - entry) × size
    // = (-1_885_000_000 × 500_000) / 1_000_000
    // = -$942.5 (e6)
    
    assert_eq!(pnl, -942_500_000); // -$942.5 (e6)
    println!("✅ Long亏损PnL计算正确: {} USDC", pnl as f64 / 1_000_000.0);
}

#[test]
fn test_calculate_realized_pnl_short_profit() {
    // 测试Short盈利PnL计算
    // 场景: Short 0.001 BTC, Entry $102,000, Exit $100,000
    
    let entry = 102_000_000_000;
    let exit = 100_000_000_000;
    let size = 1_000_000; // 0.001 BTC
    
    let pnl = calculate_realized_pnl(false, entry, exit, size).unwrap();
    
    // PnL = (entry - exit) × size
    // = (2_000_000_000 × 1_000_000) / 1_000_000
    // = $2,000 (e6)
    
    assert_eq!(pnl, 2_000_000_000); // $2,000 (e6)
    println!("✅ Short盈利PnL计算正确: {} USDC", pnl as f64 / 1_000_000.0);
}

#[test]
fn test_calculate_liquidation_fee() {
    // 测试清算手续费计算 (0.5%)
    
    let locked_usdc = 5_000_000; // $5
    let fee = calculate_liquidation_fee(locked_usdc);
    
    // Fee = 5 × 0.5% = $0.025
    assert_eq!(fee, 25_000); // $0.025 (e6)
    println!("✅ 清算手续费计算正确: {} USDC", fee as f64 / 1_000_000.0);
}

#[test]
fn test_validate_leverage() {
    // 测试杠杆验证
    
    assert!(validate_leverage(1).is_ok());
    assert!(validate_leverage(20).is_ok());
    assert!(validate_leverage(100).is_ok());
    
    assert!(validate_leverage(0).is_err()); // 0x不合法
    assert!(validate_leverage(101).is_err()); // >100x不合法
    
    println!("✅ 杠杆验证逻辑正确");
}

#[test]
fn test_is_liquidatable() {
    // 测试强平判断
    
    assert!(!is_liquidatable(15000)); // 150% - 不可强平
    assert!(!is_liquidatable(11000)); // 110% - 不可强平
    assert!(!is_liquidatable(10000)); // 100% - 不可强平（刚好）
    assert!(is_liquidatable(9999)); // 99.99% - 可强平 ✅
    assert!(is_liquidatable(5000)); // 50% - 可强平 ✅
    assert!(is_liquidatable(0)); // 0% - 可强平 ✅
    
    println!("✅ 强平判断逻辑正确");
}

// 集成测试（需要solana-program-test环境）
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::{signature::Signer, transaction::Transaction};
    
    #[tokio::test]
    async fn test_lock_margin_basic() {
        // TODO: 实现完整的集成测试
        // 需要设置Solana Program Test环境
    }
}

