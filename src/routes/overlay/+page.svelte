<script>
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { page } from '$app/state';

	let mode = $state('interactive');
	let counter = $state(0);

	onMount(async () => {
		mode = page.url.searchParams.get('mode') === 'clickthrough' ? 'clickthrough' : 'interactive';
		if (mode === 'clickthrough') {
			try {
				await getCurrentWindow().setIgnoreCursorEvents(true);
			} catch (e) {
				console.error('setIgnoreCursorEvents failed', e);
			}
		}
	});

	async function close() {
		try {
			await getCurrentWindow().close();
		} catch (e) {
			console.error('close failed', e);
		}
	}
</script>

<svelte:head>
	<style>
		html,
		body {
			background: transparent !important;
		}
	</style>
</svelte:head>

{#if mode === 'interactive'}
	<div class="card interactive" data-tauri-drag-region>
		<div class="title" data-tauri-drag-region>INTERACTIVE</div>
		<div class="hint" data-tauri-drag-region>drag me anywhere</div>
		<div class="row">
			<button onclick={() => counter++}>clicked {counter}</button>
			<button class="close" onclick={close} aria-label="close">×</button>
		</div>
	</div>
{:else}
	<div class="card clickthrough">
		<div class="title">CLICK-THROUGH</div>
		<div class="hint">try clicking the desktop behind me</div>
	</div>
{/if}

<style>
	:global(html),
	:global(body) {
		background: transparent !important;
		margin: 0;
		padding: 0;
		overflow: hidden;
	}

	.card {
		box-sizing: border-box;
		width: 100vw;
		height: 100vh;
		padding: 14px 16px;
		border-radius: 14px;
		font-family: 'DM Sans', system-ui, sans-serif;
		color: #fafafa;
		display: flex;
		flex-direction: column;
		gap: 8px;
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
	}

	.interactive {
		background: rgba(59, 130, 246, 0.55);
		border: 1px solid rgba(147, 197, 253, 0.7);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
	}

	.clickthrough {
		background: rgba(236, 72, 153, 0.45);
		border: 1px dashed rgba(251, 207, 232, 0.8);
	}

	.title {
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0.12em;
	}

	.hint {
		font-size: 11px;
		opacity: 0.85;
	}

	.row {
		margin-top: auto;
		display: flex;
		gap: 8px;
		align-items: center;
	}

	button {
		font: inherit;
		color: #fafafa;
		background: rgba(255, 255, 255, 0.18);
		border: 1px solid rgba(255, 255, 255, 0.3);
		border-radius: 8px;
		padding: 6px 10px;
		cursor: pointer;
	}

	button:hover {
		background: rgba(255, 255, 255, 0.28);
	}

	button.close {
		margin-left: auto;
		padding: 2px 8px;
		font-size: 14px;
		line-height: 1;
	}
</style>
