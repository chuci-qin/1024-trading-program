//! Trading Program State Definitions
//! 
//! 定义Trading Program的核心数据结构

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{pubkey::Pubkey, sysvar::Sysvar};

/// Side枚举：Buy或Sell
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// 保证金模式
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarginMode {
    Cross,      // 全仓
    Isolated,   // 逐仓
}

/// 清算状态
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidationStatus {
    Normal,                 // 正常
    Warning,                // 警告（150%）
    SmartHedgeTriggered,    // Smart Hedge已触发（110%）
    Liquidatable,           // 可强平（100%）
}

/// Trading Vault（全局状态，单例PDA）
/// PDA Seeds: [b"trading_vault"]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TradingVault {
    /// 账户类型标识符 "TRADEVAL" = 0x54524144_4556414c
    pub discriminator: u64,
    
    /// 数据版本
    pub version: u8,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// 预留字段（对齐）
    pub reserved_align: [u8; 6],
    
    /// Program管理员（Relayer）
    pub authority: Pubkey,
    
    /// 总锁定USDC（e6格式）
    pub total_locked_usdc_e6: i64,
    
    /// 总持仓数
    pub total_positions: u64,
    
    /// Insurance Fund总额（e6格式）
    pub insurance_fund_e6: i64,
    
    /// Fee Treasury总额（e6格式）
    pub fee_treasury_e6: i64,
    
    /// 创建时间戳（秒）
    pub created_at: i64,
    
    /// 更新时间戳（秒）
    pub updated_at: i64,
    
    /// 统计数据
    pub total_trades: u64,              // 总交易次数
    pub total_liquidations: u64,        // 总强平次数
    pub total_smart_hedges: u64,        // 总Smart Hedge次数
    
    /// 累计数据（e6格式）
    pub cumulative_volume_e6: i64,      // 累计交易量
    pub cumulative_fees_e6: i64,        // 累计手续费
    pub cumulative_pnl_e6: i64,         // 累计已实现盈亏
    
    /// 预留扩展字段
    pub reserved: [u8; 128],
}

impl TradingVault {
    pub const DISCRIMINATOR: u64 = 0x54524144_4556414c;
    pub const VERSION: u8 = 1;
    
    /// 8 + 1 + 1 + 6 + 32 + 8*7 + 8*3 + 128 = 255 bytes
    pub const SIZE: usize = 255;
    
    pub fn new(authority: Pubkey, bump: u8) -> Self {
        let now = solana_program::clock::Clock::get()
            .map(|clock| clock.unix_timestamp)
            .unwrap_or(0);
        
        Self {
            discriminator: Self::DISCRIMINATOR,
            version: Self::VERSION,
            bump,
            reserved_align: [0; 6],
            authority,
            total_locked_usdc_e6: 0,
            total_positions: 0,
            insurance_fund_e6: 0,
            fee_treasury_e6: 0,
            created_at: now,
            updated_at: now,
            total_trades: 0,
            total_liquidations: 0,
            total_smart_hedges: 0,
            cumulative_volume_e6: 0,
            cumulative_fees_e6: 0,
            cumulative_pnl_e6: 0,
            reserved: [0; 128],
        }
    }
}

/// User Position（每个用户每个市场一个PDA）
/// PDA Seeds: [b"position", user.key().as_ref(), account_id.as_bytes(), market.as_bytes()]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserPosition {
    /// 账户类型标识符 "USERPOSN" = 0x55534552_504f534e
    pub discriminator: u64,
    
    /// 数据版本
    pub version: u8,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// 预留字段（对齐）
    pub reserved_align: [u8; 6],
    
    // === 用户标识 ===
    pub wallet: Pubkey,                 // 用户钱包
    pub account_id: String,             // 账户ID（如 "test_isolated"）最大64字节
    pub market: String,                 // 市场（如 "BTC-PERP"）最大32字节
    
    // === 持仓信息 ===
    pub side: Side,                     // Buy/Sell
    pub size_e6: i64,                   // 持仓数量（e6格式，带符号）
    pub entry_price_e6: i64,            // 开仓均价（e6格式）
    pub mark_price_e6: i64,             // 标记价格（e6格式）
    pub leverage: u32,                  // 杠杆倍数
    pub margin_mode: MarginMode,        // 保证金模式
    
    // === 保证金 ===
    pub locked_usdc_e6: i64,            // 锁定的USDC（初始保证金IM）
    pub mm_e6: i64,                     // 维持保证金（MM）
    
    // === 盈亏 ===
    pub unrealized_pnl_e6: i64,         // 未实现盈亏
    pub realized_pnl_e6: i64,           // 已实现盈亏（累计）
    
    // === 风控 ===
    pub liquidation_price_e6: i64,      // 强平价格
    pub margin_ratio_bp: u32,           // 保证金率（基点，10000=100%）
    pub liquidation_status: LiquidationStatus,
    
    // === 时间戳 ===
    pub opened_at: i64,                 // 开仓时间（秒）
    pub updated_at: i64,                // 更新时间（秒）
    
    // === 预留扩展字段 ===
    pub reserved: [u8; 96],  // 增加预留空间（移除了TP/SL和Smart Hedge字段）
}

impl UserPosition {
    pub const DISCRIMINATOR: u64 = 0x55534552_504f534e;
    pub const VERSION: u8 = 1;
    
    /// 8 + 1 + 1 + 6 + 32 + (64+32) + 8*7 + 4*2 + 1 + 8*4 + 8*2 + 1 + 1 + 64 = ~350 bytes
    pub const MAX_SIZE: usize = 400;
    
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wallet: Pubkey,
        account_id: String,
        market: String,
        side: Side,
        size_e6: i64,
        entry_price_e6: i64,
        leverage: u32,
        margin_mode: MarginMode,
        locked_usdc_e6: i64,
        mm_e6: i64,
        bump: u8,
    ) -> Self {
        let now = solana_program::clock::Clock::get()
            .map(|clock| clock.unix_timestamp)
            .unwrap_or(0);
        
        // 计算强平价格
        let liquidation_price_e6 = Self::calculate_liquidation_price(
            entry_price_e6,
            side,
            leverage,
        );
        
        Self {
            discriminator: Self::DISCRIMINATOR,
            version: Self::VERSION,
            bump,
            reserved_align: [0; 6],
            wallet,
            account_id,
            market,
            side,
            size_e6,
            entry_price_e6,
            mark_price_e6: entry_price_e6,
            leverage,
            margin_mode,
            locked_usdc_e6,
            mm_e6,
            unrealized_pnl_e6: 0,
            realized_pnl_e6: 0,
            liquidation_price_e6,
            margin_ratio_bp: 20000, // 200%（初始）
            liquidation_status: LiquidationStatus::Normal,
            opened_at: now,
            updated_at: now,
            reserved: [0; 96],
        }
    }
    
    /// 计算强平价格（简化版）
    fn calculate_liquidation_price(
        entry_price_e6: i64,
        side: Side,
        leverage: u32,
    ) -> i64 {
        // 强平价格 = 入场价 × (1 ± 1/杠杆)
        // Long: liquidation = entry × (1 - 1/leverage)
        // Short: liquidation = entry × (1 + 1/leverage)
        
        let factor = 1_000_000 / leverage as i64; // 1/leverage (e6)
        
        match side {
            Side::Buy => {
                // Long: 价格下跌触发强平
                entry_price_e6 - (entry_price_e6 * factor / 1_000_000)
            }
            Side::Sell => {
                // Short: 价格上涨触发强平
                entry_price_e6 + (entry_price_e6 * factor / 1_000_000)
            }
        }
    }
    
    /// 计算未实现盈亏
    pub fn calculate_unrealized_pnl(&self, current_mark_price_e6: i64) -> i64 {
        let price_diff = match self.side {
            Side::Buy => current_mark_price_e6 - self.entry_price_e6,
            Side::Sell => self.entry_price_e6 - current_mark_price_e6,
        };
        
        // PnL = price_diff × size (e6格式)
        (price_diff as i128 * self.size_e6 as i128 / 1_000_000) as i64
    }
    
    /// 计算保证金率（基点）
    pub fn calculate_margin_ratio(&self) -> u32 {
        if self.mm_e6 == 0 {
            return u32::MAX; // 避免除以0
        }
        
        let equity = self.locked_usdc_e6 + self.unrealized_pnl_e6;
        
        if equity <= 0 {
            return 0; // 已爆仓
        }
        
        // margin_ratio = equity / mm × 10000 (基点)
        (equity as i128 * 10000 / self.mm_e6 as i128) as u32
    }
    
    /// 更新标记价格和未实现盈亏
    pub fn update_pnl(&mut self, mark_price_e6: i64) {
        self.mark_price_e6 = mark_price_e6;
        self.unrealized_pnl_e6 = self.calculate_unrealized_pnl(mark_price_e6);
        self.margin_ratio_bp = self.calculate_margin_ratio();
        
        // 更新清算状态
        self.liquidation_status = if self.margin_ratio_bp >= 15000 {
            LiquidationStatus::Normal
        } else if self.margin_ratio_bp >= 11000 {
            LiquidationStatus::Warning
        } else if self.margin_ratio_bp >= 10000 {
            LiquidationStatus::SmartHedgeTriggered
        } else {
            LiquidationStatus::Liquidatable
        };
        
        self.updated_at = solana_program::clock::Clock::get()
            .map(|clock| clock.unix_timestamp)
            .unwrap_or(0);
    }
}

// === ProtectionPool已移除 ===
// Smart Hedge功能应该在1024-core/smart-hedge-engine中实现
// trading-program只负责USDC的存入和取出！

