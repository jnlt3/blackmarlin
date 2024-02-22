use clap::Parser;
use cozy_chess::{Board, Move};
use rand::{thread_rng, Rng};
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Minimum ply to add to dataset
    #[arg(long, default_value_t = 16)]
    min_ply: usize,

    /// Maximum ply to add to dataset
    #[arg(long)]
    max_ply: Option<usize>,

    /// Maximum eval to add to dataset
    #[arg(long, default_value_t = 3000)]
    max_eval: i16,

    /// Filter out positions where the best move is a capture
    #[arg(long)]
    filter_cap: bool,

    /// Filter out positions in check
    #[arg(long)]
    filter_in_check: bool,

    /// Filter out positions where the best move is a promotion
    #[arg(long)]
    filter_promo: bool,

    /// Filter out positions where the best move gives check
    #[arg(long)]
    filter_gives_check: bool,

    /// Filter out draws chance
    #[arg(long)]
    draw_filter_chance: Option<f32>,

    /// Filter out positions that don't match next position's eval by given margin
    #[arg(long, value_name = "EVAL")]
    blunder_filter_eval: Option<u16>,

    /// Random filter out chance
    #[arg(long)]
    filter_random: Option<f32>,

    /// Text file to read from
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Text file to write to
    #[arg(short, long, value_name = "FILE")]
    out: PathBuf,

    /// Number of threads to use
    #[arg(short, long)]
    threads: usize,

    /// Information printing interval
    #[arg(short, long, value_name = "LINES", default_value_t = 1000)]
    info_interval: usize,
}

fn main() {
    let args = Args::parse();
    let in_file = BufReader::new(OpenOptions::new().read(true).open(&args.input).unwrap());
    let mut out_file = BufWriter::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(&args.out)
            .unwrap(),
    );

    let mut added = 0;
    for (index, line) in in_file.lines().enumerate() {
        let Ok(line) = line else {
            eprintln!("{:?}", line);
            continue;
        };
        let mut split = line.split(" | ");
        let fen = split.next().unwrap();
        let eval = split.next().unwrap().parse::<i16>().unwrap();
        let wdl = split.next().unwrap(); // "0" "0.5" or "1"
        let board = Board::from_str(&fen).unwrap();
        let mv = split.next().unwrap().parse::<Move>().unwrap();
        let ply = split.next().unwrap().parse::<usize>().unwrap();

        if ply < args.min_ply || ply > args.max_ply.unwrap_or(usize::MAX) {
            continue;
        }
        if eval.abs() > args.max_eval {
            continue;
        }
        if args.filter_promo && mv.promotion.is_some() {
            continue;
        }
        if args.filter_in_check && !board.checkers().is_empty() {
            continue;
        }
        if args.filter_gives_check {
            let mut next_board = board.clone();
            next_board.play(mv);
            if !next_board.checkers().is_empty() {
                continue;
            }
        }
        if args.filter_cap && board.colors(!board.side_to_move()).has(mv.to) {
            continue;
        }
        if let Some(chance) = args.filter_random {
            if thread_rng().gen::<f32>() < chance {
                continue;
            }
        }
        if let Some(chance) = args.draw_filter_chance {
            if wdl == "0.5" && thread_rng().gen::<f32>() < chance {
                continue;
            }
        }
        added += 1;
        if (index + 1) % args.info_interval == 0 {
            println!("Added {} positions", added);
            println!(
                "Filter rate: {:.4}%",
                (index - added) as f64 / index as f64 * 100.0
            );
        }
        out_file
            .write(format!("{fen} | {eval} | {wdl}\n").as_bytes())
            .unwrap();
    }
}
