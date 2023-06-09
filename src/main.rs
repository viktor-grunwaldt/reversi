mod args;
mod board;
mod board_ai;
mod communication;

use args::{Cli, Mode};
use board::{init_board, make_move, Board, Player};
use board_ai::{compute_move, pick_random_move};
use clap::Parser;
use communication::{read_response, send_message, set_up, Response};
use rand::{rngs::ThreadRng, thread_rng};

fn init_state() -> (Board, Player) {
    (init_board(), Player::Black)
}

fn play(mode: Mode, rng: &mut ThreadRng) {
    let (mut b, mut p) = init_state();
    set_up();

    loop {
        match read_response() {
            // first move
            Response::Ugo(_turn_time, _game_time) => {
                p = Player::White;
            }
            // consecutive moves
            Response::Hedid(_turn_time, _game_time, opp_move) => {
                if let Some((col, row)) = opp_move {
                    b = make_move(b, p.enemy(), row, col);
                }
            }
            Response::Onemore => {
                (b, p) = init_state();
                set_up();
                continue;
            }
            Response::Bye => {
                eprintln!("Finished playing, goodbye!");
                break;
            }
            Response::Fail => {
                eprintln!("Invalid message recieved, quitting!");
                return;
            }
        }
        let my_move = match mode {
            Mode::Random => pick_random_move(b, p, rng),
            Mode::Minimax => compute_move(b, p, 4, false),
            Mode::MinimaxSorted => compute_move(b, p, 4, true),
            Mode::TournamentMode => compute_move(b, p, 6, true),
        };

        if let Some((row, col)) = my_move {
            b = make_move(b, p, row, col);
        }

        send_message(my_move, true);
    }
}

fn main() {
    let args = Cli::parse();
    play(args.mode, &mut thread_rng());
}
