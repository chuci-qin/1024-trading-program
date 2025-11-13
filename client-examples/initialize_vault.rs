//! 初始化Trading Vault的完整示例
//! 
//! 使用trading-program-client SDK

use anyhow::Result;
use borsh::BorshSerialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

fn main() -> Result<()> {
    println!("🏗️ 初始化1024 Trading Vault");
    println!("══════════════════════════════════════════════════");
    println!();
    
    // 配置
    let rpc_url = "https://testnet-rpc.1024chain.com/rpc/";
    let program_id = Pubkey::from_str("E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw")?;
    let usdc_account = Pubkey::from_str("GaYo5tic9mdV4sp6JmP2DXntWve5Sw6SDnQzcfMcvFxe")?;
    
    println!("📋 配置:");
    println!("  Program ID: {}", program_id);
    println!("  USDC Account: {}", usdc_account);
    println!();
    
    // 读取authority keypair
    let authority = read_keypair_file("../1024-core/settlement-authority-fixed.json")
        .map_err(|e| anyhow::anyhow!("Failed to read keypair: {:?}", e))?;
    println!("  Authority: {}", authority.pubkey());
    println!();
    
    // 创建RPC client
    let rpc_client = RpcClient::new_with_commitment(
        rpc_url.to_string(),
        CommitmentConfig::confirmed(),
    );
    
    // 派生Trading Vault PDA
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(
        &[b"trading_vault"],
        &program_id,
    );
    
    println!("📐 派生的PDAs:");
    println!("  Trading Vault PDA: {}", vault_pda);
    println!();
    
    // 检查Vault是否已初始化
    match rpc_client.get_account(&vault_pda) {
        Ok(account) => {
            if account.lamports > 0 {
                println!("⚠️  Trading Vault已存在！");
                println!("   PDA: {}", vault_pda);
                println!("   Owner: {}", account.owner);
                println!("   Lamports: {}", account.lamports);
                println!("   Data length: {} bytes", account.data.len());
                println!();
                println!("✅ 无需重新初始化");
                return Ok(());
            }
        }
        Err(_) => {
            println!("📋 Trading Vault尚未初始化，继续创建...");
            println!();
        }
    }
    
    // 构造InitializeVault instruction
    use trading_program::instruction::TradingInstruction;
    
    let instruction_data = TradingInstruction::InitializeVault;
    let data = instruction_data.try_to_vec()?;
    
    let accounts = vec![
        AccountMeta::new(vault_pda, false),
        AccountMeta::new(usdc_account, false),
        AccountMeta::new(authority.pubkey(), true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
    ];
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    
    // 创建并发送交易
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );
    
    println!("📤 发送InitializeVault交易...");
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    
    println!();
    println!("✅ Trading Vault初始化成功！");
    println!();
    println!("  Vault PDA: {}", vault_pda);
    println!("  Transaction: {}", signature);
    println!();
    println!("🌐 区块浏览器:");
    println!("  https://testnet-scan.1024chain.com/tx/{}", signature);
    println!();
    
    // 验证
    println!("🔍 验证初始化...");
    let account = rpc_client.get_account(&vault_pda)?;
    println!("  ✅ Vault PDA已创建");
    println!("  Owner: {}", account.owner);
    println!("  Lamports: {}", account.lamports);
    println!("  Data length: {} bytes", account.data.len());
    println!();
    
    println!("════════════════════════════════════════════════");
    println!("🎉 Trading Vault初始化完成！");
    println!("════════════════════════════════════════════════");
    
    Ok(())
}

