use rand::prelude::*;
use std::collections::HashSet;
use std::fmt;
use std::io::stdin;
use std::str::FromStr;

// type alias for coordinates in the minesweeper grid
type Position = (usize, usize);

// fn to gen random usize in exclusive range
fn random_range(start: usize, stop: usize) -> usize {
    let mut rng = thread_rng();
    rng.gen_range(start..stop)
}

// fn to get input from user
fn get_input(msg: &str) -> String {
    // create empty buffer to read input into
    let mut input_string = String::new();
    // read input until it contains something
    while input_string.trim().is_empty() {
        // display prompt message
        println!("{}", msg);
        // read input into buffer and if err, clear and try again
        if stdin().read_line(&mut input_string).is_err() {
            input_string.clear();
            println!("Error reading input.");
        }
    }
    // return valid input
    input_string
}

// enum to store game state
#[derive(PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

// enum to store type of move made by user
#[derive(PartialEq)]
enum MoveType {
    Flag,
    Open,
}

// err to raise if move validation fails (out of bounds, etc)
#[derive(Debug, PartialEq)]
struct MoveValidationError;

impl fmt::Display for MoveValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid move")
    }
}

// enum to store all minesweeper variants
#[derive(PartialEq)]
enum MinesweeperVariant {
    Normal, // all mines in 3x3 area around square
    FarNormal, // all mines in 5x5 area around square
    KnightPaths, // all mines in knight paths from square
    BlindUp, // all mines in 3x3 area around square excl square directly above
    BlindDown, // all mines in 3x3 area around square excl square directly below
    BlindLeft, // all mines in 3x3 area around square excl square directly left
    BlindRight, // all mines in 3x3 area around square excl square directly right
    Orthogonal, // all mines orthogonally adjacent to square (distance 1)
    FarOrthogonal, // all mines orthogonally adjacent to square (distance 2)
    Diagonal, // all mines diagonally adjacent to square (distance 1)
    FarDiagonal, // all mines diagonally adjacent to square (distance 2)
    Doubled, // all mines in 3x3 area around square but orthogonally adj squares counted twice
}

// err to raise if parse from str fails
#[derive(Debug, PartialEq)]
struct VariantParseError;

impl fmt::Display for VariantParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid variant")
    }
}

// impl ability to parse from str
impl FromStr for MinesweeperVariant {
    // err to return if parsing fails
    type Err = VariantParseError;

    // fn to parse variant from str
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(Self::Normal),
            "far-normal" => Ok(Self::FarNormal),
            "knight-paths" => Ok(Self::KnightPaths),
            "blind-up" => Ok(Self::BlindUp),
            "blind-down" => Ok(Self::BlindDown),
            "blind-left" => Ok(Self::BlindLeft),
            "blind-right" => Ok(Self::BlindRight),
            "orthogonal" => Ok(Self::Orthogonal),
            "far-orthogonal" => Ok(Self::FarOrthogonal),
            "diagonal" => Ok(Self::Diagonal),
            "far-diagonal" => Ok(Self::FarDiagonal),
            "doubled" => Ok(Self::Doubled),
            _ => Err(VariantParseError),
        }
    }
}

// struct to store the minesweeper game
struct Minesweeper {
    width: usize, // width of board
    height: usize, // height of board
    mines: HashSet<Position>, // set to store mines
    open_squares: HashSet<Position>, // set to store current open positions
    flagged_squares: HashSet<Position>, // set to store current flagged positions
    all_squares: HashSet<Position>, // set to store all possible positions
    state: GameState, // game state (playing, won, lost)
    variant: MinesweeperVariant, // variant
}

impl Minesweeper {
    // fn to construct a new game from an instance of GameSettings
    fn new(settings: GameSettings) -> Self {
        Self {
            width: settings.board_width,
            height: settings.board_height,
            mines: { // generate mines
                // make an empty set of positions
                let mut mines = HashSet::<Position>::new();
                // repeat until have enough mines
                while mines.len() < settings.num_mines {
                    // generate random mine
                    let mine: Position = (random_range(0, settings.board_width), random_range(0, settings.board_height));
                    // add to mines unless mine is already there
                    if mines.contains(&mine) {
                        continue;
                    }
                    mines.insert(mine);
                }
                // return mines
                mines
            },
            open_squares: HashSet::<Position>::new(), // init
            flagged_squares: HashSet::<Position>::new(), // init
            all_squares: { // generate all positions
                let mut all_squares = HashSet::<Position>::new();
                // loop through all positions and add them to the set
                for x in 0..settings.board_width {
                    for y in 0..settings.board_height {
                        all_squares.insert((x, y));
                    }
                }
                // return all squares
                all_squares
            },
            state: GameState::Playing, // init
            variant: settings.variant,
        }
    }

    // fn to generate neighbors (as specified by the game's variant) for a specific cell on the grid
    fn neighbors(&self, x: usize, y: usize) -> Vec<Position> {
        // get neighbor offsets for game's variant
        use MinesweeperVariant::{BlindDown, BlindLeft, BlindRight, BlindUp, Diagonal, Doubled, FarDiagonal, FarNormal, FarOrthogonal, KnightPaths, Normal, Orthogonal};
        let dirs: Vec<(i64, i64)> = match self.variant {
            Normal => vec![(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)], // all mines in 3x3 area around square
            FarNormal => (-2..=2).flat_map(|x| (-2..=2).map(move |y| (x, y))).collect(), // all mines in 5x5 area around square
            KnightPaths => vec![(-1, -2), (-1, 2), (1, -2), (1, 2), (-2, -1), (-2, 1), (2, -1), (2, 1)], // all mines in knight paths from square
            BlindUp => vec![(-1, 0), (1, 0), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)], // all mines in 3x3 area around square excl square directly above
            BlindDown => vec![(-1, 0), (1, 0), (0, -1), (-1, -1), (-1, 1), (1, -1), (1, 1)], // all mines in 3x3 area around square excl square directly below
            BlindLeft => vec![(1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)], // all mines in 3x3 area around square excl square directly left
            BlindRight => vec![(-1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)], // all mines in 3x3 area around square excl square directly right
            Orthogonal => vec![(-1, 0), (1, 0), (0, -1), (0, 1)], // all mines orthogonally adjacent to square (distance 1)
            FarOrthogonal => vec![(-2, 0), (2, 0), (0, -2), (0, 2), (-1, 0), (1, 0), (0, -1), (0, 1)], // all mines orthogonally adjacent to square (distance 2)
            Diagonal => vec![(-1, -1), (1, 1), (-1, 1), (1, -1)], // all mines diagonally adjacent to square (distance 1)
            FarDiagonal => vec![(-2, -2), (2, 2), (-2, 2), (2, -2), (-1, -1), (1, 1), (-1, 1), (1, -1)], // all mines diagonally adjacent to square (distance 2)
            Doubled => vec![(-1, 0), (1, 0), (0, -1), (0, 1), (-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)], // all mines in 3x3 area around square but orthogonally adj squares counted twice
        };
        // generate list of neighbors
        let mut neighbors = Vec::<Position>::new(); // init
        // loop over neighbor offsets, destructure into individual x and y offsets
        for &(dx, dy) in &dirs {
            // apply offsets to cell specified to get neighbor
            let nx = x as i64 + dx;
            let ny = y as i64 + dy;
            // check if generated neighbor lies outside game's borders and if so ignore it
            if nx < 0 || nx >= self.width as i64 || ny < 0 || ny >= self.height as i64 {
                continue;
            }
            // convert neighbor x and y to grid position (this should never fail)
            let nx: usize = nx.try_into().unwrap_or_else(|_| unreachable!());
            let ny: usize = ny.try_into().unwrap_or_else(|_| unreachable!());
            // push neighbor to list
            neighbors.push((nx, ny));
        }
        // return neighbors
        neighbors
    }

    // fn to get the number of neighbors of a cell which are mines
    fn mines_near(&self, x: usize, y: usize) -> usize {
        self.neighbors(x, y) // get neighbors
            .iter()
            .filter(|&neighbor| self.mines.contains(neighbor)) // filter to get only those which are mines
            .count() // count the number
    }

    // fn to open a square
    // opening a square adds it to the current set of open squares
    // if it is not already there and it is not flagged (as being a mine)
    // and opens neighboring squares recursively as long as they are empty.
    // if a square is opened which contains a mine, the game is lost.
    fn open(&mut self, x: usize, y: usize) {
        // guard to check if square has already been opened
        if self.open_squares.contains(&(x, y)) {
            return;
        }
        // guard to check if square is flagged
        if self.flagged_squares.contains(&(x, y)) {
            return;
        }
        // if square is a mine, lose the game
        if self.mines.contains(&(x, y)) {
            println!("You lost!");
            self.state = GameState::Lost;
            return;
        }

        // by this point, we are safe to open this square
        // add this square to the set of open squares
        self.open_squares.insert((x, y));

        // open neighboring squares with zero mines near recursively
        // guard to check if this square has more than zero mines surrounding it
        if self.mines_near(x, y) > 0 {
            return;
        }
        // open all neighbors recursively
        for (new_x, new_y) in self.neighbors(x, y) {
            self.open(new_x, new_y);
        }
    }

    // fn to flag a square
    // flagging a square makes it impossible to open.
    // this is usually used to signal that the flagged square is probably a mine,
    // however flagging all mines is not required to win a game.
    // hence you cannot flag open squares as they are already proven to not be mines.
    fn flag(&mut self, x: usize, y: usize) {
        // guard to check if square is open
        if self.open_squares.contains(&(x, y)) {
            return;
        }

        // if we are re-flagging a flagged square, interpret that as a toggle and remove it
        if self.flagged_squares.contains(&(x, y)) {
            self.flagged_squares.remove(&(x, y));
        // else add the square to the set of flagged squares
        } else {
            self.flagged_squares.insert((x, y));
        }
    }

    // fn to determine if the game is won
    // a game is won if all non-mine squares have been dug up (ie. opened)
    fn determine_win(&mut self) {
        // create test clone of open squares to check for win without mutating original
        // this needs to be done as .extend() extends in place
        let mut test_squares = self.open_squares.clone();
        // extend test squares (open squares) by set of mines
        // a.extend(b) adds all members of b to a in place.
        test_squares.extend(self.mines.clone());
        // if the extended set is equal to the set of all possible positions,
        // this means that all non-mine squares have been opened and we have won the game.
        if test_squares == self.all_squares {
            self.state = GameState::Won;
            println!("You won!");
        }
    }

    // fn to validate a move position (used for getting a valid move from the player)
    fn validate_move_pos(raw: &str, bound: usize) -> Result<usize, MoveValidationError> {
        // parse the raw string into a grid index
        let parse_result = raw.parse::<usize>();
        // check the parsed result
        match parse_result {
            // if parsing was a success and move is within bounds, return the parsed move
            Ok(parsed_move) if (1..=bound).contains(&parsed_move) => Ok(parsed_move-1),
            // else if there was an error
            Err(_) => match raw.to_lowercase().as_str() {
                // if the raw string that we wanted to parse was a quit instruction, then quit
                "q" | "quit" => {
                    println!("Quitting...");
                    std::process::exit(0);
                },
                // else, we do not recognise user's input and return an error
                _ => Err(MoveValidationError),
            },
            // else, error
            _ => Err(MoveValidationError),
        }
    }

    // fn to get a valid move position from the player
    fn get_move_pos(&self) -> Position {
        // get raw input of move from the user
        let raw_move_x = get_input(format!("Enter move x (1-{}): ", self.width).as_str());
        // validate the move with bound of board width
        let move_x = Self::validate_move_pos(raw_move_x.trim(), self.width);

        // get raw input of move from the user
        let raw_move_y = get_input(format!("Enter move y (1-{}): ", self.height).as_str());
        // validate the move with bound of board height
        let move_y = Self::validate_move_pos(raw_move_y.trim(), self.height);

        // check if both moves were validated correctly and if so, return them
        if let (Ok(x), Ok(y)) = (move_x, move_y) {
            (x, y)
        // else we try again
        } else {
            println!("Invalid move.");
            self.get_move_pos()
        }
    }

    // fn to get a valid move type from the player
    fn get_move_type(&self) -> MoveType {
        // get raw input from player
        let move_type = get_input("Enter move type (open/flag/quit): ");
        // check input
        match move_type.to_lowercase().as_str().trim() {
            // flag command
            "f" | "flag" => MoveType::Flag,
            // open command
            "o" | "open" => MoveType::Open,
            // quit command
            "q" | "quit" => {
                println!("Quitting...");
                std::process::exit(0);
            },
            // invalid - try again
            _ => {
                println!("Invalid move type.");
                self.get_move_type()
            },
        }
    }

    // fn to display a single square
    fn write_square(&self, fmt: &mut fmt::Formatter<'_>, x: usize, y: usize) -> fmt::Result {
        // square is flagged and game not lost
        if self.flagged_squares.contains(&(x, y)) && self.state != GameState::Lost {
            write!(fmt, "F ")?;
        // square is a mine
        } else if self.mines.contains(&(x, y)) {
            // if game is lost, display mine
            if self.state == GameState::Lost {
                write!(fmt, "# ")?;
            // otherwise display unopened square (if square was opened, game would be lost)
            } else {
                write!(fmt, ". ")?;
            }
        // square is open
        } else if self.open_squares.contains(&(x, y)) {
            // display number of mines near if > 0, else opened square
            let mines_value: usize = self.mines_near(x, y);
            if mines_value > 0 {
                write!(fmt, "{} ", mines_value)?;
            } else {
                write!(fmt, "  ")?;
            }
        // square is unopened or unflagged
        } else {
            write!(fmt, ". ")?;
        }
        Ok(())
    }

    // fn to play a game of minesweeper
    fn play(&mut self) {
        // display board
        println!("{}", self);
        // while we are playing (game not lost or won)
        while self.state == GameState::Playing {
            // get move pos from player
            let (x, y) = self.get_move_pos();
            // get move type from player
            let move_type = self.get_move_type();
            // open or flag square based on move type
            match move_type {
                MoveType::Open => self.open(x, y),
                MoveType::Flag => self.flag(x, y),
            };
            // display board
            println!("{}", self);
            // check if won
            self.determine_win();
        }
    }
}

impl fmt::Display for Minesweeper {
    // fn to display the board
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // generate horizontal border eg.
        // +--------+ for a board of width 4
        // each cell takes up 2 chars
        let horiz_border = "+".to_owned() + &"-".repeat(self.width * 2 + 1) + "+\n";

        // display top border
        write!(fmt, "{}", horiz_border)?;

        // for each row
        for y in 0..self.height {
            // display left border
            write!(fmt, "| ")?;
            // for each col
            for x in 0..self.width {
                // display square at that pos
                self.write_square(fmt, x, y)?;
            }
            // display right border
            writeln!(fmt, "|")?;
        }

        // display bottom border
        write!(fmt, "{}", horiz_border)?;
        Ok(())
    }
}

// struct to store the settings for a particular game:
// board width, board height, number of mines, game variant
struct GameSettings {
    board_width: usize,
    board_height: usize,
    num_mines: usize,
    variant: MinesweeperVariant,
}

// fn to fetch an arg from command line args
// takes the position the arg should be in, the arg name,
// a function/closure to validate it with, and an err msg
// to display if the validation fails.
fn get_arg<T, E>(pos: usize, arg_name: &str, validation_fn: fn(String) -> Result<T, E>, err_msg: &str) -> T {
    // get arg at position pos, erring if absent
    let nth_arg = std::env::args() // get command line args
        .nth(pos) // get arg in pos position
        .unwrap_or_else(|| panic!("parameter {} expected in position {}", arg_name, pos)); // err if not found
    // validate arg and show err_msg on fail
    validation_fn(nth_arg) // validate arg
        .unwrap_or_else(|_| panic!("invalid string found for parameter {}: {}", arg_name, err_msg)) // err with err_msg on fail
}

// fn to build a GameSettings object from cmd line args
fn get_game_settings() -> GameSettings {
    // build GameSettings object
    GameSettings {
        board_width: get_arg(1, "board_width", |x| x.parse::<usize>(), "unable to parse to usize"), // board width
        board_height: get_arg(2, "board_height", |x| x.parse::<usize>(), "unable to parse to usize"), // board height
        num_mines: get_arg(3, "num_mines", |x| x.parse::<usize>(), "unable to parse to usize"), // number of mines
        variant: get_arg(4, "variant", |x| x.parse::<MinesweeperVariant>(), concat!( // game variant
            "invalid variant: allowed variants include:",
            "\n\tnormal",
            "\n\tfar-normal",
            "\n\tknight-paths",
            "\n\tblind-up",
            "\n\tblind-down",
            "\n\tblind-left",
            "\n\tblind-right",
            "\n\torthogonal",
            "\n\tfar-orthogonal",
            "\n\tdiagonal",
            "\n\tfar-diagonal",
            "\n\tdoubled",
        )),
    }
}

fn main() {
    // get game settings from cmd line args
    let settings = get_game_settings();
    // init game with these settings
    let mut minesweeper: Minesweeper = Minesweeper::new(settings);
    // play game
    minesweeper.play();
}
