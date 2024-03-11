#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

use pink_extension as pink;


#[pink::contract(env=PinkEnvironment)]
mod oh_my_chess {
    use super::{pink};
    use pink::PinkEnvironment;
    use scale::{Encode, Decode};
    use alloc::format;
    use serde::{Deserialize, Serialize};
    use alloc::string::String;
    use serde_json_core;
    use crate::oh_my_chess::Error::{WrongPlayerAddressArgument, ErrorInsertingToDB, CouldNotUpdateDB, ToIsOccupiedByOneOfYourPiece, PieceSelectedIsNotYours, NoPieceBoardChessFrom, OutOfBoardChessFrom, OutOfBoardChessTo, NonValidMove, NoElementFoundInDB, ErrorFetchingFromDB, NotAuthorized, NotYourTurn, YourNotInThisGameSession};
    use scale_info::TypeInfo;


    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NonValidMove,
        NotYourTurn,
        YourNotInThisGameSession,
        NoElementFoundInDB,
        CouldNotUpdateDB,
        ErrorFetchingFromDB,
        ErrorInsertingToDB,
        NotAuthorized,
        OutOfBoardChessFrom,
        OutOfBoardChessTo,
        NoPieceBoardChessFrom,
        PieceSelectedIsNotYours,
        ToIsOccupiedByOneOfYourPiece,
        WrongPlayerAddressArgument,
    }
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Option<T> = core::option::Option<T>;


    #[ink(storage)]
    pub struct OhMyChess {
        admin: AccountId,
        url: String,
        api_key: String,
    }

    impl OhMyChess {

        #[ink(constructor)]
        pub fn new(url: String, api_key: String) -> Result<Self> {
            let instance = Self {
                admin: Self::env().caller(),
                url,
                api_key,
            };
            Ok(instance)
        }

        #[ink(message)]
        pub fn start_new_game_session(&self, player_white: [u8; 32], player_black: [u8; 32]) -> Result<String> {

            let initial_board = [
                [ // 1st rank (from White's perspective)
                    Some(ChessCell { piece: Piece::Rook, player: Player::White }),
                    Some(ChessCell { piece: Piece::Knight, player: Player::White }),
                    Some(ChessCell { piece: Piece::Bishop, player: Player::White }),
                    Some(ChessCell { piece: Piece::Queen, player: Player::White }),
                    Some(ChessCell { piece: Piece::King, player: Player::White }),
                    Some(ChessCell { piece: Piece::Bishop, player: Player::White }),
                    Some(ChessCell { piece: Piece::Knight, player: Player::White }),
                    Some(ChessCell { piece: Piece::Rook, player: Player::White }),
                ],
                [ // 2nd rank
                    Some(ChessCell { piece: Piece::Pawn, player: Player::White }); 8 // All pawns
                ],
                [None; 8], // 3rd rank
                [None; 8], // 4th rank
                [None; 8], // 5th rank
                [None; 8], // 6th rank
                [ // 7th rank
                    Some(ChessCell { piece: Piece::Pawn, player: Player::Black }); 8 // All pawns
                ],
                [ // 8th rank (from Black's perspective)
                    Some(ChessCell { piece: Piece::Rook, player: Player::Black }),
                    Some(ChessCell { piece: Piece::Knight, player: Player::Black }),
                    Some(ChessCell { piece: Piece::Bishop, player: Player::Black }),
                    Some(ChessCell { piece: Piece::Queen, player: Player::Black }),
                    Some(ChessCell { piece: Piece::King, player: Player::Black }),
                    Some(ChessCell { piece: Piece::Bishop, player: Player::Black }),
                    Some(ChessCell { piece: Piece::Knight, player: Player::Black }),
                    Some(ChessCell { piece: Piece::Rook, player: Player::Black }),
                ],
            ];
            let game_state = GameState {
                board: initial_board,
                turn: Player::White, // White starts the game
                players: PlayersAddresses {
                    black: player_black.clone(),
                    white: player_white.clone(),
                },
                status: GameStatus::Ongoing,
            };
            let inserted_document_id = self.insert_game_session_to_mongodb(game_state)?;
            self.update_players_sessions_track_in_mongodb(inserted_document_id.clone(), player_black)?;
            self.update_players_sessions_track_in_mongodb(inserted_document_id.clone(), player_white)?;

            Ok(String::from(inserted_document_id.as_str()))
        }

        pub fn is_admin(&self) -> bool {
            Self::env().caller() == self.admin
        }

        #[ink(message)]
        pub fn get_url(&self) -> Result<String> {
            if self.is_admin() { Ok(self.url.clone()) }
            else { Err(NotAuthorized) }
        }

        #[ink(message)]
        pub fn set_url(&mut self, url: String) -> bool {
            if Self::env().caller() != self.admin { false }
            else {
                self.url = url;
                true
            }
        }

        #[ink(message)]
        pub fn get_api_key(&self) -> Result<String> {
            if self.is_admin() { Ok(self.api_key.clone()) }
            else { Err(NotAuthorized) }
        }

        #[ink(message)]
        pub fn set_api_key(&mut self, api_key: String) -> bool {
            if Self::env().caller() != self.admin { false }
            else {
                self.api_key = api_key;
                true
            }
        }

        #[ink(message)]
        pub fn make_move(&self, chess_move: ChessMove, session_id: String) -> Result<()> {
            let mut game_state = self.find_game_session_from_mongodb(session_id.clone())?;

            // check validity of the move
            Self::check_move_boundaries(&chess_move)?;
            Self::check_caller_turn(&game_state)?;
            Self::check_caller_owns_piece(&game_state, &chess_move)?;
            Self::check_move_validity_for_piece(&game_state, &chess_move)?;

            // update game_state: board, status
            Self::update_board_from_move(&mut game_state, &chess_move)?;
            Self::update_turn_and_status(&mut game_state, &chess_move)?;

            // update mongodb
            self.update_game_session_to_mongodb(game_state, session_id)
        }

        fn update_turn_and_status(game_state: &mut GameState, chess_move: &ChessMove) -> Result<()> {
            let (tx, ty) = chess_move.to;
            if let Some(ChessCell { piece: Piece::King, .. }) = game_state.board[tx as usize][ty as usize] {
                game_state.status = GameStatus::Finished;
            } else {
                game_state.turn = match game_state.turn {
                    Player::Black => Player::White,
                    Player::White => Player::Black,
                };
            }
            Ok(())
        }

        fn update_board_from_move(game_state: &mut GameState, chess_move: &ChessMove) -> Result<()> {
            let (fx, fy) = chess_move.from;
            let (tx, ty) = chess_move.to;

            let chess_cell_option = game_state.board[fx as usize][fy as usize].take(); // Directly take the value out
            if let Some(ChessCell { piece: Piece::Pawn, .. }) = &chess_cell_option {
                // Check if the pawn reaches the end and should be promoted to a queen
                if tx == 7 || tx == 0 {
                    game_state.board[tx as usize][ty as usize] = Some(ChessCell { piece: Piece::Queen, player: game_state.turn });
                } else {
                    game_state.board[tx as usize][ty as usize] = chess_cell_option;
                }
            } else {
                game_state.board[tx as usize][ty as usize] = chess_cell_option;
            }

            // Clear the original location after moving the piece
            game_state.board[fx as usize][fy as usize] = None;

            Ok(())
        }

        pub fn check_caller_owns_piece(game_state: &GameState, chess_move: &ChessMove) -> Result<()> {
            let player_turn = &game_state.turn;

            let ChessCell{ player: player_piece_from, .. } = match &game_state.board[chess_move.from.0 as usize][chess_move.from.1 as usize] {
                Some(ref chess_cell) => chess_cell,
                None => return Err(NoPieceBoardChessFrom), // No piece at source
            };

            if *player_turn != *player_piece_from {
                return Err(PieceSelectedIsNotYours);
            }

            let ChessCell{ player: player_piece_to, .. } = match &game_state.board[chess_move.to.0 as usize][chess_move.to.1 as usize] {
                Some(ref chess_cell) => chess_cell,
                None => return Ok(()), // No piece at source
            };

            if *player_piece_to == *player_piece_from { Err(ToIsOccupiedByOneOfYourPiece) }
            else { Ok(()) }
        }

        pub fn check_caller_turn(game_state: &GameState) -> Result<()> {
            let caller = Self::env().caller();
            let player = &game_state.turn;
            let player_address = if *player == Player::Black { game_state.players.black } else { game_state.players.white };

            if AccountId::from(player_address) != caller {
                return Err(NotYourTurn);
            }
            if caller != AccountId::from(game_state.players.black) && caller != AccountId::from(game_state.players.white) {
                // The caller is neither black nor white player, return YourNotInThisGameSession error
                return Err(YourNotInThisGameSession);
            }
            Ok(())
        }

        #[ink(message)]
        pub fn find_game_session_from_mongodb(&self, session_id: String) -> Result<GameState> {
            let method = String::from("POST"); // HTTP Method for the request
            let url = format!("{}/action/findOne?_id={}", self.url, session_id);

            let data = r#"{
                "collection":"game_sessions",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "projection":{"_id":0,"turn":1,"status":1,"players":1,"board":1}
            }"#.as_bytes().to_vec();

            // Prepare headers
            let headers = alloc::vec![
                (String::from("Content-Type"), String::from("application/json")),
                (String::from("Access-Control-Request-Headers"), String::from("*")),
                (String::from("api-key"), self.api_key.clone()),
            ];

            let response = pink::http_req!(
                method,
                url,
                data,
                headers
            );

            let game_state_res = serde_json_core::from_slice::<FindMongoDBDocumentResult>(response.body.as_slice())
                .map_err(|_| { ErrorFetchingFromDB })
                .map(|(mongodb, _)| { mongodb.document })?
                .ok_or(NoElementFoundInDB);

            game_state_res
        }

        pub fn update_players_sessions_track_in_mongodb(&self, session_id: heapless::String<32>, player_address: [u8; 32]) -> Result<()> {
            let method = String::from("POST"); // HTTP Method for the request
            let player_address_hex_string = Self::bytes_to_hex_string(player_address)?;
            let url = format!("{}/action/updateOne?_id={}", self.url, player_address_hex_string);

            let data = format!(r#"{{
                "collection":"players_sessions_trackers",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "filter":{{}},
                "update":{{"$push": {{"sessions": "{}"}} }},
                "upsert":true
            }}"#, session_id).as_bytes().to_vec();

            // Prepare headers
            let headers = alloc::vec![
                (String::from("Content-Type"), String::from("application/json")),
                (String::from("Access-Control-Request-Headers"), String::from("*")),
                (String::from("api-key"), self.api_key.clone()),
            ];

            let response = pink::http_req!(
                method,
                url,
                data,
                headers
            );

            if response.status_code == 200 { Ok(()) }
            else { Err(CouldNotUpdateDB) }
        }

        pub fn update_game_session_to_mongodb(&self, game_state: GameState, session_id: String) -> Result<()> {
            let method = String::from("POST"); // HTTP Method for the request
            let url = format!("{}/action/updateOne?_id={}", self.url, session_id);

            let json_game_state: heapless::String<8192> = serde_json_core::ser::to_string(&game_state).map_err(|_| { CouldNotUpdateDB })?;

            let data = format!(r#"{{
                "collection":"game_sessions",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "filter":{{}},
                "update":{{"$set": {}}}
            }}"#, json_game_state).as_bytes().to_vec();

            // Prepare headers
            let headers = alloc::vec![
                (String::from("Content-Type"), String::from("application/json")),
                (String::from("Access-Control-Request-Headers"), String::from("*")),
                (String::from("api-key"), self.api_key.clone()),
            ];

            let response = pink::http_req!(
                method,
                url,
                data,
                headers
            );

            if response.status_code == 200 { Ok(()) }
            else { Err(CouldNotUpdateDB) }
        }

        pub fn insert_game_session_to_mongodb(&self, game_state: GameState) -> Result<heapless::String<32>> {
            let method = String::from("POST"); // HTTP Method for the request
            let url = format!("{}/action/insertOne", self.url);

            let json_game_state: heapless::String<4096> = serde_json_core::ser::to_string(&game_state).map_err(|_| { ErrorInsertingToDB })?;

            let data = format!(r#"{{
                "collection":"game_sessions",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "document":{}
            }}"#, json_game_state).as_bytes().to_vec();

            // Prepare headers
            let headers = alloc::vec![
                (String::from("Content-Type"), String::from("application/json")),
                (String::from("Access-Control-Request-Headers"), String::from("*")),
                (String::from("api-key"), self.api_key.clone()),
            ];

            let response = pink::http_req!(
                method,
                url,
                data.clone(),
                headers
            );

            serde_json_core::from_slice::<InsertedMongoDBDocumentResult>(response.body.as_slice())
                .map_err(|_| { ErrorInsertingToDB })
                .map(|(inserted_document_mongo, _)| { inserted_document_mongo.insertedId })?
                .ok_or(ErrorInsertingToDB)
        }

        pub fn bytes_to_hex_string(bytes: [u8; 32]) -> Result<heapless::String<95>> {
            let mut s: heapless::String<95> = heapless::String::new(); // Adjust the size accordingly

            for (i, byte) in bytes.iter().enumerate() {
                s.push_str(&format!("{:02x}", byte)).map_err(|_| { WrongPlayerAddressArgument })?; // Push each byte as a hex string

                if i < bytes.len() - 1 {
                    s.push('-').map_err(|_| { WrongPlayerAddressArgument })?; // Separate bytes with a '-'
                }
            }

            Ok(s)
        }

        pub fn check_move_boundaries(chess_move: &ChessMove) -> Result<()> {
            if chess_move.from.0 > 7 || chess_move.from.1 > 7 {
                return Err(OutOfBoardChessFrom);
            }
            if chess_move.to.0 > 7 || chess_move.to.1 > 7 {
                return Err(OutOfBoardChessTo);
            }
            Ok(())
        }

        pub fn check_move_validity_for_piece(game_state: &GameState, chess_move: &ChessMove,) -> Result<()> {
            let ChessCell{ piece, .. } = match &game_state.board[chess_move.from.0 as usize][chess_move.from.1 as usize] {
                Some(ref chess_cell) => chess_cell,
                None => return Err(NoPieceBoardChessFrom), // No piece at source
            };

            match piece {
                Piece::Pawn => Self::check_move_validity_pawn(&game_state, &chess_move),
                Piece::Knight => Self::check_move_validity_knight(&chess_move),
                Piece::Bishop => Self::check_bishop_move_validity(&game_state, &chess_move),
                Piece::Rook => Self::check_move_validity_rook(&game_state, &chess_move),
                Piece::Queen => Self::check_move_validity_queen(&game_state, &chess_move),
                Piece::King => Self::check_move_validity_king(&chess_move),
            }
        }

        pub fn check_move_validity_pawn(game_state: &GameState, chess_move: &ChessMove) -> Result<()> {
            let (fx, fy) = chess_move.from;
            let (tx, ty) = chess_move.to;
            let player = &game_state.turn;

            let forward = match player {
                Player::White => 1,
                Player::Black => -1,
            };

            // Check forward move of 1 step
            if fx as i32 + forward == tx as i32 && fy == ty && game_state.board[tx as usize][ty as usize].is_none() {
                return Ok(());
            }

            // Check forward move of 2 steps
            if ((player == &Player::White && fx == 1) || (player == &Player::Black && fx == 6))
                && tx as i32 == fx as i32 + 2 * forward && fy == ty && game_state.board[tx as usize][ty as usize].is_none()
                && game_state.board[(fx as i32 + forward) as usize][fy as usize].is_none() {
                // Check if the path is clear
                return Ok(());
            }

            // Check capture move
            if fx as i32 + forward == tx as i32 && (fy as i32 - 1 == ty as i32 || fy as i32 + 1 == ty as i32) {
                if let Some(ChessCell{player: piece_player, ..}) = &game_state.board[tx as usize][ty as usize] {
                    if *player != *piece_player {
                        // Capture if it's an opponent's piece
                        return Ok(())
                    }
                }
            }

            Err(NonValidMove)
        }

        pub fn check_move_validity_knight(chess_move: &ChessMove) -> Result<()> {
            let (fx, fy, tx, ty) = (chess_move.from.0, chess_move.from.1, chess_move.to.0, chess_move.to.1);
            let dx = (fx as i32 - tx as i32).abs();
            let dy = (fy as i32 - ty as i32).abs();

            // Check L-shape move
            if (dx == 2 && dy == 1) || (dx == 1 && dy == 2) { Ok(()) } else { Err(NonValidMove) }
        }

        pub fn check_bishop_move_validity(game_state: &GameState, chess_move: &ChessMove) -> Result<()> {
            // Bishop can move diagonally
            let is_diagonal = (chess_move.from.0 as i32 - chess_move.to.0 as i32).abs() == (chess_move.from.1 as i32 - chess_move.to.1 as i32).abs();

            if is_diagonal {
                // Diagonal move: Ensure the path is clear
                if Self::is_path_clear(&(game_state.board), chess_move, &Direction::Diagonal) { Ok(()) }
                else { Err(NonValidMove) }
            } else { Err(NonValidMove) }
        }

        pub fn check_move_validity_rook(game_state: &GameState, chess_move: &ChessMove) -> Result<()> {
            // Rook can move horizontally or vertically
            let is_horizontal = chess_move.from.0 == chess_move.to.0;
            let is_vertical = chess_move.from.1 == chess_move.to.1;

            if is_horizontal {
                // Horizontal move: Ensure the path is clear
                if Self::is_path_clear(&(game_state.board), chess_move, &Direction::Horizontal) { Ok(()) }
                else { Err(NonValidMove) }
            } else if is_vertical {
                // Vertical move: Ensure the path is clear
                if Self::is_path_clear(&(game_state.board), chess_move, &Direction::Vertical) { Ok(()) }
                else { Err(NonValidMove) }
            } else {
                Err(NonValidMove)
            }
        }

        pub fn check_move_validity_king(chess_move: &ChessMove) -> Result<()> {
            // Calculate the difference in the move for both axes
            let delta_row = (chess_move.from.0 as i8 - chess_move.to.0 as i8).abs();
            let delta_col = (chess_move.from.1 as i8 - chess_move.to.1 as i8).abs();

            if delta_row <= 1 && delta_col <= 1 { Ok(()) } else { Err(NonValidMove) }
        }

        pub fn check_move_validity_queen(game_state: &GameState, chess_move: &ChessMove) -> Result<()> {
            // Queen can move horizontally, vertically, or diagonally
            let from = chess_move.from;
            let to = chess_move.to;
            let is_horizontal = from.0 == to.0;
            let is_vertical = from.1 == to.1;
            let is_diagonal = (from.0 as i32 - to.0 as i32).abs() == (from.1 as i32 - to.1 as i32).abs();

            if is_horizontal {
                if Self::is_path_clear(&(game_state.board), chess_move, &Direction::Horizontal) { Ok(()) }
                else { Err(NonValidMove) }
            } else if is_vertical {
                if Self::is_path_clear(&(game_state.board), chess_move, &Direction::Vertical) { Ok(()) }
                else { Err(crate::oh_my_chess::Error::NonValidMove) }
            } else if is_diagonal {
                if Self::is_path_clear(&(game_state.board), chess_move, &Direction::Diagonal) { Ok(()) }
                else { Err(crate::oh_my_chess::Error::NonValidMove) }
            } else {
                Err(NonValidMove)
            }
        }

        fn is_path_clear(board: &[[Option<ChessCell>; 8]; 8], chess_move: &ChessMove, direction: &Direction) -> bool {
            let from = chess_move.from;
            let to = chess_move.to;
            let (dx, dy) = (to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32);
            let step_x = dx.signum() as u8;
            let step_y = dy.signum() as u8;

            // Check if movement is according to the piece's moving pattern
            match direction {
                Direction::Horizontal => if dy != 0 { return false; },
                Direction::Vertical => if dx != 0 { return false; },
                Direction::Diagonal => if dx.abs() != dy.abs() { return false; },
            }

            let mut current_x = from.0;
            let mut current_y = from.1;

            while (current_x, current_y) != (to.0, to.1) {
                current_x += step_x;
                current_y += step_y;

                // Avoid checking the destination square for a piece
                if (current_x, current_y) == (to.0, to.1) {
                    break;
                }

                // Check if the path is clear
                if board[current_x as usize][current_y as usize].is_some() {
                    return false;
                }
            }
            true
        }
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TypeInfo)]
    pub enum Direction {
        Horizontal,
        Vertical,
        Diagonal,
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Copy, Clone, Debug, PartialEq, TypeInfo)]
    pub enum Piece {
        Pawn, Knight, Bishop, Rook, Queen, King
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Copy, Clone, Debug, PartialEq, TypeInfo)]
    pub enum Player {
        Black, White
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TypeInfo)]
    pub struct PlayersAddresses {
        black: [u8; 32],
        white: [u8; 32],
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TypeInfo)]
    pub enum GameStatus {
        Ongoing, Finished, Stalemate, Draw
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, TypeInfo)]
    pub struct GameState {
        board: [[Option<ChessCell>; 8]; 8],
        turn: Player,
        players: PlayersAddresses,
        status: GameStatus,
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Copy, Clone, Debug, PartialEq, TypeInfo)]
    pub struct ChessCell {
        piece: Piece,
        player: Player,
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TypeInfo)]
    pub struct ChessMove {
        from: (u8, u8),
        to: (u8, u8),
    }

    #[derive(Encode, Decode, Deserialize, Clone, Debug)]
    pub struct FindMongoDBDocumentResult {
        document: Option<GameState>
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct InsertedMongoDBDocumentResult {
        insertedId: Option<heapless::String<32>>
    }


}
