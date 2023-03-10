<script lang="ts">
	import { Slider } from 'fluent-svelte';
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';
	import autoAnimate from '@formkit/auto-animate';

	import type { BigNumber as BN } from 'bignumber.js';
	import TokenInput from '../../components/tokenInput.svelte';
	import type { Token } from '../../helpers/globalStore';
	import { page } from '$app/stores';
	import { LP_TOKEN_ADDRESSES, prettyAmount } from '../../helpers';
	import { actions } from '../../types';

	// Input
	let leverage = 15;

	// tokens
	let selectedBaseToken: Token | undefined;
	let selectedLPToken: Token | undefined;

	// Amounts
	let baseTokenAmount: BN | undefined;
	let selectedLPTokenAmount: BN | undefined;

	let showQuote = false;

	function calculateLPTokenAmount(prettyAmount: BN | undefined, leverage: number) {}
	// Update leverage token amount on baseTokenAmount change
	$: leveragedTokenAmount = calculateLPTokenAmount(baseTokenAmount, leverage);

	$: showQuote = baseTokenAmount && baseTokenAmount.gt(0) ? true : false;
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
							<p class="text-base">Deposit</p>
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
							<p class="text-base">Your LP tokens</p>
						</div>

						<TokenInput
							name="leverage"
							bind:tokenAmount={selectedLPTokenAmount}
							bind:selectedToken={selectedLPToken}
							allowSelectToken={false}
							allowedTokens={LP_TOKEN_ADDRESSES}
						/>
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
	</div>
	<div use:autoAnimate={{ duration: 200 }}>
		{#if showQuote}
			<div
				class="container mx-auto py-4 max-w-xs bg-slate-900  justify-items-left items-left px-5 rounded-md flex flex-col gap-2"
			>
				<h3 class="text-xl font-pixel ">LP position</h3>
				<div class="flex flex-col gap-1">
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">APR</p>
						<p class="text-base font-pixel">{`$${10.25}%`}</p>
					</div>
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Total staked in pool</p>
						<p class="text-base font-pixel">{`$${0.0}`}</p>
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
