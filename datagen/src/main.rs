use std::{path::PathBuf, sync::Arc};

use blackmarlin::bm::bm_runner::time::TimeManagementInfo;
use clap::Parser;

use crate::datagen::DataGenOptions;

mod datagen;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Maximum search depth
    #[arg(short, long)]
    depth: u32,

    /// Standard Chess = 0, FRC = 1, DFRC = 2
    #[arg(short, long, default_value_t = 0)]
    variant: u8,

    #[arg(long)]
    random_move_chance: Option<f32>,

    /// Max amount of nodes to search (quits the depth after reaching limit)
    #[arg(short, long)]
    nodes: Option<u64>,

    /// Amount of plies to randomize
    #[arg(short, long)]
    random_plies: Option<usize>,

    /// Number of FENS to generate
    #[arg(short, long)]
    pos_count: Option<usize>,

    /// Number of threads to use
    #[arg(short, long)]
    threads: usize,

    /// Text file to append the games to
    #[arg(short, long, value_name = "FILE")]
    out: PathBuf,

    /// Write to file interval in seconds
    #[arg(short, long, default_value_t = 30)]
    write_interval: u64,

    /// Adjudicate draws after 80 plies if score has been 0 for 8 consecutive plies
    #[arg(long, default_value_t = false)]
    draw_adj: bool,
}

fn main() {
    let args = Args::parse();

    let options = DataGenOptions {
        threads: args.threads,
        random_plies: args.random_plies.unwrap_or(0),
        random_move_chance: args.random_move_chance.unwrap_or(0.0),
        pos_count: args.pos_count.unwrap_or(usize::MAX),
        variant: args.variant,
        out: args.out,
        interval: args.write_interval,
        draw_adj: args.draw_adj,
    };
    let tm_options = [
        TimeManagementInfo::MaxDepth(args.depth),
        TimeManagementInfo::MaxNodes(args.nodes.unwrap_or(u64::MAX)),
    ]
    .into_iter()
    .collect::<Arc<_>>();
    datagen::gen_eval(&tm_options, options);
}
