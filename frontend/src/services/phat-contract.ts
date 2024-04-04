import {
    getClient,
    getContract, InjectedAccount,
    PinkContractPromise,
    UIKeyringProvider,
} from "@phala/sdk";
import { WsProvider } from "@polkadot/api";
import contractAbi from '@resources/oh_my_chess.json';
import {
    CreatedNewSessionResult,
    GetGameSessionResult,
    JoinSessionResult,
    ListSessionsResult, MakeChessMoveResult
} from "@/models/api-result";
import { GameSession, Player } from "@/models/game-session";



export interface CreateNewSessionArgs {
    player?: Player;
    secondPlayerAddress?: Uint8Array;
}

export interface JoinSessionArgs {
    sessionId: string;
}

export interface GetGameSessionArgs {
    sessionId: string;
}

export interface MakeChessMoveArgs {
    sessionId: string;
    chessMove: {
        from: [number, number];
        to: [number, number];
    };
}

export interface OhMyChessClient {
    contract: PinkContractPromise;
    fetchSessions: () => Promise<string[]>;
    createNewSession: (_: CreateNewSessionArgs) => Promise<string>;
    joinSession: (_: JoinSessionArgs) => Promise<void>;
    getGameSession: (_: GetGameSessionArgs) => Promise<GameSession>;
    makeChessMove: (_: MakeChessMoveArgs) => Promise<void>;
    clientId: string;
}

interface InputProps {
    networkUrl: string;
    accountAddress: InjectedAccount;
}

export const fetchInjectedAccounts = async({networkUrl}: {networkUrl: string}): Promise<InjectedAccount[]> => {
    return await UIKeyringProvider.getAllAccountsFromProvider('Oh My Chess', 'polkadot-js');
};

export const instantiateOhMyChessClient = async({networkUrl, accountAddress}: InputProps): Promise<OhMyChessClient> => {
    const client = await getClient({ transport: new WsProvider(networkUrl) });

    const accounts = await UIKeyringProvider.getAllAccountsFromProvider('Oh My Chess', 'polkadot-js');
    const provider = await UIKeyringProvider.create(client.api, 'Oh My Chess', 'polkadot-js', accountAddress);

    const contract = await getContract({
        client,
        contractId: process.env.NEXT_PUBLIC_PHAT_CONTRACT_OH_MY_CHESS as string,
        abi: contractAbi,
        provider,
    });

    const fetchSessions = async(): Promise<string[]> => {
        const contractCallOutcome = await contract.q.findPlayersSessionsTrackInMongodb({args: []});
        const listSessionsResult = contractCallOutcome.output.toJSON()?.valueOf() as ListSessionsResult;
        if ('err' in listSessionsResult) throw Error(listSessionsResult.err);
        else if ('err' in listSessionsResult.ok) throw Error(listSessionsResult.ok.err);
        else return listSessionsResult.ok.ok;
    }

    const createNewSession = async ({player, secondPlayerAddress}: CreateNewSessionArgs): Promise<string> => {
        const contractCallOutcome = await contract.q.startNewGameSession({args: [player || null, secondPlayerAddress || null]});
        const newCreatedSessionId = contractCallOutcome.output.toJSON()?.valueOf() as CreatedNewSessionResult;
        if ('err' in newCreatedSessionId) throw Error(newCreatedSessionId.err);
        else if ('err' in newCreatedSessionId.ok) throw Error(newCreatedSessionId.ok.err);
        else return newCreatedSessionId.ok.ok;
    };

    const joinSession = async ({sessionId}: JoinSessionArgs): Promise<void> => {
        const contractCallOutcome = await contract.q.joinSession({args: [sessionId]});
        const joinSessionResult = contractCallOutcome.output.toJSON()?.valueOf() as JoinSessionResult;
        if ('err' in joinSessionResult) throw Error(joinSessionResult.err);
        else if ('err' in joinSessionResult.ok) throw Error(joinSessionResult.ok.err);
        else return joinSessionResult.ok.ok;
    };

    const getGameSession = async({sessionId}: GetGameSessionArgs): Promise<GameSession> => {
        const contractCallOutcome = await contract.q.findLobbyGameSessionFromMongodb({args: [sessionId]});
        const gameSessionResult = contractCallOutcome.output.toJSON()?.valueOf() as GetGameSessionResult;
        if ('err' in gameSessionResult) throw Error(gameSessionResult.err);
        else if ('err' in gameSessionResult.ok) throw Error(gameSessionResult.ok.err);
        else return gameSessionResult.ok.ok;
    };

    const makeChessMove = async({sessionId, chessMove}: MakeChessMoveArgs): Promise<void> => {
        const contractCallOutcome = await contract.q.makeMove({args: [chessMove, sessionId]});
        const makeChessMoveResult = contractCallOutcome.output.toJSON()?.valueOf() as MakeChessMoveResult;
        if ('err' in makeChessMoveResult) throw Error(makeChessMoveResult.err);
        else if ('err' in makeChessMoveResult.ok) throw Error(makeChessMoveResult.ok.err);
        else return makeChessMoveResult.ok.ok;
    };

    const clientId = `${networkUrl}-${accountAddress.address}`

    return {contract, fetchSessions, createNewSession, joinSession, getGameSession, makeChessMove, clientId};
};

