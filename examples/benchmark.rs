//! 性能基准测试
//! 
//! 测试Trading Program的性能指标：
//! - 延迟（Latency）
//! - 吞吐量（TPS）
//! - Gas成本
//! 
//! 运行:
//! ```bash
//! cargo run --example benchmark --release
//! ```

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair, Signer},
    pubkey::Pubkey,
};
use trading_program_client::{TradingProgramClient, state::{Side, MarginMode}};
use std::env;
use std::time::{Duration, Instant};

struct BenchmarkResult {
    operation: String,
    total_ops: usize,
    successful_ops: usize,
    failed_ops: usize,
    total_duration: Duration,
    avg_latency: Duration,
    min_latency: Duration,
    max_latency: Duration,
    p50_latency: Duration,
    p95_latency: Duration,
    p99_latency: Duration,
    tps: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("\n📊 {} 性能测试结果", self.operation);
        println!("========================================");
        println!("总操作数: {}", self.total_ops);
        println!("成功: {} ({:.1}%)", self.successful_ops, 
            self.successful_ops as f64 / self.total_ops as f64 * 100.0);
        println!("失败: {} ({:.1}%)", self.failed_ops,
            self.failed_ops as f64 / self.total_ops as f64 * 100.0);
        println!("\n延迟统计:");
        println!("  平均: {:?}", self.avg_latency);
        println!("  最小: {:?}", self.min_latency);
        println!("  最大: {:?}", self.max_latency);
        println!("  P50:  {:?}", self.p50_latency);
        println!("  P95:  {:?}", self.p95_latency);
        println!("  P99:  {:?}", self.p99_latency);
        println!("\n吞吐量:");
        println!("  TPS: {:.2}", self.tps);
        println!("========================================");
    }
}

fn calculate_percentile(latencies: &mut [Duration], percentile: f64) -> Duration {
    latencies.sort();
    let index = (latencies.len() as f64 * percentile / 100.0) as usize;
    latencies[index.min(latencies.len() - 1)]
}

async fn benchmark_lock_margin(
    client: &TradingProgramClient,
    user: &Keypair,
    user_usdc: &Pubkey,
    iterations: usize,
) -> Result<BenchmarkResult> {
    println!("\n🔒 开始测试 Lock Margin...");
    
    let mut latencies = Vec::with_capacity(iterations);
    let mut successful = 0;
    let mut failed = 0;
    
    let start_all = Instant::now();
    
    for i in 0..iterations {
        let account_id = format!("bench_{}_{}", user.pubkey().to_string()[..8].to_string(), i);
        let market = "BTC-PERP".to_string();
        
        let start = Instant::now();
        
        match client.lock_margin(
            &user.pubkey(),
            user_usdc,
            account_id,
            market,
            Side::Buy,
            1_000_000, // 0.001 BTC
            100_000_000_000, // $100,000
            20,
            MarginMode::Cross,
        ).await {
            Ok(_) => {
                successful += 1;
                latencies.push(start.elapsed());
            }
            Err(_) => {
                failed += 1;
            }
        }
        
        // 避免请求过快
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    let total_duration = start_all.elapsed();
    
    if latencies.is_empty() {
        return Err(anyhow::anyhow!("所有操作都失败了"));
    }
    
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let min_latency = *latencies.iter().min().unwrap();
    let max_latency = *latencies.iter().max().unwrap();
    
    let p50 = calculate_percentile(&mut latencies, 50.0);
    let p95 = calculate_percentile(&mut latencies, 95.0);
    let p99 = calculate_percentile(&mut latencies, 99.0);
    
    let tps = successful as f64 / total_duration.as_secs_f64();
    
    Ok(BenchmarkResult {
        operation: "Lock Margin".to_string(),
        total_ops: iterations,
        successful_ops: successful,
        failed_ops: failed,
        total_duration,
        avg_latency,
        min_latency,
        max_latency,
        p50_latency: p50,
        p95_latency: p95,
        p99_latency: p99,
        tps,
    })
}

async fn benchmark_calculations() -> Result<()> {
    println!("\n🧮 基准测试：计算性能");
    println!("========================================");
    
    use trading_program::utils::*;
    
    // 测试IM计算
    let iterations = 1_000_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = calculate_initial_margin(1_000_000, 100_000_000_000, 20);
    }
    
    let elapsed = start.elapsed();
    println!("IM计算:");
    println!("  {} 次迭代", iterations);
    println!("  总时间: {:?}", elapsed);
    println!("  平均: {:?}", elapsed / iterations);
    println!("  TPS: {:.0}", iterations as f64 / elapsed.as_secs_f64());
    
    // 测试PnL计算
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = calculate_realized_pnl(true, 100_000_000_000, 102_000_000_000, 1_000_000);
    }
    
    let elapsed = start.elapsed();
    println!("\nPnL计算:");
    println!("  {} 次迭代", iterations);
    println!("  总时间: {:?}", elapsed);
    println!("  平均: {:?}", elapsed / iterations);
    println!("  TPS: {:.0}", iterations as f64 / elapsed.as_secs_f64());
    
    // 测试强平判断
    let start = Instant::now();
    
    for i in 0..iterations {
        let _ = is_liquidatable((i % 20000) as u32);
    }
    
    let elapsed = start.elapsed();
    println!("\n强平判断:");
    println!("  {} 次迭代", iterations);
    println!("  总时间: {:?}", elapsed);
    println!("  平均: {:?}", elapsed / iterations);
    println!("  TPS: {:.0}", iterations as f64 / elapsed.as_secs_f64());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 1024 Trading Program - 性能基准测试");
    println!("========================================\n");
    
    // 1. 链下计算性能测试
    benchmark_calculations().await?;
    
    // 2. 链上操作性能测试（可选，需要配置）
    if let Ok(rpc_url) = env::var("RPC_URL") {
        println!("\n\n🌐 链上操作性能测试");
        println!("========================================");
        
        let program_id: Pubkey = env::var("PROGRAM_ID")
            .unwrap_or_else(|_| "E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw".to_string())
            .parse()?;
        
        println!("RPC: {}", rpc_url);
        println!("Program ID: {}", program_id);
        
        // 加载密钥对
        if let Ok(user_path) = env::var("USER_KEYPAIR") {
            let user = read_keypair_file(&user_path)
                .map_err(|e| anyhow::anyhow!("无法读取密钥对: {}", e))?;
            
            let vault_usdc = Pubkey::new_unique();
            let insurance_fund = Pubkey::new_unique();
            let fee_treasury = Pubkey::new_unique();
            
            let client = TradingProgramClient::new(
                rpc_url.clone(),
                program_id,
                user.insecure_clone(),
                vault_usdc,
                insurance_fund,
                fee_treasury,
            );
            
            let usdc_mint: Pubkey = env::var("USDC_MINT")
                .unwrap_or_else(|_| "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string())
                .parse()?;
            
            let user_usdc = spl_associated_token_account::get_associated_token_address(
                &user.pubkey(),
                &usdc_mint
            );
            
            // 运行基准测试（少量迭代避免费用）
            let iterations = 5;
            println!("\n⚠️  链上测试将消耗真实Gas费用!");
            println!("   测试次数: {}", iterations);
            println!("   按Enter继续，Ctrl+C取消...");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            let user_clone = user.insecure_clone();
            let result = benchmark_lock_margin(&client, &user_clone, &user_usdc, iterations).await?;
            result.print();
            
            // Gas成本估算
            println!("\n💰 Gas成本估算:");
            println!("   每笔Lock Margin: ~0.0005 SOL (~$0.10 @ $200/SOL)");
            println!("   每笔Unlock Margin: ~0.0003 SOL (~$0.06 @ $200/SOL)");
            println!("   总计每轮: ~$0.16");
        } else {
            println!("\n⚠️  未设置USER_KEYPAIR，跳过链上测试");
        }
    } else {
        println!("\n⚠️  未设置RPC_URL，跳过链上测试");
        println!("   设置环境变量以启用:");
        println!("   export RPC_URL=\"https://testnet-rpc.1024chain.com/rpc/\"");
        println!("   export USER_KEYPAIR=\"/path/to/user.json\"");
    }
    
    println!("\n========================================");
    println!("✅ 性能基准测试完成!");
    println!("========================================");
    
    println!("\n性能目标对比:");
    println!("  目标延迟: < 2秒");
    println!("  目标TPS:  > 10");
    println!("  目标成功率: > 99%");
    
    Ok(())
}

