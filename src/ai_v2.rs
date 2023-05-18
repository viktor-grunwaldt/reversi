use std::{cmp::max, ops::Shl};

use itertools::Itertools;

use crate::board::*;

fn game_state(my_disks: u64, opp_disks: u64) -> [i32; 3] {
    let stone_count = (my_disks | opp_disks).count_ones();
    match stone_count {
        ..=20 => [50, 0, 0],
        21..=55 => [20, 10, 100],
        _ => [100, 500, 500],
    }
}

fn depth_from_state(my_disks: u64, opp_disks: u64) -> usize {
    let stone_count = (my_disks | opp_disks).count_ones();
    match stone_count {
        ..=20 => 2,
        21..=55 => 4,
        _ => 8,
    }
}

fn parity(my: u64, opp: u64) -> i32 {
    ((my | opp).count_zeros() as i32 % 2) * 2 - 1
}

fn hval2(my_disks: u64, opp_disks: u64, my_moves: u64, opp_moves: u64) -> i32 {
    let disk_diff = my_disks.count_ones() as i32 - opp_disks.count_ones() as i32;
    if my_moves == 0 && opp_moves == 0 {
        return disk_diff << 20;
    }
    let weights = game_state(my_disks, opp_disks);
    let my_corners = (my_disks & CORNER_MASK).count_ones() as i32;
    let opp_corners = (opp_disks & CORNER_MASK).count_ones() as i32;
    let mobility = my_moves.count_ones() as i32 - opp_moves.count_ones() as i32;
    let my_bad_corners = (my_disks & BAD_CORNER_MASK).count_ones() as i32;
    let opp_bad_corners = (opp_disks & BAD_CORNER_MASK).count_ones() as i32;
    (my_corners - opp_corners).shl(8) - (my_bad_corners - opp_bad_corners).shl(10)
        + mobility * weights[0]
        + disk_diff * weights[1]
        + parity(my_disks, opp_disks) * weights[2]
}

fn hval(my_disks: u64, opp_disks: u64, my_moves: u64, opp_moves: u64) -> i32 {
    /* end of game, win bonus is ~1 million per disk */
    if my_moves == 0 && opp_moves == 0 {
        return (my_disks.count_ones() as i32 - opp_disks.count_ones() as i32) << 20;
    }
    let my_corners = (my_disks & CORNER_MASK).count_ones() as i32;
    let opp_corners = (opp_disks & CORNER_MASK).count_ones() as i32;
    let mut score = 0;
    let (my_borders, opp_borders) = disks_on_border(my_disks, opp_disks);
    score += (my_corners - opp_corners) * 16;
    score += (my_moves.count_ones() as i32 - opp_moves.count_ones() as i32) * 2;
    score += opp_borders.count_ones() as i32 - my_borders.count_ones() as i32;
    score
}

fn negamax_ab(
    my_disks: u64,
    opp_disks: u64,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    color: i32,
) -> i32 {
    let my_moves = generate_moves(my_disks, opp_disks);
    let opp_moves = generate_moves(opp_disks, my_disks);
    // assert!(my_moves != opp_moves);
    if depth == 0 || my_moves == 0 {
        return color * hval(my_disks, opp_disks, my_moves, opp_moves);
    }
    let mut best_value = -i32::MAX;
    for idx in 0..64 {
        if (1_u64 << idx) & my_moves == 0 {
            continue;
        }
        let [my_disks, opp_disks] = resolve_move(my_disks, opp_disks, idx);
        let val = -negamax_ab(opp_disks, my_disks, depth - 1, -beta, -alpha, -color);
        best_value = max(best_value, val);
        alpha = max(alpha, val);
        if alpha >= beta {
            break;
        }
    }
    best_value
}

pub fn negamax_ab_sorted(
    my_disks: u64,
    opp_disks: u64,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    color: i32,
) -> i32 {
    let my_moves = generate_moves(my_disks, opp_disks);
    let opp_moves = generate_moves(opp_disks, my_disks);
    if depth == 0 || my_moves == 0 {
        return color * hval2(my_disks, opp_disks, my_moves, opp_moves);
    }
    let mut best_value = -i32::MAX;

    //sorting moves
    let sorted_moves = (0..64)
        .filter_map(|idx| (((1_u64 << idx) & my_moves) != 0).then_some(idx))
        .sorted_by_cached_key(|idx| {
            let [my, opp] = resolve_move(my_disks, opp_disks, *idx);
            hval2(my, opp, my_moves, opp_moves)
        })
        .collect_vec();

    for idx in sorted_moves {
        let [my_disks, opp_disks] = resolve_move(my_disks, opp_disks, idx);
        let val = -negamax_ab_sorted(my_disks, opp_disks, depth - 1, -beta, -alpha, -color);
        best_value = max(best_value, val);
        alpha = max(alpha, val);
        if alpha >= beta {
            break;
        }
    }
    best_value
}

pub fn compute_move(b: Board, p: Player, max_depth: usize) -> Option<(usize, usize)> {
    let mut max_points = -i32::MAX;
    let mut best_move = None;
    let my_moves = generate_moves_for_player(b, p);
    let p_idx = p as usize;
    for idx in 0..64 {
        if (1_u64 << idx) & my_moves == 0 {
            continue;
        }
        let eval = negamax_ab(
            b.disks[p_idx],
            b.disks[p_idx ^ 1],
            depth_from_state(b.disks[p_idx], b.disks[p_idx ^ 1]),
            -i32::MAX,
            i32::MAX,
            1,
        );
        if eval > max_points {
            max_points = eval;
            best_move = Some((idx / 8, idx % 8));
        }
    }
    best_move
}
