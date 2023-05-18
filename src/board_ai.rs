use crate::board::*;
use itertools::Itertools;
use rand::prelude::*;

#[allow(dead_code)]
pub fn pick_random_move(b: Board, p: Player, rng: &mut ThreadRng) -> Option<(usize, usize)> {
    let m = generate_moves_for_player(b, p);
    let moves = (0..64).filter(|x| (1_u64 << x) & m != 0).collect_vec();
    match moves.len() {
        0 => None,
        l => moves.get(rng.gen_range(0..l)).map(|u| (u / 8, u % 8)),
    }
}
/*
 * negamax is literally minimax, but we're using the fact that min(a,b) = -max(-a,-b)
 * in order to simplify the implementation of the algorithm. (we don't have min/max, only looking for max)
 * or -max
 */
fn negamax(
    my_disks: u64,
    opp_disks: u64,
    max_depth: usize,
    mut alpha: i32,
    beta: i32,
    eval_count: &mut u32,
) -> (i32, Option<usize>) {
    let my_moves = generate_moves(my_disks, opp_disks);
    let opp_moves = generate_moves(opp_disks, my_disks);
    /* forced to pass a move */
    if my_moves == 0 && opp_moves != 0 {
        let (x, _y) = negamax(opp_disks, my_disks, max_depth, -beta, -alpha, eval_count);
        return (-x, None);
    }
    /* game finished or max depth reached */
    let terminal_state = my_moves == 0 && opp_moves == 0;
    if terminal_state || max_depth == 0 {
        *eval_count += 1;
        return (-hval(my_disks, opp_disks, my_moves, opp_moves), None);
    }
    assert!(alpha < beta);
    let mut best_move = None;
    let mut max = -i32::MAX;
    // try my_moves.view_bits().iter_ones() later
    for next_move in (0..64).filter(|i| my_moves & (1_u64 << i) != 0) {
        let [my_disks_new, opp_disks_new] = resolve_move(my_disks, opp_disks, next_move);
        /* we discard opponent's best move and take the negation of his eval */
        let (next_eval, _) = negamax(
            my_disks_new,
            opp_disks_new,
            max_depth.checked_sub(1).unwrap(),
            -beta,
            -alpha,
            eval_count,
        );
        let next_eval = -next_eval;
        /* if this is the best evaluation, update max, alpha and best_move */
        if next_eval > max {
            max = next_eval;
            best_move = Some(next_move);
            alpha = std::cmp::max(next_eval, alpha);
            /* cutoff other branches if alpha >= beta */
            if alpha >= beta {
                break;
            }
        }
    }
    (max, best_move)
}
/*
 * since there is no way of estimating depth from compute time givem,
 * we will just give a limit to how many moves we can evaluate
 * [alpha, beta] = [-i32::MAX, i32::MAX] so that alpha = -beta
 */
fn ids_negamax(
    my_disks: u64,
    opp_disks: u64,
    start_depth: usize,
    total_eval_count: u32,
) -> Option<usize> {
    let mut eval_count: u32 = 0;
    let mut best_move = None;
    for d in start_depth.. {
        if eval_count >= total_eval_count {
            break;
        }
        let (eval, eval_move) =
            negamax(my_disks, opp_disks, d, -i32::MAX, i32::MAX, &mut eval_count);
        best_move = eval_move;
        /* if match has been already decided */
        if eval.abs() > (1 << 20) {
            break;
        }
    }
    best_move
}
pub fn compute_move(
    b: Board,
    p: Player,
    start_depth: usize,
    total_eval_count: u32,
) -> Option<(usize, usize)> {
    if !has_valid_move(b, p) {
        return None;
    }
    let player_idx = p as usize;
    ids_negamax(
        b.disks[player_idx],
        b.disks[player_idx ^ 1],
        start_depth,
        total_eval_count,
    )
    .map(|move_idx| (move_idx / 8, move_idx % 8))
}
