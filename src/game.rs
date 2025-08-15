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
    pub board: [[Option<Player>; 7]; 7],
    pub current_player: Player,
    pub status: GameStatus,
}

// --- GAME LOGIC ---

impl Game {
    // Creates a new game
    pub fn new() -> Self {
        let mut board = [[None; 7]; 7];

        // Player 1's starting positions (near corner A1 / 0,0)
        board[0][3] = Some(Player::P1);
        board[1][2] = Some(Player::P1);
        board[2][1] = Some(Player::P1);
        board[3][0] = Some(Player::P1);

        // Player 2's starting positions (near corner G7 / 6,6)
        board[3][6] = Some(Player::P2);
        board[4][5] = Some(Player::P2);
        board[5][4] = Some(Player::P2);
        board[6][3] = Some(Player::P2);

        Game {
            board,
            current_player: Player::P1,
            status: GameStatus::Ongoing,
        }
    }

    // Returns the position of the base ("bottle") for a given player
    pub fn get_goal_pos(player: Player) -> Position {
        match player {
            Player::P1 => Position { row: 0, col: 0 },
            Player::P2 => Position { row: 6, col: 6 },
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
        for r in 0..7 {
            for c in 0..7 {
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
        (0..7).contains(&row) && (0..7).contains(&col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_game() -> Game {
        Game::new()
    }

    #[test]
    fn test_initial_board_setup() {
        let game = setup_game();
        assert_eq!(game.board[0][3], Some(Player::P1));
        assert_eq!(game.board[3][0], Some(Player::P1));
        assert_eq!(game.board[3][6], Some(Player::P2));
        assert_eq!(game.board[6][3], Some(Player::P2));
        assert_eq!(game.board[0][0], None);
    }

    #[test]
    fn test_initial_turn_is_p1() {
        let game = setup_game();
        assert_eq!(game.current_player, Player::P1);
    }

    #[test]
    fn test_count_neighbors() {
        let game = setup_game();

        // Piece at (0,3) has 1 neighbor at (1,2)
        let pos = Position { row: 0, col: 3 };
        assert_eq!(game.count_neighbors(pos), 1);

        // Piece at (3,0) has 1 neighbor at (2,1)
        let pos = Position { row: 3, col: 0 };
        assert_eq!(game.count_neighbors(pos), 1);

        // Piece at (2,1) has 2 neighbors from the initial setup
        let pos = Position { row: 2, col: 1 };
        assert_eq!(game.count_neighbors(pos), 2);
    }

    #[test]
    fn test_valid_move() {
        let mut game = setup_game();
        let from = Position { row: 0, col: 3 };
        let to = Position { row: 0, col: 2 };

        let result = game.make_move(from, to);
        assert!(result.is_ok());
        assert_eq!(game.board[from.row][from.col], None);
        assert_eq!(game.board[to.row][to.col], Some(Player::P1));
        assert_eq!(game.current_player, Player::P2);
    }

    #[test]
    fn test_invalid_move_occupied_destination() {
        let mut game = setup_game();
        let from = Position { row: 0, col: 3 };
        let to = Position { row: 1, col: 2 };
        let result = game.make_move(from, to);
        assert!(result.is_err());
    }

    #[test]
    fn test_win_by_reaching_goal() {
        let mut game = setup_game();
        game.current_player = Player::P1;

        // Manually set up the board for P1 to win in one move to the opponent's goal (6,6)
        game.board = [[None; 7]; 7];
        let from = Position { row: 3, col: 3 };
        game.board[from.row][from.col] = Some(Player::P1);

        // Add 3 neighbors to the piece at (3,3) so it can move 3 steps
        game.board[2][2] = Some(Player::P2);
        game.board[2][4] = Some(Player::P2);
        game.board[4][2] = Some(Player::P2);

        let to = Position { row: 6, col: 6 };

        let result = game.make_move(from, to);

        // The move is valid and results in a win
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::Won(Player::P1));
    }

    #[test]
    fn test_win_by_opponent_no_moves() {
        let mut game = setup_game();
        game.current_player = Player::P1;

        // Set up a simple scenario where P2 has no valid moves.
        // P2's only piece is in a corner and surrounded.
        game.board = [[None; 7]; 7];
        game.board[0][0] = Some(Player::P2);
        game.board[0][1] = Some(Player::P1);
        game.board[1][0] = Some(Player::P1);
        game.board[1][1] = Some(Player::P1);

        // P1 makes a valid move. P1 piece at (3,0) moves to (4,0).
        game.board[3][0] = Some(Player::P1);
        game.board[2][1] = Some(Player::P1);
        let from = Position { row: 3, col: 0 };
        let to = Position { row: 4, col: 0 };

        let result = game.make_move(from, to);

        // The move is valid. The win condition should now be triggered
        // because P2 has no moves left.
        assert!(result.is_ok());
        assert_eq!(game.status, GameStatus::Won(Player::P1));
    }
}
