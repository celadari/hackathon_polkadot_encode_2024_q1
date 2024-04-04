import React from 'react';

interface InputProps {
    sessionUuids: string[] | undefined;
    onNewSessionClick: () => void;
    onJoinSessionClick: () => void;
    handleSelectedSessionUuid: (_: string) => void;
    selectedSessionUuid: string | undefined;
    disableButtons: boolean;
}

const VerticalMenu = ({ disableButtons, sessionUuids, onNewSessionClick, onJoinSessionClick, handleSelectedSessionUuid, selectedSessionUuid }: InputProps): React.ReactNode => {

    return (
        <nav className="h-screen fixed left-0 top-0 w-64 bg-gray-800 text-white flex flex-col">
            <div className="px-4 py-2">
                <button disabled={disableButtons} onClick={onNewSessionClick} className="text-sm w-full mb-4 p-2 bg-blue-500 hover:bg-blue-700 rounded transition duration-300 ease-in-out disabled:bg-blue-200">New Session</button>
                <button disabled={disableButtons} onClick={onJoinSessionClick} className="text-sm w-full p-2 bg-green-500 hover:bg-green-700 rounded transition duration-300 ease-in-out disabled:bg-green-200">Join Session</button>
            </div>
            <ul className="flex-grow overflow-auto">
                {sessionUuids && sessionUuids?.length > 0 ? (
                    sessionUuids.filter(uuid => !!uuid).map((uuid, index) => (
                        <li key={index} className={`truncate px-4 py-2 ${selectedSessionUuid !== uuid ? 'cursor-pointer hover:bg-gray-600' : ''} transition duration-300 ease-in-out ${selectedSessionUuid === uuid ? 'bg-gray-400' : ''}`}
                            onClick={() => handleSelectedSessionUuid(uuid)}>
                                {uuid}
                        </li>
                ))) : (
                    <li className="px-4 py-2">No Sessions</li>
                )}
            </ul>
        </nav>
    );
};

export default VerticalMenu;