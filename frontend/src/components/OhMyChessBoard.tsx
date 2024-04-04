import React, { ReactNode, useEffect, useMemo, useState } from "react";
import { Chess, Color, Square } from "chess.js";
import { Chessboard } from "react-chessboard";
import { CustomPieceFn, CustomPieceFnArgs } from "react-chessboard/dist/chessboard/types";
import usePhatContractOhMyChess from "@/hooks/usePhatContractOhMyChess";
import { GameSession, gameSessionToFen, getBoardOrientation, isPlayerTurn } from "@/models/game-session";
import { InjectedAccount } from "@phala/sdk";

export type PieceSymbolUppercase = 'P' | 'N' | 'B' | 'R' | 'Q' | 'K';
type Piece = `${Color}${PieceSymbolUppercase}`;

const boardWrapper = {
    width: `70vw`,
    maxWidth: "70vh",
    margin: "3rem auto",
};

const chessNotationToTuple = (square: Square): [number, number] => {
    // Map for columns 'a' through 'h' to 0 through 7
    const columnMap: { [key: string]: number } = {
        'a': 0, 'b': 1, 'c': 2, 'd': 3, 'e': 4, 'f': 5, 'g': 6, 'h': 7,
    };
    // Extract the column letter and row number from the notation
    const columnLetter: string = square[0];
    const rowIndex: number = parseInt(square[1], 10) - 1;
    const columnIndex: number = columnMap[columnLetter];

    return [rowIndex, columnIndex];
}


interface Props {
    networkUrl: string;
    accountAddress?: InjectedAccount;
    sessionId: string | undefined;
}

const EMPTY_FREN: string = '8/8/8/8/8/8/8/8 w - - 0 1';

const OhMyChessBoard = ({networkUrl, accountAddress, sessionId}: Props): ReactNode => {
    const [game, _] = useState<Chess>(new Chess());
    const {useQueryGameSessionFn, useMakeChessMoveMutation } = usePhatContractOhMyChess({networkUrl, accountAddress});
    const { data: gameSession } = useQueryGameSessionFn(sessionId);
    const [gamePosition, setGamePosition] = useState(EMPTY_FREN);
    const [activeSquare, setActiveSquare] = useState("");
    const [boardOrientation, setBoardOrientation] = useState<'white' | 'black' | undefined>(undefined);

    useEffect((): void => {
        if (!sessionId) {
            setGamePosition(EMPTY_FREN);
        }
    }, [sessionId]);

    useEffect((): void => {
        if (sessionId && gameSession && accountAddress) {
            const fen = gameSessionToFen(gameSession as GameSession, accountAddress as InjectedAccount);
            const orientation = getBoardOrientation(gameSession as GameSession, accountAddress as InjectedAccount);
            setBoardOrientation(orientation);
            setGamePosition(fen);
            game.load(fen);
        }
    }, [gameSession, accountAddress, sessionId]);

    const onDrop = (sourceSquare: Square, targetSquare: Square, piece: Piece): boolean => {
        try {
            const from = chessNotationToTuple(sourceSquare);
            const to = chessNotationToTuple(targetSquare);

            console.log('--session: ', sessionId, '--turn: ', gameSession?.turn, '--address: ', accountAddress?.address);

            const move = game.move({
                from: sourceSquare,
                to: targetSquare,
                promotion: (piece as unknown as string)[1].toLowerCase() ?? "q",
            });
            game.undo();

            if (move && isPlayerTurn(gameSession, accountAddress)) {
                useMakeChessMoveMutation.mutate({sessionId: sessionId as string, chessMove: {from, to}}, {
                    onSuccess: async() => {
                        game.move({
                            from: sourceSquare,
                            to: targetSquare,
                            promotion: (piece as unknown as string)[1].toLowerCase() ?? "q",
                        });

                        setGamePosition(game.fen());
                    },
                    onError: (error: Error): void => {
                        console.error(error);
                    }
                });
            }

            // exit if the game is over
            return !(game.isGameOver() || game.isDraw());
        } catch (e) {
            return false;
        }
    };


    const threeDPieces = useMemo(() => {
        const pieces: {piece: Piece, pieceHeight: number}[] = [
            { piece: "wP", pieceHeight: 1 },
            { piece: "wN", pieceHeight: 1.2 },
            { piece: "wB", pieceHeight: 1.2 },
            { piece: "wR", pieceHeight: 1.2 },
            { piece: "wQ", pieceHeight: 1.5 },
            { piece: "wK", pieceHeight: 1.6 },
            { piece: "bP", pieceHeight: 1 },
            { piece: "bN", pieceHeight: 1.2 },
            { piece: "bB", pieceHeight: 1.2 },
            { piece: "bR", pieceHeight: 1.2 },
            { piece: "bQ", pieceHeight: 1.5 },
            { piece: "bK", pieceHeight: 1.6 },
        ];

        const pieceComponents = {} as Record<Piece, CustomPieceFn>;
        pieces.forEach(({ piece, pieceHeight }, index): void => {
            pieceComponents[piece] = ({
                squareWidth,
            }: CustomPieceFnArgs): React.JSX.Element => (
                <div
                    style={{
                        width: squareWidth,
                        height: squareWidth,
                        position: "relative",
                        pointerEvents: "none",
                    }}
                >
                    <img
                        src={`chess-pieces/3d-pieces/${piece}.webp`}
                        width={squareWidth}
                        height={pieceHeight * squareWidth}
                        style={{
                            position: "absolute",
                            bottom: `${0.2 * squareWidth}px`,
                            objectFit: piece[1] === "K" ? "contain" : "cover",
                        }}
                        alt={`${index}`}/>
                </div>
            );
        });
        return pieceComponents;
    }, []);

    return (
        <div style={boardWrapper}>
            <Chessboard
                id="Styled3DBoard"
                position={gamePosition}
                onPieceDrop={onDrop}
                boardOrientation={boardOrientation}
                autoPromoteToQueen={true}
                customBoardStyle={{
                    transform: "rotateX(27.5deg)",
                    transformOrigin: "center",
                    border: "16px solid #b8836f",
                    borderStyle: "outset",
                    borderRightColor: " #b27c67",
                    borderRadius: "4px",
                    boxShadow: "rgba(0, 0, 0, 0.5) 2px 24px 24px 8px",
                    borderRightWidth: "2px",
                    borderLeftWidth: "2px",
                    borderTopWidth: "0px",
                    borderBottomWidth: "18px",
                    borderTopLeftRadius: "8px",
                    borderTopRightRadius: "8px",
                    padding: "8px 8px 12px",
                    background: "#e0c094",
                    backgroundImage: 'url("wood-pattern.png")',
                    backgroundSize: "cover",
                }}
                customPieces={threeDPieces}
                customLightSquareStyle={{
                    backgroundColor: "#e0c094",
                    backgroundImage: 'url("wood-pattern.png")',
                    backgroundSize: "cover",
                }}
                customDarkSquareStyle={{
                    backgroundColor: "#865745",
                    backgroundImage: 'url("wood-pattern.png")',
                    backgroundSize: "cover",
                }}
                animationDuration={500}
                customSquareStyles={{
                    [activeSquare]: {
                        boxShadow: "inset 0 0 1px 6px rgba(255,255,255,0.75)",
                    },
                }}
                onMouseOverSquare={(sq) => setActiveSquare(sq)}
                onMouseOutSquare={() => setActiveSquare("")}
            />
        </div>
    );
};

export default OhMyChessBoard;