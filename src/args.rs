use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Chooses a legal move with equal probability
    Random,
    /// Uses eval + negamax + alpha-beta pruning to determine next move
    Minimax {
        #[arg(short, long, default_value_t = 1)]
        depth: usize,
    },
    /// Same as minimax, but prioritises evaluating better moves
    MinimaxSorted {
        #[arg(short, long, default_value_t = 1)]
        depth: usize,
    }
}
/// Reversi agent designed to be played with ai_dueler.py
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Select AI mode
    #[command(subcommand)]
    pub mode: Mode,
}
