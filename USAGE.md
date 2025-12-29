# SSH客户端交互式使用说明

## 启动程序
```bash
cargo run --release
```

## 交互式命令

### 1. 建立SSH连接
```
ssh-config --host <主机地址> --username <用户名> [--port <端口>] [--password <密码>]
```

示例：
```
> ssh-config --host 192.168.1.100 --username admin --password mypassword
> ssh-config --host example.com --username root --port 2222
```

### 2. 执行Shell命令
连接建立后，可以执行远程命令：
```
ssh-mode --mode shell --shell "<命令>" --path <任意路径>
```

示例：
```
> ssh-mode --mode shell --shell "ls -la" --path /tmp
> ssh-mode --mode shell --shell "pwd" --path /tmp
```

注意：`--path` 参数是必需的（由于结构定义），但在shell模式下不会被使用。

### 3. 传输文件（SCP）
```
ssh-mode --mode sftp --path <本地文件路径> --shell <任意字符串>
```

示例：
```
> ssh-mode --mode sftp --path C:\test.txt --shell dummy
```

注意：`--shell` 参数是必需的（由于结构定义），但在sftp模式下不会被使用。

### 4. 关闭SSH连接
```
ssh-close
```

### 5. 退出程序
```
exit
```
或
```
quit
```

## 完整使用流程示例

```
> cargo run --release
这是一个SSH客户端工具
请输入相应的命令来操作SSH连接
输入 'exit' 或 'quit' 退出程序

> ssh-config --host 192.168.1.100 --username admin --password mypass
Authenticated successfully
Session established with 192.168.1.100:22
SSH session established

> ssh-mode --mode shell --shell "hostname" --path /tmp
myserver.example.com
Shell command executed

> ssh-mode --mode shell --shell "uptime" --path /tmp
 10:30:15 up 5 days,  2:15,  1 user,  load average: 0.15, 0.20, 0.18
Shell command executed

> ssh-close
SSH session closed
SSH session closed successfully

> exit
再见！
```

## 常见错误

1. **"SSHMode command requires an established session"**
   - 需要先使用 `ssh-config` 建立连接

2. **"SSHClose command requires an established session"**
   - 需要先使用 `ssh-config` 建立连接

3. **连接失败**
   - 检查主机地址、端口、用户名、密码是否正确
   - 确保目标主机SSH服务正在运行
   - 检查网络连接

## 注意事项

- 会话在程序运行期间保持，可以执行多个命令
- 关闭会话后可以重新建立新的连接
- 使用 Ctrl+C 可以强制终止程序
