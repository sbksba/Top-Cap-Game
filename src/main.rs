use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Declare the game and AI modules
mod ai;
mod constants;
mod game;

use crate::constants::BOARD_SIZE;
use game::{Game, GameStatus, MoveRequest, Player};

// --- AXUM ROUTES & HANDLERS ---

type AppState = Arc<Mutex<Game>>;

async fn index() -> impl axum::response::IntoResponse {
    info!("GET / requested.");
    "Visit /board to see the game state."
}

#[derive(serde::Serialize)]
struct ConfigResponse {
    board_size: usize,
}

// GET /api/config → {"boardSize":BOARD_SIZE}
async fn get_config() -> Json<ConfigResponse> {
    info!("GET /api/config requested.");
    Json(ConfigResponse {
        board_size: BOARD_SIZE,
    })
}

// Handles GET /board request. Returns the current game state as JSON.
async fn get_board(State(state): State<AppState>) -> Json<Game> {
    info!("GET /board requested.");
    let game = state.lock().unwrap();
    Json((*game).clone())
}

// Handles POST /move request. Attempts to make a move.
async fn make_move(
    State(state): State<AppState>,
    Json(payload): Json<MoveRequest>,
) -> (StatusCode, String) {
    info!(
        "POST /move requested: from ({},{}), to ({},{})",
        payload.from.row, payload.from.col, payload.to.row, payload.to.col
    );
    let mut game = state.lock().unwrap();

    if let GameStatus::Won(_) = game.status {
        error!("Move failed: Game is already over.");
        return (StatusCode::BAD_REQUEST, "Game is already over.".to_string());
    }

    match game.make_move(payload.from, payload.to) {
        Ok(_) => {
            info!("Move successful.");
            (StatusCode::OK, "Move accepted.".to_string())
        }
        Err(e) => {
            error!("Move failed: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        }
    }
}

// Handles POST /ai-move request. Triggers the AI to make its move.
async fn make_ai_move(State(state): State<AppState>) -> (StatusCode, String) {
    info!("POST /ai-move requested.");
    let mut game = state.lock().unwrap();

    if let GameStatus::Won(_) = game.status {
        error!("AI move failed: Game is already over.");
        return (StatusCode::BAD_REQUEST, "Game is already over.".to_string());
    }

    // The AI is always Player 2.
    if game.current_player != Player::P2 {
        error!("AI move failed: It's not the AI's turn.");
        return (
            StatusCode::BAD_REQUEST,
            "It's not the AI's turn.".to_string(),
        );
    }

    // Call the AI logic from the separate module
    if let Some((from, to)) = ai::find_best_move(&game) {
        match game.make_move(from, to) {
            Ok(_) => {
                info!("AI move successful.");
                (StatusCode::OK, "AI move accepted.".to_string())
            }
            Err(e) => {
                error!("AI move failed during execution: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "AI made an invalid move.".to_string(),
                )
            }
        }
    } else {
        error!("AI move failed: No valid moves found.");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "AI could not find a move.".to_string(),
        )
    }
}

// Handles POST /reset request. Resets the game to its initial state.
async fn reset_game(State(state): State<AppState>) -> (StatusCode, String) {
    info!("POST /reset requested.");
    let mut game = state.lock().unwrap();
    *game = Game::new();
    info!("Game reset successfully.");
    (StatusCode::OK, "Game reset.".to_string())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();

    info!("Starting server...");

    let shared_state = AppState::new(Mutex::new(Game::new()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));
    let app = Router::new()
        .route("/", get(index))
        .route("/api/config", get(get_config))
        .route("/board", get(get_board))
        .route("/move", post(make_move))
        .route("/ai-move", post(make_ai_move))
        .route("/reset", post(reset_game))
        .fallback_service(serve_dir)
        .with_state(shared_state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
