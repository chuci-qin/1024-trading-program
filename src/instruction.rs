//! Trading Program Instructions

use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::{Side, MarginMode};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TradingInstruction {
    /// 初始化Trading Vault（仅一次，由管理员调用）
    /// 
    /// Accounts:
    /// 0. `[writable]` TradingVault PDA - 将被创建
    /// 1. `[writable]` Vault USDC Account - 托管账户
    /// 2. `[signer, writable]` Authority (管理员) - 支付租金
    /// 3. `[]` System Program
    /// 4. `[]` Token Program
    /// 5. `[]` Rent Sysvar
    InitializeVault,
    
    /// 开仓锁定保证金（Lock Margin）
    /// 
    /// Accounts:
    /// 0. `[writable]` User Position PDA - 将被创建或更新
    /// 1. `[signer]` User - 用户钱包
    /// 2. `[writable]` User USDC Account - 用户USDC账户
    /// 3. `[writable]` Vault USDC Account - 托管USDC账户
    /// 4. `[writable]` Trading Vault - 全局状态
    /// 5. `[]` Token Program
    /// 6. `[]` System Program
    LockMargin {
        account_id: String,         // 账户ID（如 "test_isolated"）
        market: String,             // 市场（如 "BTC-PERP"）
        side: Side,                 // Buy/Sell
        size_e6: i64,               // 数量（e6格式）
        entry_price_e6: i64,        // 开仓价格（e6格式）
        leverage: u32,              // 杠杆倍数
        margin_mode: MarginMode,    // 保证金模式
    },
    
    /// 平仓返还保证金（Unlock Margin）
    /// 
    /// Accounts:
    /// 0. `[writable]` User Position PDA
    /// 1. `[signer]` User - 用户钱包
    /// 2. `[writable]` User USDC Account
    /// 3. `[writable]` Vault USDC Account
    /// 4. `[writable]` Trading Vault
    /// 5. `[]` Token Program
    UnlockMargin {
        account_id: String,
        market: String,
        close_size_e6: i64,         // 平仓数量
        exit_price_e6: i64,         // 平仓价格
    },
    
    /// 强平（Liquidation）
    /// 
    /// Accounts:
    /// 0. `[writable]` User Position PDA - 被强平的持仓
    /// 1. `[]` Position Owner - 持仓所有者
    /// 2. `[signer]` Liquidator - 清算人
    /// 3. `[writable]` Liquidator USDC Account - 清算人收费账户
    /// 4. `[writable]` User USDC Account - 用户账户（返还剩余）
    /// 5. `[writable]` Vault USDC Account
    /// 6. `[writable]` Insurance Fund Account
    /// 7. `[writable]` Fee Treasury Account
    /// 8. `[writable]` Trading Vault
    /// 9. `[]` Token Program
    Liquidate {
        account_id: String,
        market: String,
        liquidation_price_e6: i64,  // 强平价格
    },
    
    /// 更新持仓标记价格和未实现盈亏（链下定期调用）
    /// 
    /// Accounts:
    /// 0. `[writable]` User Position PDA
    /// 1. `[signer]` Authority (Relayer)
    UpdatePosition {
        account_id: String,
        market: String,
        wallet: solana_program::pubkey::Pubkey,
        mark_price_e6: i64,
    },
    
    /// 紧急提现Insurance Fund（仅管理员）
    /// 
    /// Accounts:
    /// 0. `[writable]` Insurance Fund Account
    /// 1. `[writable]` Admin USDC Account
    /// 2. `[signer]` Admin
    /// 3. `[writable]` Trading Vault
    /// 4. `[]` Token Program
    WithdrawInsuranceFund {
        amount_e6: i64,
    },
}

