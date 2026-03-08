use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[command(
    name = "todox",
    version,
    about = "Local terminal Todo tool / 本地终端 Todo 工具"
)]
pub struct Cli {
    #[arg(
        long,
        value_name = "PATH",
        help = "Override data file path / 覆盖默认数据文件路径"
    )]
    pub data_file: Option<PathBuf>,
}
