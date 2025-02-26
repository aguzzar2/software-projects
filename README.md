# Language Learning Flashcard Application

A web-based flashcard application designed to help users learn Japanese, featuring deck management, practice sessions, and user authentication.

## Features

### User Management
- Secure user registration and login system
- Personal session management
- Sign out functionality

### Deck Management
- Create custom flashcard decks
- Add/remove decks from your library
- Populate decks with English-Japanese word pairs
- View all decks in a centralized library

### Practice Mode
- Interactive flashcard practice sessions
- Real-time answer validation
- Progressive learning through sequential card presentation
- Automatic progression through deck content

## Technical Stack

- **Backend**: Rust with Rocket framework
- **Frontend**: HTML/CSS with Tera templates
- **Database**: SQLite
- **Authentication**: Custom implementation with secure password storage

## Project Structure

## Getting Started

1. **Prerequisites**
   - Rust and Cargo installed
   - SQLite

2. **Installation**
   ```bash
   git clone https://github.com/aguzzar2/software-projects.git
   cd software-projects
   cargo build
   ```

3. **Running the Application**
   ```bash
   cargo run --bin home
   ```
   The application will be available at `http://localhost:8000`

## Usage

1. **Create an Account**
   - Navigate to the login page
   - Choose "Sign Up" and create your credentials

2. **Create a Deck**
   - Click "Create Deck" from the navigation menu
   - Enter a unique deck name
   - Start adding English-Japanese word pairs

3. **Practice**
   - Select a deck from your library
   - Practice mode will present English words
   - Enter the Japanese translations
   - Receive immediate feedback on your answers

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
