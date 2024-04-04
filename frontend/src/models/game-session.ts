import { InjectedAccount } from "@phala/sdk";
import { encodeAddress } from '@polkadot/util-crypto'


export interface GameSession {
    board: ChessCell[][];
    turn: Player;
    players: PlayersAddresses;
    status: GameStatus;
}

export interface PlayersAddresses {
    black: Uint8Array,
    white: Uint8Array,
}

export enum Player {
    White = 'White',
    Black = 'Black',
}

export type ChessLocation = [number, number];

export enum Piece {
    Pawn = 'Pawn',
    Knight = 'Knight',
    Bishop = 'Bishop',
    Rook = 'Rook',
    Queen = 'Queen',
    King = 'King',
}

export interface ChessCell {
    piece: Piece;
    player: Player;
}

export enum GameStatus {
    Ongoing = 'Ongoing',
    Finished = 'Finished',
    Stalemate = 'Stalemate',
    Draw = 'Draw',
}

// Helper function to get FEN piece code
const getFenPieceCode = (piece: Piece, player: Player): string => {
    const pieceToFen = {
        [Piece.Pawn]: 'P',
        [Piece.Knight]: 'N',
        [Piece.Bishop]: 'B',
        [Piece.Rook]: 'R',
        [Piece.Queen]: 'Q',
        [Piece.King]: 'K',
    };

    const pieceCode = pieceToFen[piece];
    return player === Player.White ? pieceCode : pieceCode.toLowerCase();
};

export const gameSessionToFen = (gameSession: GameSession): string => {
    // Convert the board to FEN piece placement
    const fenRows = gameSession.board.toReversed().map(row =>
        row.map(cell =>
            cell ? getFenPieceCode(cell.piece, cell.player) : '1' // '1' represents an empty square
        ).join('')
            .replace(/1{1,8}/g, match => match.length.toString()) // Consolidate empty squares
    ).join('/');

    // Active color
    const activeColor = gameSession.turn === Player.White ? 'w' : 'b';

    // We'll assume castling availability and en passant target square are not provided and default to '-'
    // If your GameSession object includes this information, modify this section accordingly
    const castlingAvailability = '-';
    const enPassantTargetSquare = '-';

    // We'll also assume the halfmove and fullmove counters are not provided and default to '0' and '1'
    // You would need to provide these values if your GameSession object includes them
    const halfmoveClock = '0';
    const fullmoveNumber = '1';

    // Construct the full FEN string
    return `${fenRows} ${activeColor} ${castlingAvailability} ${enPassantTargetSquare} ${halfmoveClock} ${fullmoveNumber}`;
};

export const getBoardOrientation = (gameSession: GameSession, accountAddress: InjectedAccount): 'white' | 'black' => {
    if (gameSession.players.black && encodeAddress(accountAddress.address) === encodeAddress(gameSession.players.black)) return 'black';
    else if (gameSession.players.white && encodeAddress(accountAddress.address) === encodeAddress(gameSession.players.white)) return 'white';
    else throw Error('Current address cannot play for this address');
};

export const checkIfIsPlayerTurn = (gameSession: GameSession | undefined, accountAddress: InjectedAccount | undefined): boolean => {
    if (accountAddress && gameSession?.players.black && encodeAddress(accountAddress.address) === encodeAddress(gameSession.players.black) && gameSession.turn === Player.Black) return true;
    else if (accountAddress && gameSession?.players.white && encodeAddress(accountAddress.address) === encodeAddress(gameSession.players.white) && gameSession.turn === Player.White) return true;
    else return false;
};

