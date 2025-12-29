# SSH 客户端工具 (ssh_client_rs)

一个功能完善的交互式 SSH 客户端工具，支持配置文件管理、连接配置保存、环境变量等功能。
# 这是一个大学生用来探索Github Action跨平台编译的项目，路人请回避（）
## 主要特性

- **交互式命令行界面** - 友好的 REPL 交互体验
- **配置文件支持** - 使用 TOML 格式保存默认配置和连接配置
- **连接配置管理** - 保存和重用常用的 SSH 连接配置
- **环境变量支持** - 通过环境变量设置默认参数
- **智能提示符** - 连接后显示当前用户和主机信息
- **无冗余参数** - 每个命令只需输入必要的参数
- **文件传输** - 支持上传和下载文件

## 快速开始

### 安装和运行

```bash
cargo build --release
./target/release/ssh_client_rs
```

或者直接运行：

```bash
cargo run --release
```

### 基本使用

#### 1. 使用命令行参数连接

```bash
> connect --host example.com --username root --password mypass
✓ 认证成功
✓ 已连接到 example.com:22

[root@example.com] >
```

#### 2. 使用保存的配置连接

```bash
# 首次连接并保存配置
> connect --host example.com --username root --password mypass
✓ 已连接到 example.com:22

# 保存当前连接为 "server1"
[root@example.com] > save server1
✓ 连接配置已保存为 'server1'

# 下次可以直接使用配置名连接
> connect -p server1
✓ 已连接到 example.com:22
```

#### 3. 执行远程命令

```bash
[root@example.com] > shell ls -la
total 48
drwx------  5 root root 4096 Dec 29 10:30 .
drwxr-xr-x 19 root root 4096 Dec 28 15:20 ..
...

[root@example.com] > shell "uname -a"
Linux example.com 5.15.0-89-generic #99-Ubuntu SMP x86_64 GNU/Linux
```

#### 4. 文件传输

```bash
# 上传文件
[root@example.com] > upload ./local.txt /tmp/remote.txt
✓ 文件上传成功: ./local.txt -> /tmp/remote.txt

# 下载文件
[root@example.com] > download /tmp/remote.txt ./downloaded.txt
✓ 文件下载成功: /tmp/remote.txt -> ./downloaded.txt
```

## 命令详解

### 连接管理

#### `connect` - 建立 SSH 连接

**方式 1: 使用保存的配置**
```bash
connect -p <配置名>
connect --profile <配置名>
```

**方式 2: 直接指定参数**
```bash
connect --host <地址> --username <用户名> [选项]
```

**选项：**
- `--host` - SSH 服务器地址（可通过 `SSH_HOST` 环境变量设置）
- `-u, --username` - 用户名（可通过 `SSH_USERNAME` 环境变量设置）
- `--port` - SSH 端口，默认 22（可通过 `SSH_PORT` 环境变量设置）
- `-p, --password` - 密码，不提供则使用 SSH Agent（可通过 `SSH_PASSWORD` 环境变量设置）

**示例：**
```bash
# 最简单的连接（使用默认端口 22）
> connect --host 192.168.1.100 --username admin

# 指定端口和密码
> connect --host example.com --username root --port 2222 --password mypass

# 使用保存的配置
> connect -p myserver
```

#### `close` - 关闭当前连接

```bash
[user@host] > close
✓ SSH 连接已关闭
```

### 远程操作

#### `shell` - 执行远程命令

```bash
shell <命令>
```

**示例：**
```bash
> shell pwd
/home/user

> shell "ps aux | grep nginx"
root      1234  0.0  0.1 125684  2156 ?  Ss   10:30   0:00 nginx: master
```

#### `upload` - 上传文件到远程服务器

```bash
upload <本地路径> <远程路径>
```

**示例：**
```bash
> upload ./config.json /etc/myapp/config.json
✓ 文件上传成功: ./config.json -> /etc/myapp/config.json
```

#### `download` - 从远程服务器下载文件

```bash
download <远程路径> <本地路径>
```

**示例：**
```bash
> download /var/log/app.log ./app.log
✓ 文件下载成功: /var/log/app.log -> ./app.log
```

### 配置管理

#### `save` - 保存当前连接为配置

```bash
save <配置名>
```

**示例：**
```bash
# 连接后保存配置
[root@example.com] > save production-server
✓ 连接配置已保存为 'production-server'
```

#### `list` - 列出所有保存的配置

```bash
> list
已保存的连接配置:
  • myserver - admin@192.168.1.100:22
  • production - root@example.com:22
  • dev-server - dev@10.0.0.5:2222
```

### 其他命令

- `help` - 显示帮助信息
- `exit` / `quit` - 退出程序

## 配置文件

### 配置文件位置

- **Windows**: `C:\Users\<用户名>\AppData\Roaming\ssh_client_rs\config.toml`
- **Linux**: `~/.config/ssh_client_rs/config.toml`
- **macOS**: `~/Library/Application Support/ssh_client_rs/config.toml`

### 配置文件格式

```toml
# 默认配置
[defaults]
port = 22
username = "admin"  # 可选，设置默认用户名

# 保存的连接配置
[profiles.myserver]
host = "192.168.1.100"
username = "admin"
password = "mypass"  # 可选，不建议保存密码

[profiles.production]
host = "example.com"
port = 2222
username = "root"
# 不保存密码，连接时会使用 SSH Agent
```

### 手动编辑配置

你可以直接编辑配置文件来添加或修改连接配置。配置文件会在首次运行时自动创建。

## 环境变量

以下环境变量可用于设置默认值：

- `SSH_HOST` - 默认主机地址
- `SSH_PORT` - 默认端口
- `SSH_USERNAME` - 默认用户名
- `SSH_PASSWORD` - 默认密码

**示例（Linux/macOS）：**
```bash
export SSH_USERNAME=admin
export SSH_PORT=2222
./ssh_client_rs
```

**示例（Windows PowerShell）：**
```powershell
$env:SSH_USERNAME="admin"
$env:SSH_PORT="2222"
.\ssh_client_rs.exe
```

## 完整使用示例

```
$ cargo run --release
SSH 客户端工具 v0.1.0
输入 'help' 查看帮助，'exit' 或 'quit' 退出程序

> connect --host 192.168.1.100 --username admin --password mypass
✓ 认证成功
✓ 已连接到 192.168.1.100:22

[admin@192.168.1.100] > shell hostname
myserver.local

[admin@192.168.1.100] > shell "df -h"
Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   15G   33G  31% /
...

[admin@192.168.1.100] > save myserver
✓ 连接配置已保存为 'myserver'

[admin@192.168.1.100] > upload ./test.txt /tmp/test.txt
✓ 文件上传成功: ./test.txt -> /tmp/test.txt

[admin@192.168.1.100] > close
✓ SSH 连接已关闭

> list
已保存的连接配置:
  • myserver - admin@192.168.1.100:22

> connect -p myserver
✓ 认证成功
✓ 已连接到 192.168.1.100:22

[admin@192.168.1.100] > exit
再见！
```

## 架构改进

相比旧版本，新版本进行了以下重大改进：

### 1. 命令结构优化
- **旧版**: `ssh-mode --mode shell --shell "cmd" --path /dummy`（需要输入无用的 --path 参数）
- **新版**: `shell cmd`（只需输入必要参数）

### 2. 配置管理
- **新增**: 支持保存和加载连接配置
- **新增**: 支持配置文件（TOML 格式）
- **新增**: 支持通过配置名快速连接

### 3. 默认值支持
- **新增**: 端口默认为 22，无需每次指定
- **新增**: 可在配置文件中设置默认用户名
- **新增**: 支持环境变量设置默认值

### 4. 用户体验改进
- **新增**: 智能提示符显示当前连接信息
- **新增**: 友好的错误提示（带 ✓ 和 ✗ 标记）
- **新增**: 内置帮助命令
- **改进**: 更清晰的命令命名（connect, shell, upload, download）

### 5. 功能扩展
- **新增**: 文件下载功能
- **改进**: 文件上传使用正确的文件大小
- **新增**: 配置列表展示功能

## 技术栈

- **Rust 2021 Edition**
- **ssh2** - SSH 协议实现（使用 vendored-openssl）
- **clap 4** - 命令行参数解析
- **serde + toml** - 配置文件序列化
- **dirs** - 跨平台配置目录

## 跨平台编译

本项目配置了 GitHub Actions 进行跨平台编译：

- Windows (x86_64-pc-windows-msvc)
- Linux (x86_64-unknown-linux-gnu)
- macOS (x86_64-apple-darwin, aarch64-apple-darwin)

详见 `.github/workflows/` 目录。

## 常见问题

### 1. 认证失败
- 检查用户名和密码是否正确
- 如果不提供密码，确保 SSH Agent 已启动并加载了密钥

### 2. 连接超时
- 检查主机地址和端口是否正确
- 确保目标主机的 SSH 服务正在运行
- 检查防火墙设置

### 3. 文件传输失败
- 确保有足够的权限写入目标目录
- 检查磁盘空间是否充足
- 确保路径使用正确的格式（Windows 使用 `\` 或 `/`，Linux/macOS 使用 `/`）

### 4. 配置文件找不到
配置文件会在首次运行时自动创建。如果遇到问题，可以手动创建配置目录：

**Windows**: `mkdir %APPDATA%\ssh_client_rs`
**Linux/macOS**: `mkdir -p ~/.config/ssh_client_rs`

## 许可证

本项目是一个用于学习 Rust 和 GitHub Actions 的示例项目。

## 贡献

欢迎提交 Issue 和 Pull Request！
