'use client'
import React, { useState, useEffect } from 'react';
import {InjectedAccount} from "@phala/sdk";
import { fetchInjectedAccounts } from "@/services/phat-contract";
import Select, { type StylesConfig} from 'react-select'

interface Props {
    handleSelectNetworkUrlAndAccount: (_: string, __: InjectedAccount) => void;
}

const selectStyles: StylesConfig<{ value: InjectedAccount | null; label?: InjectedAccount["address"]; }, false> = {
    option: (styles, {isFocused, isSelected}) => ({
        ...styles,
        color: !isSelected && isFocused ? "black" : "black"
    })
}

const HorizontalMenu = ({ handleSelectNetworkUrlAndAccount }: Props) => {
    const [accounts, setAccounts] = useState<InjectedAccount[]>([]);
    const [networkUrl, setNetworkUrl] = useState<string>(process.env.NEXT_PUBLIC_RPC_LOCALNET_URL as string);
    const [accountSelected, setAccountSelected] = useState<InjectedAccount | null>(null);
    const [isMounted, setIsMounted] = useState(false);
    const [ disableChangeSelectionButton, setDisableChangeSelectionButton ] = useState<boolean>(false);

    useEffect(() => setIsMounted(true), []);

    useEffect((): void => {
        fetchInjectedAccounts({networkUrl}).then(fetchedAccounts => {
            setAccounts(fetchedAccounts);
        });
    }, [networkUrl]);

    return (
        <nav className="flex items-center justify-end gap-4 bg-gray-800 text-white p-4">
            {/* Network selection dropdown */}
            <div className="flex items-left">
                <label htmlFor="network-select" className="hidden">Select Network</label>
                <select
                    id="network-select"
                    value={networkUrl}
                    onChange={(e): void => {
                        if (e.target?.value) {
                            if (e.target.value !== networkUrl) setDisableChangeSelectionButton(false);
                            setNetworkUrl(e.target.value);
                        }
                    }}
                    className="bg-gray-700 border border-gray-600 rounded px-3 py-2"
                >
                    <option key={"mainnet"} value={process.env.NEXT_PUBLIC_RPC_MAINNET_URL as string}>
                        Mainnet
                    </option>
                    <option key={"poc6-testnet"} value={process.env.NEXT_PUBLIC_RPC_TESTNET_URL as string}>
                        POC6 Testnet
                    </option>
                    <option key={"local-net"} value={process.env.NEXT_PUBLIC_RPC_LOCALNET_URL as string}>
                        Local Net
                    </option>
                </select>
            </div>

            {/* Account selection dropdown */}
            <div className="flex items-center min-w-[500px]">
                <label htmlFor="account-select" className="hidden">Select Account</label>
                {isMounted ? (
                    <Select
                        id="account-select"
                        value={{value: accountSelected, label: accountSelected?.address}}
                        onChange={e => {
                            if (e?.value) {
                                if (e.value !== accountSelected) setDisableChangeSelectionButton(false);
                                setAccountSelected(e.value);
                            }
                        }}
                        className="bg-gray-700 border border-gray-600 rounded px-3 py-2 w-full"
                        options={accounts.map(account => ({value: account, label: account.address}))}
                        styles={selectStyles}
                    />
                ): null}
            </div>
            <div>
                <button disabled={!accountSelected || disableChangeSelectionButton}
                    onClick={(): void => {
                        handleSelectNetworkUrlAndAccount(networkUrl, accountSelected as InjectedAccount);
                        setDisableChangeSelectionButton(true);
                    }}
                    className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded disabled:bg-blue-300"
                >
                    Change Selection (Network and/or account)
                </button>
            </div>
        </nav>
    );
};

export default HorizontalMenu;