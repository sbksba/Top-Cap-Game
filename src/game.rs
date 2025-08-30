use crate::constants::{BOARD_SIZE, GOAL_P1, GOAL_P2, P1_START, P2_START};
use serde::{Deserialize, Serialize};

// --- DATA STRUCTURES ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    P1, // Represented by 🔴
    P2, // Represented by 🔵
}

impl Player {
    // Returns the opponent player
    pub fn opponent(&self) -> Player {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

// To represent the state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Ongoing,
    Won(Player),
}

// Coordinates on the board (0-6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

// This is the payload the client sends to make a move.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MoveRequest {
    pub from: Position,
    pub to: Position,
}

// Main game structure
#[derive(Debug, Serialize, Clone)]
pub struct Game {
    pub board: [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    pub current_player: Player,
    pub status: GameStatus,
}

// --- GAME LOGIC ---

impl Game {
    // Creates a new game
    pub fn new() -> Self {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];

        // ---- Place P1 -------------------------------------------------------
        for &(r, c) in P1_START.iter() {
            board[r][c] = Some(Player::P1);
        }

        // ---- Place P2 -------------------------------------------------------
        for &(r, c) in P2_START.iter() {
            board[r][c] = Some(Player::P2);
        }

        Game {
            board,
            current_player: Player::P1,
            status: GameStatus::Ongoing,
        }
    }

    // Returns the position of the base ("bottle") for a given player
    pub fn get_goal_pos(player: Player) -> Position {
        match player {
            Player::P1 => Position {
                row: GOAL_P1.0,
                col: GOAL_P1.1,
            },
            Player::P2 => Position {
                row: GOAL_P2.0,
                col: GOAL_P2.1,
            },
        }
    }

    /// Attempts to make a move. Updates the game state internally.
    pub fn make_move(&mut self, from: Position, to: Position) -> Result<(), &'static str> {
        // Validation 1: The starting square must contain a piece of the current player
        match self.board[from.row][from.col] {
            Some(p) if p == self.current_player => {}
            _ => return Err("Invalid starting square or that's not your piece."),
        }

        // Validation 2: The move must be in the list of valid moves
        let valid_moves = self.get_valid_moves_for_piece(from);
        if !valid_moves.contains(&to) {
            return Err("Illegal move.");
        }

        // The move is valid, execute it
        self.board[to.row][to.col] = self.board[from.row][from.col].take();

        // Victory check 1: Reach the opponent's base
        if to == Self::get_goal_pos(self.current_player.opponent()) {
            self.status = GameStatus::Won(self.current_player);
            return Ok(());
        }

        // Pass to the next player
        self.current_player = self.current_player.opponent();

        // Victory check 2: The opponent has no more possible moves
        if !self.has_any_valid_moves(self.current_player) {
            self.status = GameStatus::Won(self.current_player.opponent());
        }

        Ok(())
    }

    /// Calculates all valid moves for a piece at a given position.
    pub fn get_valid_moves_for_piece(&self, pos: Position) -> Vec<Position> {
        let mut moves = Vec::new();
        let move_dist = self.count_neighbors(pos) as isize;

        if move_dist == 0 {
            return moves; // A piece with no neighbors cannot move
        }

        // Test the 8 possible directions
        for &dir in &[
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ] {
            let new_row_isize = pos.row as isize + dir.0 * move_dist;
            let new_col_isize = pos.col as isize + dir.1 * move_dist;

            if Self::is_on_board(new_row_isize, new_col_isize) {
                let target_pos = Position {
                    row: new_row_isize as usize,
                    col: new_col_isize as usize,
                };
                if self.is_move_valid(pos, target_pos) {
                    moves.push(target_pos);
                }
            }
        }
        moves
    }

    /// Checks if a player has at least one valid move on the entire board.
    pub fn has_any_valid_moves(&self, player: Player) -> bool {
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                if self.board[r][c] == Some(player)
                    && !self
                        .get_valid_moves_for_piece(Position { row: r, col: c })
                        .is_empty()
                {
                    return true;
                }
            }
        }
        false
    }

    // --- HELPER VALIDATION FUNCTIONS ---

    /// Counts the number of adjacent pieces to a square.
    pub fn count_neighbors(&self, pos: Position) -> u8 {
        let mut count = 0;
        for r_offset in -1..=1 {
            for c_offset in -1..=1 {
                if r_offset == 0 && c_offset == 0 {
                    continue;
                } // Ignore the square itself

                let check_row = pos.row as isize + r_offset as isize;
                let check_col = pos.col as isize + c_offset as isize;

                if Self::is_on_board(check_row, check_col)
                    && self.board[check_row as usize][check_col as usize].is_some()
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// Checks if a move from `from` to `to` respects all rules.
    fn is_move_valid(&self, from: Position, to: Position) -> bool {
        // Must be on the board
        if !Self::is_on_board(to.row as isize, to.col as isize) {
            return false;
        }
        // The destination square must be empty
        if self.board[to.row][to.col].is_some() {
            return false;
        }
        // Cannot move to its own base
        if to == Self::get_goal_pos(self.current_player) {
            return false;
        }
        // Must have a clear path
        if !self.is_path_clear(from, to) {
            return false;
        }

        true
    }

    /// Checks that the path between two points is empty (no jumping).
    fn is_path_clear(&self, from: Position, to: Position) -> bool {
        let dr = (to.row as isize - from.row as isize).signum(); // Row direction: -1, 0, or 1
        let dc = (to.col as isize - from.col as isize).signum(); // Column direction: -1, 0, or 1
        let mut current_pos = from;

        // Move square by square until the second-to-last square of the path
        while (current_pos.row as isize + dr) as usize != to.row
            || (current_pos.col as isize + dc) as usize != to.col
        {
            current_pos.row = (current_pos.row as isize + dr) as usize;
            current_pos.col = (current_pos.col as isize + dc) as usize;
            if self.board[current_pos.row][current_pos.col].is_some() {
                return false; // Obstacle found
            }
        }
        true
    }

    /// Checks if coordinates (as i8 for calculations) are on the board.
    fn is_on_board(row: isize, col: isize) -> bool {
        let max = BOARD_SIZE as isize;
        (0..max).contains(&row) && (0..max).contains(&col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper – creates a fresh game with the standard initial placement
    fn setup_game() -> Game {
        Game::new()
    }

    // Initial board layout
    #[test]
    fn test_initial_board_setup() {
        let game = setup_game();

        // Verify the four P1 pieces
        for &(r, c) in crate::constants::P1_START.iter() {
            assert_eq!(
                game.board[r][c],
                Some(Player::P1),
                "P1 should be at ({},{})",
                r,
                c
            );
        }

        // Verify the four P2 pieces
        for &(r, c) in crate::constants::P2_START.iter() {
            assert_eq!(
                game.board[r][c],
                Some(Player::P2),
                "P2 should be at ({},{})",
                r,
                c
            );
        }

        // Ensure an arbitrary empty square (e.g., top‑left corner) is indeed empty
        assert_eq!(game.board[0][0], None);
    }

    // The first turn belongs to P1
    #[test]
    fn test_initial_turn_is_p1() {
        let game = setup_game();
        assert_eq!(game.current_player, Player::P1);
    }

    // Neighbor counting – using the predefined start positions
    #[test]
    fn test_count_neighbors() {
        let game = setup_game();

        // (0,3) has exactly one neighbor at (1,2)
        let pos = Position { row: 0, col: 3 };
        assert_eq!(game.count_neighbors(pos), 1);

        // (3,0) has exactly one neighbor at (2,1)
        let pos = Position { row: 3, col: 0 };
        assert_eq!(game.count_neighbors(pos), 1);

        // (2,1) has two neighbors ((1,2) and (3,0))
        let pos = Position { row: 2, col: 1 };
        assert_eq!(game.count_neighbors(pos), 2);
    }

    // Valid move (single step left)
    #[test]
    fn test_valid_move() {
        let mut game = setup_game();
        let from = Position { row: 0, col: 3 };
        let to = Position { row: 0, col: 2 }; // one step left

        let result = game.make_move(from, to);
        assert!(result.is_ok());

        // Origin must become empty
        assert_eq!(game.board[from.row][from.col], None);
        // Destination must contain P1's piece
        assert_eq!(game.board[to.row][to.col], Some(Player::P1));

        // Turn should switch to P2
        assert_eq!(game.current_player, Player::P2);
    }

    // Invalid move – destination already occupied
    #[test]
    fn test_invalid_move_occupied_destination() {
        let mut game = setup_game();
        let from = Position { row: 0, col: 3 };
        // (1,2) is already occupied by P1 at the start
        let to = Position { row: 1, col: 2 };

        let result = game.make_move(from, to);
        assert!(result.is_err());
    }

    // Victory by reaching the opponent's goal square
    #[test]
    fn test_win_by_reaching_goal_corrected() {
        let mut game = setup_game();
        game.current_player = Player::P1;

        let goal = Position {
            row: crate::constants::GOAL_P2.0,
            col: crate::constants::GOAL_P2.1,
        };

        // Choose a start square that is aligned with the goal
        //    (same diagonal, same row, or same column)
        // We walk backwards along the diagonal from the goal until we stay
        // inside the board.
        let mut offset = 1usize;
        let start = loop {
            if goal.row >= offset && goal.col >= offset {
                break Position {
                    row: goal.row - offset,
                    col: goal.col - offset,
                };
            }
            offset += 1;
            if offset > BOARD_SIZE {
                panic!("Unable to find a start square aligned with the goal");
            }
        };

        game.board = [[None; BOARD_SIZE]; BOARD_SIZE];
        game.board[start.row][start.col] = Some(Player::P1);

        // Determine how many squares the piece will travel.
        // This is also the exact number of adjacent opponent pieces required.
        let steps = ((goal.row as isize - start.row as isize).abs())
            .max((goal.col as isize - start.col as isize).abs()) as usize;

        // Place exactly `steps` Player 2 pieces around the start piece
        let mut placed = 0usize;
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let r = (start.row as isize + dr) as usize;
                let c = (start.col as isize + dc) as usize;
                if r < BOARD_SIZE && c < BOARD_SIZE && placed < steps {
                    game.board[r][c] = Some(Player::P2);
                    placed += 1;
                }
            }
        }

        // If, for some reason, `steps` exceeds the eight possible neighbours
        // (this can only happen on extremely small boards), fill the remaining
        // required pieces along the same row.
        let mut extra = 0usize;
        while placed < steps {
            let r = start.row;
            let c = (start.col + extra + 1) % BOARD_SIZE;
            if game.board[r][c].is_none() {
                game.board[r][c] = Some(Player::P2);
                placed += 1;
            }
            extra += 1;
        }

        assert!(
            game.board[goal.row][goal.col].is_none(),
            "Goal square was already occupied"
        );

        let result = game.make_move(start, goal);
        assert!(
            result.is_ok(),
            "The move should have been accepted, but it failed: {:?}",
            result.err()
        );
        assert_eq!(
            game.status,
            GameStatus::Won(Player::P1),
            "After reaching the opponent’s goal, the game status should be Won(P1)"
        );
    }

    // Victory because the opponent has no legal moves left
    #[test]
    fn test_win_by_opponent_no_moves() {
        let mut game = setup_game();
        game.current_player = Player::P1;

        // Build a situation where P2 cannot move at all.
        // Place P2 in the top‑left corner and surround it with P1 pieces.
        game.board = [[None; BOARD_SIZE]; BOARD_SIZE];
        game.board[0][0] = Some(Player::P2); // corner
        game.board[0][1] = Some(Player::P1);
        game.board[1][0] = Some(Player::P1);
        game.board[1][1] = Some(Player::P1);

        // Perform a valid move for P1 (e.g., (3,0) → (4,0))
        game.board[3][0] = Some(Player::P1);
        game.board[2][1] = Some(Player::P1);
        let from = Position { row: 3, col: 0 };
        let to = Position { row: 4, col: 0 };

        let result = game.make_move(from, to);
        assert!(result.is_ok());

        // After this move, P2 has no possible moves → P1 wins
        assert_eq!(game.status, GameStatus::Won(Player::P1));
    }
}
