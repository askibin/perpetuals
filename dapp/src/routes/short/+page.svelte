<script lang="ts">
	import { Slider } from 'fluent-svelte';
	import { walletStore } from '@svelte-on-solana/wallet-adapter-core';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';
	import autoAnimate from '@formkit/auto-animate';

	import { BigNumber as BN } from 'bignumber.js';
	import TokenInput from '../../components/tokenInput.svelte';
	import type { Token } from '../../helpers/globalStore';
	import { page } from '$app/stores';
	import { prettyAmount } from '../../helpers';
	import { actions } from '../../types';

	// Input
	let leverage = 15;
	let defaultTokenBalance = BN(0);

	// tokens
	let selectedBaseToken: Token | undefined;
	let selectedLeverageToken: Token | undefined;

	let borrowFeePerHr = 0.0001;
	let availableLiquidityUSD = '200000';

	// Amounts
	let baseTokenAmount = '0';
	let leveragedTokenAmount = '0';

	let showQuote = false;

	function calculateLeveragedTokenAmount(prettyAmount: string, leverage: number) {
		const amount = prettyAmount.replaceAll(',', '');
		if (amount === '') {
			return '0';
		}
		const amountBN = new BN(amount);
		const amountUSD = amountBN.times(selectedBaseToken?.priceUSD ?? '0');
		const leverageBN = new BN(leverage);
		const leverageUSD = amountUSD.times(leverageBN);
		const leverageAmount = leverageUSD.dividedBy(selectedLeverageToken?.priceUSD ?? '0');
		return leverageAmount.isNaN() ? '0' : leverageAmount.toString();
	}
	// Update leverage token amount on baseTokenAmount change
	$: leveragedTokenAmount = calculateLeveragedTokenAmount(baseTokenAmount, leverage);
</script>

<div class="container flex flex-col gap-5">
	<div
		class={`container box ${$page.route.id.replace(
			'/',
			''
		)}-border mx-auto py-4 max-w-xs bg-slate-900  justify-items-center items-center px-5 rounded-md`}
	>
		<div class="flex flex-col gap-5">
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
					<img class="cursor-pointer" height="10px" width="auto" src="drop.png" />
				</div>
			</div>
			<div class="container flex flex-col gap-5">
				<div class="container max-w-lg">
					<div class="container flex flex-col j">
						<div class="container flex flex-row justify-between ">
							<p class="text-base">Select base token</p>
							{#if $walletStore.connected && selectedBaseToken?.symbol}
								<div class="flex flex-row items-center gap-1">
									<p class="text-base">{defaultTokenBalance}</p>
									<p class="text-sm">{`${selectedBaseToken?.symbol} balance`}</p>
								</div>
							{/if}
						</div>
						<TokenInput
							name={'base'}
							bind:tokenAmount={baseTokenAmount}
							bind:selectedToken={selectedBaseToken}
							leverage={0}
						/>
					</div>
				</div>

				<div class="container max-w-lg">
					<div class="container flex flex-col j">
						<div class="container flex flex-row justify-between ">
							<p class="text-base">You long position</p>
						</div>

						<TokenInput
							name="leverage"
							bind:tokenAmount={leveragedTokenAmount}
							bind:selectedToken={selectedLeverageToken}
							bind:leverage
						/>
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
						<button
							on:click={() => (showQuote = !showQuote)}
							class="container bg-fuchsia-500 rounded-md">Place Order</button
						>
					{:else}
						<div class="flex  justify-center">
							<WalletMultiButton>Connect to place order</WalletMultiButton>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>
	<div use:autoAnimate={{ duration: 200 }}>
		{#if showQuote}
			<div
				class="container mx-auto py-4 max-w-xs bg-slate-900  justify-items-left items-left px-5 rounded-md flex flex-col gap-2"
			>
				<h3 class="text-xl font-pixel ">Short position</h3>
				<div class="flex flex-col gap-1">
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Entry price</p>
						<p class="text-base font-pixel">{`$${prettyAmount(baseTokenAmount)}`}</p>
					</div>
					<div class="flex flex-row justify-between">
						<p class="text-base font-pixel">Exit price</p>
						<p class="text-base font-pixel">{`$${prettyAmount(baseTokenAmount)}`}</p>
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
