use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;
use clap::{Parser,Args ,Subcommand};
#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    SSHConfig(SSHConfigArgs),
    SSHMode(SSHModeArgs),
    SSHClose,
}
#[derive(Args, Debug)]
struct SSHConfigArgs {
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long)]
    username: String,
    #[arg(long)]
    password: Option<String>,
}
#[derive(Args, Debug)]
struct SSHModeArgs {
    #[arg(long)]     
    mode: String,
    #[arg(long)]
    path: std::path::PathBuf,
    #[arg(long)]
    shell:String,
}
fn establish_ssh_session(args: &SSHConfigArgs) -> Result<Session, Box<dyn std::error::Error>> {
    let tcp = TcpStream::connect(format!("{}:{}", args.host, args.port.unwrap_or(22)))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    if let Some(ref password) = args.password {
        session.userauth_password(&args.username, password)?;
    } else {
        session.userauth_agent(&args.username)?;
    }
    if !session.authenticated() {
        return Err(Box::from("Authentication failed"));
    }
    print!("Authenticated successfully\n");
    print!("Session established with {}:{}\n", args.host, args.port.unwrap_or(22));
    Ok(session)
}
fn establish_ssh_mode(session: &mut Session, args: &SSHModeArgs) -> Result<String, Box<dyn std::error::Error>> {
    match args.mode.as_str() {
        "sftp" => {
            let mut sftp = session.scp_send(&args.path, 0o644, 10, None)?;
            let mut localfile = std::fs::File::open(&args.path)?;
            std::io::copy(&mut localfile, &mut sftp)?;
            Ok("SFTP transfer completed".to_string())
        },
        "shell" => {
            let mut channel = session.channel_session()?;
            channel.exec(&args.shell)?;
            let mut s = String::new();
            channel.read_to_string(&mut s)?;
            println!("{}", s);
            Ok("Shell command executed".to_string())
        },
        _ => {
            Err(Box::from("Unsupported mode"))
        }
    }
}
fn handle_ssh_close(session: Session) -> Result<(), Box<dyn std::error::Error>> {
    session.disconnect(None, "Closing session", None)?;
    println!("SSH session closed");
    Ok(())
}
fn main() {
    println!("这是一个SSH客户端工具");
    println!("请输入相应的命令来操作SSH连接");
    println!("输入 'exit' 或 'quit' 退出程序\n");

    let mut session: Option<Session> = None;

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            eprintln!("读取输入失败");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            if let Some(s) = session {
                let _ = handle_ssh_close(s);
            }
            println!("再见！");
            break;
        }

        // 将输入解析为参数数组
        let args: Vec<&str> = input.split_whitespace().collect();
        let args_with_program = std::iter::once("ssh-client").chain(args.iter().copied()).collect::<Vec<_>>();

        let cli = match Cli::try_parse_from(args_with_program) {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
    match &cli.command {
        Commands::SSHConfig(args) => {
            match establish_ssh_session(args) {
                Ok(s) => {
                    session = Some(s);
                    println!("SSH session established");
                },
                Err(e) => {
                    eprintln!("Error establishing SSH session: {}", e);
                }
            }
        },
        Commands::SSHMode(args) => {
          if session.is_none() {
                eprintln!("SSHMode command requires an established session");
          }else{
            match establish_ssh_mode(session.as_mut().unwrap(), args) {
                Ok(msg) => {
                    println!("{}", msg);
                },
                Err(e) => {
                    eprintln!("Error establishing SSH mode: {}", e);
                }
            }
          }
        },
        Commands::SSHClose => {
            if session.is_none() {
                eprintln!("SSHClose command requires an established session");
            } else {
                match handle_ssh_close(session.take().unwrap()) {
                    Ok(_) => {
                        println!("SSH session closed successfully");
                        session = None;
                    },
                    Err(e) => {
                        eprintln!("Error closing SSH session: {}", e);
                        session = None;
                    }
                }
            }
        }
    }
   }
}
