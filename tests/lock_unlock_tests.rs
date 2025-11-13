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

// 集成测试（使用solana-program-test）
#[cfg(test)]
mod integration_tests {
    use super::*;
    use borsh::BorshSerialize;
    use solana_program::{pubkey::Pubkey, system_program};
    use solana_program_test::*;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use trading_program::{
        instruction::TradingInstruction,
        state::{Side, MarginMode, TradingVault, UserPosition},
        processor,
    };
    use spl_token;
    
    /// 辅助函数：创建测试环境
    async fn setup_test_env() -> (
        BanksClient,
        Keypair,
        solana_program::hash::Hash,
        Pubkey,
        Keypair,
        Pubkey,
        Pubkey,
    ) {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "trading_program",
            program_id,
            processor!(processor::process_instruction),
        );
        
        // 添加SPL Token program
        program_test.add_program(
            "spl_token",
            spl_token::id(),
            None,
        );
        
        let (banks_client, payer, recent_blockhash) = program_test.start().await;
        
        // 创建USDC mint和账户
        let usdc_mint = Pubkey::new_unique();
        let vault_usdc = Pubkey::new_unique();
        let user_wallet = Keypair::new();
        let user_usdc = Pubkey::new_unique();
        
        (banks_client, payer, recent_blockhash, program_id, user_wallet, vault_usdc, user_usdc)
    }
    
    #[tokio::test]
    #[ignore] // 需要真实环境或完整的SPL Token setup
    async fn test_initialize_vault() {
        let (mut banks_client, payer, recent_blockhash, program_id, _, vault_usdc, _) = setup_test_env().await;
        
        // 派生Trading Vault PDA
        let (vault_pda, _bump) = Pubkey::find_program_address(
            &[b"trading_vault"],
            &program_id,
        );
        
        // 创建InitializeVault instruction
        let instruction_data = TradingInstruction::InitializeVault;
        let serialized = instruction_data.try_to_vec().unwrap();
        
        let instruction = solana_program::instruction::Instruction {
            program_id,
            accounts: vec![
                solana_program::instruction::AccountMeta::new(vault_pda, false),
                solana_program::instruction::AccountMeta::new(vault_usdc, false),
                solana_program::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_program::instruction::AccountMeta::new_readonly(system_program::id(), false),
                solana_program::instruction::AccountMeta::new_readonly(spl_token::id(), false),
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::sysvar::rent::id(),
                    false
                ),
            ],
            data: serialized,
        };
        
        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        
        // 注意：这个测试需要完整的Token Account设置
        // 在实际部署环境中测试
        println!("✅ Initialize vault test prepared (需要真实环境验证)");
    }
    
    #[tokio::test]
    async fn test_lock_margin_calculation() {
        // 测试保证金计算的准确性
        let size_e6 = 1_000_000; // 0.001 BTC
        let price_e6 = 101_885_000_000; // $101,885
        let leverage = 20;
        
        let im = calculate_initial_margin(size_e6, price_e6, leverage).unwrap();
        
        // 验证：IM = (size × price) / leverage
        assert_eq!(im, 5_094_250_000); // $5.09425
        
        println!("✅ Lock margin calculation test passed");
    }
    
    #[tokio::test]
    async fn test_unlock_margin_pnl_calculation() {
        // 测试平仓盈亏计算
        
        // Long盈利
        let entry = 101_885_000_000;
        let exit = 103_000_000_000;
        let size = 1_000_000;
        
        let pnl = calculate_realized_pnl(true, entry, exit, size).unwrap();
        assert_eq!(pnl, 1_115_000_000); // $1.115
        
        // Long亏损
        let exit_loss = 100_000_000_000;
        let pnl_loss = calculate_realized_pnl(true, entry, exit_loss, size).unwrap();
        assert_eq!(pnl_loss, -1_885_000_000); // -$1.885
        
        // Short盈利
        let pnl_short = calculate_realized_pnl(false, entry, exit_loss, size).unwrap();
        assert_eq!(pnl_short, 1_885_000_000); // $1.885
        
        println!("✅ Unlock margin PnL calculation test passed");
    }
}

