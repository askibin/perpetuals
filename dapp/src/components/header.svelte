<script lang="ts">
	import { WalletProvider } from '@svelte-on-solana/wallet-adapter-ui';
	import { AnchorConnectionProvider } from '@svelte-on-solana/wallet-adapter-anchor';
	import { onMount } from 'svelte';
	import { clusterApiUrl } from '@solana/web3.js';
	const localStorageKey = 'walletAdapter';
	const network = clusterApiUrl('devnet');
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

<div class="container flex flex-row justify-end ">
	<div class="flex flex-row justify-end w-10">
		<WalletProvider {localStorageKey} {wallets} autoConnect />
	</div>
</div>
