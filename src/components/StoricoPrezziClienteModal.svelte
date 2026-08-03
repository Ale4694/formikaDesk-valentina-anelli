<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { formatCurrency, formatDate, formatRelativeTime, setError } from '../lib/stores'
  import { api } from '../lib/api'
  import type { ArticoloVendutoCliente, MovimentoVenditaCliente, TipoDocumento, UltimoDocumentoVendita, RigaDocumento } from '../lib/types'

  export let clienteId: number
  export let clienteNome: string = ''
  export let articoloCorrente: { ricambioId: number; codiceInterno: string; descrizione: string } | null = null
  // Attiva il pulsante "Ricarica ultimo documento" solo su documento vuoto
  // (nessuna riga con ricambio_id valorizzato): e' l'unica azione che scrive
  // piu' righe insieme, su un documento gia' avviato rischierebbe duplicati.
  export let documentoVuoto: boolean = false

  const dispatch = createEventDispatcher<{
    close: void
    select: {
      ricambioId: number
      codiceArticolo: string
      descrizione: string
      prezzoUnitario: number
      scontoPercentuale: number | null
    }
    ricarica: { righe: RigaDocumento[] }
  }>()

  let query = ''
  let loading = true
  let articoli: ArticoloVendutoCliente[] = []
  let storicoArticolo: MovimentoVenditaCliente[] = []
  let loadingStorico = false
  let highlightIndex = 0
  let rowEls: (HTMLElement | null)[] = []
  let searchInputEl: HTMLInputElement
  let ultimoDocumento: UltimoDocumentoVendita | null = null
  let loadingRicarica = false

  const tipoLabel: Partial<Record<TipoDocumento, string>> = {
    fattura: 'Fattura', ddt: 'DDT', nota_credito: 'Nota credito',
    vendita_banco: 'Vendita Banco', buono: 'Buono', preventivo: 'Preventivo',
    fattura_differita: 'Fattura Differita', ddt_fornitore: 'DDT Fornitore',
  }

  onMount(() => {
    // Il focus resta sul campo che ha aperto il pannello (F2 non lo
    // sposta da solo): lo spostiamo qui esplicitamente, l'attributo
    // autofocus non è affidabile nel webview su un nodo montato a runtime.
    searchInputEl?.focus()
    caricaLista()
    caricaUltimoDocumento()
    if (articoloCorrente) caricaStorico(articoloCorrente.ricambioId)
  })

  async function caricaUltimoDocumento() {
    try {
      ultimoDocumento = await api.storicoCliente.getUltimoDocumento(clienteId)
    } catch {
      ultimoDocumento = null
    }
  }

  async function ricaricaUltimoDocumento() {
    if (!ultimoDocumento || !documentoVuoto || loadingRicarica) return
    loadingRicarica = true
    try {
      const doc = await api.documenti.get(ultimoDocumento.documento_id)
      dispatch('ricarica', { righe: doc.righe })
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento ultimo documento')
    } finally {
      loadingRicarica = false
    }
  }

  async function caricaLista() {
    loading = true
    try {
      articoli = await api.storicoCliente.getArticoliVenduti(clienteId)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento storico prezzi')
    } finally {
      loading = false
    }
  }

  async function caricaStorico(ricambioId: number) {
    loadingStorico = true
    try {
      storicoArticolo = await api.storicoCliente.getMovimentiArticolo(clienteId, ricambioId)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento storico articolo')
    } finally {
      loadingStorico = false
    }
  }

  // Vendite più frequenti in cima: chi apre il pannello sa il cliente,
  // non il codice, e vuole subito "cosa gli vendo di solito".
  $: filtered = (query.trim()
    ? articoli.filter(a => {
        const q = query.trim().toLowerCase()
        return a.codice_articolo.toLowerCase().includes(q) || a.descrizione.toLowerCase().includes(q)
      })
    : articoli
  ).slice().sort((a, b) => b.numero_vendite - a.numero_vendite)

  $: if (rowEls[highlightIndex]) rowEls[highlightIndex]?.scrollIntoView({ block: 'nearest' })

  function selezionaArticolo(a: ArticoloVendutoCliente) {
    dispatch('select', {
      ricambioId: a.articolo_id,
      codiceArticolo: a.codice_articolo,
      descrizione: a.descrizione,
      prezzoUnitario: a.prezzo_ultimo,
      scontoPercentuale: null,
    })
  }

  function selezionaMovimento(m: MovimentoVenditaCliente) {
    dispatch('select', {
      ricambioId: m.articolo_id,
      codiceArticolo: m.codice_articolo,
      descrizione: m.descrizione,
      prezzoUnitario: m.prezzo_unitario,
      scontoPercentuale: m.sconto_perc,
    })
  }

  function close() {
    dispatch('close')
  }

  function onQueryInput() {
    highlightIndex = 0
  }

  // Su <svelte:window>, non sul campo di ricerca: il focus può restare
  // sul campo che ha aperto il pannello (F2 non lo sposta da solo), quindi
  // un listener solo sull'input o sulla card non riceverebbe mai l'evento.
  // Il componente esiste solo mentre il pannello è aperto, quindi questo
  // listener sparisce da solo alla chiusura e non intercetta gli Escape
  // gestiti dai typeahead di NuovaFattura quando il pannello è chiuso.
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
      const a = filtered[highlightIndex]
      if (a) selezionaArticolo(a)
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
        Storico prezzi{clienteNome ? ` — ${clienteNome}` : ''}
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
        placeholder="Cerca articolo per codice o descrizione…"
      />
      {#if !loading}
        <span class="text-xs text-gray-500 shrink-0">
          {query.trim() ? `${filtered.length} di ${articoli.length}` : `${articoli.length} articol${articoli.length === 1 ? 'o' : 'i'}`}
        </span>
      {/if}
      <kbd class="text-xs text-gray-600 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 shrink-0">Esc</kbd>
    </div>

    <!-- Ricarica ultimo documento -->
    {#if ultimoDocumento}
      <div class="px-4 py-2.5 border-b border-gray-800 shrink-0">
        <button
          class="btn text-xs w-full justify-center disabled:opacity-60 disabled:cursor-not-allowed
            {documentoVuoto
              ? 'bg-green-950/20 hover:bg-green-900/30 text-green-400 border border-green-600/60 hover:border-green-500'
              : 'bg-gray-800 hover:bg-gray-700 text-gray-200 border border-gray-700'}"
          disabled={!documentoVuoto || loadingRicarica}
          on:click={ricaricaUltimoDocumento}
        >
          {loadingRicarica
            ? 'Caricamento…'
            : `↻ Ricarica ${tipoLabel[ultimoDocumento.tipo_documento] ?? ultimoDocumento.tipo_documento} ${ultimoDocumento.numero} del ${formatDate(ultimoDocumento.data)}`}
        </button>
        {#if !documentoVuoto}
          <!-- Testo sempre visibile, non un title: su un bottone disabled
               i browser Chromium/WebKit non disparano hover/title, quindi
               un tooltip da solo non avrebbe mai comunicato il motivo. -->
          <p class="text-[10px] text-gray-500 mt-1 text-center">Disponibile solo su un documento vuoto</p>
        {/if}
      </div>
    {/if}

    <div class="overflow-y-auto min-h-0">
      <!-- Articolo corrente -->
      {#if articoloCorrente}
        <div class="px-4 py-3 border-b border-gray-800 bg-gray-900/40">
          <h3 class="text-xs font-semibold text-gray-300 mb-2">
            Articolo corrente: <span class="font-mono text-brand-400">{articoloCorrente.codiceInterno}</span> — {articoloCorrente.descrizione}
          </h3>
          {#if loadingStorico}
            <div class="flex justify-center py-4">
              <div class="w-4 h-4 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            </div>
          {:else if storicoArticolo.length === 0}
            <p class="text-xs text-gray-600 italic">Nessuna vendita precedente di questo articolo a questo cliente.</p>
          {:else}
            <table class="w-full text-xs">
              <thead>
                <tr class="text-gray-500">
                  <th class="text-left pb-2 font-medium">Data</th>
                  <th class="text-left pb-2 font-medium">Documento</th>
                  <th class="text-right pb-2 font-medium">Qtà</th>
                  <th class="text-right pb-2 font-medium">Prezzo</th>
                  <th class="text-right pb-2 font-medium">Sc. %</th>
                  <th class="text-right pb-2 font-medium">Totale</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800/50">
                {#each storicoArticolo as m}
                  <tr class="table-row-hover cursor-pointer" on:click={() => selezionaMovimento(m)}>
                    <td class="py-2 pr-3 text-gray-400 whitespace-nowrap">
                      {formatDate(m.data)} <span class="text-gray-500 text-[10px]">· {formatRelativeTime(m.data)}</span>
                    </td>
                    <td class="py-2 pr-3 text-gray-400">{tipoLabel[m.tipo_documento] ?? m.tipo_documento} {m.numero_documento}</td>
                    <td class="py-2 pr-3 text-right text-gray-300">{m.quantita}</td>
                    <td class="py-2 pr-3 text-right font-mono text-gray-300">{formatCurrency(m.prezzo_unitario)}</td>
                    <td class="py-2 pr-3 text-right text-gray-400">{m.sconto_perc}%</td>
                    <td class="py-2 pr-3 text-right font-mono text-green-400">{formatCurrency(m.totale_riga)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>
      {/if}

      <!-- Lista completa -->
      {#if loading}
        <div class="flex justify-center py-10">
          <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if filtered.length === 0}
        <p class="text-sm text-gray-500 text-center py-10">
          {articoli.length === 0 ? 'Nessuna vendita registrata per questo cliente.' : 'Nessun articolo corrisponde alla ricerca.'}
        </p>
      {:else}
        <table class="w-full text-sm table-fixed">
          <colgroup>
            <col class="w-[12%]" />
            <col class="w-[42%]" />
            <col class="w-[13%]" />
            <col class="w-[10%]" />
            <col class="w-[23%]" />
          </colgroup>
          <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide sticky top-0">
            <tr>
              <th class="px-3 py-2.5 text-left">Codice</th>
              <th class="px-3 py-2.5 text-left">Descrizione</th>
              <th class="px-3 py-2.5 text-right">Ultimo</th>
              <th class="px-3 py-2.5 text-right">Vendite</th>
              <th class="px-3 py-2.5 text-left">Ultima vendita</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-800">
            {#each filtered as a, i (a.articolo_id)}
              <tr
                bind:this={rowEls[i]}
                class="table-row-hover cursor-pointer {i === highlightIndex ? 'bg-brand-600/20 ring-1 ring-inset ring-brand-500/60' : ''}"
                on:click={() => selezionaArticolo(a)}
                on:mouseenter={() => highlightIndex = i}
              >
                <td class="px-3 py-3.5 font-mono text-brand-400 text-xs">{a.codice_articolo}</td>
                <td class="px-3 py-3.5 text-gray-200 truncate" title={a.descrizione}>{a.descrizione}</td>
                <td class="px-3 py-3.5 text-right font-mono text-green-400 font-bold text-base">{formatCurrency(a.prezzo_ultimo)}</td>
                <td class="px-3 py-3.5 text-right text-gray-400 text-xs">{a.numero_vendite}</td>
                <td class="px-3 py-3.5 text-gray-400 text-xs">
                  <div>{formatDate(a.ultima_vendita)}</div>
                  <div class="text-gray-500 text-[10px]">{formatRelativeTime(a.ultima_vendita)}</div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </div>
</div>
