import { useState, useEffect } from 'react';
import {
    instantiateOhMyChessClient,
    CreateNewSessionArgs,
    JoinSessionArgs,
    OhMyChessClient,
    MakeChessMoveArgs
} from "@/services/phat-contract";
import { useMutation, UseMutationResult, useQuery, useQueryClient, UseQueryResult } from "@tanstack/react-query";
import { InjectedAccount } from "@phala/sdk";
import {GameSession} from "@/models/game-session";

interface InputProps {
    networkUrl: string;
    accountAddress?: InjectedAccount;
}

interface ReturnProps  {
    useQuerySessions: UseQueryResult<string[], Error>;
    useCreateNewSessionMutation: UseMutationResult<string, Error, CreateNewSessionArgs, any>;
    useJoinSessionMutation: UseMutationResult<void, Error, JoinSessionArgs, any>;
    useMakeChessMoveMutation: UseMutationResult<void, Error, MakeChessMoveArgs, any>;
    useQueryGameSessionFn: (_: string | undefined) => UseQueryResult<GameSession | undefined, Error>;
}

const useCreateNewSessionMutationFunc = (createNewSession: (_: CreateNewSessionArgs) => Promise<string>): UseMutationResult<string, Error, CreateNewSessionArgs, any> => {
    const queryClient = useQueryClient();
    const key = 'create-new-game-session';

    return useMutation({
        mutationKey: [key],
        mutationFn: createNewSession,
        onSuccess: (): void => {
            // Invalidate and refetch QuickBooks accounts data
            queryClient.invalidateQueries({ queryKey: [key] });
        },
    });
};

const useJoinSessionMutationFunc = (joinSession: (_: JoinSessionArgs) => Promise<void>): UseMutationResult<void, Error, JoinSessionArgs, any> => {
    const queryClient = useQueryClient();
    const key = 'join-game-session';

    return useMutation({
        mutationKey: [key],
        mutationFn: joinSession,
        onSuccess: (): void => {
            // Invalidate and refetch QuickBooks accounts data
            queryClient.invalidateQueries({ queryKey: [key] });
        },
    });
};

const useMakeChessMoveMutationFunc = (makeChessMove: (_: MakeChessMoveArgs) => Promise<void>): UseMutationResult<void, Error, MakeChessMoveArgs, any> => {
    const queryClient = useQueryClient();
    const key = 'make-chess-game';

    return useMutation({
        mutationKey: [key],
        mutationFn: makeChessMove,
        onSuccess: (): void => {
            // Invalidate and refetch QuickBooks accounts data
            queryClient.invalidateQueries({ queryKey: [key] });
        },
    });
};

const usePhatContractOhMyChess = ({networkUrl, accountAddress}: InputProps): ReturnProps => {
    const [ohMyChessClient, setOhMyChessClient] = useState<OhMyChessClient | null>(null);

    useEffect((): void => {
        const instantiateClient = async (): Promise<void> => {
            try {
                if (accountAddress) {
                    setOhMyChessClient(await instantiateOhMyChessClient({networkUrl: networkUrl, accountAddress}));
                }
            } catch (e) {
                console.error(e);
            }
        };
        instantiateClient();
    }, [networkUrl, accountAddress]);

    const useQuerySessions = useQuery({ queryKey: ['sessions-list', ohMyChessClient?.clientId], queryFn: ohMyChessClient?.fetchSessions, enabled: !!ohMyChessClient});
    const useCreateNewSessionMutation = useCreateNewSessionMutationFunc(ohMyChessClient ? ohMyChessClient.createNewSession : (_: CreateNewSessionArgs): Promise<string> => {throw Error('OhMyChessClient not instantiated yet');});
    const useJoinSessionMutation = useJoinSessionMutationFunc(ohMyChessClient ? ohMyChessClient.joinSession : (_: JoinSessionArgs): Promise<void> => {throw Error('OhMyChessClient not instantiated yet');});
    const useMakeChessMoveMutation = useMakeChessMoveMutationFunc(ohMyChessClient ? ohMyChessClient.makeChessMove : (_: MakeChessMoveArgs): Promise<void> => {throw Error('OhMyChessClient not instantiated yet');});
    const useQueryGameSessionFn = (sessionId: string | undefined) => useQuery({queryKey: ['game-session', ohMyChessClient?.clientId, sessionId], queryFn: () => ohMyChessClient?.getGameSession({sessionId: sessionId as string}), enabled: (!!sessionId && !!ohMyChessClient), refetchInterval: 4000});

    return { useQuerySessions, useCreateNewSessionMutation, useJoinSessionMutation, useMakeChessMoveMutation, useQueryGameSessionFn };
};

export default usePhatContractOhMyChess;