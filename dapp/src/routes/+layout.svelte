<script lang="ts">
	import { onMount } from 'svelte';

	import Header from '../components/header.svelte';
	import { SvelteToast } from '@zerodevx/svelte-toast';
	import { Metaplex } from '@metaplex-foundation/js';
	import * as metadata from '@metaplex-foundation/mpl-token-metadata';

	import '../app.css';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';
	import { tokensStore, type Tokens } from '../helpers/globalStore';
	import { getTokenMetaDataFromSolanaFM, LP_TOKEN_ADDRESSES } from '../helpers';

	// Start with whitelist of tokens to search for
	const tokenAddresses = ['DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263'];

	onMount(async () => {
		const allTokens = await Promise.all(
			tokenAddresses.map((address) => {
				return getTokenMetaDataFromSolanaFM(address);
			})
		);
		const lpTokens = await Promise.all(
			LP_TOKEN_ADDRESSES.map((address) => {
				return {
					address,
					amount: 0,
					priceUSD: 1,
					symbol: 'SAND',
					name: 'BLIZZARD SAND LP TOKEN',
					icon: '',
					website: '',
					twitter: '',
					tag: [],
					decimals: 6,
					coingeckoId: '',
					holder: 0
				};
			})
		);
		tokensStore.set([...allTokens, ...lpTokens]);
	});

	const toastOptions = {
		reversed: true
	};
</script>

<div class="contaniner flex flex-col gap-20">
	<Header />
	<div>
		<div class="wrapper-app">
			<div class=" container mx-auto flex flex-col m-6">
				<slot />
			</div>
		</div>
	</div>
	<SvelteToast options={toastOptions} />
</div>

<style style="postcss">
	:root {
		--toastContainerTop: auto;
		--toastContainerRight: 0;
		--toastContainerBottom: 4rem;
	}

	:global(body) {
		margin: 0;
		background-color: #000000;
		color: aliceblue;
	}

	:global(p) {
		font-family: pixel;
		font-size: 24px;
	}

	@keyframes glower {
		0% {
			background-position: 0 0;
		}

		100% {
			background-position: 400% 400%;
		}
	}
	:global(.box) {
		position: relative;
		display: block;
	}
	:global(.long-border:before) {
		content: '';
		position: absolute;
		border-radius: 5px;
		left: -2px;
		top: -2px;
		background: linear-gradient(45deg, transparent, #00ff66, transparent);
		background-size: 400%;
		width: calc(100% + 5px);
		height: calc(100% + 5px);
		z-index: -1;
		animation: glower 10s linear infinite;
	}

	:global(.short-border:before) {
		content: '';
		position: absolute;
		border-radius: 5px;
		left: -2px;
		top: -2px;
		background: linear-gradient(45deg, transparent, red, transparent);
		background-size: 400%;
		width: calc(100% + 5px);
		height: calc(100% + 5px);
		z-index: -1;
		animation: glower 10s linear infinite;
	}

	:global(.swap-border:before) {
		content: '';
		position: absolute;
		border-radius: 5px;
		left: -2px;
		top: -2px;
		background: linear-gradient(45deg, transparent, blue, transparent);
		background-size: 400%;
		width: calc(100% + 5px);
		height: calc(100% + 5px);
		z-index: -1;
		animation: glower 10s linear infinite;
	}

	:global(.earn-border:before) {
		content: '';
		position: absolute;
		border-radius: 5px;
		left: -2px;
		top: -2px;
		background: linear-gradient(45deg, transparent, pink, transparent);
		background-size: 400%;
		width: calc(100% + 5px);
		height: calc(100% + 5px);
		z-index: -1;
		animation: glower 10s linear infinite;
	}

	.wrapper-app {
		height: 100vh;
		font-family: 'Gill Sans', 'Gill Sans MT', Calibri, 'Trebuchet MS', sans-serif;
	}

	:global(.active-action) {
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
