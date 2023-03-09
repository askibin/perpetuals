<script lang="ts">
	import { tokensStore, type Token, type Tokens } from '../helpers/globalStore';

	let searchRef: HTMLInputElement = null;
	export let tokenSearchTerm: string = '';
	export let filteredTokens: Tokens = [];
	export let selectedToken: Token | undefined;
	export let showTokenDropdown: boolean;

	let selectedTokenIndex: number = 0;
	let selectedTokenId: string = '';
	let previousMod = 0;

	const handleKeydown = (event: KeyboardEvent) => {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			if (selectedTokenIndex === -1) {
				selectedTokenIndex = 0;
			} else {
				if (event.key === 'ArrowDown') {
					selectedTokenIndex += 1;
				} else {
					selectedTokenIndex -= 1;
				}
				selectedTokenIndex = selectedTokenIndex % filteredTokens.length;
			}
			selectedTokenId = filteredTokens[selectedTokenIndex].symbol;

			const searchTokenList = document.getElementById('searchTokenList');
			const hoverToken = document.getElementById(selectedTokenId);

			const currentMod = hoverToken.offsetTop % searchTokenList.clientHeight;

			if (previousMod > currentMod) {
				searchTokenList.scrollTo(0, hoverToken.offsetTop);
			}
			previousMod = hoverToken.offsetTop % searchTokenList.clientHeight;
		}

		if (event.key === 'Enter' && selectedTokenIndex !== -1) {
			selectedToken = filteredTokens[selectedTokenIndex];
			selectedTokenIndex = -1;
		}
	};

	$: {
		if (tokenSearchTerm) {
			filteredTokens = $tokensStore.filter((token) =>
				token.symbol.toLowerCase().includes(tokenSearchTerm.toLowerCase())
			);
		} else {
			filteredTokens = $tokensStore;
		}
	}

	$: {
		if (showTokenDropdown && searchRef) {
			setTimeout(() => {
				searchRef.focus();
			}, 200);
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

<div>
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
			<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
			<li
				tabindex={index}
				id={`${token.symbol}`}
				class={`flex flex-row gap-5 h-10 justify-between hover:bg-sky-700 focus:bg-sky-700 cursor-pointer ${
					index === selectedTokenIndex ? 'bg-sky-700' : ''
				}`}
			>
				<div class="flex flex-row ml-5 gap-5 items-center">
					<img class="w-8" src={token.icon} alt="token logo" />
					<button
						on:click={() => {
							selectedToken = token;
							showTokenDropdown = false;
							selectedToken.priceUSD = 1.1;
						}}
						class="flex flex-col "
					>
						<div>{token.symbol}</div>
						<div class="text-xs text-slate-600">{token.name}</div>
					</button>
				</div>
				{#if token.amount}
					<div class="flex flex-col mr-5">
						<div class="text-xs text-slate-100">{token.amount}</div>
						<div class="text-xs text-slate-100">{`$ ${token.amount}`}</div>
					</div>
				{/if}
			</li>
		{/each}
	</ul>
</div>
