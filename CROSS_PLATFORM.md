# 跨平台编译指南

## ✅ 项目跨平台兼容性

这个SSH客户端项目**完全支持**以下平台：
- ✅ Windows (x86_64)
- ✅ macOS (ARM64 / Apple Silicon)
- ✅ macOS (x86_64 / Intel)
- ✅ Linux (x86_64)

## 🚀 推荐的构建方式

### 方案1：GitHub Actions自动构建（最推荐）⭐

**优点**：
- 自动为4个平台构建
- 无需本地配置复杂工具链
- 产物可直接下载
- 适合CI/CD

**使用方法**：
1. 将项目推送到GitHub
2. GitHub Actions会自动触发构建
3. 在Actions标签页下载各平台的二进制文件

**配置文件已创建**：`.github/workflows/build.yml`

---

### 方案2：在目标平台上直接构建

**macOS ARM64上构建**：
```bash
# 在Apple Silicon Mac上
cargo build --release
# 输出: target/release/ssh_client_rs
```

**Windows上构建**：
```bash
# 在Windows PC上
cargo build --release
# 输出: target/release/ssh_client_rs.exe
```

**Linux上构建**：
```bash
# 在Linux上
cargo build --release
# 输出: target/release/ssh_client_rs
```

---

### 方案3：使用cargo-zigbuild（较简单的交叉编译）

如果确实需要在Windows上交叉编译到macOS：

```bash
# 1. 安装zig
winget install -e --id Zig.Zig

# 2. 安装cargo-zigbuild
cargo install cargo-zigbuild

# 3. 交叉编译到macOS ARM64
cargo zigbuild --release --target aarch64-apple-darwin

# 注意：此方法仍可能遇到OpenSSL依赖问题
```

---

### 方案4：使用Docker（跨平台构建）

```dockerfile
# Dockerfile示例
FROM rust:latest
RUN rustup target add aarch64-apple-darwin
WORKDIR /app
COPY . .
RUN cargo build --release --target aarch64-apple-darwin
```

---

## ⚠️ Windows到macOS交叉编译的困难

在Windows上直接交叉编译到macOS存在以下问题：

1. **需要macOS SDK** - 苹果许可限制，难以合法获取
2. **需要专用链接器** - 如 `lld` 或 `ld64`
3. **C依赖库** - `libssh2`、`openssl` 需要ARM64 macOS版本
4. **配置复杂** - 需要设置大量环境变量和工具链

**结论**：不推荐在Windows上直接交叉编译到macOS。

---

## 📦 发布流程（使用GitHub Actions）

1. **推送代码到GitHub**：
```bash
git add .
git commit -m "Add cross-platform build"
git push origin master
```

2. **查看构建状态**：
   - 访问仓库的 "Actions" 标签
   - 等待构建完成（约5-10分钟）

3. **下载产物**：
   - 在Actions页面点击最新的workflow运行
   - 在 "Artifacts" 区域下载各平台的二进制文件

4. **创建Release（可选）**：
```bash
git tag v0.1.0
git push origin v0.1.0
```
   - 创建GitHub Release后，二进制文件会自动附加到Release中

---

## 🎯 结论

- **开发阶段**：在各自平台上直接编译
- **发布阶段**：使用GitHub Actions自动构建所有平台
- **紧急情况**：可尝试 `cargo-zigbuild`

**推荐流程**：将项目推送到GitHub，让Actions自动构建ARM64 macOS版本！
