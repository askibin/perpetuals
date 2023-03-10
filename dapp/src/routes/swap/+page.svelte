<script lang="ts">
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';
	import autoAnimate from '@formkit/auto-animate';

	import { BigNumber, type BigNumber as BN } from 'bignumber.js';
	import TokenInput from '../../components/tokenInput.svelte';
	import type { Token } from '../../helpers/globalStore';
	import { page } from '$app/stores';
	import { prettyAmount } from '../../helpers';
	import { actions } from '../../types';

	// Input
	let leverage = 15;

	// tokens
	let selectedBaseToken: Token | undefined;
	let selectedLeverageToken: Token | undefined;

	let borrowFeePerHr = 0.0001;
	let availableLiquidityUSD = '200000';

	// Amounts
	let baseTokenAmount: BN | undefined;
	let leveragedTokenAmount: BN | undefined;

	let showQuote = false;

	function calculateLeveragedTokenAmount(prettyAmount: BN | undefined, leverage: number) {
		if (!prettyAmount) return undefined;

		const amountUSD = prettyAmount.times(selectedBaseToken?.priceUSD ?? '0');
		const leverageBN = new BigNumber(leverage);
		const leverageUSD = amountUSD.times(leverageBN);
		const leverageAmount = leverageUSD.dividedBy(selectedLeverageToken?.priceUSD ?? '0');
		return leverageAmount;
	}
	// Update leverage token amount on baseTokenAmount change
	$: leveragedTokenAmount = calculateLeveragedTokenAmount(baseTokenAmount, leverage);

	$: showQuote = baseTokenAmount && baseTokenAmount.gt(0);
</script>

<div class="container flex flex-col gap-5">
	<div
		class={`container box ${$page.route.id.replace(
			'/',
			''
		)}-border mx-auto py-4 max-w-xs bg-slate-900  justify-items-center items-center px-5 rounded-md`}
	>
		<div class="flex flex-col gap-8">
			<div class=" flex flex-row justify-center">
				<div class="container flex flex-row bg-black justify-center ext-8xl sky-300">
					{#each actions as action}
						<div class="py-2">
							<a
								class={`px-4 py-2 rounded-base text-sm font-pixel ${
									$page.route.id === action.path ? 'active-action' : ''
								}`}
								href={action.path}>{action.name}</a
							>
						</div>
					{/each}
				</div>
				<div class="w-12 flex justify-center">
					<img
						alt="liquidity options"
						class="cursor-pointer"
						height="10px"
						width="auto"
						src="drop.png"
					/>
				</div>
			</div>
			<div class="container flex flex-col gap-5">
				<div class="container max-w-lg">
					<div class="container flex flex-col j">
						<div class="container flex flex-row justify-between ">
							<p class="text-base">From</p>
							{#if $walletStore.connected && selectedBaseToken?.symbol}
								<div class="flex flex-row items-center gap-1">
									<p class="text-base">{prettyAmount(selectedBaseToken.amount.toString())}</p>
									<p class="text-sm">{`${selectedBaseToken?.symbol}`}</p>
								</div>
							{/if}
						</div>
						<TokenInput
							name={'base'}
							bind:tokenAmount={baseTokenAmount}
							bind:selectedToken={selectedBaseToken}
						/>
					</div>
				</div>

				<div class="container max-w-lg">
					<div class="container flex flex-col j">
						<div class="container flex flex-row justify-between ">
							<p class="text-base">To</p>
						</div>

						<TokenInput
							name="leverage"
							bind:tokenAmount={leveragedTokenAmount}
							bind:selectedToken={selectedLeverageToken}
						/>
					</div>
				</div>
			</div>
			<div class="container max-w-lg">
				{#if $walletStore.connected}
					<button class="container bg-fuchsia-500 rounded-md">Swap</button>
				{:else}
					<div class="flex  justify-center">
						<WalletMultiButton>Connect to place order</WalletMultiButton>
					</div>
				{/if}
			</div>
		</div>
	</div>
	<div use:autoAnimate={{ duration: 200 }}>
		{#if showQuote}
			<div
				class="container mx-auto py-4 max-w-xs bg-slate-900  justify-items-left items-left px-5 rounded-md flex flex-col gap-2"
			>
				<h3 class="text-xl font-pixel ">Long position</h3>
				<div class="flex flex-col gap-1">
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Entry price</p>
						<p class="text-base font-pixel">{`$${prettyAmount(baseTokenAmount.toString())}`}</p>
					</div>
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Exit price</p>
						<p class="text-base font-pixel">{`$${prettyAmount(baseTokenAmount.toString())}`}</p>
					</div>
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Borrow fee</p>
						<p class="text-base font-pixel">{`${borrowFeePerHr}% / 1hr`}</p>
					</div>
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Available liquidity</p>
						<p class="text-base font-pixel">{`$${prettyAmount(availableLiquidityUSD)}`}</p>
					</div>
				</div>
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
