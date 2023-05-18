mod board;
mod communication;
mod board_ai;
mod args;


use board::{Board, Player, init_board, make_move};
use communication::{set_up, read_response, Response, send_message};
use board_ai::{compute_move, pick_random_move};
use args::{Cli, Mode};
use clap::Parser;
use rand::{thread_rng, rngs::ThreadRng};

fn init_state() -> (Board, Player) {
    (init_board(), Player::Black )
}

fn play(mode:Mode, rng:&mut ThreadRng) {
    let (mut b,mut p) = init_state();
    set_up();
    loop {
        match read_response() {
            // first move
            Response::UGO(_turn_time , _game_time) => {
                p = Player::White;
            },
            // consecutive moves
            Response::HEDID(_turn_time , _game_time, opp_move) => {
                if let Some((col,row)) = opp_move {
                    b = make_move(b, p.enemy(), row, col);
                }
            },
            Response::ONEMORE => {
                (b, p) = init_state();
                set_up();
                continue;
            },
            Response::BYE => {
                eprintln!("Finished playing, goodbye!");
                break;
            },
            Response::FAIL => {
                eprintln!("Invalid message recieved, quitting!");
                return;
            }
        }

        let my_move = match mode {
            Mode::Random => pick_random_move(b, p, rng),
            Mode::Minimax { depth } => compute_move(b, p, depth, 1_0),
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
