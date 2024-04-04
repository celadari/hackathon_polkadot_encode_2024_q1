import {ReactNode, useState} from "react";
import Modal from "@/components/Modal";
import { JoinSessionArgs } from "@/services/phat-contract";

interface Props {
    show: boolean;
    closeModal: () => void;
    handleJoinSession: (_: JoinSessionArgs) => void;
}

export const JoinSessionModal = ({
                                      show,
                                      closeModal,
                                      handleJoinSession,
                                      }: Props): ReactNode => {
    const [sessionUUID, setSessionUUID] = useState<string>('');

    return (
        <Modal show={show} title={"Join Session"} closeModal={closeModal}>
            <div>
                <input
                    type="text"
                    placeholder="Session ID"
                    value={sessionUUID}
                    onChange={(e) => setSessionUUID(e.target.value)}
                />
            </div>
            <button onClick={(): void => {
                handleJoinSession({sessionId: sessionUUID});
                setSessionUUID('');
            }}>Join Session</button>
        </Modal>
    );
};