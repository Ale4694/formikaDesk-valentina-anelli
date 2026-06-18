<script lang="ts">
  import { createEventDispatcher, onMount, tick } from 'svelte'
  import { documenti, clienti, formatCurrency, formatDate, setError, setSuccess, currentView, printData, editDocumentoId } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { Documento } from '../../lib/types'
  import PagamentoModal from '../PagamentoModal.svelte'
  import ConfirmModal from '../ConfirmModal.svelte'

  const dispatch = createEventDispatcher()

  let showPagamentoModal = false
  let pagamentoDocId: number | null = null
  let pagamentoDocNumero = ''

  // Modifica ed elimina
  const TIPI_SCARICO = ['fattura', 'ddt', 'vendita_banco', 'buono', 'fattura_differita', 'ddt_fornitore']
  let confirmDeleteOpen = false
  let docToDelete: Documento | null = null

  $: confirmDeleteTitle = docToDelete?.stato === 'confermato'
    ? 'Elimina documento confermato'
    : 'Elimina documento'

  $: confirmDeleteMessage = docToDelete
    ? docToDelete.stato === 'confermato' && TIPI_SCARICO.includes(docToDelete.tipo_documento)
      ? `Questo documento è confermato e ha movimentato il magazzino — eliminandolo la giacenza degli articoli sarà ripristinata. Continuare?`
      : docToDelete.stato === 'confermato'
        ? `Questo documento è confermato. Sei sicuro di voler procedere con l'eliminazione?`
        : `Sei sicuro di voler eliminare il documento ${docToDelete.numero}?`
    : ''

  function modifica(doc: Documento) {
    editDocumentoId.set(doc.id)
    currentView.set('nuova-fattura')
  }

  function apriElimina(doc: Documento) {
    docToDelete = doc
    confirmDeleteOpen = true
  }

  async function confermaElimina() {
    if (!docToDelete) return
    confirmDeleteOpen = false
    const id = docToDelete.id
    docToDelete = null
    try {
      await api.documenti.delete(id)
      documenti.update(list => list.filter(d => d.id !== id))
      await loadPage()
    } catch (e: any) {
      setError(e?.message ?? 'Errore eliminazione documento')
    }
  }

  const statoBadge: Record<string, string> = {
    bozza: 'badge-gray', confermato: 'badge-blue', pagato: 'badge-green', annullato: 'badge-red'
  }
  const tipoLabel: Record<string, string> = {
    fattura:           'Fattura',
    preventivo:        'Preventivo',
    ddt:               'DDT',
    nota_credito:      'Nota credito',
    vendita_banco:     'Vendita Banco',
    buono:             'Buono',
    fattura_differita: 'Fattura Differita',
    ddt_fornitore:     'DDT Fornitore',
  }
  const tipoBadge: Record<string, string> = {
    fattura:           'badge-blue',
    preventivo:        'badge-gray',
    ddt:               'badge-green',
    nota_credito:      'badge-red',
    vendita_banco:     'badge-purple',
    buono:             'badge-gray',
    fattura_differita: 'badge-indigo',
    ddt_fornitore:     'badge-orange',
  }

  // Filtro stato
  const statiFilter = ['tutti', 'bozza', 'confermato', 'pagato', 'annullato'] as const
  let statoAttivo: typeof statiFilter[number] = 'tutti'

  // Filtro tipo
  const tipiFilter = ['tutti', 'fattura', 'ddt', 'preventivo', 'nota_credito', 'vendita_banco', 'buono', 'fattura_differita', 'ddt_fornitore'] as const
  let tipoAttivo: typeof tipiFilter[number] = 'tutti'

  // Ricerca testuale
  let search = ''
  let searchTimer: ReturnType<typeof setTimeout>

  function onSearchInput(e: Event) {
    search = (e.target as HTMLInputElement).value
    clearTimeout(searchTimer)
    searchTimer = setTimeout(() => { page = 0; loadPage() }, 350)
  }

  // Paginazione server-side
  const PAGE_SIZE = 50
  let page = 0
  let pageTotal = 0
  let pageItems: Documento[] = []
  let pageLoading = false
  let mostraStorici = false

  $: documentiVisibili = mostraStorici
    ? pageItems
    : pageItems.filter(d => d.data >= '2026-01-01')

  // Ordinamento
  type SortDir = 'asc' | 'desc'
  let sortCol = 'data'
  let sortDir: SortDir = 'desc'

  async function loadPage() {
    pageLoading = true
    try {
      const res = await api.documenti.getPaginated(page, PAGE_SIZE, statoAttivo, tipoAttivo, search.trim(), sortCol, sortDir)
      pageItems = res.items
      pageTotal = res.total
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento documenti')
    } finally {
      pageLoading = false
    }
  }

  onMount(() => loadPage())

  function toggleSort(col: string) {
    if (sortCol === col) { sortDir = sortDir === 'asc' ? 'desc' : 'asc' }
    else { sortCol = col; sortDir = 'asc' }
    page = 0
    loadPage()
  }

  function onFilterChange() {
    page = 0
    clearTimeout(searchTimer)
    loadPage()
  }

  function nomeCliente(id: number | null) {
    if (!id) return '—'
    return $clienti.find(c => c.id === id)?.ragione_sociale ?? '—'
  }

  async function cambiaStato(id: number, stato: string) {
    try {
      const updated = await api.documenti.updateStato(id, stato)
      documenti.update(list => list.map(d => d.id === id ? updated : d))
      pageItems = pageItems.map(d => d.id === id ? updated : d)
    } catch (e: any) { setError(e?.message ?? 'Errore') }
  }

  function apriPagamento(doc: Documento) {
    pagamentoDocId = doc.id
    pagamentoDocNumero = doc.numero
    showPagamentoModal = true
  }

  async function confermaPagamento(e: CustomEvent<{
    data_pagamento: string
    metodo_pagamento: string
    riferimento_pagamento: string | null
    note_pagamento: string | null
  }>) {
    if (pagamentoDocId === null) return
    showPagamentoModal = false
    try {
      const updated = await api.documenti.updateStato(pagamentoDocId, 'pagato', e.detail)
      documenti.update(list => list.map(d => d.id === updated.id ? updated : d))
      pageItems = pageItems.map(d => d.id === updated.id ? updated : d)
    } catch (err: any) { setError(err?.message ?? 'Errore registrazione pagamento') }
    pagamentoDocId = null
  }

  async function stampa(id: number) {
    try {
      const doc = await api.documenti.get(id)
      printData.set(doc)
      await tick()
      window.print()
    } catch (e: any) {
      setError(e?.message ?? 'Errore stampa')
    }
  }

  async function apriPdfAllegato(id: number) {
    try {
      await api.ddtFornitore.apriPdf(id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore apertura PDF')
    }
  }

  async function generaXmlFatturaPa(id: number) {
    try {
      const filePath = await api.fatturaPa.genera(id)
      if (filePath) {
        setSuccess(`XML salvato in: ${filePath}`)
      }
    } catch (e: any) {
      const msg = e?.message || (typeof e === 'string' ? e : JSON.stringify(e)) || 'Errore generazione XML FatturaPA'
      setError(msg)
    }
  }
</script>

<PagamentoModal
  show={showPagamentoModal}
  numeroDocumento={pagamentoDocNumero}
  on:conferma={confermaPagamento}
  on:annulla={() => { showPagamentoModal = false; pagamentoDocId = null }}
/>

<ConfirmModal
  open={confirmDeleteOpen}
  title={confirmDeleteTitle}
  message={confirmDeleteMessage}
  confirmLabel="Elimina"
  confirmClass="btn-danger"
  onConfirm={confermaElimina}
  onCancel={() => { confirmDeleteOpen = false; docToDelete = null }}
/>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Documenti</h1>
    <button class="btn-primary" on:click={() => currentView.set('nuova-fattura')}>+ Nuovo documento</button>
  </div>

  <!-- Ricerca testuale -->
  <input
    class="input max-w-sm"
    placeholder="Cerca per numero o cliente..."
    value={search}
    on:input={onSearchInput}
  />

  <!-- Filtro stato -->
  <div class="flex gap-1.5 flex-wrap">
    {#each statiFilter as s}
      <button
        class="px-3 py-1 rounded-full text-xs font-medium transition-colors duration-100
          {statoAttivo === s
            ? 'bg-brand-600 text-white'
            : 'bg-gray-800 text-gray-400 hover:text-gray-200 hover:bg-gray-700'}"
        on:click={() => { statoAttivo = s; onFilterChange() }}
      >
        {s === 'tutti' ? 'Tutti' : s.charAt(0).toUpperCase() + s.slice(1)}
      </button>
    {/each}
  </div>

  <!-- Filtro tipo documento -->
  <div class="flex gap-1.5 flex-wrap">
    {#each tipiFilter as t}
      <button
        class="px-3 py-1 rounded-full text-xs font-medium transition-colors duration-100
          {tipoAttivo === t
            ? 'bg-gray-600 text-white'
            : 'bg-gray-800/60 text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
        on:click={() => { tipoAttivo = t; onFilterChange() }}
      >
        {t === 'tutti' ? 'Tutti i tipi' : (tipoLabel[t] ?? t)}
      </button>
    {/each}
  </div>

  <button
    class="text-xs text-gray-500 underline mt-1"
    on:click={() => mostraStorici = !mostraStorici}
  >
    {mostraStorici ? 'Nascondi documenti precedenti al 2026' : 'Mostra documenti precedenti al 2026'}
  </button>

  <div class="card overflow-hidden">
    <table class="w-full text-sm">
      <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
        <tr>
          {#each [
            { col: 'tipo_documento',   label: 'Tipo',    align: 'left'  },
            { col: 'numero',           label: 'Numero',  align: 'left'  },
            { col: 'data',             label: 'Data',    align: 'left'  },
            { col: 'cliente_id',       label: 'Cliente', align: 'left'  },
            { col: 'totale_documento', label: 'Totale',  align: 'right' },
            { col: 'stato',            label: 'Stato',   align: 'left'  },
          ] as h}
            <th
              class="px-4 py-3 text-{h.align} cursor-pointer select-none hover:text-gray-200 whitespace-nowrap"
              on:click={() => toggleSort(h.col)}
            >
              {h.label}
              {#if sortCol === h.col}
                <span class="ml-0.5">{sortDir === 'asc' ? '↑' : '↓'}</span>
              {:else}
                <span class="ml-0.5 opacity-25">↕</span>
              {/if}
            </th>
          {/each}
          <th class="px-4 py-3"></th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-800">
        {#each documentiVisibili as d}
          <tr class="table-row-hover">
            <td class="px-4 py-3">
              <span class="{tipoBadge[d.tipo_documento] ?? 'badge-gray'} text-xs">
                {tipoLabel[d.tipo_documento] ?? d.tipo_documento}
              </span>
            </td>
            <td class="px-4 py-3 font-mono text-brand-400">{d.numero}</td>
            <td class="px-4 py-3 text-gray-400">{formatDate(d.data)}</td>
            <td class="px-4 py-3 text-gray-200">{nomeCliente(d.cliente_id)}</td>
            <td class="px-4 py-3 text-right font-medium text-green-400">{formatCurrency(d.totale_documento)}</td>
            <td class="px-4 py-3"><span class={statoBadge[d.stato]}>{d.stato}</span></td>
            <td class="px-4 py-3">
              <div class="flex items-center gap-1.5">
                {#if d.stato === 'bozza'}
                  <button class="btn-secondary text-xs px-2 py-1" on:click={() => cambiaStato(d.id, 'confermato')}>Conferma</button>
                {/if}
                {#if d.stato !== 'pagato' && d.stato !== 'annullato'}
                  <button class="btn-secondary text-xs px-2 py-1 text-green-400" on:click={() => apriPagamento(d)}>Segna pagato</button>
                {/if}
                <button
                  class="btn-secondary text-xs px-2 py-1 flex items-center gap-1"
                  title="Stampa / Esporta PDF"
                  on:click={() => stampa(d.id)}
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"/>
                  </svg>
                  PDF
                </button>
                {#if d.tipo_documento === 'fattura' || d.tipo_documento === 'nota_credito' || d.tipo_documento === 'fattura_differita'}
                  <button
                    class="btn-secondary text-xs px-2 py-1 flex items-center gap-1 text-yellow-400 hover:text-yellow-300"
                    title="Genera XML FatturaPA"
                    on:click={() => generaXmlFatturaPa(d.id)}
                  >
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>
                    </svg>
                    XML FatturaPA
                  </button>
                {/if}
                {#if d.tipo_documento === 'ddt_fornitore' && d.pdf_allegato}
                  <button
                    class="btn-secondary text-xs px-2 py-1 flex items-center gap-1 text-orange-400 hover:text-orange-300"
                    title="Visualizza PDF originale"
                    on:click={() => apriPdfAllegato(d.id)}
                  >
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                    </svg>
                    PDF orig.
                  </button>
                {/if}
                <button
                  class="btn-secondary text-xs px-2 py-1 flex items-center gap-1"
                  title="Modifica documento"
                  on:click={() => modifica(d)}
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                  </svg>
                  Modifica
                </button>
                <button
                  class="btn-secondary text-xs px-2 py-1 flex items-center gap-1 text-red-400 hover:text-red-300"
                  title="Elimina documento"
                  on:click={() => apriElimina(d)}
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                  </svg>
                  Elimina
                </button>
              </div>
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="7" class="py-16 text-center">
              <div class="flex flex-col items-center gap-3 text-gray-600">
                {#if pageLoading}
                  <div class="w-7 h-7 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
                {:else}
                  <svg class="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                  </svg>
                  <p class="text-sm">
                    {#if statoAttivo !== 'tutti' || tipoAttivo !== 'tutti'}
                      Nessun documento con i filtri selezionati
                    {:else}
                      Nessun documento — crea la prima fattura
                    {/if}
                  </p>
                  {#if statoAttivo === 'tutti' && tipoAttivo === 'tutti'}
                    <button class="btn-primary text-xs mt-1" on:click={() => currentView.set('nuova-fattura')}>
                      + Nuovo documento
                    </button>
                  {/if}
                {/if}
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if pageTotal > PAGE_SIZE}
      <div class="flex items-center justify-between px-4 py-2.5 border-t border-gray-800 text-xs text-gray-500">
        <button
          class="btn-secondary text-xs px-3 py-1 disabled:opacity-40"
          disabled={page === 0}
          on:click={() => { page--; loadPage() }}
        >← Prec.</button>
        <span>{page * PAGE_SIZE + 1}–{Math.min((page + 1) * PAGE_SIZE, pageTotal)} di {pageTotal}</span>
        <button
          class="btn-secondary text-xs px-3 py-1 disabled:opacity-40"
          disabled={(page + 1) * PAGE_SIZE >= pageTotal}
          on:click={() => { page++; loadPage() }}
        >Succ. →</button>
      </div>
    {/if}
  </div>
</div>
