use std::{env, str::FromStr};

use base64::prelude::*;
use clap::{Parser, Subcommand};
use keyring::Entry;
use reqwest::{Client, Response};
use serde_json::json;
use tackle_box::{
    connection::{
        client_service_client::ClientServiceClient, match_monitor_response::EventType,
        MatchMonitorRequest, MatchPlayerRequest,
    },
    contracts::{
        grpc::MatchMetadata,
        payloads::{
            JoinMatchPayload, LoginPayload, LoginResponse, NewAgentPayload, NewMatchPayload,
            RegisterPayload,
        },
    },
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{metadata::MetadataValue, transport::Channel, Request, Status};
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Error)]
enum ClientError {
    #[error("internal error")]
    KeyRing(#[from] keyring::Error),
    #[error("already logined")]
    AlreadyLogin,
    #[error("connection error")]
    Connection(#[from] reqwest::Error),
    #[error("serde error")]
    Serde,
    #[error("api error with {0}")]
    ApiError(String),
    #[error("not login")]
    NotLogin,
    #[error("grpc error")]
    GrpcError(#[from] tonic::transport::Error),
    #[error("grpc connect")]
    GrpcConnectError(#[from] Status),
    #[error("sub process error")]
    SubProcess,
    #[error("std handle error")]
    StdinHandler,
}

const SERVICE_NAME: &str = "TACKLEBOX";
const SERVICE_URL: &str = "127.0.0.1:3000";
const SERVICE_GRPC_URL: &str = "127.0.0.1:50050";
const MAIN_AUTH_USER: &str = "cli_main_token";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 管理 Agent 注册、配置和密钥
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// 管理比赛的创建、状态和监控
    Match {
        #[command(subcommand)]
        command: MatchCommands,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// 通用系统信息和健康检查
    Info,
}

// --- Agent 子命令集 ---
#[derive(Subcommand, Debug)]
enum AgentCommands {
    /// 运行一个本地 Agent 进程，通过 I/O 流接入比赛
    Run {
        /// Agent 可执行文件或脚本的路径 (例如: python my_agent.py)
        path: String,
        /// 使用的 Agent
        #[arg(short, long)]
        agent_id: Uuid,
    },
    // /// 创建一个新的 Agent
    // Create {
    //     /// 新 Agent 的名称
    //     name: String,
    //     game_type: String,
    //     #[arg(short, long, default_value = "0.0.1")]
    //     version: String,
    //     #[arg(short, long)]
    //     description: Option<String>,
    // },
    // /// 更新一个Agent信息
    // Update {
    //     name: String,
    //     game_type: Option<String>,
    //     version: Option<String>,
    //     description: Option<String>,
    // },
    // /// 列出所有Agent
    // List,
}

// --- Match 子命令集 ---
#[derive(Subcommand, Debug)]
enum MatchCommands {
    // /// 提交一个新的比赛请求
    // Create {
    //     /// 比赛名称
    //     match_name: String,
    //     /// 游戏类型
    //     game_type: String,
    //     /// 总共进行的场次
    //     #[arg(default_value_t = 50)]
    //     total_games: i32,
    //     /// 参与的Agent
    //     with_agent_names: Vec<String>,
    //     /// 比赛描述
    //     description: Option<String>,
    //     /// 密码
    //     password: Option<String>,
    // },
    // /// 实时监控一个比赛的状态和日志 (使用 WebSocket)
    // Monitor {
    //     /// 比赛名称
    //     match_name: String,
    // },
    // Join {
    //     /// 比赛名称
    //     match_name: String,
    //     /// Agent名称
    //     agent_name: String,
    // },
    // /// 列出所有正在运行的比赛
    // List,
}

// --- Profile 子命令集 ---
#[derive(Subcommand, Debug)]
enum ProfileCommands {
    /// 注册
    Register {
        /// 用户名称
        username: String,
        /// 用户密码
        password: String,
        /// Email
        email: String,
    },
    Login {
        /// 用户名称
        username: String,
        /// 用户密码
        password: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Profile { command } => match command {
            ProfileCommands::Login { username, password } => {
                handle_login(username, password).await?
            }
            ProfileCommands::Register {
                username,
                password,
                email,
            } => {
                println!(
                    "username: {}, password: {}, email: {}",
                    &username, &password, &email
                );
                handle_register(username, password, email).await?
            }
        },
        Commands::Agent { command } => match command {
            // AgentCommands::Create {
            //     name,
            //     version,
            //     game_type,
            //     description,
            // } => handle_create_agent(name, version, game_type, description).await?,
            AgentCommands::Run { path, agent_id } => handle_run_agent(path, agent_id).await?,
            // AgentCommands::List => handle_list_agents().await?,
            _ => {}
        },
        Commands::Match { command } => match command {
            // MatchCommands::Create {
            //     match_name,
            //     description,
            //     game_type,
            //     total_games,
            //     with_agent_names,
            //     password,
            // } => {
            //     handle_create_match(
            //         match_name,
            //         description,
            //         game_type,
            //         total_games,
            //         with_agent_names,
            //         password,
            //     )
            //     .await?
            // }
            // MatchCommands::Join {
            //     match_name,
            //     agent_name,
            // } => handle_join_match(match_name, agent_name).await?,
            _ => {}
        },
        _ => (),
    }

    Ok(())
}

fn get_auth_token() -> Result<String, ClientError> {
    println!(
        "Keyring GET attempt: Service={}, User={}",
        SERVICE_NAME, MAIN_AUTH_USER
    );
    let token = match Entry::new(SERVICE_NAME, MAIN_AUTH_USER) {
        Ok(entry) => entry.get_password()?,
        Err(_) => match env::var("TACKLE_BOX_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(ClientError::NotLogin);
            }
        },
    };
    Ok(token)
}

async fn process_error(resp: Response) -> Result<Response, ClientError> {
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ClientError::ApiError(format!(
            "Action failed: Status={}, Body={}",
            status, body
        )));
    }
    Ok(resp)
}

async fn handle_login(username: String, password: String) -> Result<(), ClientError> {
    println!("Attempting to log in as {}...", username);
    let client = Client::new();
    let payload = LoginPayload { username, password };
    let resp = client
        .post(format!("http://{}/api/v1/auth/login", SERVICE_URL))
        .json(&json!(payload))
        .send()
        .await?;

    // 检查 HTTP 状态码
    let resp = process_error(resp).await?;

    // 反序列化响应
    let login_response: LoginResponse = resp.json().await?;
    let LoginResponse {
        user_id,
        token,
        token_type: _,
    } = login_response;
    println!("Login successful!");
    match Entry::new(SERVICE_NAME, MAIN_AUTH_USER) {
        Ok(entry) => entry.set_password(&token)?,
        Err(_) => {
            println!(
                "keyring unable work there, please add token to TACKLE_BOX_TOKEN to auth: {}",
                token
            );
        }
    };

    Ok(())
}

async fn handle_register(
    username: String,
    password: String,
    email: String,
) -> Result<(), ClientError> {
    println!("Attempting to register new user {}...", username);
    let client = Client::builder()
        .http1_only()
        .user_agent("Custom-Rust-Client/1.0")
        .build()?;
    let payload = RegisterPayload {
        username,
        password,
        email,
    };
    let resp = client
        .post(format!("http://{}/api/v1/auth/register", SERVICE_URL))
        .json(&json!(payload))
        .send()
        .await?;

    println!("{:?}", &resp);

    let resp = process_error(resp).await?;

    let register_response: LoginResponse = resp.json().await?;
    let LoginResponse {
        user_id,
        token,
        token_type: _,
    } = register_response;
    match Entry::new(SERVICE_NAME, MAIN_AUTH_USER) {
        Ok(entry) => entry.set_password(&token)?,
        Err(_) => {
            println!(
                "keyring unable work there, please add token to TACKLE_BOX_TOKEN to auth: {}",
                token
            );
        }
    };

    println!("Registration successful!");
    Ok(())
}

// async fn handle_create_agent(
//     name: String,
//     version: String,
//     game_type: String,
//     description: Option<String>,
// ) -> Result<(), ClientError> {
//     let token = get_auth_token()?;
//     let client = Client::new();
//     let payload = NewAgentPayload {
//         name,
//         version,
//         game_type,
//         description,
//     };

//     let resp = client
//         .post(format!("http://{}/api/v1/agent/new", SERVICE_URL,))
//         .header("Authorization", format!("Bearer {}", token))
//         .json(&json!(payload))
//         .send()
//         .await?;

//     let resp = process_error(resp).await?;
//     println!("Creating Agent successful!");
//     Ok(())
// }

// async fn handle_list_agents() -> Result<(), ClientError> {
//     let token = get_auth_token()?;
//     let client = Client::new();

//     let resp = client
//         .get(format!("http://{}/api/v1/agent/agents", SERVICE_URL))
//         .header("Authorization", format!("Bearer {}", token))
//         .send()
//         .await?;

//     let resp = process_error(resp).await?;
//     Ok(())
// }

// async fn handle_create_match(
//     name: String,
//     description: Option<String>,
//     game_type: String,
//     total_games: i32,
//     with_agent_names: Vec<String>,
//     password: Option<String>,
// ) -> Result<(), ClientError> {
//     let token = get_auth_token()?;
//     let client = Client::new();
//     let payload = NewMatchPayload {
//         name,
//         game_type,
//         total_games,
//         with_agent_names,
//         password,
//     };

//     let resp = client
//         .post(format!("http://{}/api/v1/match/new", SERVICE_URL,))
//         .json(&json!(payload))
//         .header("Authorization", format!("Bearer {}", token))
//         .send()
//         .await?;
//     let resp = process_error(resp).await?;

//     println!("Creating match successful!");
//     Ok(())
// }

// async fn handle_join_match(match_id: Uuid, agent_ids: Vec<Uuid>) -> Result<(), ClientError> {
//     let token = get_auth_token()?;
//     let client = Client::new();
//     let payload = JoinMatchPayload {
//         match_id,
//         agent_ids,
//     };

//     let resp = client
//         .post(format!("http://{}/api/v1/match/join", SERVICE_URL))
//         .json(&json!(payload))
//         .header("Authorization", format!("Bearer {}", token))
//         .send()
//         .await?;
//     println!("Join match successful!");
//     Ok(())
// }

// async fn handle_monitor_match(match_id: Uuid) -> Result<(), ClientError> {
//     let token = get_auth_token()?;
//     let channel = Channel::from_shared(format!("http://{}", SERVICE_GRPC_URL))
//         .unwrap()
//         .connect()
//         .await?;

//     let auth_token: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();
//     let mut client = ClientServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
//         req.metadata_mut()
//             .insert("authorization", auth_token.clone());
//         Ok(req)
//     });
//     let request = MatchMonitorRequest {};
//     let resp = client.match_monitor(request).await?;
//     let mut in_stream = resp.into_inner();
//     loop {
//         let Ok(Some(msg)) = in_stream.message().await else {
//             break;
//         };
//         let Some(event) = msg.event_type else {
//             break;
//         };
//         match event {
//             EventType::MatchUpdate(up) => {
//                 println!("Current Status: {}", up.current_status);
//                 println!("New Message: {}", up.message);
//             }
//             EventType::ScoreChange(s) => {
//                 println!(
//                     "Turn {} with Score Changes: {:?}",
//                     s.source_i_turn, s.agent_scores
//                 );
//             }
//         }
//     }
//     Ok(())
// }

async fn handle_run_agent(path: String, agent_id: Uuid) -> Result<(), ClientError> {
    let token = get_auth_token()?;

    let channel = Channel::from_shared(format!("http://{}", SERVICE_GRPC_URL))
        .unwrap()
        .connect()
        .await?;

    println!("Launching Agent: {}", path);

    let mut child = Command::new("python")
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|_| ClientError::SubProcess)?;

    let mut python_stdin = child
        .stdin
        .take()
        .ok_or_else(|| ClientError::StdinHandler)?;
    let python_stdout = child
        .stdout
        .take()
        .ok_or_else(|| ClientError::StdinHandler)?;

    let auth_token: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();
    let metadata_bytes = match serde_json::to_vec(&MatchMetadata::MatchPlayer { agent_id }) {
        Ok(bytes) => bytes,
        Err(e) => {
            // 序列化失败，返回 gRPC 错误
            println!("{}", e.to_string());
            return Err(ClientError::ApiError(format!(
                "Metadata serialization failed: {}",
                e
            )));
        }
    };
    let metadata = BASE64_STANDARD.encode(metadata_bytes.clone());
    let message_metadata =
        MetadataValue::from_str(&metadata).map_err(|e| ClientError::ApiError(e.to_string()))?;

    let mut client = ClientServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut()
            .insert("authorization", auth_token.clone());
        req.metadata_mut()
            .insert("x-message-metadata", message_metadata.clone());
        Ok(req)
    });
    debug!("successful make client");

    let (tx, rx) = mpsc::channel(16);
    let request_stream = ReceiverStream::new(rx);

    let response = client
        // request_stream 才是实现了 Stream Trait 的类型
        .match_player(tonic::Request::new(request_stream))
        .await?;

    debug!("get response successful");

    let mut server_responses = response.into_inner();

    let agent_feed_handle = tokio::spawn(async move {
        println!("Task A: Listening for server states...");
        while let Some(response) = server_responses.message().await.transpose() {
            let response = match response {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("gRPC response stream error: {}", e);
                    break;
                }
            };
            debug!("response is {:?}", &response);

            let state = response.state.as_bytes();

            let mut data_to_write = Vec::with_capacity(state.len() + 1);
            data_to_write.extend_from_slice(state);
            data_to_write.extend_from_slice(b"\n");
            if let Err(e) = python_stdin.write_all(&data_to_write).await {
                eprintln!("Failed to write state to Agent stdin (pipe closed): {}", e);
                break;
            }
        }

        // 关键：当 gRPC 响应流结束时，关闭 Agent 进程的 stdin
        // 这会发送 EOF 信号，让 Python 优雅退出
        let _ = python_stdin.shutdown().await;
        println!("Task A finished. Closed Agent stdin.");
    });

    let action_dispatch_handle = tokio::spawn(async move {
        println!("Task B: Listening for Agent actions...");
        let mut reader = BufReader::new(python_stdout);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF: Python Agent 已关闭 stdout
                    println!("Agent stdout stream closed. Exiting Task B.");
                    break;
                }
                Ok(_) => {
                    // 1. 反序列化 Agent Action JSON
                    // 2. 构造 gRPC 请求消息
                    let grpc_req = MatchPlayerRequest {
                        action: line.trim().to_string(),
                    };

                    // 3. 发送给 gRPC Server
                    if let Err(e) = tx.send(grpc_req).await {
                        eprintln!("Failed to send action to gRPC Server (tx closed): {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from Agent stdout: {}", e);
                    break;
                }
            }
        }
    });

    let agent_monitor_handle = tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => println!("Task C: Agent process exited with status: {}", status),
            Err(e) => eprintln!("Task C: Error waiting for Agent process: {}", e),
        }
    });

    // 4. 等待所有任务完成
    let _ = tokio::try_join!(
        agent_feed_handle,
        action_dispatch_handle,
        agent_monitor_handle
    );
    Ok(())
}
