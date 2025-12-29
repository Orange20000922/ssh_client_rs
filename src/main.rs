use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;
use std::collections::HashMap;
use clap::{Parser, Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============ 配置文件结构 ============
#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    #[serde(default)]
    defaults: ConfigDefaults,
    #[serde(default)]
    profiles: HashMap<String, ConnectionProfile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigDefaults {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default)]
    username: Option<String>,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            port: default_port(),
            username: None,
        }
    }
}

fn default_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectionProfile {
    host: String,
    #[serde(default)]
    port: Option<u16>,
    username: String,
    #[serde(default)]
    password: Option<String>,
}

// ============ CLI 命令结构 ============
#[derive(Parser, Debug)]
#[command(name = "ssh-client")]
#[command(about = "一个简单的 SSH 客户端工具", long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 建立 SSH 连接
    Connect(ConnectArgs),
    /// 执行 Shell 命令
    Shell(ShellArgs),
    /// 上传文件 (SFTP)
    Upload(UploadArgs),
    /// 下载文件 (SFTP)
    Download(DownloadArgs),
    /// 关闭 SSH 连接
    Close,
    /// 保存当前连接为配置
    Save(SaveProfileArgs),
    /// 列出所有保存的配置
    List,
}

#[derive(Args, Debug)]
struct ConnectArgs {
    /// 连接配置名称（如果使用已保存的配置）
    #[arg(short, long)]
    profile: Option<String>,

    /// SSH 服务器地址
    #[arg(long, env = "SSH_HOST")]
    host: Option<String>,

    /// SSH 端口（默认: 22）
    #[arg(long, env = "SSH_PORT")]
    port: Option<u16>,

    /// 用户名
    #[arg(long, short, env = "SSH_USERNAME")]
    username: Option<String>,

    /// 密码（不提供则使用 SSH Agent）
    #[arg(long, short, env = "SSH_PASSWORD")]
    password: Option<String>,
}

#[derive(Args, Debug)]
struct ShellArgs {
    /// 要执行的 Shell 命令
    command: String,
}

#[derive(Args, Debug)]
struct UploadArgs {
    /// 本地文件路径
    local: PathBuf,

    /// 远程文件路径
    remote: PathBuf,
}

#[derive(Args, Debug)]
struct DownloadArgs {
    /// 远程文件路径
    remote: PathBuf,

    /// 本地文件路径
    local: PathBuf,
}

#[derive(Args, Debug)]
struct SaveProfileArgs {
    /// 配置名称
    name: String,
}

// ============ 配置管理 ============
struct ConfigManager {
    config: Config,
    config_path: PathBuf,
}

impl ConfigManager {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("无法获取配置目录")?
            .join("ssh_client_rs");

        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");

        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        Ok(Self { config, config_path })
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn get_profile(&self, name: &str) -> Option<&ConnectionProfile> {
        self.config.profiles.get(name)
    }

    fn save_profile(&mut self, name: String, profile: ConnectionProfile) -> Result<(), Box<dyn std::error::Error>> {
        self.config.profiles.insert(name, profile);
        self.save()?;
        Ok(())
    }

    fn list_profiles(&self) -> Vec<&String> {
        self.config.profiles.keys().collect()
    }

    fn get_defaults(&self) -> &ConfigDefaults {
        &self.config.defaults
    }
}

// ============ SSH 会话管理 ============
struct SessionManager {
    session: Option<Session>,
    current_connection: Option<ConnectionProfile>,
}

impl SessionManager {
    fn new() -> Self {
        Self {
            session: None,
            current_connection: None,
        }
    }

    fn connect(&mut self, profile: ConnectionProfile) -> Result<(), Box<dyn std::error::Error>> {
        let port = profile.port.unwrap_or(22);
        let tcp = TcpStream::connect(format!("{}:{}", profile.host, port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        if let Some(ref password) = profile.password {
            session.userauth_password(&profile.username, password)?;
        } else {
            session.userauth_agent(&profile.username)?;
        }

        if !session.authenticated() {
            return Err(Box::from("认证失败"));
        }

        println!("✓ 认证成功");
        println!("✓ 已连接到 {}:{}", profile.host, port);

        self.session = Some(session);
        self.current_connection = Some(profile);
        Ok(())
    }

    fn exec_shell(&mut self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let session = self.session.as_mut().ok_or("未建立 SSH 连接")?;
        let mut channel = session.channel_session()?;
        channel.exec(command)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }

    fn upload(&mut self, local: &PathBuf, remote: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let session = self.session.as_mut().ok_or("未建立 SSH 连接")?;
        let local_file = std::fs::File::open(local)?;
        let file_size = local_file.metadata()?.len();

        let mut remote_file = session.scp_send(remote, 0o644, file_size, None)?;
        std::io::copy(&mut std::io::BufReader::new(local_file), &mut remote_file)?;

        println!("✓ 文件上传成功: {} -> {}", local.display(), remote.display());
        Ok(())
    }

    fn download(&mut self, remote: &PathBuf, local: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let session = self.session.as_mut().ok_or("未建立 SSH 连接")?;
        let (mut remote_file, _stat) = session.scp_recv(remote)?;
        let mut local_file = std::fs::File::create(local)?;
        std::io::copy(&mut remote_file, &mut local_file)?;

        println!("✓ 文件下载成功: {} -> {}", remote.display(), local.display());
        Ok(())
    }

    fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(session) = self.session.take() {
            session.disconnect(None, "正常断开连接", None)?;
            println!("✓ SSH 连接已关闭");
            self.current_connection = None;
        } else {
            return Err(Box::from("当前没有活动的 SSH 连接"));
        }
        Ok(())
    }

    fn get_current_connection(&self) -> Option<&ConnectionProfile> {
        self.current_connection.as_ref()
    }

    fn is_connected(&self) -> bool {
        self.session.is_some()
    }
}

// ============ 主程序 ============
fn main() {
    println!("SSH 客户端工具 v0.1.0");
    println!("输入 'help' 查看帮助，'exit' 或 'quit' 退出程序\n");

    let mut config_manager = match ConfigManager::new() {
        Ok(cm) => cm,
        Err(e) => {
            eprintln!("⚠ 无法加载配置文件: {}", e);
            eprintln!("将使用默认配置继续...\n");
            ConfigManager {
                config: Config::default(),
                config_path: PathBuf::new(),
            }
        }
    };

    let mut session_manager = SessionManager::new();

    loop {
        // 显示提示符
        if session_manager.is_connected() {
            if let Some(conn) = session_manager.get_current_connection() {
                print!("[{}@{}] > ", conn.username, conn.host);
            } else {
                print!("> ");
            }
        } else {
            print!("> ");
        }
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        // 读取用户输入
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            eprintln!("读取输入失败");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // 处理退出命令
        if input == "exit" || input == "quit" {
            if session_manager.is_connected() {
                let _ = session_manager.close();
            }
            println!("再见！");
            break;
        }

        // 处理帮助命令
        if input == "help" || input == "--help" {
            print_help();
            continue;
        }

        // 解析命令
        let args: Vec<&str> = input.split_whitespace().collect();
        let args_with_program = std::iter::once("ssh-client")
            .chain(args.iter().copied())
            .collect::<Vec<_>>();

        let cli = match Cli::try_parse_from(args_with_program) {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        // 执行命令
        match cli.command {
            Commands::Connect(args) => {
                match handle_connect(&mut config_manager, &mut session_manager, args) {
                    Ok(_) => {},
                    Err(e) => eprintln!("✗ 连接失败: {}", e),
                }
            },
            Commands::Shell(args) => {
                if !session_manager.is_connected() {
                    eprintln!("✗ 请先使用 'connect' 命令建立连接");
                    continue;
                }
                match session_manager.exec_shell(&args.command) {
                    Ok(output) => println!("{}", output),
                    Err(e) => eprintln!("✗ 执行命令失败: {}", e),
                }
            },
            Commands::Upload(args) => {
                if !session_manager.is_connected() {
                    eprintln!("✗ 请先使用 'connect' 命令建立连接");
                    continue;
                }
                match session_manager.upload(&args.local, &args.remote) {
                    Ok(_) => {},
                    Err(e) => eprintln!("✗ 上传失败: {}", e),
                }
            },
            Commands::Download(args) => {
                if !session_manager.is_connected() {
                    eprintln!("✗ 请先使用 'connect' 命令建立连接");
                    continue;
                }
                match session_manager.download(&args.remote, &args.local) {
                    Ok(_) => {},
                    Err(e) => eprintln!("✗ 下载失败: {}", e),
                }
            },
            Commands::Close => {
                match session_manager.close() {
                    Ok(_) => {},
                    Err(e) => eprintln!("✗ {}", e),
                }
            },
            Commands::Save(args) => {
                match handle_save_profile(&mut config_manager, &session_manager, args) {
                    Ok(_) => {},
                    Err(e) => eprintln!("✗ 保存配置失败: {}", e),
                }
            },
            Commands::List => {
                let profiles = config_manager.list_profiles();
                if profiles.is_empty() {
                    println!("没有保存的连接配置");
                } else {
                    println!("已保存的连接配置:");
                    for name in profiles {
                        if let Some(profile) = config_manager.get_profile(name) {
                            println!("  • {} - {}@{}:{}",
                                name,
                                profile.username,
                                profile.host,
                                profile.port.unwrap_or(22)
                            );
                        }
                    }
                }
            },
        }
    }
}

fn handle_connect(
    config_manager: &ConfigManager,
    session_manager: &mut SessionManager,
    args: ConnectArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let profile = if let Some(profile_name) = args.profile {
        // 使用保存的配置
        let saved_profile = config_manager.get_profile(&profile_name)
            .ok_or(format!("配置 '{}' 不存在", profile_name))?;

        // 允许命令行参数覆盖保存的配置
        ConnectionProfile {
            host: args.host.unwrap_or_else(|| saved_profile.host.clone()),
            port: args.port.or(saved_profile.port),
            username: args.username.unwrap_or_else(|| saved_profile.username.clone()),
            password: args.password.or_else(|| saved_profile.password.clone()),
        }
    } else {
        // 从命令行参数创建配置
        let defaults = config_manager.get_defaults();
        let host = args.host.ok_or("必须提供 --host 或使用 -p/--profile")?;
        let username = args.username
            .or_else(|| defaults.username.clone())
            .ok_or("必须提供 --username 或在配置文件中设置默认用户名")?;

        ConnectionProfile {
            host,
            port: args.port,
            username,
            password: args.password,
        }
    };

    session_manager.connect(profile)?;
    Ok(())
}

fn handle_save_profile(
    config_manager: &mut ConfigManager,
    session_manager: &SessionManager,
    args: SaveProfileArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let current = session_manager.get_current_connection()
        .ok_or("当前没有活动的连接，无法保存配置")?;

    config_manager.save_profile(args.name.clone(), current.clone())?;
    println!("✓ 连接配置已保存为 '{}'", args.name);
    Ok(())
}

fn print_help() {
    println!("可用命令:");
    println!();
    println!("连接管理:");
    println!("  connect -p <配置名>                    使用保存的配置连接");
    println!("  connect --host <地址> --username <用户>  直接连接（可选: --port --password）");
    println!("  close                                  关闭当前连接");
    println!();
    println!("远程操作:");
    println!("  shell <命令>                           执行远程命令");
    println!("  upload <本地路径> <远程路径>           上传文件");
    println!("  download <远程路径> <本地路径>         下载文件");
    println!();
    println!("配置管理:");
    println!("  save <配置名>                          保存当前连接为配置");
    println!("  list                                   列出所有保存的配置");
    println!();
    println!("其他:");
    println!("  help                                   显示此帮助信息");
    println!("  exit / quit                            退出程序");
    println!();
    println!("环境变量支持:");
    println!("  SSH_HOST, SSH_PORT, SSH_USERNAME, SSH_PASSWORD");
    println!();
    println!("配置文件位置:");
    if let Some(config_dir) = dirs::config_dir() {
        println!("  {}", config_dir.join("ssh_client_rs").join("config.toml").display());
    }
}
