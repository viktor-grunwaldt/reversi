use itertools::Itertools;
use std::{fmt, ops::BitAnd};

/// Represents the current state of the game.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Player {
    Black,
    White,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Board {
    pub disks: [u64; 2],
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut charr = [['_'; 8]; 8];
        let mut bl = self.disks[0];
        let mut wh = self.disks[1];

        for row in &mut charr {
            for j in row.iter_mut().take(8) {
                *j = match (bl & 1, wh & 1) {
                    (0, 0) => '.',
                    (1, 0) => '#',
                    (0, 1) => 'O',
                    _ => panic!(),
                };
                bl >>= 1;
                wh >>= 1;
            }
        }
        write!(f, "{}", charr.map(|x| x.iter().join("")).join("\n"))
    }
}

impl Player {
    pub fn enemy(self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}
#[allow(dead_code)]
pub fn show_possible_moves(b: Board, p: Player) -> String {
    let mut charr = b
        .to_string()
        .split_ascii_whitespace()
        .map(|x| x.chars().collect_vec())
        .collect_vec();
    let idx = p as usize;
    let mut moves = generate_moves(b.disks[idx], b.disks[idx ^ 1]);
    for row in charr.iter_mut().take(8) {
        for e in row {
            if moves.bitand(1) != 0 {
                *e = '*';
            }
            moves >>= 1;
        }
    }

    charr.iter().map(|x| x.iter().join("")).join("\n")
}
static MASKS: [u64; 8] = [
    0x7F7F7F7F7F7F7F7F, /* Right. */
    0x007F7F7F7F7F7F7F, /* Down-right. */
    0xFFFFFFFFFFFFFFFF, /* Down. */
    0x00FEFEFEFEFEFEFE, /* Down-left. */
    0xFEFEFEFEFEFEFEFE, /* Left. */
    0xFEFEFEFEFEFEFE00, /* Up-left. */
    0xFFFFFFFFFFFFFFFF, /* Up. */
    0x7F7F7F7F7F7F7F00, /* Up-right. */
];
static LSHIFTS: [u64; 8] = [
    0, /* Right. */
    0, /* Down-right. */
    0, /* Down. */
    0, /* Down-left. */
    1, /* Left. */
    9, /* Up-left. */
    8, /* Up. */
    7, /* Up-right. */
];
static RSHIFTS: [u64; 8] = [
    1, /* Right. */
    9, /* Down-right. */
    8, /* Down. */
    7, /* Down-left. */
    0, /* Left. */
    0, /* Up-left. */
    0, /* Up. */
    0, /* Up-right. */
];
// XX...
// X....
pub static CORNER_MASK: u64 = 0x8100_0000_0000_0081;

fn shift(disks: u64, dir: usize) -> u64 {
    MASKS[dir]
        & match dir {
            ..=3 => disks >> RSHIFTS[dir],
            4..=7 => disks << LSHIFTS[dir],
            _ => panic!(),
        }
}
// https://www.chessprogramming.org/Dumb7Fill
pub fn generate_moves(my_disks: u64, opp_disks: u64) -> u64 {
    let mut legal_moves: u64 = 0;
    let mut x;
    let empty_cells = !(my_disks | opp_disks);
    // if compiler is smart, he will either unroll the loop or use simd instructions
    for dir in 0..8 {
        /* Get opponent disks adjacent to my disks in direction dir. */
        x = shift(my_disks, dir) & opp_disks;
        /* Add opponent disks adjacent to those, and so on. */
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;

        legal_moves |= shift(x, dir) & empty_cells;
    }
    legal_moves
}

pub fn generate_moves_for_player(b: Board, p: Player) -> u64 {
    let idx = p as usize;
    generate_moves(b.disks[idx], b.disks[idx ^ 1])
}

pub fn has_valid_move(b: Board, p: Player) -> bool {
    generate_moves_for_player(b, p) != 0
}

fn is_valid_move(b: Board, p: Player, row: usize, col: usize) -> bool {
    assert!((0..8).contains(&row) && (0..8).contains(&col));
    let mask = 1 << (row * 8 + col);
    (generate_moves_for_player(b, p) & mask) != 0
}

pub fn resolve_move(my_disks: u64, opp_disks: u64, idx: usize) -> [u64; 2] {
    assert!(idx < 64);
    let mut x: u64;
    let mut bounding_disk;
    let mut captured_disks: u64 = 0;
    let new_disk: u64 = 1 << idx;
    let my_new_disks = my_disks | new_disk;
    for dir in 0..8 {
        /* Find opponent disk adjacent to the new disk. */
        x = shift(new_disk, dir) & opp_disks;

        /* Add any adjacent opponent disk to that one, and so on. */
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;
        x |= shift(x, dir) & opp_disks;

        /* Determine whether the disks were captured. */
        bounding_disk = shift(x, dir) & my_new_disks;
        captured_disks |= if bounding_disk != 0 { x } else { 0 };
    }
    /* A valid move must capture disks. */
    assert!(captured_disks != 0);
    let my_disks = my_new_disks ^ captured_disks;
    let opp_disks = opp_disks ^ captured_disks;
    /* The sets must still be disjoint. */
    assert!((my_disks & opp_disks) == 0);
    [my_disks, opp_disks]
}

pub fn disks_on_border(my_disks: u64, opp_disks: u64) -> (u64, u64) {
    let empty_cells = !(my_disks | opp_disks);
    let mut my_borders = 0;
    let mut opp_borders = 0;
    for i in 0..8 {
        let x = shift(empty_cells, i);
        my_borders |= x & my_disks;
        opp_borders |= x & opp_disks;
    }

    (my_borders, opp_borders)
}

pub fn make_move(b: Board, p: Player, row: usize, col: usize) -> Board {
    let v = is_valid_move(b, p, row, col);
    if !v {
        eprintln!("invalid move, {b} {p:?} {row} {col}");
    }
    assert!(v);
    let player_idx = p as usize;
    let new_disks = resolve_move(b.disks[player_idx], b.disks[player_idx ^ 1], row * 8 + col);
    Board {
        disks: [new_disks[player_idx], new_disks[player_idx ^ 1]],
    }
}

pub fn init_board() -> Board {
    Board {
        disks: [0x0000_0010_0800_0000, 0x0000_0008_1000_0000],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn foo() {
        todo!()
    }
}
