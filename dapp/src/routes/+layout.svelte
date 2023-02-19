<script lang="ts">
   import { page } from '$app/stores';

	
	import { onMount } from 'svelte';
	import { clusterApiUrl } from '@solana/web3.js';
	import { WalletProvider } from '@svelte-on-solana/wallet-adapter-ui';
	import { AnchorConnectionProvider } from '@svelte-on-solana/wallet-adapter-anchor';
	import '../app.css';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';

	const localStorageKey = 'walletAdapter';
	const network = clusterApiUrl('devnet');
	let wallets;


	console.log("page: ",$page.route.id)

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

<div>
	<WalletProvider {localStorageKey} {wallets} autoConnect />
	<div>
		<div class="wrapper-app">
			<div>
				<WalletMultiButton />
			</div>
			<div class="title">
				<h1 class="text-8xl sky-300 font-pixel">SANDBLIZZARD</h1>
			</div>

			<div class=" container mx-auto gray-200 flex flex-col m-6 gap-y-1 bg-slate-800">
				<p class="font-pixel mx-auto text-4xl">Start earning</p>
				<div class="container mx-auto flex flex-row max-w-md bg-slate-600 justify-center">
					<div
						class="container flex flex-column bg-slate-500 mx-5 mt-5 gap-10 justify-center ext-8xl sky-300 "
					>
						<div><a class={`${$page.route.id === "/long" ? "active-action":""}`} href="/long">Long</a></div>
						<div><a href="/short">Short</a></div>
						<div>
							<a href="/swap">Swap</a>
						</div>
						<div><a href="/earn">Earn</a></div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
<slot />

<style style="postcss">
	:global(body) {
		padding: 100px;
		margin: 0;
		background-color: #000000;
		color: aliceblue;
	}

	:global(p) {
		font-family: pixel;
		font-size: 24px;
	}
	.wrapper-app {
		height: 100vh;
		font-family: 'Gill Sans', 'Gill Sans MT', Calibri, 'Trebuchet MS', sans-serif;
	}

	.active-action {
		background-color: white;
		color: blue;
	}
	.title {
		text-align: center;
		color: white;
		font-size: 20px;
		margin-bottom: 40px;
	}
</style>
