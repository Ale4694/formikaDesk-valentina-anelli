<script lang="ts">
  import { createEventDispatcher, onDestroy } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'

  export let updateInfo: { version: string; notes: string; date: string }
  export let currentVersion: string = ''

  const dispatch = createEventDispatcher()

  let installing = false
  let progress = 0
  let error = ''
  let unlisten: UnlistenFn | null = null

  async function startInstall() {
    installing = true
    error = ''
    progress = 0

    unlisten = await listen<number>('update-progress', (event) => {
      progress = event.payload
    })

    try {
      await invoke('install_update')
    } catch (e: any) {
      error = e?.message ?? String(e)
      installing = false
      unlisten?.()
      unlisten = null
    }
  }

  function dismiss() {
    dispatch('dismiss')
  }

  onDestroy(() => {
    unlisten?.()
  })
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
  <div class="bg-gray-900 border border-gray-700 rounded-xl shadow-2xl w-full max-w-md mx-4">
    <!-- Header -->
    <div class="px-6 pt-6 pb-4 border-b border-gray-700">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-blue-600/20 border border-blue-500/30 flex items-center justify-center shrink-0">
          <svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
          </svg>
        </div>
        <div>
          <h2 class="text-base font-semibold text-white">Aggiornamento disponibile</h2>
          <p class="text-xs text-gray-400 mt-0.5">FormikaDesk si aggiorna automaticamente</p>
        </div>
      </div>
    </div>

    <!-- Body -->
    <div class="px-6 py-4 space-y-4">
      <!-- Version comparison -->
      <div class="flex items-center gap-3">
        <div class="flex-1 bg-gray-800 rounded-lg px-3 py-2 text-center">
          <p class="text-xs text-gray-500 mb-0.5">Versione attuale</p>
          <p class="text-sm font-mono font-medium text-gray-300">{currentVersion || '—'}</p>
        </div>
        <svg class="w-4 h-4 text-blue-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
        </svg>
        <div class="flex-1 bg-blue-900/30 border border-blue-500/30 rounded-lg px-3 py-2 text-center">
          <p class="text-xs text-blue-400 mb-0.5">Nuova versione</p>
          <p class="text-sm font-mono font-medium text-blue-300">{updateInfo.version}</p>
        </div>
      </div>

      <!-- Release notes -->
      {#if updateInfo.notes}
        <div class="bg-gray-800 rounded-lg p-3">
          <p class="text-xs font-medium text-gray-400 mb-1.5">Note di rilascio</p>
          <p class="text-sm text-gray-300 whitespace-pre-wrap leading-relaxed">{updateInfo.notes}</p>
        </div>
      {/if}

      <!-- Progress bar -->
      {#if installing}
        <div class="space-y-2">
          <div class="flex items-center justify-between text-xs text-gray-400">
            <span>{progress > 0 ? 'Download in corso...' : 'Avvio download...'}</span>
            {#if progress > 0}
              <span>{progress}%</span>
            {/if}
          </div>
          <div class="w-full bg-gray-700 rounded-full h-2 overflow-hidden">
            <div
              class="h-2 rounded-full transition-all duration-300"
              style="width: {progress > 0 ? progress : 100}%; background-color: #185FA5; {progress === 0 ? 'animation: pulse 1.5s ease-in-out infinite;' : ''}"
            ></div>
          </div>
          {#if progress === 100}
            <p class="text-xs text-gray-400 text-center">Installazione... l'app si riavvierà a breve</p>
          {/if}
        </div>
      {/if}

      <!-- Error -->
      {#if error}
        <div class="bg-red-900/30 border border-red-700/50 rounded-lg px-3 py-2">
          <p class="text-xs text-red-400">{error}</p>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    {#if !installing}
      <div class="px-6 pb-6 flex gap-3">
        <button
          on:click={dismiss}
          class="flex-1 px-4 py-2 text-sm font-medium text-gray-400 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors"
        >
          Più tardi
        </button>
        <button
          on:click={startInstall}
          class="flex-1 px-4 py-2 text-sm font-medium text-white rounded-lg transition-colors"
          style="background-color: #185FA5;"
          on:mouseenter={(e) => (e.currentTarget.style.backgroundColor = '#1a6fbe')}
          on:mouseleave={(e) => (e.currentTarget.style.backgroundColor = '#185FA5')}
        >
          Aggiorna ora
        </button>
      </div>
    {:else}
      <div class="px-6 pb-6">
        <p class="text-xs text-gray-500 text-center">Non chiudere l'applicazione durante l'aggiornamento</p>
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
