<script lang="ts">
	// Input
	let amount: string;
	let textTimeout: any;
	const handleAmountChange = (am: KeyboardEvent) => {
		clearTimeout(textTimeout);
		textTimeout = setTimeout(() => {
			console.log('start');
			const parts = (am.target as HTMLInputElement).value.replaceAll(',', '').split('.');
			const numberPart = parts[0];
			const decimalPart = parts[1];
			amount =
				numberPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',') + (decimalPart ? '.' + decimalPart : '');
		}, 500);
	};

	const actions = [
		{
			name: 'Long',
			path: '/long'
		},
		{
			name: 'Short',
			path: '/short'
		},
		{
			name: 'Swap',
			path: '/swap'
		},
		{
			name: 'Earn',
			path: '/earn'
		}
	];
</script>

<div class="container">
	<div class="container max-w-lg">
		<div class="container flex flex-col j">
			<div class="container flex flex-row justify-between ">
				<p class="text-base">You pay</p>
				<div class="flex flex-row items-center gap-1">
					<p class="text-base">4,3223</p>
					<p class="text-sm">sol balance</p>
				</div>
			</div>
			<div class="container bg-slate-800 py-1 px-5 flex flex-row justify-between rounded-md  ">
				<div class="flex items-center">
					<img />
					<p>SOL</p>
					<p>></p>
				</div>
				<div class="flex flex-col">
					<input
						bind:value={amount}
						on:keypress={handleAmountChange}
						placeholder="0.0"
						name="amount"
						type="text"
						class="text-base outline-none  text-right bg-transparent placeholder-shown:border-gray-500"
					/>
					<p class="text-sm text-slate-600 text-right">$ 0</p>
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
					<img />
					<p>SOL</p>
					<p>></p>
				</div>
				<div class="flex flex-col">
					<p class="text-base text-slate-800">5.5</p>
					<p class="text-sm">$ 90.19</p>
				</div>
			</div>
		</div>
	</div>

	<div class="container max-w-lg">
		<div class="container flex flex-row justify-between items-center ">
			<p class="text-small">Leverage</p>
			<div>--------------</div>
			<div>15x</div>
		</div>
	</div>

	<div class="container max-w-lg">
		<button class="container bg-fuchsia-500 rounded-md">Place Order</button>
	</div>
</div>
