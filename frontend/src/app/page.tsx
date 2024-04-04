'use client';
import OhMyChessBoard from "@/components/OhMyChessBoard";
import React, {useCallback, useState} from "react";
import VerticalMenuView from "@/components/VerticalMenuView";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {InjectedAccount} from "@phala/sdk";
import HorizontalMenu from "@/components/HorizontalMenu";
import {ReactQueryDevtools} from "@tanstack/react-query-devtools";

const queryClient = new QueryClient();

export default function Home(): React.JSX.Element {
    const [selectedSessionUuid, setSelectedSessionUuid] = useState<string | undefined>(undefined);
    const [accountAddress, setAccountAddress] = useState<InjectedAccount | undefined>(undefined);
    const [networkUrl, setNetworkUrl] = useState<string>(process.env.NEXT_PUBLIC_RPC_LOCALNET_URL as string);

    const handleSelectNetworkUrlAndAccount = useCallback((_networkUrl: string, __accountSelected: InjectedAccount) => {
        setNetworkUrl(_networkUrl);
        setAccountAddress(__accountSelected);
        setSelectedSessionUuid(undefined);
    }, []);

    const handleSelectedSessionUuid = useCallback((sessionId: string): void => {
        setSelectedSessionUuid(sessionId);
    }, []);

    return (
        <>
          <QueryClientProvider client={queryClient}>
              <ReactQueryDevtools></ReactQueryDevtools>

              <div>
                  <HorizontalMenu handleSelectNetworkUrlAndAccount={handleSelectNetworkUrlAndAccount} />
                  <VerticalMenuView selectedSessionUuid={selectedSessionUuid} handleSelectedSessionUuid={handleSelectedSessionUuid} accountAddress={accountAddress} networkUrl={networkUrl}/>
                  <main className="flex min-h-screen flex-col items-center justify-between p-24">
                      <div>
                          <OhMyChessBoard accountAddress={accountAddress} sessionId={selectedSessionUuid} networkUrl={networkUrl} />
                      </div>
                  </main>

              </div>
          </QueryClientProvider>
        </>
    );
}
