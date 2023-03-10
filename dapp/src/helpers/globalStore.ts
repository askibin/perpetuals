import { writable } from 'svelte/store';
import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
import { toast } from '@zerodevx/svelte-toast';
import { getWalletBalances, type HeliusGetWalletBalanceResponse, type SolScanTokenMeta } from '.';
import BigNumber from 'bignumber.js';

import Moralis from 'moralis';
import type { SelectorItem } from './types';

const errorToastOptions = {
	classes: ['bg-red-900', 'font-pixel', 'drop-shadow', 'shadow-red-600'],
	theme: {
		'--toastBarBackground': '#7f1d1d'
	}
};

enum ToastId {
	LoadWalletBalance = 1,
	LoadPrices = 2
}

export type Token = SolScanTokenMeta;
export type Tokens = SolScanTokenMeta[];

export type TokenMap = Record<string, Token>;

export const tokensStore = writable<Tokens>([]);

export interface Pool extends SelectorItem {
	id: string;
	tvl: string;
	apr: string;
}
export const defaultPool = writable<Pool>(null);

export const hydrateTokensStore = (
	tokens: Tokens,
	walletBalances: HeliusGetWalletBalanceResponse
): Tokens => {
	return tokens
		.map((token) => {
			const walletToken = walletBalances.tokens.find((t) => t.mint === token.address);
			if (walletToken) {
				token.amount = new BigNumber(walletToken.amount).shiftedBy(-token.decimals).toNumber();
			}
			return token;
		})
		.sort((a, b) => {
			if (b.amount && a.amount) return b.amount - a.amount;
			if (b.amount) return 1;
			if (a.amount) return -1;
		});
};

// Load wallet balances
walletStore.subscribe(async (wallet) => {
	if (wallet.publicKey) {
		try {
			const walletBalances = await getWalletBalances(wallet.publicKey);
			tokensStore.update((tokens) => hydrateTokensStore(tokens, walletBalances));
		} catch (err) {
			toast.pop(1);
			toast.push('Failed to load wallet balances. Please try again later. ' + err.message, {
				id: ToastId.LoadWalletBalance,
				...errorToastOptions
			});
		}
	}
});

const getTokenUsdPrice = async (tokenAddress: string): Promise<number> => {
	const response = await Moralis.SolApi.token.getTokenPrice({
		address: tokenAddress,
		network: 'mainnet'
	});
	return response.toJSON().usdPrice;
};

tokensStore.subscribe(async (tokens) => {
	return await Promise.all(
		tokens.map(async (token) => {
			//if (!token.extensions.coingeckoId) return token;
			// const price = await getTokenUsdPrice(token.address);
			// const priceUsd = price[token.extensions.coingeckoId].usd;
			// token.priceUSD = priceUsd;
			// token.amountUSD = new BigNumber(token.amount)
			// 	.div(priceUsd)
			// 	.shiftedBy(-token.decimals)
			// 	.toNumber();
			// return token;
		})
	);
});
