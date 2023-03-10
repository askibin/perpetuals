<script lang="ts">
	import type { Pool } from '../helpers/globalStore';
	import type { SelectorItem } from '../helpers/types';

	let listRef: HTMLUListElement = null;

	export let items: Pool[] = [];
	export let selectedItem = undefined;
	export let showDropdown: boolean = false;

	let selectedIndex: number = 0;
	let selectedId: string = '';
	let previousMod = 0;

	const handleKeydown = (event: KeyboardEvent) => {
		event.preventDefault();
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			if (selectedIndex === -1) {
				selectedIndex = 0;
			} else {
				if (event.key === 'ArrowDown') {
					selectedIndex += 1;
				} else {
					selectedIndex -= 1;
					if (selectedIndex < 0) selectedIndex = 0;
				}
				console.log('selectedIndex: ', selectedIndex, 'items.length: ', items.length);
				selectedIndex = Math.abs(selectedIndex) % items.length;
			}
			console.log('selectedIndex: ', selectedIndex, 'filteredItems: ', items[selectedIndex]);

			selectedId = items[selectedIndex].id;

			const searchTokenList = document.getElementById('searchTokenList');
			const hoverToken = document.getElementById(selectedId);

			const currentMod = hoverToken.offsetTop % searchTokenList.clientHeight;

			console.log('currentMod: ', currentMod, 'previousMod: ', previousMod);
			if (previousMod >= currentMod) {
				searchTokenList.scrollTo(0, hoverToken.offsetTop);
			}
			previousMod = hoverToken.offsetTop % searchTokenList.clientHeight;
		}

		if (event.key === 'Enter' && selectedIndex !== -1) {
			selectedItem = items[selectedIndex];
			selectedIndex = -1;
		}
	};
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="relative ">
	<div class="flex flex-row justify-between">
		<button
			id="search"
			placeholder="Select pool"
			on:click={() => {
				showDropdown = !showDropdown;
				selectedIndex = 0;
				listRef.focus();
			}}
			class="z-1000  text-slate-200 outline-none text-left bg-transparent placeholder-shown:border-gray-500"
			><div class="">
				{#if selectedItem}
					<p class="font-pixel">{selectedItem.value}</p>
				{:else}
					<p class="font-pixel text-slate-400">Select pool</p>
				{/if}
			</div>
		</button>
		<button
			on:click={() => {
				selectedIndex = 0;
				selectedItem = undefined;
				showDropdown = false;
			}}>{'x'}</button
		>
	</div>

	<ul
		bind:this={listRef}
		id="searchTokenList"
		class={`flex absolute top-0 left-0 right-0 flex-col h-12  z-10 absolute  w-120 overflow-scroll  bg-slate-800 rounded-md ${
			showDropdown ? '' : 'hidden'
		}`}
	>
		{#each items as item, index}
			<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
			<li
				tabindex={index}
				id={`${item.id}`}
				class={`hover:bg-sky-700 focus:bg-sky-700 cursor-pointer ${
					index === selectedIndex ? 'bg-sky-700' : ''
				}`}
			>
				<div
					class=""
					on:click={() => {
						selectedIndex = index;
						selectedItem = item;
						showDropdown = false;
					}}
					on:keydown={(event) => {
						if (event.key === 'Enter') {
							selectedIndex = index;
							showDropdown = false;
						}
					}}
				>
					<slot {item} />
				</div>
			</li>
		{/each}
	</ul>
</div>
