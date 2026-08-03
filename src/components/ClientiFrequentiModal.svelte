<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { formatDate, formatRelativeTime, setError } from '../lib/stores'
  import { api } from '../lib/api'
  import type { ClienteTopDocumenti } from '../lib/types'

  const dispatch = createEventDispatcher<{
    close: void
    select: { clienteId: number; ragioneSociale: string }
  }>()

  const annoCorrente = new Date().getFullYear()

  let loading = true
  let clienti: ClienteTopDocumenti[] = []
  let highlightIndex = 0
  let rowEls: (HTMLElement | null)[] = []

  onMount(() => {
    // Nessun focus() qui: il trigger è il campo di ricerca cliente sotto,
    // che deve restare a fuoco e continuare a ricevere ciò che si digita.
    // Niente campo di ricerca interno: la ricerca è già quel campo.
    caricaLista()
  })

  async function caricaLista() {
    loading = true
    try {
      clienti = await api.dashboard.getClientiTopDocumenti()
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento clienti più attivi')
    } finally {
      loading = false
    }
  }

  $: if (rowEls[highlightIndex]) rowEls[highlightIndex]?.scrollIntoView({ block: 'nearest' })

  function seleziona(c: ClienteTopDocumenti) {
    dispatch('select', { clienteId: c.cliente_id, ragioneSociale: c.ragione_sociale })
  }

  function close() {
    dispatch('close')
  }

  // Su <svelte:window>, non su un campo interno: qui non c'è nessun campo
  // interno che possa tenere il focus, dato che il pannello non lo ruba
  // mai al campo cliente sottostante. Il componente esiste solo mentre il
  // pannello è aperto, quindi il listener sparisce da solo alla chiusura.
  // Non intercetta altro che Escape/frecce/Invio: digitare continua a
  // raggiungere il campo cliente esattamente come prima.
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.preventDefault(); close(); return }
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (clienti.length) highlightIndex = Math.min(highlightIndex + 1, clienti.length - 1)
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (clienti.length) highlightIndex = Math.max(highlightIndex - 1, 0)
      return
    }
    if (e.key === 'Enter') {
      e.preventDefault()
      const c = clienti[highlightIndex]
      if (c) seleziona(c)
    }
  }
</script>

<svelte:window on:keydown={onKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50" on:click={close}></div>

<div class="fixed top-10 left-1/2 -translate-x-1/2 w-full max-w-2xl z-50 px-4">
  <div class="card shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">

    <!-- Titolo -->
    <div class="flex items-center justify-between px-4 py-2.5 border-b border-gray-800 shrink-0">
      <h2 class="text-sm font-semibold text-white truncate">Clienti più attivi {annoCorrente}</h2>
      <kbd class="text-xs text-gray-600 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 shrink-0">Esc</kbd>
    </div>

    <div class="overflow-y-auto min-h-0">
      {#if loading}
        <div class="flex justify-center py-10">
          <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if clienti.length === 0}
        <p class="text-sm text-gray-500 text-center py-10">Nessun cliente con documenti quest'anno.</p>
      {:else}
        <table class="w-full text-sm table-fixed">
          <colgroup>
            <col class="w-[42%]" />
            <col class="w-[20%]" />
            <col class="w-[13%]" />
            <col class="w-[25%]" />
          </colgroup>
          <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide sticky top-0">
            <tr>
              <th class="px-3 py-2.5 text-left">Ragione sociale</th>
              <th class="px-3 py-2.5 text-left">Città</th>
              <th class="px-3 py-2.5 text-right">Doc.</th>
              <th class="px-3 py-2.5 text-left">Ultimo documento</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-800">
            {#each clienti as c, i (c.cliente_id)}
              <tr
                bind:this={rowEls[i]}
                class="table-row-hover cursor-pointer {i === highlightIndex ? 'bg-brand-600/20 ring-1 ring-inset ring-brand-500/60' : ''}"
                on:click={() => seleziona(c)}
                on:mouseenter={() => highlightIndex = i}
              >
                <td class="px-3 py-3 text-gray-200 truncate" title={c.ragione_sociale}>{c.ragione_sociale}</td>
                <td class="px-3 py-3 text-gray-400 text-xs truncate">{c.citta ?? '—'}</td>
                <td class="px-3 py-3 text-right font-mono text-brand-400 font-bold text-base">{c.numero_documenti}</td>
                <td class="px-3 py-3 text-gray-400 text-xs">
                  <div>{formatDate(c.ultimo_documento)}</div>
                  <div class="text-gray-500 text-[10px]">{formatRelativeTime(c.ultimo_documento)}</div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </div>
</div>
