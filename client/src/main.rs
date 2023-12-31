mod config;
mod downloader;
mod source;
mod utils;

use crate::downloader::download_files;
use crate::source::{call_server_update, get_client_config, get_files_list, update_file};
use crate::utils::{countdown, remove_files, setup_logger};
use clap::Parser;
use log::LevelFilter;
use log::{debug, error, info, warn};
use std::path::PathBuf;
use url::Url;

#[derive(Parser)]
#[command(name = "rr_updater Client")]
#[command(author = "kexue <xana278@foxmail.com>")]
#[command(version)]
#[command(about = "从服务端同步文件到本地", long_about = None)]
struct Cli {
    /// 指定配置文件
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// DEBUG 模式
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// 不计算 SHA
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    no_sha: bool,

    /// 不对文件进行操作
    #[arg(long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,

    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    update: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.debug {
        0 => {
            setup_logger(LevelFilter::Info)?;
            info!("Debug mode is OFF");
        }
        1 => {
            setup_logger(LevelFilter::Debug)?;
            debug!("Debug mode is on");
        }
        _ => {
            setup_logger(LevelFilter::Debug)?;
            error!("What r u doing?");
        }
    }

    let config = get_client_config(cli.config.as_deref());

    let host = match Url::parse(config.client.host.as_str()) {
        Ok(r) => r,
        Err(_) => {
            let host = Url::parse("http://127.0.0.1:8520").unwrap();
            error!("host 配置错误, URL不合法");
            warn!("使用默认host: {}", &host);
            host
        }
    };

    let key = config.client.key.clone();

    if cli.update {
        call_server_update(&host, key);
        return Ok(());
    }

    let syncs = &config.sync;

    if cli.no_sha {
        warn!("跳过 SHA 计算");
    } else {
        if syncs.len() == 0 {
            error!("没有配置 sync");
        }
        for sync in syncs {
            // 每个配置
            let local = update_file(sync);

            let files_items = get_files_list(&host, &sync, &local);
            if !cli.dry_run {
                remove_files(&files_items);
                download_files(host.to_owned(), files_items);
            } else {
                warn!("不进行数据更新");
            }
        }
    }
    info!("程序运行完成，5秒后自动退出");
    countdown(5);
    Ok(())
}
