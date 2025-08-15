# Top-Cap Game

![Project Status](https://img.shields.io/badge/status-active-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

Top-Cap is a strategic board game implemented with a Rust backend and a simple, responsive web frontend. Play against a friend in local multiplayer or challenge the built-in AI opponent.

## Features

* Two Game Modes: Choose between playing against a human opponent (Player vs. Player) or a challenging AI (Player vs. AI).

* Minimax AI: The AI opponent uses the classic minimax algorithm to predict your moves and find the optimal strategy to win.

* Modular Architecture: The project is cleanly separated into distinct modules for the game logic, AI, and server, making the codebase easy to read, maintain, and expand.

* Simple Web Interface: The game board is displayed in your browser with a clean, responsive design built with Tailwind CSS.

* Real-time Updates: The frontend communicates with the Rust backend via a simple REST API to keep the game state synchronized.

## Technologies Used

* Backend: Rust with the Axum web framework.

* AI: Implemented from scratch in Rust using the minimax algorithm.

* Frontend: HTML, CSS (Tailwind CSS), and plain JavaScript.

## Getting Started

Follow these steps to set up and run the Top-Cap game locally on your machine.

### Prerequisites

You need to have Rust installed on your system. If you don't, you can install it using rustup:
Bash

` curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh `

### Building and Running with Podman

Alternatively, you can build a container image for the application and run it with Podman.

Build the container image from the Dockerfile in the root directory:

`podman build -t top-cap-game .`

Run the container and map the internal port 3000 to your machine's port 3000:

`podman run -d -p 3000:3000 top-cap-game`

The server will be accessible at http://127.0.0.1:3000.

### Running the Server Natively

If you prefer to run the server directly on your machine without a container, use the following command:

`cargo run`

The server will start on http://127.0.0.1:3000.

### Playing the Game

Open your web browser and navigate to http://127.0.0.1:3000.

The game's rules will be displayed in a pop-up. Read them and click "Got It!" to proceed.

Choose your game mode: Player vs. AI or Player vs. Player.

Start playing!

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue. If you'd like to contribute code, please fork the repository and open a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](https://www.google.com/search?q=LICENSE) file for details.
