<script>
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { check } from '@tauri-apps/plugin-updater';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { onMount } from 'svelte';
	import UploadList from '$lib/UploadList.svelte';
	import SettingsView from '$lib/SettingsView.svelte';

	let uploads = $state([]);
	let config = $state(null);
	let showSettings = $state(false);
	let updateVersion = $state(null);
	let updateStatus = $state('idle');

	onMount(async () => {
		uploads = await invoke('get_uploads');
		config = await invoke('get_config');

		const unlistenUploads = await listen('upload-changed', (event) => {
			uploads = event.payload;
		});

		const unlistenUpdate = await listen('update-available', (event) => {
			updateVersion = event.payload;
		});

		return () => {
			unlistenUploads();
			unlistenUpdate();
		};
	});

	async function installUpdate() {
		updateStatus = 'updating';
		try {
			const update = await check();
			if (update) {
				await update.downloadAndInstall();
				await relaunch();
			}
		} catch (e) {
			console.error('Update failed:', e);
			updateStatus = 'error';
		}
	}
</script>

<div class="flex flex-col h-screen">
	<!-- Header -->
	<div class="flex items-center justify-between px-3 py-2.5 border-b border-zinc-800">
		<h1 class="text-sm font-semibold text-zinc-200">Storm Uploader</h1>
		<button
			onclick={() => { showSettings = !showSettings; }}
			class="rounded-md p-1 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors"
			title={showSettings ? 'Back to uploads' : 'Settings'}
		>
			{#if showSettings}
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
				</svg>
			{:else}
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
						d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
				</svg>
			{/if}
		</button>
	</div>

	<!-- Update banner -->
	{#if updateVersion}
		<div class="flex items-center justify-between px-3 py-1.5 bg-blue-900/50 border-b border-blue-800 text-xs">
			<span class="text-blue-200">v{updateVersion} available</span>
			{#if updateStatus === 'updating'}
				<span class="text-blue-300">Updating...</span>
			{:else if updateStatus === 'error'}
				<button
					onclick={installUpdate}
					class="text-blue-300 hover:text-blue-100 font-medium"
				>Retry</button>
			{:else}
				<button
					onclick={installUpdate}
					class="text-blue-300 hover:text-blue-100 font-medium"
				>Update</button>
			{/if}
		</div>
	{/if}

	<!-- Content -->
	{#if showSettings && config}
		<SettingsView {config} onSave={(c) => { config = c; }} />
	{:else}
		<UploadList {uploads} />
	{/if}
</div>
