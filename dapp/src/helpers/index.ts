import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { BigNumber as BN } from 'bignumber.js';
const getHeliusRPC = (cluster: web3.Cluster): string => {
	switch (cluster) {
		case 'mainnet-beta':
			return `https://rpc.helius.xyz/?api-key=${process.env.HELIUS_API_KEY}`;
		case 'devnet':
			return `https://rpc-devnet.helius.xyz/?api-key=${process.env.HELIUS_API_KEY}`;
		default:
			throw new Error('Invalid cluster');
	}
};

/**
 * getTokenBalance gets the balance of a specific token for a specific owner
 * @param tokenMintAddress
 * @param ownerAddress
 * @returns
 */
export const getTokenBalance = async (
	tokenMintAddress: string,
	ownerAddress?: web3.PublicKey
): Promise<BN> => {
	if (!ownerAddress) return new BN(0);
	const connection = new web3.Connection(getHeliusRPC('mainnet-beta'), 'confirmed');
	console.log('ownerAddress', ownerAddress.toString(), 'tokenMintAddress', tokenMintAddress);
	try {
		const ata = await spl.getAssociatedTokenAddress(
			new web3.PublicKey(tokenMintAddress),
			ownerAddress
		);
		const mint = await spl.getMint(connection, new web3.PublicKey(tokenMintAddress));
		const tokenAccount = await spl.getAccount(connection, ata);
		console.log('tokenAccount', tokenAccount);
		console.log('mint.decimals', mint.decimals);
		return new BN(tokenAccount.amount.toString()).shiftedBy(-mint.decimals);
	} catch (err) {
		return new BN(0);
	}
};
