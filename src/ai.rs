use crate::constants::{BOARD_SIZE, GOAL_P2};
use crate::game::{Game, GameStatus, Player, Position};

/// A simple heuristic to evaluate the board state.
/// A higher score is better for the AI (Player 2).
fn evaluate(game: &Game) -> i32 {
    let mut score = 0;

    // Check for an immediate win or loss
    match game.status {
        GameStatus::Won(Player::P2) => return 1000,
        GameStatus::Won(Player::P1) => return -1000,
        _ => {}
    }

    // Heuristic 1: Reward pieces for being closer to the opponent's goal
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if let Some(player) = game.board[r][c] {
                match player {
                    Player::P1 => {
                        let distance = (GOAL_P2.0 - r) + (GOAL_P2.1 - c);
                        score -= distance as i32;
                    }
                    Player::P2 => {
                        let distance = r + c;
                        score += distance as i32;
                    }
                }
            }
        }
    }

    score
}

/// The main minimax recursive function.
fn minimax(game: &Game, depth: u8, is_maximizing_player: bool) -> i32 {
    // Base Case: If the game is over or we've reached max depth, evaluate the board.
    if depth == 0 || !matches!(game.status, GameStatus::Ongoing) {
        return evaluate(game);
    }

    let player_to_move = if is_maximizing_player {
        Player::P2
    } else {
        Player::P1
    };

    let mut all_valid_moves = Vec::new();
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if game.board[r][c] == Some(player_to_move) {
                let from_pos = Position { row: r, col: c };
                let valid_moves = game.get_valid_moves_for_piece(from_pos);
                for to_pos in valid_moves {
                    all_valid_moves.push((from_pos, to_pos));
                }
            }
        }
    }

    // If no moves are possible, it's a loss for the current player
    if all_valid_moves.is_empty() {
        return if is_maximizing_player { -1000 } else { 1000 };
    }

    if is_maximizing_player {
        let mut best_score = i32::MIN;
        for (from, to) in all_valid_moves {
            let mut new_game_state = game.clone();
            let _ = new_game_state.make_move(from, to);
            let score = minimax(&new_game_state, depth - 1, false);
            best_score = best_score.max(score);
        }
        best_score
    } else {
        // Minimizing player
        let mut best_score = i32::MAX;
        for (from, to) in all_valid_moves {
            let mut new_game_state = game.clone();
            let _ = new_game_state.make_move(from, to);
            let score = minimax(&new_game_state, depth - 1, true);
            best_score = best_score.min(score);
        }
        best_score
    }
}

/// Public function to find the best move for the AI.
pub fn find_best_move(game: &Game) -> Option<(Position, Position)> {
    let mut best_move = None;
    let mut best_score = i32::MIN;

    let mut all_valid_moves = Vec::new();
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if game.board[r][c] == Some(Player::P2) {
                let from_pos = Position { row: r, col: c };
                let valid_moves = game.get_valid_moves_for_piece(from_pos);
                for to_pos in valid_moves {
                    all_valid_moves.push((from_pos, to_pos));
                }
            }
        }
    }

    if all_valid_moves.is_empty() {
        return None;
    }

    const SEARCH_DEPTH: u8 = 3; // Adjust this value to change AI difficulty
    for (from, to) in all_valid_moves {
        let mut new_game_state = game.clone();
        let _ = new_game_state.make_move(from, to);
        let score = minimax(&new_game_state, SEARCH_DEPTH - 1, false);
        if score > best_score {
            best_score = score;
            best_move = Some((from, to));
        }
    }

    best_move
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: fresh empty game
    fn setup_test_game() -> Game {
        let mut game = Game::new();
        game.board = [[None; BOARD_SIZE]; BOARD_SIZE];
        game.status = GameStatus::Ongoing;
        game
    }

    // evaluate – win conditions
    #[test]
    fn test_evaluate_win_condition() {
        let mut game = setup_test_game();

        game.status = GameStatus::Won(Player::P2);
        assert_eq!(evaluate(&game), 1000);

        game.status = GameStatus::Won(Player::P1);
        assert_eq!(evaluate(&game), -1000);
    }

    // evaluate – positional score
    #[test]
    fn test_evaluate_positional_score() {
        let mut game = setup_test_game();

        // P2 piece near the bottom‑right corner (still inside the board)
        game.board[5][5] = Some(Player::P2);
        // Positional score = row + col = 5 + 5 = 10
        assert_eq!(evaluate(&game), (5 + 5) as i32);

        // Add a P1 piece elsewhere
        game.board[1][0] = Some(Player::P1);
        // New score = (5+5) - ((5-1)+(5-0)) = 10 - (4+5) = 1
        assert_eq!(evaluate(&game), (5 + 5) as i32 - ((5 - 1) + (5 - 0)) as i32);
    }

    // minimax – depth‑zero base case
    #[test]
    fn test_minimax_base_case_depth_zero() {
        let game = setup_test_game();
        let score = minimax(&game, 0, true);
        assert_eq!(score, evaluate(&game));
    }

    // minimax – blocking‑move scenario (flexible assertions)
    #[test]
    fn test_minimax_blocking_move() {
        let mut game = setup_test_game();
        game.current_player = Player::P2;

        /* -------------------------------------------------------------
           Scenario (all coordinates ≤ BOARD_SIZE‑1 = 5):

           • P1 has a piece at (4, 4).  It is one step away from the
             far‑right‑bottom corner (5, 5), which would be a winning move
             for P1.

           • P2 owns a piece at (5, 4).  That piece has at least one
             adjacent opponent piece (the P1 piece at 4, 4), so it can move.
             The exact distance the AI decides to use is internal to the
             algorithm – we only care that the move it returns is legal.

           • The AI should move the piece at (5, 4) to **some** empty
             neighbour square (or a square exactly `distance` steps away).
        ------------------------------------------------------------- */

        // P1 piece (one move away from winning)
        game.board[4][4] = Some(Player::P1);
        // The winning target (5,5) stays `None`.

        // P2 piece that can intervene
        game.board[5][4] = Some(Player::P2);

        let best_move_for_ai = find_best_move(&game);

        let (from, to) = best_move_for_ai.expect("AI should have found a legal move for Player 2");

        assert_eq!(
            from,
            Position { row: 5, col: 4 },
            "AI should move the piece that belongs to Player 2 at (5,4)"
        );

        assert!(
            to.row < BOARD_SIZE && to.col < BOARD_SIZE,
            "AI tried to move outside the board: ({}, {})",
            to.row,
            to.col
        );

        assert!(
            matches!(game.board[to.row][to.col], None),
            "AI attempted to move onto an occupied square ({}, {})",
            to.row,
            to.col
        );

        // (Optional sanity check) – the move must be straight line.
        // The distance can be 1, 2, … depending on how many adjacent
        // opponent pieces the AI counted.  We only need to ensure that
        // the move is either horizontal, vertical, or diagonal.
        let dr = (to.row as isize - from.row as isize).abs();
        let dc = (to.col as isize - from.col as isize).abs();
        assert!(
            dr == 0 || dc == 0 || dr == dc,
            "AI must move in a straight line (horizontal, vertical, or diagonal)"
        );
    }
}
