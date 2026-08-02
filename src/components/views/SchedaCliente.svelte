<script lang="ts">
  import {
    clienti, schedaClienteId, currentView, editDocumentoId,
    formatCurrency, formatDate, setError, setSuccess,
  } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type {
    ArticoloVendutoCliente, MovimentoVenditaCliente, PrezzoCliente,
    Ricambio, TipoDocumento, DocumentoCliente, RiepilogoCliente,
  } from '../../lib/types'
  import ConfirmModal from '../ConfirmModal.svelte'

  $: clienteId = $schedaClienteId
  $: cliente = clienteId != null ? $clienti.find(c => c.id === clienteId) : null

  // Stesse etichette/colori gia' in uso in Documenti.svelte, per non
  // introdurre un secondo linguaggio visivo per lo stesso concetto.
  // Set completo (8 tipi): il tab "Movimenti"/"Articoli venduti" vede solo
  // i 4 tipi presenti in v_storico_vendite, il tab "Documenti" li vede tutti.
  const tipoLabel: Partial<Record<TipoDocumento, string>> = {
    fattura:            'Fattura',
    preventivo:         'Preventivo',
    ddt:                'DDT',
    nota_credito:       'Nota credito',
    vendita_banco:      'Vendita Banco',
    buono:              'Buono',
    fattura_differita:  'Fattura Differita',
    ddt_fornitore:      'DDT Fornitore',
  }
  const tipoBadge: Partial<Record<TipoDocumento, string>> = {
    fattura:            'badge-blue',
    preventivo:         'badge-gray',
    ddt:                'badge-green',
    nota_credito:       'badge-red',
    vendita_banco:      'badge-purple',
    buono:              'badge-gray',
    fattura_differita:  'badge-indigo',
    ddt_fornitore:      'badge-orange',
  }
  const statoBadge: Record<string, string> = {
    bozza: 'badge-gray', confermato: 'badge-blue', pagato: 'badge-green', annullato: 'badge-red',
  }

  function tornaClienti() {
    schedaClienteId.set(null)
    currentView.set('clienti')
  }

  function apriDocumento(documentoId: number) {
    editDocumentoId.set(documentoId)
    currentView.set('nuova-fattura')
  }

  // ---------------------------------------------------------------
  // Riepilogo in cima, sempre visibile
  // ---------------------------------------------------------------
  let riepilogo: RiepilogoCliente | null = null

  async function caricaRiepilogo(id: number) {
    riepilogo = null
    try {
      riepilogo = await api.storicoCliente.getRiepilogo(id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento riepilogo cliente')
    }
  }

  const annoCorrente = new Date().getFullYear()

  type Tab = 'articoli' | 'movimenti' | 'prezzi' | 'documenti'
  let tab: Tab = 'articoli'
  const tabs: { id: Tab; label: string }[] = [
    { id: 'articoli',   label: 'Articoli venduti' },
    { id: 'movimenti',  label: 'Movimenti' },
    { id: 'prezzi',     label: 'Prezzi dedicati' },
    { id: 'documenti',  label: 'Documenti' },
  ]

  // ---------------------------------------------------------------
  // Tab "Articoli venduti"
  // ---------------------------------------------------------------
  let articoli: ArticoloVendutoCliente[] = []
  let loadingArticoli = false
  let searchArticoli = ''
  // Frequenza di vendita DESC di default: chi apre la scheda spesso
  // non conosce il codice articolo, sa solo il cliente, e vuole
  // subito "cosa gli vendo di solito" senza dover cercare o riordinare.
  let sortColArt: keyof ArticoloVendutoCliente = 'numero_vendite'
  let sortDirArt: 'asc' | 'desc' = 'desc'
  let expandedArticoloId: number | null = null
  let movimentiArticoloMap: Record<number, MovimentoVenditaCliente[]> = {}
  let loadingMovArticolo: number | null = null

  async function caricaArticoli(id: number) {
    loadingArticoli = true
    try {
      articoli = await api.storicoCliente.getArticoliVenduti(id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento articoli venduti')
    } finally {
      loadingArticoli = false
    }
  }

  function toggleSortArt(col: keyof ArticoloVendutoCliente) {
    if (sortColArt === col) {
      sortDirArt = sortDirArt === 'asc' ? 'desc' : 'asc'
    } else {
      sortColArt = col
      sortDirArt = (col === 'codice_articolo' || col === 'descrizione') ? 'asc' : 'desc'
    }
  }

  $: filteredArticoli = searchArticoli.trim()
    ? articoli.filter(a => {
        const q = searchArticoli.trim().toLowerCase()
        return a.codice_articolo.toLowerCase().includes(q)
          || a.descrizione.toLowerCase().includes(q)
      })
    : articoli

  $: sortedArticoli = [...filteredArticoli].sort((a, b) => {
    const av = a[sortColArt]
    const bv = b[sortColArt]
    if (typeof av === 'number' && typeof bv === 'number') {
      return sortDirArt === 'asc' ? av - bv : bv - av
    }
    const cmp = String(av).localeCompare(String(bv), 'it')
    return sortDirArt === 'asc' ? cmp : -cmp
  })

  async function toggleArticolo(articoloId: number) {
    if (expandedArticoloId === articoloId) {
      expandedArticoloId = null
      return
    }
    expandedArticoloId = articoloId
    if (!movimentiArticoloMap[articoloId] && clienteId != null) {
      loadingMovArticolo = articoloId
      try {
        movimentiArticoloMap[articoloId] =
          await api.storicoCliente.getMovimentiArticolo(clienteId, articoloId)
        movimentiArticoloMap = { ...movimentiArticoloMap }
      } catch (e: any) {
        setError(e?.message ?? 'Errore caricamento movimenti articolo')
      } finally {
        loadingMovArticolo = null
      }
    }
  }

  // ---------------------------------------------------------------
  // Tab "Movimenti"
  // ---------------------------------------------------------------
  let movimenti: MovimentoVenditaCliente[] = []
  let loadingMovimenti = false

  async function caricaMovimenti(id: number) {
    loadingMovimenti = true
    try {
      movimenti = await api.storicoCliente.getMovimenti(id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento movimenti')
    } finally {
      loadingMovimenti = false
    }
  }

  // ---------------------------------------------------------------
  // Tab "Documenti" — elenco completo, non filtrato, non passa da
  // v_storico_vendite: tutti i tipi e tutti gli stati, inclusi
  // annullati/bozze/preventivi/note di credito.
  // ---------------------------------------------------------------
  let documentiCliente: DocumentoCliente[] = []
  let loadingDocumenti = false
  const DOCUMENTI_LIMIT = 200
  let mostraTuttiDocumenti = false

  async function caricaDocumentiCliente(id: number) {
    loadingDocumenti = true
    try {
      documentiCliente = await api.documentiCliente.getAll(id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento documenti cliente')
    } finally {
      loadingDocumenti = false
    }
  }

  $: documentiVisibili = mostraTuttiDocumenti
    ? documentiCliente
    : documentiCliente.slice(0, DOCUMENTI_LIMIT)

  // ---------------------------------------------------------------
  // Tab "Prezzi dedicati"
  // ---------------------------------------------------------------
  let prezzi: PrezzoCliente[] = []
  let loadingPrezzi = false

  async function caricaPrezzi(id: number) {
    loadingPrezzi = true
    try {
      prezzi = await api.prezziCliente.list(id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento prezzi dedicati')
    } finally {
      loadingPrezzi = false
    }
  }

  let showForm = false
  let editingPrezzoId: number | null = null
  let formRicambioId: number | null = null
  let formPrezzo = 0
  let formSconto: number | null = null
  let formNote = ''
  let formError = ''
  let formSaving = false

  interface TA { query: string; results: Ricambio[]; open: boolean }
  let ricambioTA: TA = { query: '', results: [], open: false }
  let taTimer: ReturnType<typeof setTimeout>

  function resetForm() {
    showForm = false
    editingPrezzoId = null
    formRicambioId = null
    formPrezzo = 0
    formSconto = null
    formNote = ''
    formError = ''
    ricambioTA = { query: '', results: [], open: false }
  }

  function startAdd() {
    resetForm()
    showForm = true
  }

  function startEdit(p: PrezzoCliente) {
    resetForm()
    showForm = true
    editingPrezzoId = p.id
    formRicambioId = p.ricambio_id
    formPrezzo = p.prezzo
    formSconto = p.sconto_perc
    formNote = p.note ?? ''
    ricambioTA = { query: `${p.codice_interno} — ${p.descrizione}`, results: [], open: false }
  }

  function onRicambioInput(value: string) {
    ricambioTA = { query: value, results: ricambioTA.results, open: !!value.trim() }
    formRicambioId = null
    clearTimeout(taTimer)
    if (!value.trim()) { ricambioTA = { query: '', results: [], open: false }; return }
    taTimer = setTimeout(async () => {
      try {
        const results = await api.ricambi.search(value)
        ricambioTA = { query: value, results, open: true }
      } catch { /* la ricerca fallita non blocca il form */ }
    }, 300)
  }

  function selectRicambio(r: Ricambio) {
    formRicambioId = r.id
    ricambioTA = { query: `${r.codice_interno} — ${r.descrizione}`, results: [], open: false }
    if (!formPrezzo) formPrezzo = r.prezzo_vendita
  }

  async function salvaPrezzo() {
    if (clienteId == null) return
    if (!formRicambioId) { formError = 'Seleziona un articolo'; return }
    if (formPrezzo < 0) { formError = 'Il prezzo non può essere negativo'; return }
    formError = ''
    formSaving = true
    try {
      const saved = await api.prezziCliente.upsert({
        cliente_id: clienteId,
        ricambio_id: formRicambioId,
        prezzo: formPrezzo,
        sconto_perc: formSconto,
        note: formNote.trim() || null,
      })
      prezzi = [...prezzi.filter(p => p.ricambio_id !== saved.ricambio_id), saved]
        .sort((a, b) => a.codice_interno.localeCompare(b.codice_interno, 'it'))
      setSuccess('Prezzo dedicato salvato')
      resetForm()
    } catch (e: any) {
      formError = e?.message ?? 'Errore salvataggio'
    } finally {
      formSaving = false
    }
  }

  let confirmOpen = false
  let pendingDeleteId: number | null = null
  let pendingDeleteLabel = ''

  function richiediElimina(p: PrezzoCliente) {
    pendingDeleteId = p.id
    pendingDeleteLabel = `${p.codice_interno} — ${p.descrizione}`
    confirmOpen = true
  }

  async function eseguiElimina() {
    if (pendingDeleteId === null) return
    const id = pendingDeleteId
    confirmOpen = false
    pendingDeleteId = null
    try {
      await api.prezziCliente.delete(id)
      prezzi = prezzi.filter(p => p.id !== id)
    } catch (e: any) {
      setError(e?.message ?? 'Errore eliminazione')
    }
  }

  // ---------------------------------------------------------------
  // Caricamento dati al cambio cliente
  // ---------------------------------------------------------------
  let loadedFor: number | null = null
  $: if (clienteId != null && clienteId !== loadedFor) {
    loadedFor = clienteId
    expandedArticoloId = null
    movimentiArticoloMap = {}
    mostraTuttiDocumenti = false
    resetForm()
    caricaRiepilogo(clienteId)
    caricaArticoli(clienteId)
    caricaMovimenti(clienteId)
    caricaPrezzi(clienteId)
    caricaDocumentiCliente(clienteId)
  }
</script>

{#if clienteId == null || !cliente}
  <div class="p-6 space-y-4">
    <p class="text-sm text-gray-500">Nessun cliente selezionato.</p>
    <button class="btn-secondary" on:click={tornaClienti}>← Torna ai clienti</button>
  </div>
{:else}
  <ConfirmModal
    bind:open={confirmOpen}
    title="Elimina prezzo dedicato"
    message={`Eliminare il prezzo dedicato per "${pendingDeleteLabel}"?`}
    onConfirm={eseguiElimina}
    onCancel={() => { confirmOpen = false; pendingDeleteId = null }}
  />

  <div class="p-6 space-y-5">
    <div class="flex items-center justify-between">
      <div>
        <button class="text-xs text-gray-500 hover:text-gray-300 mb-1" on:click={tornaClienti}>
          ← Torna ai clienti
        </button>
        <h1 class="text-xl font-semibold text-white">{cliente.ragione_sociale}</h1>
        {#if cliente.partita_iva || cliente.citta}
          <p class="text-xs text-gray-500">
            {#if cliente.partita_iva}P.IVA: {cliente.partita_iva}{/if}
            {#if cliente.partita_iva && cliente.citta} · {/if}
            {cliente.citta ?? ''}
          </p>
        {/if}
      </div>
    </div>

    <!-- Riepilogo discreto: sempre visibile, sopra i tab -->
    <div class="flex flex-wrap gap-x-6 gap-y-1 text-xs text-gray-500 -mt-2">
      <span>
        Ultima vendita
        <span class="text-gray-300">
          {riepilogo?.ultima_vendita ? formatDate(riepilogo.ultima_vendita) : '—'}
        </span>
      </span>
      <span>
        Documenti
        <span class="text-gray-300">{riepilogo ? riepilogo.numero_documenti : '—'}</span>
      </span>
      <span>
        Fatturato {annoCorrente}
        <span class="text-gray-300">
          {riepilogo ? formatCurrency(riepilogo.fatturato_anno_corrente) : '—'}
        </span>
      </span>
    </div>

    <!-- Tab bar -->
    <div class="flex gap-2 border-b border-gray-800 pb-0">
      {#each tabs as t}
        <button
          class="px-3 py-2 text-xs font-medium border-b-2 transition-colors
            {tab === t.id
              ? 'border-brand-500 text-white'
              : 'border-transparent text-gray-500 hover:text-gray-300'}"
          on:click={() => tab = t.id}
        >
          {t.label}
        </button>
      {/each}
    </div>

    {#if tab === 'articoli'}
      <div class="card">
        <div class="px-4 py-3 border-b border-gray-800">
          <input class="input max-w-sm" placeholder="Cerca articolo..." bind:value={searchArticoli} />
        </div>
        {#if loadingArticoli}
          <div class="flex justify-center py-10">
            <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        {:else if sortedArticoli.length === 0}
          <p class="text-sm text-gray-500 text-center py-10">
            {articoli.length === 0 ? 'Nessuna vendita registrata per questo cliente.' : 'Nessun articolo corrisponde alla ricerca.'}
          </p>
        {:else}
          <table class="w-full text-sm">
            <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
              <tr>
                {#each [
                  { col: 'codice_articolo', label: 'Codice' },
                  { col: 'descrizione',     label: 'Descrizione' },
                  { col: 'prezzo_ultimo',   label: 'Ultimo',    align: 'right' },
                  { col: 'prezzo_min',      label: 'Min',       align: 'right' },
                  { col: 'prezzo_max',      label: 'Max',       align: 'right' },
                  { col: 'prezzo_medio',    label: 'Medio',     align: 'right' },
                  { col: 'quantita_totale', label: 'Q.tà tot.', align: 'right' },
                  { col: 'numero_vendite',  label: 'Vendite',   align: 'right' },
                  { col: 'ultima_vendita',  label: 'Ultima vendita' },
                ] as h}
                  <th
                    class="px-3 py-3 {h.align === 'right' ? 'text-right' : 'text-left'} cursor-pointer select-none hover:text-gray-200 whitespace-nowrap"
                    on:click={() => toggleSortArt(h.col as keyof ArticoloVendutoCliente)}
                  >
                    {h.label}
                    {#if sortColArt === h.col}
                      <span class="ml-0.5">{sortDirArt === 'asc' ? '↑' : '↓'}</span>
                    {:else}
                      <span class="ml-0.5 opacity-25">↕</span>
                    {/if}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-800">
              {#each sortedArticoli as a (a.articolo_id)}
                <tr
                  class="table-row-hover cursor-pointer"
                  on:click={() => toggleArticolo(a.articolo_id)}
                >
                  <td class="px-3 py-3 font-mono text-brand-400 text-xs">{a.codice_articolo}</td>
                  <td class="px-3 py-3 text-gray-200">{a.descrizione}</td>
                  <td class="px-3 py-3 text-right font-mono text-green-400 font-medium">{formatCurrency(a.prezzo_ultimo)}</td>
                  <td class="px-3 py-3 text-right font-mono text-gray-400">{formatCurrency(a.prezzo_min)}</td>
                  <td class="px-3 py-3 text-right font-mono text-gray-400">{formatCurrency(a.prezzo_max)}</td>
                  <td class="px-3 py-3 text-right font-mono text-gray-400">{formatCurrency(a.prezzo_medio)}</td>
                  <td class="px-3 py-3 text-right text-gray-400">{a.quantita_totale}</td>
                  <td class="px-3 py-3 text-right text-gray-400">{a.numero_vendite}</td>
                  <td class="px-3 py-3 text-gray-400">{formatDate(a.ultima_vendita)}</td>
                </tr>
                {#if expandedArticoloId === a.articolo_id}
                  <tr>
                    <td colspan="9" class="bg-gray-900/60 px-3 py-3">
                      {#if loadingMovArticolo === a.articolo_id}
                        <div class="flex justify-center py-4">
                          <div class="w-4 h-4 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
                        </div>
                      {:else}
                        {@const mv = movimentiArticoloMap[a.articolo_id] ?? []}
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
                                <th class="text-right pb-2 font-medium">Sc. %</th>
                                <th class="text-right pb-2 font-medium">Totale</th>
                              </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-800/50">
                              {#each mv as m}
                                <tr>
                                  <td class="py-2 pr-3 text-gray-400">{formatDate(m.data)}</td>
                                  <td class="py-2 pr-3">
                                    <button
                                      class="{tipoBadge[m.tipo_documento] ?? 'badge-gray'} text-xs"
                                      on:click|stopPropagation={() => apriDocumento(m.documento_id)}
                                    >
                                      {tipoLabel[m.tipo_documento] ?? m.tipo_documento} {m.numero_documento}
                                    </button>
                                  </td>
                                  <td class="py-2 pr-3 text-right text-gray-300">{m.quantita}</td>
                                  <td class="py-2 pr-3 text-right font-mono text-gray-300">{formatCurrency(m.prezzo_unitario)}</td>
                                  <td class="py-2 pr-3 text-right text-gray-400">{m.sconto_perc}%</td>
                                  <td class="py-2 pr-3 text-right font-mono text-green-400">{formatCurrency(m.totale_riga)}</td>
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
    {:else if tab === 'movimenti'}
      <div class="card overflow-hidden">
        {#if loadingMovimenti}
          <div class="flex justify-center py-10">
            <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        {:else if movimenti.length === 0}
          <p class="text-sm text-gray-500 text-center py-10">Nessun movimento registrato per questo cliente.</p>
        {:else}
          <table class="w-full text-sm">
            <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
              <tr>
                <th class="px-4 py-3 text-left">Data</th>
                <th class="px-4 py-3 text-left">Documento</th>
                <th class="px-4 py-3 text-left">Articolo</th>
                <th class="px-4 py-3 text-left">Qtà</th>
                <th class="px-4 py-3 text-right">Prezzo</th>
                <th class="px-4 py-3 text-left">Sc. %</th>
                <th class="px-4 py-3 text-right">Totale</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-800">
              {#each movimenti as m}
                <tr class="table-row-hover">
                  <td class="px-4 py-3 text-gray-400">{formatDate(m.data)}</td>
                  <td class="px-4 py-3">
                    <button
                      class="{tipoBadge[m.tipo_documento] ?? 'badge-gray'} text-xs"
                      on:click={() => apriDocumento(m.documento_id)}
                    >
                      {tipoLabel[m.tipo_documento] ?? m.tipo_documento} {m.numero_documento}
                    </button>
                  </td>
                  <td class="px-4 py-3 text-gray-300">
                    <span class="font-mono text-brand-400 text-xs">{m.codice_articolo}</span>
                    <span class="text-gray-400 text-xs"> — {m.descrizione}</span>
                  </td>
                  <td class="px-4 py-3 text-gray-300">{m.quantita}</td>
                  <td class="px-4 py-3 text-right font-mono text-gray-300">{formatCurrency(m.prezzo_unitario)}</td>
                  <td class="px-4 py-3 text-gray-400">{m.sconto_perc}%</td>
                  <td class="px-4 py-3 text-right font-mono text-green-400 font-medium">{formatCurrency(m.totale_riga)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {:else if tab === 'documenti'}
      <div class="space-y-2">
        <div class="card overflow-hidden">
          {#if loadingDocumenti}
            <div class="flex justify-center py-10">
              <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            </div>
          {:else if documentiCliente.length === 0}
            <p class="text-sm text-gray-500 text-center py-10">Nessun documento per questo cliente.</p>
          {:else}
            <table class="w-full text-sm">
              <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
                <tr>
                  <th class="px-4 py-3 text-left">Data</th>
                  <th class="px-4 py-3 text-left">Tipo</th>
                  <th class="px-4 py-3 text-left">Numero</th>
                  <th class="px-4 py-3 text-right">Totale</th>
                  <th class="px-4 py-3 text-left">Stato</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each documentiVisibili as d (d.documento_id)}
                  <tr
                    class="table-row-hover cursor-pointer"
                    on:click={() => apriDocumento(d.documento_id)}
                  >
                    <td class="px-4 py-3 text-gray-400">{formatDate(d.data)}</td>
                    <td class="px-4 py-3">
                      <span class="{tipoBadge[d.tipo_documento] ?? 'badge-gray'} text-xs">
                        {tipoLabel[d.tipo_documento] ?? d.tipo_documento}
                      </span>
                    </td>
                    <td class="px-4 py-3 font-mono text-brand-400">{d.numero}</td>
                    <td class="px-4 py-3 text-right font-mono font-medium text-green-400">{formatCurrency(d.totale)}</td>
                    <td class="px-4 py-3">
                      <span class={statoBadge[d.stato] ?? 'badge-gray'}>{d.stato}</span>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>
        {#if !mostraTuttiDocumenti && documentiCliente.length > DOCUMENTI_LIMIT}
          <button
            class="text-xs text-gray-500 hover:text-gray-300 underline"
            on:click={() => mostraTuttiDocumenti = true}
          >
            Mostra tutti i {documentiCliente.length} documenti (visualizzati gli ultimi {DOCUMENTI_LIMIT})
          </button>
        {/if}
      </div>
    {:else}
      <div class="space-y-4">
        {#if !showForm}
          <button class="btn-primary text-xs" on:click={startAdd}>+ Aggiungi prezzo dedicato</button>
        {:else}
          <div class="card p-4 space-y-3">
            <h2 class="text-sm font-semibold text-white">
              {editingPrezzoId ? 'Modifica prezzo dedicato' : 'Nuovo prezzo dedicato'}
            </h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div class="col-span-2 relative">
                <label class="label">Articolo *</label>
                <input
                  class="input text-xs"
                  placeholder="Cerca codice o descrizione..."
                  value={ricambioTA.query}
                  disabled={editingPrezzoId !== null}
                  on:input={e => onRicambioInput((e.target as HTMLInputElement).value)}
                  on:keydown={e => { if (e.key === 'Escape') ricambioTA = { ...ricambioTA, open: false } }}
                  on:blur={() => setTimeout(() => ricambioTA = { ...ricambioTA, open: false }, 150)}
                />
                {#if ricambioTA.open && ricambioTA.results.length > 0}
                  <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-60 overflow-y-auto">
                    {#each ricambioTA.results as r (r.id)}
                      <button
                        class="w-full text-left px-3 py-2 hover:bg-gray-700 flex flex-col gap-0.5"
                        on:mousedown|preventDefault={() => selectRicambio(r)}
                      >
                        <span class="font-mono text-brand-400 text-xs">{r.codice_interno}</span>
                        <span class="text-gray-300 text-xs truncate">{r.descrizione}</span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
              <div>
                <label class="label">Prezzo € *</label>
                <input class="input text-xs" type="number" min="0" step="0.01" bind:value={formPrezzo} />
              </div>
              <div>
                <label class="label">Sconto %</label>
                <input class="input text-xs" type="number" min="0" max="100" step="0.01" bind:value={formSconto} />
              </div>
              <div class="col-span-2 md:col-span-4">
                <label class="label">Note</label>
                <input class="input text-xs" bind:value={formNote} placeholder="es. accordo commerciale del..." />
              </div>
            </div>
            {#if formError}<p class="text-xs text-red-400">{formError}</p>{/if}
            <div class="flex gap-2">
              <button class="btn-primary text-xs" on:click={salvaPrezzo} disabled={formSaving}>
                {formSaving ? 'Salvataggio...' : 'Salva'}
              </button>
              <button class="btn-secondary text-xs" on:click={resetForm}>Annulla</button>
            </div>
          </div>
        {/if}

        <div class="card overflow-hidden">
          {#if loadingPrezzi}
            <div class="flex justify-center py-10">
              <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            </div>
          {:else if prezzi.length === 0}
            <p class="text-sm text-gray-500 text-center py-10">Nessun prezzo dedicato per questo cliente.</p>
          {:else}
            <table class="w-full text-sm">
              <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
                <tr>
                  <th class="px-4 py-3 text-left">Codice</th>
                  <th class="px-4 py-3 text-left">Descrizione</th>
                  <th class="px-4 py-3 text-left">Prezzo</th>
                  <th class="px-4 py-3 text-left">Sconto</th>
                  <th class="px-4 py-3 text-left">Note</th>
                  <th class="px-4 py-3 text-left">Aggiornato</th>
                  <th class="px-4 py-3"></th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-800">
                {#each prezzi as p (p.id)}
                  <tr class="table-row-hover">
                    <td class="px-4 py-3 font-mono text-brand-400 text-xs">{p.codice_interno}</td>
                    <td class="px-4 py-3 text-gray-200">{p.descrizione}</td>
                    <td class="px-4 py-3 font-mono text-green-400 font-medium">{formatCurrency(p.prezzo)}</td>
                    <td class="px-4 py-3 text-gray-400">{p.sconto_perc != null ? `${p.sconto_perc}%` : '—'}</td>
                    <td class="px-4 py-3 text-gray-500 text-xs">{p.note ?? '—'}</td>
                    <td class="px-4 py-3 text-gray-500 text-xs">{p.aggiornato_il}</td>
                    <td class="px-4 py-3">
                      <div class="flex gap-1 justify-end">
                        <button class="btn-secondary text-xs px-2 py-1" on:click={() => startEdit(p)}>Modifica</button>
                        <button class="btn-danger text-xs px-2 py-1" on:click={() => richiediElimina(p)}>Elimina</button>
                      </div>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}
