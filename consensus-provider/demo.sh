#!/bin/bash

# Consensus Provider 演示脚本
# 用于展示去中心化GPU算力共享平台的各种功能

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 项目路径
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$PROJECT_DIR/config"
BINARY="$PROJECT_DIR/../target/release/consensus-provider"

# 检查二进制文件是否存在
check_binary() {
    if [ ! -f "$BINARY" ]; then
        echo -e "${RED}❌ 找不到二进制文件: $BINARY${NC}"
        echo -e "${YELLOW}请先编译项目:${NC}"
        echo "  cd $PROJECT_DIR && cargo build --release"
        exit 1
    fi
}

# 显示帮助信息
show_help() {
    echo -e "${BLUE}Consensus Provider 演示脚本${NC}"
    echo "用于展示去中心化GPU算力共享平台的各种功能"
    echo
    echo -e "${YELLOW}使用方法:${NC}"
    echo "  $0 [演示类型] [选项]"
    echo
    echo -e "${YELLOW}演示类型:${NC}"
    echo "  basic      - 基础功能演示 (默认)"
    echo "  multi      - 多节点演示"
    echo "  cheating   - 作弊检测演示"
    echo "  full       - 完整功能演示"
    echo "  help       - 显示此帮助信息"
    echo
    echo -e "${YELLOW}选项:${NC}"
    echo "  -c, --config FILE     配置文件路径 (默认: config/default.json)"
    echo "  -s, --server URL      共识服务器URL (默认: http://localhost:7465)"
    echo "  -h, --help           显示帮助信息"
    echo
    echo -e "${YELLOW}示例:${NC}"
    echo "  $0 basic                    # 基础功能演示"
    echo "  $0 multi                    # 多节点演示"
    echo "  $0 cheating -c config/cheating.json  # 作弊检测演示"
    echo "  $0 full -s http://192.168.1.100:7465  # 完整演示"
    echo
}

# 基础功能演示
run_basic_demo() {
    echo -e "${BLUE}🔧 启动基础功能演示${NC}"
    echo

    check_binary

    echo -e "${CYAN}执行命令:${NC}"
    echo "  $BINARY --demo basic"
    echo

    $BINARY --demo basic
}

# 多节点演示
run_multi_demo() {
    local config_file="${1:-config/default.json}"
    local server_url="${2:-http://localhost:7465}"

    echo -e "${BLUE}🏗️  启动多节点演示${NC}"
    echo "配置文件: $config_file"
    echo "服务器地址: $server_url"
    echo

    check_binary

    # 检查配置文件是否存在
    if [ ! -f "$config_file" ]; then
        echo -e "${RED}❌ 找不到配置文件: $config_file${NC}"
        echo -e "${YELLOW}可用的配置文件:${NC}"
        ls -1 config/*.json 2>/dev/null || echo "  无配置文件"
        exit 1
    fi

    echo -e "${CYAN}执行命令:${NC}"
    echo "  $BINARY --demo multi --config $config_file --server $server_url"
    echo

    $BINARY --demo multi --config "$config_file" --server "$server_url"
}

# 作弊检测演示
run_cheating_demo() {
    local config_file="${1:-config/cheating.json}"
    local server_url="${2:-http://localhost:7465}"

    echo -e "${BLUE}🎭 启动作弊检测演示${NC}"
    echo "配置文件: $config_file"
    echo "服务器地址: $server_url"
    echo

    check_binary

    # 检查配置文件是否存在
    if [ ! -f "$config_file" ]; then
        echo -e "${YELLOW}⚠️  找不到配置文件: $config_file${NC}"
        echo -e "${CYAN}使用默认配置:${NC} config/default.json"
        config_file="config/default.json"
    fi

    echo -e "${CYAN}执行命令:${NC}"
    echo "  $BINARY --demo cheating --config $config_file --server $server_url"
    echo

    $BINARY --demo cheating --config "$config_file" --server "$server_url"
}

# 完整演示
run_full_demo() {
    local config_file="${1:-config/default.json}"
    local server_url="${2:-http://localhost:7465}"

    echo -e "${BLUE}🎊 启动完整功能演示${NC}"
    echo "配置文件: $config_file"
    echo "服务器地址: $server_url"
    echo

    check_binary

    echo -e "${CYAN}执行命令:${NC}"
    echo "  $BINARY --demo full --config $config_file --server $server_url"
    echo

    $BINARY --demo full --config "$config_file" --server "$server_url"
}

# 解析命令行参数
parse_args() {
    DEMO_TYPE="basic"
    CONFIG_FILE="config/default.json"
    SERVER_URL="http://localhost:7465"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -c|--config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            -s|--server)
                SERVER_URL="$2"
                shift 2
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            basic|multi|cheating|full|help)
                DEMO_TYPE="$1"
                shift
                ;;
            *)
                echo -e "${RED}❌ 未知参数: $1${NC}"
                echo
                show_help
                exit 1
                ;;
        esac
    done
}

# 主函数
main() {
    cd "$PROJECT_DIR"

    parse_args "$@"

    case $DEMO_TYPE in
        basic)
            run_basic_demo
            ;;
        multi)
            run_multi_demo "$CONFIG_FILE" "$SERVER_URL"
            ;;
        cheating)
            run_cheating_demo "$CONFIG_FILE" "$SERVER_URL"
            ;;
        full)
            run_full_demo "$CONFIG_FILE" "$SERVER_URL"
            ;;
        help)
            show_help
            ;;
        *)
            echo -e "${RED}❌ 无效的演示类型: $DEMO_TYPE${NC}"
            show_help
            exit 1
            ;;
    esac
}

# 如果脚本被直接执行，运行主函数
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi


