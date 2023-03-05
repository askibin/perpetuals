import * as web3 from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { BigNumber as BN } from 'bignumber.js';
const getHeliusRPC = (cluster: web3.Cluster): string => {
	switch (cluster) {
		case 'mainnet-beta':
			return `https://rpc.helius.xyz?api-key=${process.env.HELIUS_API_KEY}`;
		case 'devnet':
			return `https://rpc-devnet.helius.xyz?api-key=${process.env.HELIUS_API_KEY}`;
		default:
			throw new Error('Invalid cluster');
	}
};

const getHeliusAPI = (path?: string): string => {
	return `https://api.helius.xyz/${path}?api-key=${process.env.HELIUS_API_KEY}`;
};

export const getConnection = (cluster: web3.Cluster = 'mainnet-beta') =>
	new web3.Connection(getHeliusRPC(cluster), 'confirmed');

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
	try {
		const ata = await spl.getAssociatedTokenAddress(
			new web3.PublicKey(tokenMintAddress),
			ownerAddress
		);
		const mint = await spl.getMint(connection, new web3.PublicKey(tokenMintAddress));
		const tokenAccount = await spl.getAccount(connection, ata);
		return new BN(tokenAccount.amount.toString()).shiftedBy(-mint.decimals);
	} catch (err) {
		return new BN(0);
	}
};

export type SolScanTokenMeta = {
	address: string;
	amount: number;
	priceUSD: number;
	symbol: string;
	name: string;
	icon: string;
	website: string;
	twitter: string;
	tag: string[];
	decimals: number;
	coingeckoId: string;
	holder: number;
};
export const getTokenMetaDataFromSolScan = async (
	tokenAddress: string
): Promise<SolScanTokenMeta> => {
	const baseApi = `https://public-api.solscan.io/`;
	const path = `token/meta/${tokenAddress}`;
	const uri = baseApi + path;
	const response = await fetch(uri, {
		headers: {
			'Content-Type': 'application/json'
		}
	});
	if (!response.ok) throw new Error(`status: ${response.status} ${response.statusText}`);
	const data: SolScanTokenMeta = await response.json();
	return {
		address: tokenAddress,
		amount: 0,
		priceUSD: 1,
		...data
	};
};

export type SolanaFMData = {
	address: string;
	amount: number;
	priceUSD: number;
	mint: string;
	tokenName: string;
	symbol: string;
	decimals: number;
	description: string;
	logo: string;
	tags: [];
	verified: string;
	network: string[];
	metadataToken: string;
};

export const getTokenMetaDataFromSolanaFM = async (
	tokenAddress: string
): Promise<SolScanTokenMeta> => {
	const baseApi = `https://api.solana.fm/v0/`;
	const path = `tokens/${tokenAddress}`;
	const uri = baseApi + path;
	const response = await fetch(uri, {
		headers: {
			'Content-Type': 'application/json'
		}
	});
	if (!response.ok) throw new Error(`status: ${response.status} ${response.statusText}`);
	const data: {
		status: string;
		message: string;
		result: {
			tokenHash: string;
			data: SolanaFMData;
		};
	} = await response.json();

	return {
		address: tokenAddress,
		amount: 0,
		priceUSD: 1,
		symbol: data.result.data.symbol,
		name: data.result.data.tokenName,
		icon: data.result.data.logo,
		website: data.result.data.logo,
		twitter: data.result.data.logo,
		tag: data.result.data.tags,
		coingeckoId: '',
		decimals: data.result.data.decimals,
		holder: 0
	};
};

export type HeliusGetWalletBalanceResponse = {
	nativeBalance: number;
	tokens: HeliusToken[];
};

export const amountWithDecimals = (amount: string): string => {
	const parts = amount.split('.');
	const numberPart = parts[0];
	const decimalPart = parts[1];
	return numberPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',') + (decimalPart ? '.' + decimalPart : '');
};

type HeliusToken = {
	amount: number;
	decimals: number;
	mint: string;
	tokenAccount: string;
};

/**
 * getWalletBalances gets the balances of all tokens for a specific wallet
 * @param wallet
 * @returns
 */
export const getWalletBalances = async (
	wallet: web3.PublicKey
): Promise<HeliusGetWalletBalanceResponse> => {
	const path = `v0/addresses/${wallet.toString()}/balances`;
	const endpoint = getHeliusAPI(path);
	const response = await fetch(endpoint);
	if (!response.ok) throw new Error(`status: ${response.status} ${response.statusText}`);
	const data: HeliusGetWalletBalanceResponse = await response.json();

	// filter out NFTS
	data.tokens = data.tokens.filter((token) => token.decimals !== 0);
	return data;
};

type CoinGeckoSimplePrice = Record<string, Record<string, number>>;

export const fetchCoingeckoPriceFromId = async (
	id: string,
	currency: string
): Promise<CoinGeckoSimplePrice> => {
	const cgUrl =
		'https://api.coingecko.com/api/v3/simple/price?ids=' + id + '&vs_currencies=' + currency;
	const response = await fetch(cgUrl);
	if (!response.ok) throw new Error('Failed to fetch simple price' + response.statusText);
	return await response.json();
};

export const prettyAmount = (amount: string): string => {
	if (amount) {
		const val = amount.replaceAll(',', '');
		return amountWithDecimals(val);
	}
	return '0.0';
};
