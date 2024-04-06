'use client';
import VerticalMenu from "@/components/VerticalMenu";
import React, {ReactNode, useCallback, useState} from "react";
import ServerError from "@/components/ServerError";
import Loader from "@/components/Loader";
import usePhatContractOhMyChess from "@/hooks/usePhatContractOhMyChess";
import {CreateNewSessionArgs, JoinSessionArgs} from "@/services/phat-contract";
import {CreateNewSessionModal} from "@/components/CreateNewSessionModal";
import {JoinSessionModal} from "@/components/JoinSessionModal";
import {InjectedAccount} from "@phala/sdk";

interface Props {
    handleSelectedSessionUuid: (_: string) => void;
    networkUrl: string;
    accountAddress?: InjectedAccount;
    selectedSessionUuid: string | undefined;
}



const VerticalMenuView = ({networkUrl, accountAddress, handleSelectedSessionUuid, selectedSessionUuid}: Props): ReactNode => {
    const { useQuerySessions: {isLoading, isError, data, refetch}, useCreateNewSessionMutation, useJoinSessionMutation, useQueryGameSessionFn } = usePhatContractOhMyChess({networkUrl, accountAddress});
    const [isNewSessionModalOpen, setNewSessionModalOpen] = useState(false);
    const [isJoinSessionModalOpen, setJoinSessionModalOpen] = useState(false);

    const openNewSessionModal = useCallback((): void => {
        if (!isNewSessionModalOpen) {
            setNewSessionModalOpen(true);
        }
    }, [isNewSessionModalOpen]);

    const openJoinSessionModal = useCallback((): void => {
        if (!isJoinSessionModalOpen) {
            setJoinSessionModalOpen(true);
        }
    }, [isJoinSessionModalOpen]);

    const handleCreateSession = useCallback((createNewSessionArgs: CreateNewSessionArgs): void => {
        useCreateNewSessionMutation.mutate(createNewSessionArgs, {
            onSuccess: async(newCreateSessionId: string): Promise<void> => {
                await refetch();
                handleSelectedSessionUuid(newCreateSessionId);
                setNewSessionModalOpen(false);
            },
            onError: (error: Error): void => console.error(error)
        });
    }, [useCreateNewSessionMutation, handleSelectedSessionUuid, refetch]);

    const handleJoinSession = useCallback((joinSessionArgs: JoinSessionArgs): void => {
        useJoinSessionMutation.mutate(joinSessionArgs, {
            onSuccess: async(): Promise<void> => {
                await refetch();
                handleSelectedSessionUuid(joinSessionArgs.sessionId);
                setJoinSessionModalOpen(false);
            },
            onError: (err: Error): void => {
                console.error(err);
            },
        })
    }, [useJoinSessionMutation, handleSelectedSessionUuid, refetch]);

    if (isLoading) {
        return (
            <div className="flex justify-center mt-8">
                <Loader />
            </div>
        );
    }

    if (isError) {
        return <ServerError />;
    }

    return (
        <>
            <VerticalMenu disableButtons={!accountAddress} selectedSessionUuid={selectedSessionUuid} handleSelectedSessionUuid={handleSelectedSessionUuid} sessionUuids={data} onNewSessionClick={openNewSessionModal} onJoinSessionClick={openJoinSessionModal}/>
            <CreateNewSessionModal handleCreateSession={handleCreateSession} show={isNewSessionModalOpen} closeModal={() => setNewSessionModalOpen(false)}/>
            <JoinSessionModal handleJoinSession={handleJoinSession} show={isJoinSessionModalOpen} closeModal={() => setJoinSessionModalOpen(false)} />
        </>
    );
};

export default VerticalMenuView;