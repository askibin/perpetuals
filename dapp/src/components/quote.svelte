<script lang="ts">
	import { prettyAmount } from '../helpers';
	export let baseTokenAmount: string | undefined;
	export let borrowFeePerHr: number;
	export let availableLiquidityUSD: string;
</script>

<div
	class="container mx-auto py-4 max-w-xs bg-slate-900  justify-items-left items-left px-5 rounded-md flex flex-col gap-2"
>
	<h3 class="text-xl font-pixel ">Long position</h3>
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
