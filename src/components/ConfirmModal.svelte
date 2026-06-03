<script lang="ts">
  export let open = false
  export let title = 'Conferma'
  export let message = ''
  export let confirmLabel = 'Elimina'
  export let confirmClass = 'btn-danger'
  export let onConfirm: () => void = () => {}
  export let onCancel: () => void = () => { open = false }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return
    if (e.key === 'Escape') onCancel()
    if (e.key === 'Enter') onConfirm()
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <!-- Overlay -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center"
    role="dialog"
    aria-modal="true"
  >
    <!-- Sfondo scuro -->
    <div
      class="absolute inset-0 bg-black/60 backdrop-blur-sm"
      on:click={onCancel}
    ></div>

    <!-- Card modale -->
    <div class="relative card w-full max-w-sm mx-4 p-6 shadow-2xl space-y-4 animate-in">
      <div class="flex items-start gap-3">
        <div class="w-9 h-9 rounded-full bg-red-900/40 border border-red-800/50 flex items-center justify-center shrink-0 mt-0.5">
          <svg class="w-4 h-4 text-red-400" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
        </div>
        <div>
          <h2 class="text-base font-semibold text-white">{title}</h2>
          <p class="text-sm text-gray-400 mt-1">{message}</p>
        </div>
      </div>

      <div class="flex gap-2 justify-end pt-1">
        <button class="btn-secondary" on:click={onCancel}>Annulla</button>
        <button class={confirmClass} on:click={onConfirm}>{confirmLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .animate-in {
    animation: fadeScale 0.15s ease-out;
  }
  @keyframes fadeScale {
    from { opacity: 0; transform: scale(0.95); }
    to   { opacity: 1; transform: scale(1); }
  }
</style>
