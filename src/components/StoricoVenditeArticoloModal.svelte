<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { formatCurrency, formatDate, formatRelativeTime, setError, schedaClienteId, currentView } from '../lib/stores'
  import { api } from '../lib/api'
  import type { ClienteVendutoArticolo, MovimentoVenditaCliente, TipoDocumento } from '../lib/types'

  export let articoloId: number
  export let articoloCodice: string = ''
  export let articoloDescrizione: string = ''

  const dispatch = createEventDispatcher<{ close: void }>()

  let query = ''
  let loading = true
  let clienti: ClienteVendutoArticolo[] = []
  let highlightIndex = 0
  let rowEls: (HTMLElement | null)[] = []
  let searchInputEl: HTMLInputElement

  let expandedClienteId: number | null = null
  let movimentiMap: Record<number, MovimentoVenditaCliente[]> = {}
  let loadingMovimenti: number | null = null

  const tipoLabel: Partial<Record<TipoDocumento, string>> = {
    fattura: 'Fattura', ddt: 'DDT', nota_credito: 'Nota credito',
    vendita_banco: 'Vendita Banco', buono: 'Buono', preventivo: 'Preventivo',
    fattura_differita: 'Fattura Differita', ddt_fornitore: 'DDT Fornitore',
  }

  onMount(() => {
    searchInputEl?.focus()
    caricaLista()
  })

  async function caricaLista() {
    loading = true
    try {
      clienti = await api.storicoArticolo.getClientiPerArticolo(articoloId)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento storico vendite articolo')
    } finally {
      loading = false
    }
  }

  async function toggleEspandi(c: ClienteVendutoArticolo) {
    if (expandedClienteId === c.cliente_id) { expandedClienteId = null; return }
    expandedClienteId = c.cliente_id
    if (movimentiMap[c.cliente_id]) return
    loadingMovimenti = c.cliente_id
    try {
      const mv = await api.storicoCliente.getMovimentiArticolo(c.cliente_id, articoloId)
      movimentiMap = { ...movimentiMap, [c.cliente_id]: mv }
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento movimenti')
    } finally {
      loadingMovimenti = null
    }
  }

  function apriScheda(c: ClienteVendutoArticolo) {
    schedaClienteId.set(c.cliente_id)
    currentView.set('scheda-cliente')
    close()
  }

  function close() {
    dispatch('close')
  }

  function onQueryInput() {
    highlightIndex = 0
  }

  // Vendite più frequenti in cima, come nel pannello cliente speculare.
  $: filtered = (query.trim()
    ? clienti.filter(c => c.ragione_sociale.toLowerCase().includes(query.trim().toLowerCase()))
    : clienti
  ).slice().sort((a, b) => b.numero_vendite - a.numero_vendite)

  $: if (rowEls[highlightIndex]) rowEls[highlightIndex]?.scrollIntoView({ block: 'nearest' })

  // Stesso pattern di StoricoPrezziClienteModal: su <svelte:window>, non sul
  // campo di ricerca, perché il focus può restare sul campo/riga che ha
  // aperto F2. Il componente esiste solo mentre il pannello è aperto.
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.preventDefault(); close(); return }
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (filtered.length) highlightIndex = Math.min(highlightIndex + 1, filtered.length - 1)
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (filtered.length) highlightIndex = Math.max(highlightIndex - 1, 0)
      return
    }
    if (e.key === 'Enter') {
      e.preventDefault()
      const c = filtered[highlightIndex]
      if (c) apriScheda(c)
    }
  }
</script>

<svelte:window on:keydown={onKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50" on:click={close}></div>

<div class="fixed top-10 left-1/2 -translate-x-1/2 w-full max-w-4xl z-50 px-4">
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="card shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">

    <!-- Titolo -->
    <div class="px-4 py-2.5 border-b border-gray-800 shrink-0">
      <h2 class="text-sm font-semibold text-white truncate">
        Storico vendite
        {#if articoloCodice}
          — <span class="font-mono text-brand-400">{articoloCodice}</span>
        {/if}
        {#if articoloDescrizione}<span class="text-gray-400 font-normal"> {articoloDescrizione}</span>{/if}
      </h2>
    </div>

    <!-- Input ricerca -->
    <div class="flex items-center gap-3 px-4 py-3 border-b border-gray-800 shrink-0">
      <svg class="w-4 h-4 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
      </svg>
      <input
        bind:this={searchInputEl}
        bind:value={query}
        on:input={onQueryInput}
        class="flex-1 bg-transparent text-gray-100 placeholder-gray-500 text-sm outline-none"
        placeholder="Cerca cliente…"
      />
      {#if !loading}
        <span class="text-xs text-gray-500 shrink-0">
          {query.trim() ? `${filtered.length} di ${clienti.length}` : `${clienti.length} client${clienti.length === 1 ? 'e' : 'i'}`}
        </span>
      {/if}
      <kbd class="text-xs text-gray-600 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 shrink-0">Esc</kbd>
    </div>

    <div class="overflow-y-auto min-h-0">
      {#if loading}
        <div class="flex justify-center py-10">
          <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if filtered.length === 0}
        <p class="text-sm text-gray-500 text-center py-10">
          {clienti.length === 0 ? 'Nessuna vendita registrata per questo articolo.' : 'Nessun cliente corrisponde alla ricerca.'}
        </p>
      {:else}
        <table class="w-full text-sm table-fixed">
          <colgroup>
            <col class="w-[5%]" />
            <col class="w-[42%]" />
            <col class="w-[16%]" />
            <col class="w-[12%]" />
            <col class="w-[25%]" />
          </colgroup>
          <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide sticky top-0">
            <tr>
              <th class="px-2 py-2.5"></th>
              <th class="px-3 py-2.5 text-left">Cliente</th>
              <th class="px-3 py-2.5 text-right">Ultimo prezzo</th>
              <th class="px-3 py-2.5 text-right">Vendite</th>
              <th class="px-3 py-2.5 text-left">Ultima vendita</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-800">
            {#each filtered as c, i (c.cliente_id)}
              <tr
                bind:this={rowEls[i]}
                class="table-row-hover cursor-pointer {i === highlightIndex ? 'bg-brand-600/20 ring-1 ring-inset ring-brand-500/60' : ''}"
                on:click={() => apriScheda(c)}
                on:mouseenter={() => highlightIndex = i}
              >
                <td class="px-2 py-3.5">
                  <button
                    class="text-gray-500 hover:text-gray-200 leading-none px-1"
                    title="Mostra movimenti"
                    on:click|stopPropagation={() => toggleEspandi(c)}
                  >
                    {expandedClienteId === c.cliente_id ? '▾' : '▸'}
                  </button>
                </td>
                <td class="px-3 py-3.5 text-gray-200 truncate" title={c.ragione_sociale}>{c.ragione_sociale}</td>
                <td class="px-3 py-3.5 text-right font-mono text-green-400 font-bold text-base">{formatCurrency(c.prezzo_ultimo)}</td>
                <td class="px-3 py-3.5 text-right text-gray-400 text-xs">{c.numero_vendite}</td>
                <td class="px-3 py-3.5 text-gray-400 text-xs">
                  <div>{formatDate(c.ultima_vendita)}</div>
                  <div class="text-gray-500 text-[10px]">{formatRelativeTime(c.ultima_vendita)}</div>
                </td>
              </tr>
              {#if expandedClienteId === c.cliente_id}
                <tr>
                  <td colspan="5" class="bg-gray-900/60 px-4 py-3">
                    {#if loadingMovimenti === c.cliente_id}
                      <div class="flex justify-center py-4">
                        <div class="w-4 h-4 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
                      </div>
                    {:else}
                      {@const mv = movimentiMap[c.cliente_id] ?? []}
                      {#if mv.length === 0}
                        <p class="text-xs text-gray-600 italic">Nessun movimento.</p>
                      {:else}
                        <table class="w-full text-xs">
                          <thead>
                            <tr class="text-gray-500">
                              <th class="text-left pb-2 font-medium">Data</th>
                              <th class="text-left pb-2 font-medium">Documento</th>
                              <th class="text-right pb-2 font-medium">Qtà</th>
                              <th class="text-right pb-2 font-medium">Prezzo</th>
                            </tr>
                          </thead>
                          <tbody class="divide-y divide-gray-800/50">
                            {#each mv as m}
                              <tr>
                                <td class="py-2 pr-3 text-gray-400 whitespace-nowrap">
                                  {formatDate(m.data)} <span class="text-gray-500 text-[10px]">· {formatRelativeTime(m.data)}</span>
                                </td>
                                <td class="py-2 pr-3 text-gray-400">{tipoLabel[m.tipo_documento] ?? m.tipo_documento} {m.numero_documento}</td>
                                <td class="py-2 pr-3 text-right text-gray-300">{m.quantita}</td>
                                <td class="py-2 pr-3 text-right font-mono text-gray-300">{formatCurrency(m.prezzo_unitario)}</td>
                              </tr>
                            {/each}
                          </tbody>
                        </table>
                      {/if}
                    {/if}
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </div>
</div>
