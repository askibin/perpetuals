<script lang="ts">
	import { page } from '$app/stores';

	import Header from '../components/header.svelte';

	import '../app.css';
	import { WalletMultiButton } from '@svelte-on-solana/wallet-adapter-ui';

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

<div>
	<Header />
	<div>
		<div class="wrapper-app">
			<div>
				<WalletMultiButton />
			</div>
			<div class="title">
				<h1 class="text-8xl sky-300 font-pixel">SANDBLIZZARD</h1>
				<h2 class="text-4xl sky-300 font-pixel">PERPETUAL DEX</h2>
			</div>

			<div class=" container mx-auto flex flex-col m-6">
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
						<slot />
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style style="postcss">
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
	.box {
		position: relative;
		display: block;
	}
	.long-border:before {
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

	.short-border:before {
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

	.swap-border:before {
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
