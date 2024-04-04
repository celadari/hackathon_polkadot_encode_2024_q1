import {ReactNode, useState} from "react";
import Modal from "@/components/Modal";
import {CreateNewSessionArgs} from "@/services/phat-contract";
import { Player } from "@/models/game-session";

interface Props {
    show: boolean;
    closeModal: () => void;
    handleCreateSession: (_: CreateNewSessionArgs) => void;
}

export const CreateNewSessionModal = ({
                    show,
                    closeModal,
                    handleCreateSession,
               }: Props): ReactNode => {
    const [player, setPlayer] = useState<Player | undefined>(undefined);
    const [addressInput, setAddressInput] = useState<string | undefined>(undefined);

    return (
        <Modal show={show} title={"New Session"} closeModal={closeModal}>
            <div>
                <select onChange={(e) => setPlayer(e.target.value as Player | undefined)}>
                    <option value={undefined}>Select Player</option>
                    <option value={Player.White}>{Player.White}</option>
                    <option value={Player.Black}>{Player.Black}</option>
                </select>
            </div>
            <div>
                <input
                    type="text"
                    placeholder="Second player address in hex (64 characters)"
                    value={addressInput}
                    onChange={(e) => setAddressInput(e.target.value)}
                />
            </div>
            <button onClick={() => handleCreateSession({player})}>Create and join New Session</button>
        </Modal>
    );
};