<script lang="ts">
	import { prettyAmount } from '../helpers';
	import TokenSelector from './tokenSelector.svelte';
	import { BigNumber as BN } from 'bignumber.js';
	import { tokensStore, type Token } from '../helpers/globalStore';

	// imported vars
	export let name: string;
	export let tokenAmount: BN | undefined;
	export let selectedToken: Token;
	export let allowSelectToken: boolean = true;
	// Allow only specific tokens to be selected
	export let allowedTokens: string[] | undefined = undefined;

	// local vars
	let showTokenDropdown = false;
	let tokenAmountPretty: string | undefined = undefined;
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
			tokenAmount = new BN(inputValue.replaceAll(',', ''));
			console.log('tokenAmount', tokenAmount.toString(), inputValue);
			tokenAmountPretty = prettyAmount(inputValue);
		}, 500);
	};

	tokensStore.subscribe((tokens) => {
		if (tokens.length > 0) {
			let allowTokens = tokens;
			if (allowedTokens)
				allowTokens = tokens.filter((token) => allowedTokens.includes(token.address));
			selectedToken = allowTokens[0];
			selectedToken.priceUSD = 1.1;
			filteredTokens = allowTokens;
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
			if (allowSelectToken) showTokenDropdown = !showTokenDropdown;
		}}
		class={`flex items-center gap-2 ${showTokenDropdown ? 'hidden' : ''}`}
	>
		{#if selectedToken?.icon !== ''}
			<img class="w-5" src={selectedToken?.icon} alt="selected token icon" />
		{/if}
		<p>{selectedToken?.symbol}</p>
	</button>
	<div class="flex flex-col">
		<input
			bind:value={tokenAmountPretty}
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
