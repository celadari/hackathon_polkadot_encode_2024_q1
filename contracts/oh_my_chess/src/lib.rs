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
    use crate::oh_my_chess::Error::{GameSessionIsOver, CouldNotFindKingPiece, SessionNeedsSecondPlayer, ImpossibleError, AlreadyPlayingAsOpponent, ThisSessionContainsAlreadyTwoPlayers, AlreadyInThisGameSession, WrongPlayerAddressArgument, ErrorInsertingToDB, CouldNotUpdateDB, ToIsOccupiedByOneOfYourPiece, PieceSelectedIsNotYours, NoPieceBoardChessFrom, OutOfBoardChessFrom, OutOfBoardChessTo, NonValidMove, NoElementFoundInDB, ErrorFetchingFromDB, NotAuthorized, NotYourTurn, NotInThisGameSession};
    use scale_info::TypeInfo;


    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        GameSessionIsOver,
        NonValidMove,
        NotYourTurn,
        NotInThisGameSession,
        AlreadyInThisGameSession,
        ThisSessionContainsAlreadyTwoPlayers,
        NoElementFoundInDB,
        AlreadyPlayingAsOpponent,
        ImpossibleError,
        CouldNotFindKingPiece,
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
        SessionNeedsSecondPlayer,
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
        pub fn start_new_game_session(&self, player: Option<Player>, second_player_address: Option<[u8; 32]>) -> Result<String> {
            let caller: [u8; 32] = *Self::env().caller().as_ref();
            let player = player.unwrap_or(Player::White); // Default to White if no player specified
            let (player_white_address_opt, player_black_address_opt): (Option<[u8; 32]>, Option<[u8; 32]>) = match player {
                Player::White => (Some(caller), second_player_address),
                Player::Black => (second_player_address, Some(caller)),
            };

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
            let game_state = GameStateLobby {
                board: initial_board,
                turn: Player::White, // White starts the game
                players: PlayersAddressesLobby {
                    black: player_black_address_opt,
                    white: player_white_address_opt,
                },
                status: GameStatus::Ongoing,
            };
            let inserted_document_id = self.insert_game_session_to_mongodb(game_state)?;
            if let Some(player_black_address) = player_black_address_opt {
                self.update_players_sessions_track_in_mongodb(inserted_document_id.clone(), player_black_address)?;
            }
            if let Some(player_white_address) = player_white_address_opt {
                self.update_players_sessions_track_in_mongodb(inserted_document_id.clone(), player_white_address)?;
            }

            Ok(String::from(inserted_document_id.as_str()))
        }

        #[ink(message)]
        pub fn join_session(&self, session_id: String) -> Result<()> {
            // Attempt to fetch the game session from the database using session_id
            let mut game_state_lobby = self.find_lobby_game_session_from_mongodb(session_id.clone())?;

            // Check if the caller is already part of the game
            let caller: [u8; 32] = *Self::env().caller().as_ref();
            let caller_some = Some(caller);

            if game_state_lobby.players.white == caller_some || game_state_lobby.players.black == caller_some {
                return Err(AlreadyInThisGameSession);
            }

            // Check for a free spot in the game and join the game
            match (game_state_lobby.players.white, game_state_lobby.players.black) {
                (None, Some(black)) if black == caller => {
                    // The player is trying to join as white but is already playing as black
                    return Err(AlreadyPlayingAsOpponent);
                },
                (Some(white), None) if white == caller => {
                    // The player is trying to join as black but is already playing as white
                    return Err(AlreadyPlayingAsOpponent);
                },
                (None, Some(_)) => {
                    // There is a free spot for white, and the caller is not playing as black
                    game_state_lobby.players.white = Some(caller);
                },
                (Some(_), None) => {
                    // There is a free spot for black, and the caller is not playing as white
                    game_state_lobby.players.black = Some(caller);
                },
                _ => {
                    return Err(ThisSessionContainsAlreadyTwoPlayers);
                }
            }

            let game_state = match (game_state_lobby.players.white, game_state_lobby.players.black) {
                (Some(white), Some(black)) => GameState {
                    board: game_state_lobby.board,
                    turn: game_state_lobby.turn,
                    players: PlayersAddresses {
                        white,
                        black,
                    },
                    status: game_state_lobby.status,
                },
                // If this branch is reached, then the session is in an invalid state for conversion
                _ => return Err(ImpossibleError),
            };

            let session_id_heapless: heapless::String<32> = heapless::String::from(session_id.clone().as_str());

            // Update the game session in the database
            self.update_game_session_to_mongodb(game_state, session_id.clone())?;

            self.update_players_sessions_track_in_mongodb(session_id_heapless, caller)?;

            Ok(())
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
            let player = game_state.turn;

            // check validity of the move
            self.check_for_game_state(&game_state)?;
            Self::check_move_boundaries(&chess_move)?;
            self.check_caller_turn(&game_state)?;
            self.check_caller_owns_piece(&game_state, &chess_move)?;
            Self::check_move_validity_for_piece(&(game_state.board), &player, &chess_move)?;
            Self::check_if_move_puts_in_check(&(game_state.board), &player, &chess_move)?;

            // update game_state: board, status
            Self::update_board_from_move(&mut game_state.board, &player, &chess_move)?;
            Self::update_turn_and_status(&mut game_state)?;

            // update mongodb
            self.update_game_session_to_mongodb(game_state, session_id)
        }

        fn check_for_game_state(&self, game_state: &GameState) -> Result<()> {
            if game_state.status == GameStatus::Ongoing {
                Ok(())
            } else {
                Err(GameSessionIsOver)
            }
        }

        fn check_if_move_puts_in_check(board: &Board, player: &Player, chess_move: &ChessMove) -> Result<()> {
            let mut temp_game_state = board.clone();
            Self::update_board_from_move(&mut temp_game_state, &player, &chess_move)?;

            // Check if the move has put the current player's king in check
            if Self::is_in_check(&temp_game_state, player)? {
                return Err(NonValidMove);
            }
            Ok(())
        }

        fn update_turn_and_status(game_state: &mut GameState) -> Result<()> {
            let current_player = game_state.turn;
            let opposing_player = match current_player {
                Player::Black => Player::White,
                Player::White => Player::Black,
            };

            if Self::is_check_mate(&(game_state.board), &opposing_player)? {
                game_state.status = match current_player {
                    Player::Black => GameStatus::WonByPlayerBlack,
                    Player::White => GameStatus::WonByPlayerWhite,
                }
            } else {
                game_state.turn = opposing_player;
            }
            Ok(())
        }

        fn update_board_from_move(board: &mut Board, player: &Player, chess_move: &ChessMove) -> Result<()> {
            let (fx, fy) = chess_move.from;
            let (tx, ty) = chess_move.to;

            let chess_cell_option = board[fx as usize][fy as usize].take(); // Directly take the value out
            if let Some(ChessCell { piece: Piece::Pawn, .. }) = &chess_cell_option {
                // Check if the pawn reaches the end and should be promoted to a queen
                if tx == 7 || tx == 0 {
                    board[tx as usize][ty as usize] = Some(ChessCell { piece: Piece::Queen, player: *player });
                } else {
                    board[tx as usize][ty as usize] = chess_cell_option;
                }
            } else {
                board[tx as usize][ty as usize] = chess_cell_option;
            }

            // Clear the original location after moving the piece
            board[fx as usize][fy as usize] = None;

            Ok(())
        }

        pub fn check_caller_owns_piece(&self, game_state: &GameState, chess_move: &ChessMove) -> Result<()> {
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

        pub fn check_caller_turn(&self, game_state: &GameState) -> Result<()> {
            let caller = Self::env().caller();
            let player = &game_state.turn;
            let player_address = if *player == Player::Black { game_state.players.black } else { game_state.players.white };

            if caller != AccountId::from(game_state.players.black) && caller != AccountId::from(game_state.players.white) {
                // The caller is neither black nor white player, return YourNotInThisGameSession error
                return Err(NotInThisGameSession);
            }
            if AccountId::from(player_address) != caller {
                return Err(NotYourTurn);
            }

            Ok(())
        }

        #[ink(message)]
        pub fn find_lobby_game_session_from_mongodb(&self, session_id: String) -> Result<GameStateLobby> {
            let method = String::from("POST"); // HTTP Method for the request
            let url = format!("{}/action/findOne", self.url);

            let data = format!(r#"{{
                "collection":"game_sessions",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "filter": {{"_id": {{"$oid": "{}"}}}},
                "projection":{{"_id":0,"turn":1,"status":1,"players":1,"board":1}}
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

            let game_state_res = serde_json_core::from_slice::<FindMongoDBDocumentResult>(response.body.as_slice())
                .map_err(|_| { ErrorFetchingFromDB })
                .map(|(mongodb, _)| { mongodb.document })?
                .ok_or(NoElementFoundInDB);

            game_state_res
        }

        pub fn find_game_session_from_mongodb(&self, session_id: String) -> Result<GameState> {
            let fetched_game_state_lobby: GameStateLobby = self.find_lobby_game_session_from_mongodb(session_id)?;

            // Check if both player addresses are defined
            let players_addresses = match (fetched_game_state_lobby.players.white, fetched_game_state_lobby.players.black) {
                (Some(white), Some(black)) => PlayersAddresses { white, black },
                _ => return Err(SessionNeedsSecondPlayer),
            };

            // Convert to GameState, ensuring all fields are properly populated
            let game_state = GameState {
                board: fetched_game_state_lobby.board,
                turn: fetched_game_state_lobby.turn,
                players: players_addresses,
                status: fetched_game_state_lobby.status,
            };

            Ok(game_state)
        }

        pub fn update_players_sessions_track_in_mongodb(&self, session_id: heapless::String<32>, player_address: [u8; 32]) -> Result<()> {
            let method = String::from("POST"); // HTTP Method for the request
            let player_address_hex_string = Self::bytes_to_hex_string(player_address)?;
            let url = format!("{}/action/updateOne?_id={}", self.url, player_address_hex_string);

            let data = format!(r#"{{
                "collection":"players_sessions_trackers",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "filter":{{ "_id": "{}" }},
                "update":{{"$push": {{"sessions": "{}"}} }},
                "upsert":true
            }}"#, player_address_hex_string, session_id).as_bytes().to_vec();

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

            if response.status_code == 200 || response.status_code == 201 { Ok(()) }
            else { Err(CouldNotUpdateDB) }
        }

        #[ink(message)]
        pub fn find_players_sessions_track_in_mongodb(&self) -> Result<[Option<String>; 10]> {
            let caller: [u8; 32] = *Self::env().caller().as_ref();
            let method = String::from("POST");
            let player_address_hex_string = Self::bytes_to_hex_string(caller)?;
            let url = format!("{}/action/findOne?_id={}", self.url, player_address_hex_string);

            let data = format!(r#"{{
                "collection":"players_sessions_trackers",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "filter":{{"_id": "{}"}},
                "projection":{{"sessions": 1, "_id": 0}}
            }}"#, player_address_hex_string).as_bytes().to_vec();

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

            if response.status_code != 200 {
                return Err(ErrorFetchingFromDB);
            }

            let sessions_vec = serde_json_core::from_slice::<FindMongoDBTrackDocumentResult>(&response.body)
                .map_err(|_| { ErrorFetchingFromDB })
                .map(|(mongodb, _)| { mongodb.document })?;

            let sessions_vec = if let Some(vec) = sessions_vec {
                vec
            } else {
                let mut sessions_array: [Option<String>; 10] = Default::default();
                for i in 0..sessions_array.len() {
                    sessions_array[i] = None;
                }
                return Ok(sessions_array);
            };

            let mut sessions_array: [Option<String>; 10] = Default::default();
            for (index, session) in sessions_vec.sessions.iter().enumerate() {
                if index < sessions_array.len() {
                    sessions_array[index] = Some(String::from(session.as_str())); // Fill the array up to its capacity or the number of elements in the Vec
                } else {
                    break; // If the Vec has more elements than the array can hold, stop filling the array
                }
            }

            Ok(sessions_array)
        }

        pub fn update_game_session_to_mongodb(&self, game_state: GameState, session_id: String) -> Result<()> {
            let method = String::from("POST"); // HTTP Method for the request
            let url = format!("{}/action/updateOne", self.url);

            let json_game_state: heapless::String<4096> = serde_json_core::ser::to_string(&game_state).map_err(|_| { CouldNotUpdateDB })?;

            let data = format!(r#"{{
                "collection":"game_sessions",
                "database":"hackathon",
                "dataSource":"Cluster0",
                "filter": {{"_id": {{"$oid": "{}"}}}},
                "update":{{"$set": {}}}
            }}"#, session_id, json_game_state).as_bytes().to_vec();

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

        pub fn insert_game_session_to_mongodb(&self, game_state: GameStateLobby) -> Result<heapless::String<32>> {
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

        pub fn check_move_validity_for_piece(board: &Board, player: &Player, chess_move: &ChessMove) -> Result<()> {
            let ChessCell{ piece, .. } = match &board[chess_move.from.0 as usize][chess_move.from.1 as usize] {
                Some(ref chess_cell) => chess_cell,
                None => return Err(NoPieceBoardChessFrom), // No piece at source
            };

            match piece {
                Piece::Pawn => Self::check_move_validity_pawn(&board, &player, &chess_move),
                Piece::Knight => Self::check_move_validity_knight(&chess_move),
                Piece::Bishop => Self::check_move_validity_bishop(&board, &chess_move),
                Piece::Rook => Self::check_move_validity_rook(&board, &chess_move),
                Piece::Queen => Self::check_move_validity_queen(&board, &chess_move),
                Piece::King => Self::check_move_validity_king(&chess_move),
            }
        }

        // Helper function to find the king's position for the given player
        pub fn find_king_position(board: &Board, player: &Player) -> Result<(u8, u8)> {
            for x in 0..8 {
                for y in 0..8 {
                    if let Some(ChessCell { piece: Piece::King, player: cell_player }) = board[x][y] {
                        if cell_player == *player {
                            return Ok((x as u8, y as u8));
                        }
                    }
                }
            }

            Err(CouldNotFindKingPiece) // It should be impossible to not find the king
        }

        pub fn is_check_mate(board: &Board, player: &Player) -> Result<bool> {
            let (king_x, king_y) = Self::find_king_position(board, player)?;

            // For each cell on the board, check if it's an opponent's piece that can threaten the king
            for x in 0..8 {
                for y in 0..8 {
                    if let Some(ChessCell { piece: _, player: cell_player }) = board[x][y] {
                        if cell_player != (*player) {
                            let threat_move = ChessMove { from: (x as u8, y as u8), to: (king_x, king_y) };
                            // Check if this opponent's piece can move to capture the king
                            if Self::check_move_validity_for_piece(&board, &player, &threat_move).is_ok() {
                                // If this piece can threaten the king, check for a response
                                if !Self::has_response_to_threat(&board, &player, (x, y))? {
                                    return Ok(false);
                                }
                            }
                        }
                    }
                }
            }

            // If we've checked all pieces and there is always response to threat, the player can escape from check
            Ok(true)
        }

        fn has_response_to_threat(board: &Board, player: &Player, threat_position: (usize, usize)) -> Result<bool> {
            let (king_x, king_y) = Self::find_king_position(board, player)?;
            let (threat_x, threat_y) = threat_position;

            // Calculate direction of the threat
            let dx = (king_x as i32 - threat_x as i32).signum();
            let dy = (king_y as i32 - threat_y as i32).signum();

            let mut check_x = threat_x as i32 + dx;
            let mut check_y = threat_y as i32 + dy;

            // Iterate through the path from the threat to the king
            while (check_x != king_x as i32 || check_y != king_y as i32) && (check_x >= 0 && check_x < 8 && check_y >= 0 && check_y < 8) {
                // For each player's piece, check if moving it to (check_x, check_y) blocks the threat
                for x in 0..8 {
                    for y in 0..8 {
                        if let Some(ChessCell { player: cell_player, .. }) = board[x][y] {
                            if cell_player == *player {
                                let potential_block_move = ChessMove { from: (x as u8, y as u8), to: (check_x as u8, check_y as u8) };
                                if Self::check_move_validity_for_piece(&board, &player, &potential_block_move).is_ok() {
                                    let mut temp_board = board.clone();
                                    Self::update_board_from_move(&mut temp_board, &player, &potential_block_move)?;
                                    if !Self::is_in_check(&temp_board, &player)? {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                }

                check_x += dx;
                check_y += dy;
            }

            Ok(false)
        }

        pub fn is_in_check(board: &Board, player: &Player) -> Result<bool> {
            let king_position = Self::find_king_position(&board, &player)?;
            for x in 0..8 {
                for y in 0..8 {
                    if let Some(ChessCell { piece: _, player: cell_player }) = board[x][y] {
                        if cell_player != *player {
                            // Temporarily make a move
                            let chess_move = ChessMove { from: (x as u8, y as u8), to: king_position };
                            if Self::check_move_validity_for_piece(board, &player, &chess_move).is_ok() {
                                // If any opponent's piece can move to the king's position, the player is in check
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            Ok(false)
        }

        pub fn check_move_validity_pawn(board: &Board, player: &Player, chess_move: &ChessMove) -> Result<()> {
            let (fx, fy) = chess_move.from;
            let (tx, ty) = chess_move.to;

            let forward = match player {
                Player::White => 1,
                Player::Black => -1,
            };

            // Check forward move of 1 step
            if fx as i32 + forward == tx as i32 && fy == ty && board[tx as usize][ty as usize].is_none() {
                return Ok(());
            }

            // Check forward move of 2 steps
            if ((player == &Player::White && fx == 1) || (player == &Player::Black && fx == 6))
                && tx as i32 == fx as i32 + 2 * forward && fy == ty && board[tx as usize][ty as usize].is_none()
                && board[(fx as i32 + forward) as usize][fy as usize].is_none() {
                // Check if the path is clear
                return Ok(());
            }

            // Check capture move
            if fx as i32 + forward == tx as i32 && (fy as i32 - 1 == ty as i32 || fy as i32 + 1 == ty as i32) {
                if let Some(ChessCell{player: piece_player, ..}) = board[tx as usize][ty as usize] {
                    if *player != piece_player {
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

        pub fn check_move_validity_bishop(board: &Board, chess_move: &ChessMove) -> Result<()> {
            // Bishop can move diagonally
            let is_diagonal = (chess_move.from.0 as i32 - chess_move.to.0 as i32).abs() == (chess_move.from.1 as i32 - chess_move.to.1 as i32).abs();

            if is_diagonal {
                // Diagonal move: Ensure the path is clear
                if Self::is_path_clear(&board, chess_move) { Ok(()) }
                else { Err(NonValidMove) }
            } else { Err(NonValidMove) }
        }

        pub fn check_move_validity_rook(board: &Board, chess_move: &ChessMove) -> Result<()> {
            // Rook can move horizontally or vertically
            let is_horizontal = chess_move.from.0 == chess_move.to.0;
            let is_vertical = chess_move.from.1 == chess_move.to.1;

            if is_horizontal {
                // Horizontal move: Ensure the path is clear
                if Self::is_path_clear(&board, chess_move) { Ok(()) }
                else { Err(NonValidMove) }
            } else if is_vertical {
                // Vertical move: Ensure the path is clear
                if Self::is_path_clear(&board, chess_move) { Ok(()) }
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

        pub fn check_move_validity_queen(board: &Board, chess_move: &ChessMove) -> Result<()> {
            // Queen can move horizontally, vertically, or diagonally
            let from = chess_move.from;
            let to = chess_move.to;
            let is_horizontal = from.0 == to.0;
            let is_vertical = from.1 == to.1;
            let is_diagonal = (from.0 as i32 - to.0 as i32).abs() == (from.1 as i32 - to.1 as i32).abs();

            if is_horizontal {
                if Self::is_path_clear(&board, chess_move) { Ok(()) }
                else { Err(NonValidMove) }
            } else if is_vertical {
                if Self::is_path_clear(&board, chess_move) { Ok(()) }
                else { Err(NonValidMove) }
            } else if is_diagonal {
                if Self::is_path_clear(&board, chess_move) { Ok(()) }
                else { Err(NonValidMove) }
            } else {
                Err(NonValidMove)
            }
        }

        fn is_path_clear(board: &[[Option<ChessCell>; 8]; 8], chess_move: &ChessMove) -> bool {
            let from = (chess_move.from.0 as i16, chess_move.from.1 as i16);
            let to = (chess_move.to.0 as i16, chess_move.to.1 as i16);
            let (dx, dy) = (to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32);
            let step_x = dx.signum() as i16;
            let step_y = dy.signum() as i16;

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
    pub struct PlayersAddressesLobby {
        black: Option<[u8; 32]>,
        white: Option<[u8; 32]>,
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TypeInfo)]
    pub enum GameStatus {
        Ongoing, WonByPlayerBlack, WonByPlayerWhite, Stalemate, Draw
    }

    pub type Board = [[Option<ChessCell>; 8]; 8];

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, TypeInfo)]
    pub struct GameState {
        board: Board,
        turn: Player,
        players: PlayersAddresses,
        status: GameStatus,
    }

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, TypeInfo)]
    pub struct GameStateLobby {
        board: [[Option<ChessCell>; 8]; 8],
        turn: Player,
        players: PlayersAddressesLobby,
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
        document: Option<GameStateLobby>
    }

    #[derive(Deserialize, Clone, Debug)]
    pub struct MongoDBSessionDocument {
        sessions: heapless::Vec<heapless::String<32>, 10>
    }

    #[derive(Deserialize, Clone, Debug)]
    pub struct FindMongoDBTrackDocumentResult {
        document: Option<MongoDBSessionDocument>
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct InsertedMongoDBDocumentResult {
        insertedId: Option<heapless::String<32>>
    }


}
