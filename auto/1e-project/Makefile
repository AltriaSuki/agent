# 开发自动化工具集

.PHONY: help check test fmt lint fix build clean install-hooks

# 默认目标
help:
	@echo "🛠️  日省录 - 开发工具"
	@echo ""
	@echo "常用命令:"
	@echo "  make check        - 运行所有检查（推荐 commit 前使用）"
	@echo "  make test         - 运行测试"
	@echo "  make fmt          - 格式化代码"
	@echo "  make lint         - 运行 linter"
	@echo "  make fix          - 自动修复问题"
	@echo "  make build        - 编译项目"
	@echo "  make install-hooks - 安装 Git hooks"
	@echo ""
	@echo "Android 相关:"
	@echo "  make android      - 构建 Android 库"
	@echo "  make apk          - 构建 APK"
	@echo ""

# 完整检查（等同于 CI）
check: fmt lint test
	@echo "✅ 所有检查通过！"

# 运行测试
test:
	@echo "🧪 运行测试..."
	@cargo test --quiet

# 格式化代码
fmt:
	@echo "📝 格式化代码..."
	@cargo fmt --all

# 运行 Clippy
lint:
	@echo "🔎 运行 Clippy..."
	@cargo clippy --all-targets --all-features -- \
		-D warnings \
		-W clippy::all \
		-W clippy::pedantic \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc

# 自动修复问题
fix:
	@echo "🔧 自动修复问题..."
	@cargo fmt --all
	@cargo clippy --fix --allow-dirty --allow-staged
	@echo "✅ 已尝试自动修复"

# 编译
build:
	@echo "🔨 编译项目..."
	@cargo build

# 编译 release 版本
build-release:
	@echo "🔨 编译 release 版本..."
	@cargo build --release --features server

# 清理
clean:
	@echo "🧹 清理构建文件..."
	@cargo clean
	@rm -rf android/app/build
	@rm -rf android/app/src/main/jniLibs

# 安装 Git hooks
install-hooks:
	@bash scripts/setup-hooks.sh

# 运行开发服务器
dev:
	@echo "🚀 启动开发服务器..."
	@RUST_LOG=debug cargo run --features server

# 运行系统测试
system-test:
	@echo "🔧 运行系统测试..."
	@bash scripts/system_test.sh

# 压力测试
stress-test:
	@echo "⚡ 运行压力测试..."
	@REQUESTS=200 CONCURRENCY=20 bash scripts/stress_test.sh

# Android 构建
android:
	@echo "📱 构建 Android 库..."
	@cargo build --target aarch64-linux-android --release --lib
	@cargo build --target armv7-linux-androideabi --release --lib
	@bash scripts/sync_android_assets.sh

# 构建 APK
apk: android
	@echo "📦 构建 APK..."
	@cd android && ./gradlew assembleRelease
	@echo "✅ APK: android/app/build/outputs/apk/release/app-release.apk"

# 安装到设备
install-apk: apk
	@echo "📲 安装到设备..."
	@adb install -r android/app/build/outputs/apk/release/app-release.apk

# 代码覆盖率
coverage:
	@echo "📊 生成代码覆盖率报告..."
	@cargo tarpaulin --out Html --output-dir coverage
	@echo "✅ 报告: coverage/index.html"

# 安全审计
audit:
	@echo "🔒 运行安全审计..."
	@cargo audit

# 更新依赖
update:
	@echo "⬆️  更新依赖..."
	@cargo update
	@cargo audit

# 生成文档
docs:
	@echo "📚 生成文档..."
	@cargo doc --no-deps --open

# 全量检查（包括测试、审计、覆盖率）
full-check: check audit coverage
	@echo "✅ 全量检查完成！"

# 快速检查（只检查格式和 lint，不运行测试）
quick-check: fmt lint
	@echo "⚡ 快速检查完成！"

# 监听文件变化自动运行测试（需要安装 cargo-watch）
watch:
	@command -v cargo-watch >/dev/null 2>&1 || { echo "请先安装: cargo install cargo-watch"; exit 1; }
	@echo "👀 监听文件变化..."
	@cargo watch -x test

# Benchmark
bench:
	@echo "📈 运行性能基准测试..."
	@cargo bench

# 一键发布流程
release: full-check build-release apk
	@echo "🎉 准备发布！"
	@echo "  - Web 服务器: target/release/daily-reckoning"
	@echo "  - Android APK: android/app/build/outputs/apk/release/app-release.apk"
