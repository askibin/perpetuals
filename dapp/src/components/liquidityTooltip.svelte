<script lang="ts">
	import { BigNumber as BN } from 'bignumber.js';
	import { Flyout, Button } from 'fluent-svelte';
	import { prettyAmount } from '../helpers';
	import { defaultPool, type Pool } from '../helpers/globalStore';
	import Selector from './selector.svelte';

	let showPoolSelector = false;

	// Pool selection
	let DUMMY_POOLS: Pool[] = [
		{ value: 'Index pool', label: 'Index pool', id: '1', tvl: '1000000', apr: '10.23' },
		{ value: 'Stable pool', label: 'Stable pool', id: '2', tvl: '2000000', apr: '9.54' },
		{ value: 'Low cap pool', label: 'Low cap pool', id: '3', tvl: '3000000', apr: '11.1' }
	];

	let selectedPool = $defaultPool;
	$: {
		console.log('selected pool: ', selectedPool);
		defaultPool.set(selectedPool);
		if (selectedPool) showPoolSelector = false;
	}
</script>

<div class="w-12 flex justify-center items-center">
	<Button
		on:click={() => {
			showPoolSelector = !showPoolSelector;
		}}
		class="!border-none !p-0 flex flex-col"
	>
		{#if $defaultPool}
			<img
				alt="liquidity options"
				class="cursor-pointer absolute h-7 "
				height="5px"
				src="drop.png"
			/>
			<p class="font-pixel text-sm z-10">{`${$defaultPool.label}`}</p>
		{:else}
			<img
				alt="liquidity options"
				class="cursor-pointer "
				height="10px"
				width="auto"
				src="drop.png"
			/>
		{/if}
	</Button>
	<Flyout
		bind:open={showPoolSelector}
		placement={'bottom'}
		alignment={'end'}
		class=" !flex !justify-center"
	>
		<svelte:fragment slot="flyout"
			><div class="rounded-md !bg-slate-800 p-4 flex flex-col gap-2">
				<p class="font-pixel text-sm">
					We will always use the optimal pool. If you still want to choose a pool, pick here.
				</p>
				<Selector
					bind:selectedItem={selectedPool}
					let:item
					items={DUMMY_POOLS}
					showDropdown={false}
				>
					<div class="flex flex-row justify-around items-center">
						<p class="font-pixel text-sm ">{item.label}</p>
						<div class="flex flex-col ">
							<p class="font-pixel text-sm">{`TVL: $${prettyAmount(item?.tvl)}`}</p>
							<p class="font-pixel text-sm">{`APR: ${item?.apr}%`}</p>
						</div>
					</div>
				</Selector>
			</div></svelte:fragment
		>
	</Flyout>
</div>
