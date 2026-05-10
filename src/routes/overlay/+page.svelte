<script>
	import { onMount, onDestroy } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { listen } from '@tauri-apps/api/event';
	import { page } from '$app/state';

	let mode = $state('interactive');
	let counter = $state(0);
	/** @type {'blocking' | 'interactable'} */
	let blockerMode = $state('blocking');
	let flashing = $state(false);
	/** @type {(() => void) | undefined} */
	let unlisten;
	/** @type {ReturnType<typeof setTimeout> | undefined} */
	let flashTimer;

	onMount(async () => {
		const m = page.url.searchParams.get('mode');
		mode = m === 'clickthrough' || m === 'blocker' ? m : 'interactive';

		if (mode === 'clickthrough') {
			try {
				await getCurrentWindow().setIgnoreCursorEvents(true);
			} catch (e) {
				console.error('setIgnoreCursorEvents failed', e);
			}
		}

		if (mode === 'blocker') {
			unlisten = await listen('blocker://mode-changed', (event) => {
				const next = event?.payload;
				if (next === 'blocking' || next === 'interactable') {
					blockerMode = next;
				}
			});
		}
	});

	onDestroy(() => {
		unlisten?.();
		if (flashTimer) clearTimeout(flashTimer);
	});

	async function close() {
		try {
			await getCurrentWindow().close();
		} catch (e) {
			console.error('close failed', e);
		}
	}

	function onBlockerMouseDown() {
		if (blockerMode !== 'blocking') return;
		flashing = false;
		// Force a tick so the class re-applies and the animation re-runs.
		requestAnimationFrame(() => {
			flashing = true;
			if (flashTimer) clearTimeout(flashTimer);
			flashTimer = setTimeout(() => {
				flashing = false;
			}, 220);
		});
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
{:else if mode === 'clickthrough'}
	<div class="card clickthrough">
		<div class="title">CLICK-THROUGH</div>
		<div class="hint">try clicking the desktop behind me</div>
	</div>
{:else}
	<div
		class="blocker {blockerMode} {flashing ? 'flash' : ''}"
		onmousedown={onBlockerMouseDown}
		role="presentation"
	>
		{#if blockerMode === 'interactable'}
			<div class="blocker-label">Map Blocker — Ctrl+Shift+B to lock</div>
		{/if}
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

	.blocker {
		box-sizing: border-box;
		width: 100vw;
		height: 100vh;
		font-family: 'DM Sans', system-ui, sans-serif;
		color: #fafafa;
		display: flex;
		align-items: flex-end;
		justify-content: flex-start;
		padding: 6px 8px;
		transition: background 120ms ease-out;
	}

	.blocker.blocking {
		background: transparent;
		border: 1px dashed rgba(255, 100, 100, 0.35);
	}

	.blocker.blocking.flash {
		animation: flashPulse 220ms ease-out;
	}

	@keyframes flashPulse {
		0% {
			background: rgba(255, 80, 80, 0.18);
		}
		100% {
			background: transparent;
		}
	}

	.blocker.interactable {
		background: rgba(30, 30, 35, 0.55);
		border: 2px dashed rgba(236, 72, 153, 0.85);
	}

	.blocker-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		background: rgba(0, 0, 0, 0.55);
		padding: 4px 8px;
		border-radius: 6px;
	}
</style>
