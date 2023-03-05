<script lang="ts">
	import { WalletMultiButton, WalletProvider } from '@svelte-on-solana/wallet-adapter-ui';
	import { onMount } from 'svelte';
	import { clusterApiUrl } from '@solana/web3.js';
	const localStorageKey = 'walletAdapter';
	let wallets;

	onMount(async () => {
		const {
			PhantomWalletAdapter,
			SlopeWalletAdapter,
			SolflareWalletAdapter,
			SolletExtensionWalletAdapter,
			TorusWalletAdapter
		} = await import('@solana/wallet-adapter-wallets');

		const walletsMap = [
			new PhantomWalletAdapter(),
			new SlopeWalletAdapter(),
			new SolflareWalletAdapter(),
			new SolletExtensionWalletAdapter(),
			new TorusWalletAdapter()
		];

		wallets = walletsMap;
	});
</script>

<WalletProvider {localStorageKey} {wallets} autoConnect />
<div class="flex justify-between px-10 items-center ">
	<div class="flex flex-col gap-0 w-auto">
		<h2 class="text-6xl text-sky-300 font-pixel m-0 p-0">BLIZZARD</h2>
		<p class=" px-10 text-lg font-pixel">a PERPETUAL DEX</p>
	</div>

	<div>
		<WalletMultiButton />
	</div>
</div>
