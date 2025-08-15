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
    // P2's goal is (0,0), P1's goal is (6,6)
    for r in 0..7 {
        for c in 0..7 {
            if let Some(player) = game.board[r][c] {
                match player {
                    Player::P1 => {
                        let distance = (6 - r) + (6 - c);
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
    for r in 0..7 {
        for c in 0..7 {
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
    for r in 0..7 {
        for c in 0..7 {
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

    fn setup_test_game() -> Game {
        let mut game = Game::new();
        game.board = [[None; 7]; 7];
        game.status = GameStatus::Ongoing;
        game
    }

    #[test]
    fn test_evaluate_win_condition() {
        let mut game = setup_test_game();
        game.status = GameStatus::Won(Player::P2);
        assert_eq!(evaluate(&game), 1000);

        game.status = GameStatus::Won(Player::P1);
        assert_eq!(evaluate(&game), -1000);
    }

    #[test]
    fn test_evaluate_positional_score() {
        let mut game = setup_test_game();

        game.board[5][6] = Some(Player::P2);
        assert_eq!(evaluate(&game), (5 + 6) as i32);

        game.board[1][0] = Some(Player::P1);
        assert_eq!(evaluate(&game), (5 + 6) as i32 - ((6 - 1) + (6 - 0)) as i32);
    }

    #[test]
    fn test_minimax_base_case_depth_zero() {
        let game = setup_test_game();
        let score = minimax(&game, 0, true);
        assert_eq!(score, evaluate(&game));
    }

    #[test]
    fn test_minimax_blocking_move() {
        let mut game = setup_test_game();
        game.current_player = Player::P2;

        // P1 is one move away from winning
        game.board[5][5] = Some(Player::P1);
        game.board[6][6] = None;

        // P2 can move to block P1's winning move.
        game.board[6][5] = Some(Player::P2);

        // The piece at (6,5) has a neighbor at (5,5), which gives it a move distance of 1.
        // It can move to (5,4) to block the opponent.

        let best_move_for_ai = find_best_move(&game);

        assert_eq!(
            best_move_for_ai,
            Some((Position { row: 6, col: 5 }, Position { row: 5, col: 4 }))
        );
    }
}
