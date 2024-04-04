import * as fs from 'node:fs';
import { getClient, PinkCodePromise, KeyringPairProvider, OnChainRegistry } from '@phala/sdk';
import { Struct, u128 } from "@polkadot/types";
import path from "path";
import arg from "arg";
import BN from "bn.js";


interface CliArgs {
    contractName: string;
    contractFilePath: string;
    clusterId: string;
    adminProfile: string;
    mongoAtlasUrl: string;
    mongoAtlasApiKey: string;
    endpoint: string;
}

interface FrameSystemAccountInfo extends Struct {
    data: {
        free: u128;
    }
}

function parseArguments(rawArgs: string[]): CliArgs {
    const args = arg(
        {
            // Types
            '--contract_file_path': String,
            '--contract_name': String,
            '--cluster_id': String,
            '--admin_profile': String,
            '--mongo_atlas_url': String,
            '--mongo_atlas_api_key': String,
            '--endpoint': String,
            // Aliases
            '-n': '--contract_name',
            '-p': '--contract_file_path',
            '-c': '--cluster_id',
            '-a': '--admin_profile',
            '-u': '--mongo_atlas_url',
            '-k': '--mongo_atlas_api_key',
            '-e': '--endpoint',
        },
        {
            argv: rawArgs.slice(2),
        }
    );

    return {
        contractName: args['--contract_name'] || 'oh_my_chess',
        contractFilePath: args['--contract_file_path'] || path.join(__dirname, '..', 'contracts', 'oh_my_chess', 'target', 'ink', 'oh_my_chess.contract'),
        clusterId: args['--cluster_id'] || '0x0000000000000000000000000000000000000000000000000000000000000001',
        adminProfile: args['--admin_profile'] || 'Alice',
        mongoAtlasUrl: args['--mongo_atlas_url'],
        mongoAtlasApiKey: args['--mongo_atlas_api_key'],
        endpoint: args['--endpoint'] || 'ws://localhost:9944/ws',
    };
}

function formatPha(pha: u128 | BN): number {
    if ('toBigInt' in pha) {
        return Number(pha.toBigInt() / BigInt(1e10)) / 100;
    }
    return Number(BigInt(pha.toString()) / BigInt(1e10)) / 100;
}

let client: OnChainRegistry;

async function main(argv: string[]): Promise<void> {
    const args = parseArguments(argv);

    client = await getClient({ transport: args.endpoint });
    const provider = await KeyringPairProvider.create(client.api, client.alice);
    const accountInfo = await client.api.query.system.account<FrameSystemAccountInfo>(provider.address);

    const free = formatPha(accountInfo.data.free);
    if (free < 20) {
        console.log('Not enough balance. Please transfer some tokens not less then 20 PHA to', provider.address)
        return process.exit(1)
    }
    console.log(`Account ${provider.address} has ${free} PHA.`)

    console.log('Cluster ID:', client.clusterId);
    console.log('Pruntime Endpoint URL:', client.pruntimeURL);

    const balance = await client.getClusterBalance(provider.address);
    const total = formatPha(balance.total);
    const freeInCluster = formatPha(balance.free);
    console.log('Cluster Balance:', total, freeInCluster);

    if (freeInCluster < 500) {
        console.log('Transfer to cluster...')
        try {
            await provider.send(client.transferToCluster(provider.address, 1e12 * 500));
        } catch (err) {
            console.error(`Transfer to cluster failed: ${err}`)
            return process.exit(1)
        }
    }

    const source = fs.readFileSync(args.contractFilePath, 'utf-8');
    const pinkCodePromise = new PinkCodePromise(client, source);

    // if needed to check if exist at some point, just use 'await pinkCodePromise.hasExists();'
    const submittableResult = await pinkCodePromise.send({ provider });
    await submittableResult.waitFinalized();

    const blueprint = pinkCodePromise.getBlueprint();

    const result = await blueprint.send.new({ provider }, args.mongoAtlasUrl, args.mongoAtlasApiKey);
    await result.waitFinalized();
    const contract = result.contract;
    console.log(contract);
    console.log(`contractKey: ${contract.contractKey}`);
    console.log(`contractId: ${result.contractId}`);
}

main(process.argv).then().catch(err => {
    console.error("Failed to run the script:", err);
}).finally(async () => {
    await client.api.disconnect();
});



