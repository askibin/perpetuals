<script lang="ts">
	import { Slider } from 'fluent-svelte';
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';
	import { onMount } from 'svelte';
	import { getTokenBalance } from '../../helpers';
	import { token } from '@project-serum/anchor/dist/cjs/utils';
	import { BigNumber as BN } from 'bignumber.js';
	import Assistance from '../../components/assistance.svelte';
	import { seed } from '@project-serum/anchor/dist/cjs/idl';

	const amountWithDecimals = (amount: string): string => {
		const parts = amount.split('.');
		const numberPart = parts[0];
		const decimalPart = parts[1];
		return (
			numberPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',') + (decimalPart ? '.' + decimalPart : '')
		);
	};

	// Input
	let displayAmount: string = '0';
	let textTimeout: any;
	const handleAmountChange = (am: KeyboardEvent) => {
		clearTimeout(textTimeout);
		textTimeout = setTimeout(() => {
			const inputValue = (am.target as HTMLInputElement).value;
			if (inputValue) {
				const val = inputValue.replaceAll(',', '');
				displayAmount = amountWithDecimals(val);
				defaultTokenPrice = new BN(val).times(new BN(defaultToken.priceUSD));
			}
		}, 500);
	};

	type Token = {
		address: string;
		chainId: number;
		decimals: number;
		logoURI: string;
		name: string;
		symbol: string;
		priceUSD?: number;
	};

	let leverage = 15;
	let tokens: Token[] = [];
	let filteredTokens: Token[] = [];
	let defaultToken: Token | undefined;
	let defaultTokenBalance = BN(0);

	$: {
		// FIXME: Display loader instead
		defaultTokenBalance = new BN(0);
		getTokenBalance(defaultToken?.address ?? '', $walletStore?.publicKey).then((res) => {
			defaultTokenBalance = res;
		});
	}

	let defaultTokenPrice = new BN(0);
	const tokenMock = ['Saber2gLauYim4Mvftnrasomsv6NvAuncvMEZwcLpD1'];
	onMount(async () => {
		const allTokens = await fetch('https://cache.jup.ag/all-tokens');
		const json: Token[] = await allTokens.json();
		tokens = json.filter((token) => tokenMock.includes(token.address));
		tokens = json;
		defaultToken = tokens[0];
		defaultToken.priceUSD = 1.1;
	});

	let searchRef: HTMLInputElement = null;
	let showTokenDropdown = false;
	let leveragePosition = '0';
	let leveragePositionUSD = '0';
	$: {
		const amount = displayAmount.replaceAll(',', '');
		leveragePosition = new BN(leverage).times(new BN(amount)).toString();
		leveragePositionUSD = amountWithDecimals(
			new BN(leveragePosition).times(defaultToken?.priceUSD ?? '0').toString()
		);
		leveragePosition = amountWithDecimals(leveragePosition);
	}

	// Token search
	let tokenSearchTerm = '';
	$: {
		if (tokenSearchTerm) {
			filteredTokens = tokens.filter((token) =>
				token.symbol.toLowerCase().includes(tokenSearchTerm.toLowerCase())
			);
		} else {
			filteredTokens = tokens;
		}
	}

	$: {
		if (showTokenDropdown && searchRef) {
			setTimeout(() => {
				searchRef.focus();
			}, 200);
		}
	}

	let selectedTokenIndex: number = -1;
	let selectedTokenId: string = '';
	let previousMod = 0;
	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			showTokenDropdown = false;
		}
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			const items = filteredTokens.map((token) => document.getElementById(token.symbol));

			if (selectedTokenIndex === -1) {
				selectedTokenIndex = 0;
			} else {
				if (event.key === 'ArrowDown') {
					selectedTokenIndex += 1;
				} else {
					selectedTokenIndex -= 1;
				}
			}
			selectedTokenId = filteredTokens[selectedTokenIndex].symbol;
		}

		if (event.key === 'Enter' && selectedTokenIndex !== -1) {
			defaultToken = filteredTokens[selectedTokenIndex];
			showTokenDropdown = false;
			selectedTokenIndex = -1;
		}

		const searchTokenList = document.getElementById('searchTokenList');
		const hoverToken = document.getElementById(selectedTokenId);
		console.log(
			'searchTokenList.clientHeight',
			searchTokenList.clientHeight,
			'searchTokenList.scrollTop',
			searchTokenList.scrollTop,
			'offsetTop',
			hoverToken.offsetTop
		);

		const currentMod = hoverToken.offsetTop % searchTokenList.clientHeight;

		console.log('previousMod', previousMod, 'currentMod', currentMod);
		if (previousMod > currentMod) {
			searchTokenList.scrollTo(0, hoverToken.offsetTop);
		}
		previousMod = hoverToken.offsetTop % searchTokenList.clientHeight;
	};
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="container flex flex-col gap-5">
	<div class="container max-w-lg">
		<div class="container flex flex-col j">
			<div class="container flex flex-row justify-between ">
				<p class="text-base">You pay</p>
				{#if $walletStore.connected}
					<div class="flex flex-row items-center gap-1">
						<p class="text-base">{defaultTokenBalance}</p>
						<p class="text-sm">{`${defaultToken?.symbol} balance`}</p>
					</div>
				{/if}
			</div>
			<div
				class="container relative bg-slate-800 py-1 px-5 flex flex-row justify-between rounded-md  "
			>
				<div class={`flex flex-col justify-center ${showTokenDropdown ? '' : 'hidden'}`}>
					<input
						id="searchToken"
						bind:this={searchRef}
						bind:value={tokenSearchTerm}
						type="text"
						placeholder="Search"
						class="z-1000 w-20 text-slate-200 outline-none text-left bg-transparent placeholder-shown:border-gray-500"
					/>
					<ul
						id="searchTokenList"
						class="flex gap-2  left-0 right-0 flex-col  z-10 absolute max-h-60 w-120 overflow-scroll top-12 bg-slate-800 rounded-md"
					>
						{#each filteredTokens as token, index}
							<li
								tabindex={index}
								id={`${token.symbol}`}
								class={`flex flex-row gap-5 h-10 justify-between hover:bg-sky-700 focus:bg-sky-700 cursor-pointer ${
									index === selectedTokenIndex ? 'bg-sky-700' : ''
								}`}
							>
								<div class="flex flex-row ml-5 gap-5 items-center">
									<img class="w-8" src={token.logoURI} alt="token logo" />
									<button
										on:click={() => {
											defaultToken = token;
											showTokenDropdown = false;
											defaultToken.priceUSD = 1.1;
										}}
										class="flex flex-col "
									>
										<div>{token.symbol}</div>
										<div class="text-xs text-slate-600">{token.name}</div>
									</button>
								</div>
							</li>
						{/each}
					</ul>
				</div>
				<button
					on:click={() => {
						showTokenDropdown = !showTokenDropdown;
					}}
					class={`flex items-center gap-2 ${showTokenDropdown ? 'hidden' : ''}`}
				>
					<img class="w-5" src={defaultToken?.logoURI} />
					<p>{defaultToken?.symbol}</p>
				</button>
				<div class="flex flex-col">
					<input
						bind:value={displayAmount}
						on:keypress={handleAmountChange}
						placeholder="0.0"
						name="amount"
						type="text"
						class="text-base outline-none text-right bg-transparent placeholder-shown:border-gray-500"
					/>
					<p class="text-sm text-slate-600 text-right">{`$ ${defaultTokenPrice}`}</p>
				</div>
			</div>
		</div>
	</div>

	<div class="container max-w-lg">
		<div class="container flex flex-col j">
			<div class="container flex flex-row justify-between ">
				<p class="text-base">You short position</p>
			</div>
			<div class="container bg-slate-800 py-1 px-2 flex flex-row justify-between rounded-md  ">
				<div class="flex items-center">
					<button class="flex items-center gap-2">
						<img class="w-5" src={defaultToken?.logoURI} />
						<p>{defaultToken?.symbol}</p>
					</button>
				</div>
				<div class="flex flex-col">
					<p
						class="text-base outline-none text-right bg-transparent placeholder-shown:border-gray-500"
					>
						{leveragePosition.toString() ?? 0}
					</p>
					<p class="text-sm">{`$ ${leveragePositionUSD}`}</p>
				</div>
			</div>
		</div>
	</div>

	<div class="container max-w-lg z-1">
		<div class=" z-1 container flex flex-row justify-between items-center gap-2 ">
			<Slider
				class="z-1"
				bind:value={leverage}
				tooltip={false}
				step={5}
				max={100}
				ticks={[5, 10, 15, 20, 100]}
				suffix="%"
			/>

			<div class="relative ">
				<input
					bind:value={leverage}
					name="amount"
					type="text"
					class="px-3 text-base rounded font-pixel text-lg outline-none w-14 text-left bg-slate-800  placeholder-shown:border-gray-500"
				/>
				<div class="input-x" />
			</div>
		</div>
	</div>

	<div class="container max-w-lg">
		{#if $walletStore.connected}
			<button class="container bg-fuchsia-500 rounded-md">Place Order</button>
		{:else}
			<div class="flex  justify-center">
				<WalletMultiButton>Connect to place order</WalletMultiButton>
			</div>
		{/if}
	</div>
</div>

<style style="sass">
	:global(.slider-thumb) {
		background-color: white !important;
		z-index: 1 !important;
	}
	:global(.slider-rail) {
		background-color: black !important;
		z-index: 0 !important;
	}
	:global(.slider-track) {
		background-color: #1d4ed8 !important;
		z-index: 0 !important;
	}

	:global(.slider-tick-bar) {
		background-color: blue !important;
	}

	.input-x::after {
		content: 'x';
		position: absolute;
		right: 2px;
		top: 0;
		color: #94a3b8;
	}
</style>
