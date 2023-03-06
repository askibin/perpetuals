<script lang="ts">
	import { prettyAmount } from '../helpers';
	import TokenSelector from './tokenSelector.svelte';
	import { BigNumber as BN } from 'bignumber.js';
	import { tokensStore, type Token } from '../helpers/globalStore';

	// imported vars
	export let name: string;
	export let tokenAmount: string | undefined;
	export let leverage: number;
	export let selectedToken: Token;

	// local vars
	let showTokenDropdown = false;
	let tokenSearchTerm = '';
	let filteredTokens = [];

	let textTimeout: any;
	let tokenAmountUSD: string | undefined = undefined;

	const handleAmountChange = (am: Event) => {
		clearTimeout(textTimeout);
		textTimeout = setTimeout(() => {
			const inputValue = (am.target as HTMLInputElement).value;
			if (!inputValue) return;
			if (inputValue === '0') {
				tokenAmount = undefined;
				return;
			}
			tokenAmount = prettyAmount(inputValue);
		}, 500);
	};

	//$: tokenAmount = prettyAmount(tokenAmount);

	tokensStore.subscribe((tokens) => {
		if (tokens.length > 0) {
			selectedToken = tokens[0];
			selectedToken.priceUSD = 1.1;
			filteredTokens = tokens;
		}
	});

	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'Escape') {
			showTokenDropdown = false;
		}
		if (event.key === 'Enter') {
			showTokenDropdown = false;
		}
	};
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="container relative bg-slate-800 py-1 px-5 flex flex-row justify-between rounded-md ">
	<div class={`flex flex-col justify-center ${showTokenDropdown ? '' : 'hidden'}`}>
		<TokenSelector {tokenSearchTerm} {filteredTokens} bind:selectedToken bind:showTokenDropdown />
	</div>
	<button
		on:click={() => {
			showTokenDropdown = !showTokenDropdown;
		}}
		class={`flex items-center gap-2 ${showTokenDropdown ? 'hidden' : ''}`}
	>
		<img class="w-5" src={selectedToken?.icon} alt="selected token icon" />
		<p>{selectedToken?.symbol}</p>
	</button>
	<div class="flex flex-col">
		<input
			bind:value={tokenAmount}
			on:input={handleAmountChange}
			placeholder="0.0"
			name={`token-${name}`}
			type="text"
			class="text-base outline-none text-right bg-transparent placeholder-shown:border-gray-500"
		/>
		{#if tokenAmountUSD}
			<p class="text-sm text-slate-600 text-right">{`$ ${tokenAmountUSD}`}</p>
		{/if}
	</div>
</div>
