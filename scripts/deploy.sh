#!/bin/bash
# 1024 Trading Program - Testnet部署脚本

set -e

echo "🚀 1024 Trading Program Deployment Script"
echo "=========================================="
echo ""

# 配置
RPC_URL="https://testnet-rpc.1024chain.com/rpc/"
NETWORK="1024chain-testnet"
PROGRAM_NAME="trading-program"

# 颜色
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 检查Solana CLI
if ! command -v solana &> /dev/null; then
    echo -e "${RED}❌ Error: Solana CLI not found${NC}"
    echo "Please install: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

echo -e "${GREEN}✅ Solana CLI found${NC}"
solana --version

# 设置RPC
echo ""
echo "📡 Setting RPC to $RPC_URL"
solana config set --url $RPC_URL

# 检查余额
echo ""
echo "💰 Checking deployer balance..."
BALANCE=$(solana balance)
echo "Balance: $BALANCE"

# 构建Program
echo ""
echo "🔨 Building Program..."
cargo build-sbf

PROGRAM_SO="target/deploy/${PROGRAM_NAME}.so"

if [ ! -f "$PROGRAM_SO" ]; then
    echo -e "${RED}❌ Error: Program binary not found at $PROGRAM_SO${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Program built successfully${NC}"
ls -lh $PROGRAM_SO

# 部署Program
echo ""
echo "📤 Deploying Program to $NETWORK..."
echo ""

DEPLOY_OUTPUT=$(solana program deploy $PROGRAM_SO --output json)
PROGRAM_ID=$(echo $DEPLOY_OUTPUT | grep -o '"programId":"[^"]*"' | cut -d'"' -f4)

if [ -z "$PROGRAM_ID" ]; then
    echo -e "${RED}❌ Error: Failed to extract Program ID${NC}"
    echo "Deploy output: $DEPLOY_OUTPUT"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Program deployed successfully!${NC}"
echo ""
echo "Program ID: $PROGRAM_ID"
echo ""

# 保存Program ID
echo "$PROGRAM_ID" > program-id.txt
echo "Program ID saved to program-id.txt"

# 更新lib.rs中的Program ID
echo ""
echo "📝 Updating src/lib.rs with Program ID..."

sed -i.bak "s/solana_program::declare_id!(\"[^\"]*\");/solana_program::declare_id!(\"$PROGRAM_ID\");/" src/lib.rs

echo -e "${GREEN}✅ src/lib.rs updated${NC}"

# 查看Program信息
echo ""
echo "📊 Program Information:"
solana program show $PROGRAM_ID

# 创建初始化脚本
echo ""
echo "📝 Creating initialization script..."

cat > scripts/initialize-vault.sh << EOF
#!/bin/bash
# Initialize Trading Vault

set -e

PROGRAM_ID="$PROGRAM_ID"
RPC_URL="$RPC_URL"

echo "🏗️  Initializing Trading Vault..."
echo "Program ID: \$PROGRAM_ID"
echo ""

# TODO: 创建并运行initialize instruction
# 需要Rust客户端或使用anchor

echo "⚠️  Manual initialization required"
echo "Run: cargo run --example initialize"
EOF

chmod +x scripts/initialize-vault.sh

echo -e "${GREEN}✅ Initialization script created${NC}"

# 完成
echo ""
echo "=========================================="
echo -e "${GREEN}🎉 Deployment Complete!${NC}"
echo "=========================================="
echo ""
echo "Program ID: $PROGRAM_ID"
echo "Network: $NETWORK"
echo "RPC: $RPC_URL"
echo ""
echo "Next steps:"
echo "1. Run: ./scripts/initialize-vault.sh"
echo "2. Update 1024-core with Program ID"
echo "3. Run end-to-end tests"
echo ""
echo "Program ID saved to: program-id.txt"
echo ""

