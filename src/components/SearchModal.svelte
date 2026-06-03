<script lang="ts">
  import { searchOpen, currentView, setError } from '../lib/stores'
  import { api } from '../lib/api'
  import type { RisultatoRicerca } from '../lib/types'

  let query = ''
  let risultati: RisultatoRicerca[] = []
  let loading = false
  let timer: ReturnType<typeof setTimeout>
  let inputEl: HTMLInputElement

  $: if ($searchOpen) {
    query = ''
    risultati = []
  }

  function onInput() {
    clearTimeout(timer)
    if (query.trim().length < 2) {
      risultati = []
      loading = false
      return
    }
    loading = true
    timer = setTimeout(async () => {
      try {
        risultati = await api.ricerca.searchGlobal(query)
      } catch (e: any) {
        setError(e?.message ?? 'Errore ricerca')
        risultati = []
      } finally {
        loading = false
      }
    }, 280)
  }

  function close() {
    searchOpen.set(false)
  }

  function seleziona(r: RisultatoRicerca) {
    close()
    if (r.tipo === 'cliente') currentView.set('clienti')
    else if (r.tipo === 'ricambio') currentView.set('magazzino')
    else if (r.tipo === 'documento') currentView.set('documenti')
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close()
  }

  const icona: Record<string, string> = {
    cliente: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z',
    ricambio: 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4',
    documento: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
  }

  const tipoColor: Record<string, string> = {
    cliente: 'text-blue-400',
    ricambio: 'text-emerald-400',
    documento: 'text-purple-400',
  }
</script>

{#if $searchOpen}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50" on:click={close}></div>

  <div class="fixed top-24 left-1/2 -translate-x-1/2 w-full max-w-lg z-50 px-4">
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="card shadow-2xl overflow-hidden" on:keydown={onKeydown}>

      <!-- Input -->
      <div class="flex items-center gap-3 px-4 py-3 border-b border-gray-800">
        <svg class="w-4 h-4 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <input
          bind:this={inputEl}
          bind:value={query}
          on:input={onInput}
          class="flex-1 bg-transparent text-gray-100 placeholder-gray-500 text-sm outline-none"
          placeholder="Cerca clienti, ricambi, documenti…"
          autofocus
        />
        {#if loading}
          <div class="w-4 h-4 border-2 border-brand-500 border-t-transparent rounded-full animate-spin shrink-0"></div>
        {/if}
        <kbd class="text-xs text-gray-600 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 shrink-0">Esc</kbd>
      </div>

      <!-- Risultati -->
      {#if risultati.length > 0}
        <div class="max-h-72 overflow-y-auto">
          {#each risultati as r}
            <button
              class="w-full text-left px-4 py-2.5 hover:bg-gray-800 flex items-center gap-3 transition-colors"
              on:click={() => seleziona(r)}
            >
              <div class="w-7 h-7 rounded-md bg-gray-800 flex items-center justify-center shrink-0 {tipoColor[r.tipo]}">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={icona[r.tipo]}/>
                </svg>
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-sm text-gray-200 truncate">{r.titolo}</div>
                {#if r.sottotitolo}
                  <div class="text-xs text-gray-500 truncate">{r.sottotitolo}</div>
                {/if}
              </div>
              <span class="text-xs text-gray-600 capitalize shrink-0">{r.tipo}</span>
            </button>
          {/each}
        </div>
      {:else if !loading && query.trim().length >= 2}
        <div class="py-10 text-center text-sm text-gray-600">
          Nessun risultato per "<span class="text-gray-400">{query}</span>"
        </div>
      {:else}
        <div class="px-4 py-3 text-xs text-gray-600 space-y-1">
          <div>Cerca in <span class="text-gray-400">clienti</span>, <span class="text-gray-400">ricambi</span>, <span class="text-gray-400">documenti</span></div>
        </div>
      {/if}

    </div>
  </div>
{/if}
