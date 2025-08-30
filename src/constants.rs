pub const BOARD_SIZE: usize = 6;
pub const P1_START: [(usize, usize); 4] = [(0, 3), (1, 2), (2, 1), (3, 0)];
pub const P2_START: [(usize, usize); 4] = [
    (BOARD_SIZE - 1, BOARD_SIZE - 4),
    (BOARD_SIZE - 2, BOARD_SIZE - 3),
    (BOARD_SIZE - 3, BOARD_SIZE - 2),
    (BOARD_SIZE - 4, BOARD_SIZE - 1),
];
pub const GOAL_P1: (usize, usize) = (0, 0);
pub const GOAL_P2: (usize, usize) = (BOARD_SIZE - 1, BOARD_SIZE - 1);
