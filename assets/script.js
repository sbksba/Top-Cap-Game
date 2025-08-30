const API_URL = "http://127.0.0.1:3000";
let selectedPiece = null;
let currentPlayer = null;
let gameMode = 'two-player'; // Default to two-player mode
let BOARD_SIZE = 6; // fallback – will be overwritten as soon as we get the real value

// UI Elements
const boardElement = document.getElementById('gameBoard');
const statusElement = document.getElementById('gameStatus');
const playerIconElement = document.getElementById('currentPlayerIcon');
const resetButton = document.getElementById('resetButton');
const messageBox = document.getElementById('messageBox');
const messageText = document.getElementById('messageText');
const rulesModal = document.getElementById('rulesModal');
const closeModalButton = document.getElementById('closeModalButton');
const modeSelection = document.getElementById('modeSelection');
const gameContainer = document.getElementById('gameContainer');
const soloModeButton = document.getElementById('soloModeButton');
const twoPlayerModeButton = document.getElementById('twoPlayerModeButton');

// Function to display a temporary message to the user
function showMessage(text, type = 'info') {
    messageText.textContent = text;
    messageBox.className = 'p-4 rounded-lg w-full';
    
    // Set colors based on message type
    switch (type) {
        case 'success':
            messageBox.classList.add('bg-green-100', 'text-green-800', 'border-l-4', 'border-green-500');
            break;
        case 'error':
            messageBox.classList.add('bg-red-100', 'text-red-800', 'border-l-4', 'border-red-500');
            break;
        case 'info':
        default:
            messageBox.classList.add('bg-yellow-100', 'text-yellow-800', 'border-l-4', 'border-yellow-500');
            break;
    }
    messageBox.style.display = 'block';
    setTimeout(() => {
        messageBox.style.display = 'none';
    }, 3000);
}

// Fetch the configuration (board size) once on startup
async function fetchConfig() {
    try {
        const resp = await fetch(`${API_URL}/api/config`);
        if (!resp.ok) throw new Error(`status ${resp.status}`);
        const cfg = await resp.json();
        BOARD_SIZE = cfg.board_size ?? BOARD_SIZE;
        // Push the size to CSS (see step 3)
        document.documentElement.style.setProperty('--board-size', BOARD_SIZE);
    } catch (e) {
        console.error("Could not fetch config, using default size", e);
        // Still set the CSS variable so the grid works with the fallback
        document.documentElement.style.setProperty('--board-size', BOARD_SIZE);
    }
}


// Fetches the current game state from the Rust server
async function fetchBoardState() {
    try {
        const response = await fetch(`${API_URL}/board`);
        if (!response.ok) {
            throw new Error(`Server responded with status: ${response.status}`);
        }
        const game = await response.json();
        renderBoard(game);
        updateGameStatus(game);
        return game;
    } catch (error) {
        console.error("Failed to fetch board state:", error);
        showMessage("Failed to connect to the server. Is it running?", 'error');
    }
}

// Renders the game board based on the game state
function renderBoard(game) {
    boardElement.innerHTML = '';
    game.board.forEach((row, rowIndex) => {
        row.forEach((player, colIndex) => {
            const cell = document.createElement('div');
            const cellColor = (rowIndex + colIndex) % 2 === 0 ? 'bg-gray-700' : 'bg-gray-500';
            cell.className = `cell ${cellColor} rounded-md`;
            
            cell.dataset.row = rowIndex;
            cell.dataset.col = colIndex;

            // Add a piece if one exists on this cell
            if (player !== null) {
                const piece = document.createElement('div');
                piece.className = `piece ${player === 'P1' ? 'p1' : 'p2'}`;
                cell.appendChild(piece);
            }

            cell.addEventListener('click', handleBoardClick);
            boardElement.appendChild(cell);
        });
    });
}

// Updates the game status display
function updateGameStatus(game) {
    currentPlayer = game.current_player;
    if (game.status === 'Ongoing') {
        statusElement.textContent = `Player ${currentPlayer === 'P1' ? '1' : '2'}'s turn`;
        playerIconElement.className = `w-6 h-6 rounded-full ${currentPlayer === 'P1' ? 'bg-red-500' : 'bg-blue-500'}`;
    } else {
        const winner = game.status.Won;
        if (winner) {
            statusElement.textContent = `Player ${winner === 'P1' ? '1' : '2'} wins!`;
            playerIconElement.className = `w-6 h-6 rounded-full ${winner === 'P1' ? 'bg-red-500' : 'bg-blue-500'}`;
        }
    }
}

// Handles a click on the board
function handleBoardClick(event) {
    const cell = event.currentTarget;
    const row = parseInt(cell.dataset.row);
    const col = parseInt(cell.dataset.col);

    // If a piece is already selected, this is a move attempt
    if (selectedPiece) {
        makeMove(selectedPiece.row, selectedPiece.col, row, col);
        selectedPiece = null;
        document.querySelectorAll('.cell').forEach(c => c.classList.remove('highlight'));
    } else {
        // This is a piece selection
        const pieceElement = cell.querySelector('.piece');
        if (pieceElement) {
            const isP1 = pieceElement.classList.contains('p1');
            const piecePlayer = isP1 ? 'P1' : 'P2';

            // Only allow selecting your own pieces
            if (piecePlayer === currentPlayer) {
                selectedPiece = { row, col };
                document.querySelectorAll('.cell').forEach(c => c.classList.remove('highlight'));
                cell.classList.add('highlight');
            }
        }
    }
}

// Sends a move request to the server
async function makeMove(fromRow, fromCol, toRow, toCol) {
    const moveRequest = {
        from: { row: fromRow, col: fromCol },
        to: { row: toRow, col: toCol }
    };

    try {
        const response = await fetch(`${API_URL}/move`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(moveRequest),
        });
        
        if (!response.ok) {
            const message = await response.text();
            showMessage(message, 'error');
        }
        
        const game = await fetchBoardState();
        
        // Only make AI move if in solo mode and it's the AI's turn
        if (gameMode === 'solo' && game && game.current_player === 'P2' && game.status === 'Ongoing') {
            setTimeout(makeAiMove, 500); 
        }

    } catch (error) {
        console.error("Failed to make move:", error);
        showMessage("Failed to connect to the server.", 'error');
    }
}

// New function to trigger the AI's move
async function makeAiMove() {
    showMessage("AI is thinking...", "info");
    try {
        const response = await fetch(`${API_URL}/ai-move`, {
            method: 'POST',
        });
        
        await fetchBoardState();

    } catch (error) {
        console.error("AI failed to make move:", error);
        showMessage("AI failed to make a move.", 'error');
    }
}

// Resets the game by calling the server's reset endpoint
async function resetGame() {
    try {
        const response = await fetch(`${API_URL}/reset`, {
            method: 'POST',
        });
        const message = await response.text();
        if (!response.ok) {
            showMessage(message, 'error');
        }
        await fetchBoardState();
        // If in solo mode, let AI take its turn
        if (gameMode === 'solo' && currentPlayer === 'P2') {
            setTimeout(makeAiMove, 500);
        }
    } catch (error) {
        console.error("Failed to reset game:", error);
        showMessage("Failed to connect to the server.", 'error');
    }
}

// Event listener for the reset button
resetButton.addEventListener('click', resetGame);

// Event listener to close the rules modal
closeModalButton.addEventListener('click', () => {
    rulesModal.style.display = 'none';
});

// Event listeners for mode selection
soloModeButton.addEventListener('click', () => {
    gameMode = 'solo';
    modeSelection.classList.add('hidden');
    gameContainer.classList.remove('hidden');
    fetchBoardState();
});

twoPlayerModeButton.addEventListener('click', () => {
    gameMode = 'two-player';
    modeSelection.classList.add('hidden');
    gameContainer.classList.remove('hidden');
    fetchBoardState();
});

(async () => {
    await fetchConfig();
    rulesModal.style.display = 'flex';
})();
