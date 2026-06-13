<script lang="ts">
  import { fornitori, ricambi, setError, setSuccess, currentView } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { Fornitore, RigaDdtParsed, RigaDdtImport } from '../../lib/types'

  type RigaUI = RigaDdtParsed & {
    includi: boolean
    ricambio_id_edit: number | null
    carica_magazzino: boolean
  }

  let fase: 'selezione' | 'anteprima' | 'completato' = 'selezione'
  let loading = false
  let loadingCreaFornitore = false

  // Fase selezione
  let pdfPath = ''
  let pdfNome = ''

  // Typeahead fornitore
  let fornitoreId = 0
  let fornitoreTAQuery = ''
  let fornitoreTAOpen = false

  // Banner P.IVA non in anagrafica
  let fornitoreRilevatoPiva: string | null = null
  let showBannerCreaFornitore = false
  let ragioneSocialeNuova = ''   // fallback quando typeahead è vuoto

  // Fase anteprima
  let righe: RigaUI[] = []
  let numeroDdt = ''
  let dataDdt = new Date().toISOString().split('T')[0]

  // Fase completata
  let documentoCreato: number | null = null
  let nCaricati = 0

  $: righeInclude = righe.filter(r => r.includi)
  $: righeMatchate = righeInclude.filter(r => r.ricambio_id_edit !== null)
  $: righeDaCaricare = righeInclude.filter(r => r.carica_magazzino && r.ricambio_id_edit !== null)
  $: puoImportare = fornitoreId > 0 && numeroDdt.trim().length > 0 && righeInclude.length > 0

  // ── Typeahead fornitore ──────────────────────────────────────────────────────

  function cercaFornitore(query: string): Fornitore[] {
    if (!query.trim()) return []
    const q = query.toLowerCase()
    return $fornitori.filter(f =>
      f.ragione_sociale.toLowerCase().includes(q) ||
      (f.partita_iva ?? '').includes(q)
    ).slice(0, 8)
  }

  function onFornitoreInput() {
    fornitoreId = 0
    fornitoreTAOpen = fornitoreTAQuery.trim().length > 0
  }

  function selectFornitore(f: Fornitore) {
    fornitoreId = f.id
    fornitoreTAQuery = f.ragione_sociale
    fornitoreTAOpen = false
  }

  function clearFornitore() {
    fornitoreId = 0
    fornitoreTAQuery = ''
    fornitoreTAOpen = false
  }

  // ── Azioni principali ────────────────────────────────────────────────────────

  async function selezionaPdf() {
    try {
      const selected = await api.ddtFornitore.selezionaPdf()
      if (selected) {
        pdfPath = selected
        pdfNome = selected.split(/[\\/]/).pop() ?? selected
      }
    } catch (e: any) {
      setError(e?.message ?? 'Errore selezione file')
    }
  }

  async function analizzaPdf() {
    if (!pdfPath) { setError('Seleziona un file PDF'); return }

    loading = true
    fornitoreId = 0
    fornitoreTAQuery = ''
    fornitoreTAOpen = false
    fornitoreRilevatoPiva = null
    showBannerCreaFornitore = false
    ragioneSocialeNuova = ''

    try {
      const result = await api.ddtFornitore.parse(pdfPath)

      if (result.fornitore_id_match) {
        fornitoreId = result.fornitore_id_match
        const f = $fornitori.find(f => f.id === result.fornitore_id_match)
        fornitoreTAQuery = f?.ragione_sociale ?? ''
        showBannerCreaFornitore = false
      } else if (result.fornitore_rilevato_piva) {
        fornitoreRilevatoPiva = result.fornitore_rilevato_piva
        showBannerCreaFornitore = true
      }

      if (result.righe.length === 0) {
        setError('Nessuna riga articolo trovata nel PDF. Verifica che il formato corrisponda (es. "LPR F2081P  descrizione  PZ  2,00")')
        return
      }

      righe = result.righe.map(r => ({
        ...r,
        includi: true,
        ricambio_id_edit: r.ricambio_id,
        carica_magazzino: r.ricambio_id !== null,
      }))
      searchQueries = Array(righe.length).fill('')
      searchOpen = Array(righe.length).fill(false)
      fase = 'anteprima'
    } catch (e: any) {
      setError(e?.message ?? 'Errore analisi PDF')
    } finally {
      loading = false
    }
  }

  async function creaFornitoreRapido() {
    const nomeFornitore = fornitoreTAQuery.trim() || ragioneSocialeNuova.trim()
    if (!nomeFornitore) {
      setError('Inserisci la ragione sociale nel campo fornitore o nel banner')
      return
    }
    loadingCreaFornitore = true
    try {
      const f = await api.ddtFornitore.creaFornitoreRapido(nomeFornitore, fornitoreRilevatoPiva)
      fornitori.update(list => [...list, f])
      fornitoreId = f.id
      fornitoreTAQuery = f.ragione_sociale
      showBannerCreaFornitore = false
    } catch (e: any) {
      setError(e?.message ?? 'Errore creazione fornitore')
    } finally {
      loadingCreaFornitore = false
    }
  }

  async function importa() {
    if (!fornitoreId) { setError('Seleziona un fornitore'); return }
    if (!numeroDdt.trim()) { setError('Inserisci il numero del DDT'); return }
    if (!dataDdt) { setError('Inserisci la data del DDT'); return }
    if (righeInclude.length === 0) { setError('Seleziona almeno una riga'); return }

    loading = true
    try {
      const righeDaImportare: RigaDdtImport[] = righeInclude.map(r => ({
        codice_fornitore: r.codice_fornitore,
        descrizione: r.descrizione,
        um: r.um,
        quantita: r.quantita,
        ricambio_id: r.ricambio_id_edit,
        carica_magazzino: r.carica_magazzino && r.ricambio_id_edit !== null,
      }))

      const docId = await api.ddtFornitore.importa(
        fornitoreId,
        numeroDdt.trim(),
        dataDdt,
        righeDaImportare,
        pdfPath,
      )

      documentoCreato = docId
      nCaricati = righeDaCaricare.length
      fase = 'completato'
      setSuccess(`DDT Fornitore ${numeroDdt} importato — ${nCaricati} articoli caricati in magazzino`)
    } catch (e: any) {
      setError(e?.message ?? 'Errore importazione DDT')
    } finally {
      loading = false
    }
  }

  // ── Ricerca articoli per match manuale ────────────────────────────────────────

  function cercaRicambio(query: string): typeof $ricambi {
    if (!query.trim()) return []
    const q = query.toLowerCase()
    return $ricambi.filter(r =>
      r.codice_interno.toLowerCase().includes(q) ||
      (r.codice_oem ?? '').toLowerCase().includes(q) ||
      r.descrizione.toLowerCase().includes(q)
    ).slice(0, 10)
  }

  let searchQueries: string[] = []
  let searchOpen: boolean[] = []

  function onSearchInput(idx: number, e: Event) {
    searchQueries[idx] = (e.target as HTMLInputElement).value
    searchOpen[idx] = searchQueries[idx].length > 1
    searchQueries = [...searchQueries]
  }

  function selezionaRicambio(idx: number, ricambioId: number, codiceInterno: string, desc: string) {
    righe[idx].ricambio_id_edit = ricambioId
    righe[idx].codice_interno_match = codiceInterno
    righe[idx].descrizione_match = desc
    righe[idx].carica_magazzino = true
    searchQueries[idx] = ''
    searchOpen[idx] = false
    righe = [...righe]
  }

  function deselezionaRicambio(idx: number) {
    righe[idx].ricambio_id_edit = null
    righe[idx].codice_interno_match = null
    righe[idx].descrizione_match = null
    righe[idx].carica_magazzino = false
    righe = [...righe]
  }

  function reset() {
    fase = 'selezione'
    pdfPath = ''
    pdfNome = ''
    righe = []
    numeroDdt = ''
    dataDdt = new Date().toISOString().split('T')[0]
    fornitoreId = 0
    fornitoreTAQuery = ''
    fornitoreTAOpen = false
    fornitoreRilevatoPiva = null
    showBannerCreaFornitore = false
    ragioneSocialeNuova = ''
    documentoCreato = null
    nCaricati = 0
  }
</script>

<div class="p-6 space-y-6 max-w-5xl">
  <div class="flex items-center gap-3">
    <h1 class="text-xl font-bold text-white">Importa DDT Fornitore</h1>
    {#if fase !== 'selezione'}
      <span class="text-sm text-gray-400">— {pdfNome}</span>
    {/if}
  </div>

  <!-- ── FASE 1: Selezione PDF ── -->
  {#if fase === 'selezione'}
    <div class="bg-gray-800 rounded-xl p-6 space-y-5 max-w-lg">
      <p class="text-sm text-gray-400">
        Seleziona il PDF del DDT fornitore. Il fornitore verrà riconosciuto automaticamente dall'intestazione.
      </p>
      <div class="space-y-2">
        <label class="text-sm text-gray-300 font-medium">File PDF</label>
        <div class="flex gap-2">
          <input
            type="text"
            readonly
            value={pdfNome || ''}
            placeholder="Nessun file selezionato"
            class="flex-1 bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white text-sm"
          />
          <button on:click={selezionaPdf} class="btn-secondary text-sm px-3 py-2">
            Sfoglia
          </button>
        </div>
        {#if pdfPath}
          <p class="text-xs text-gray-500 truncate">{pdfPath}</p>
        {/if}
      </div>

      <button
        on:click={analizzaPdf}
        disabled={loading || !pdfPath}
        class="btn-primary w-full disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="animate-pulse">Analisi in corso…</span>
        {:else}
          Analizza PDF
        {/if}
      </button>
    </div>
  {/if}

  <!-- ── FASE 2: Anteprima ── -->
  {#if fase === 'anteprima'}

    <!-- Banner P.IVA non in anagrafica (visibile solo se fornitore non ancora selezionato) -->
    {#if showBannerCreaFornitore && fornitoreId === 0}
      <div class="bg-orange-900/40 border border-orange-600/60 rounded-xl px-4 py-3 space-y-2.5">
        <div class="flex items-start gap-3">
          <svg class="w-5 h-5 text-orange-400 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span class="text-sm text-orange-200">
            Fornitore con P.IVA <strong class="font-mono">{fornitoreRilevatoPiva}</strong> non presente in anagrafica
          </span>
        </div>

        <!-- Se il typeahead ha testo → mostra anteprima del nome; altrimenti input fallback -->
        {#if fornitoreTAQuery.trim()}
          <div class="pl-8 text-sm text-orange-300">
            Verrà creato come: <strong>{fornitoreTAQuery}</strong>
          </div>
        {:else}
          <div class="pl-8">
            <input
              type="text"
              bind:value={ragioneSocialeNuova}
              placeholder="Ragione sociale fornitore…"
              class="w-full max-w-xs bg-gray-700 border border-gray-600 rounded-lg px-3 py-1.5 text-white text-sm focus:outline-none focus:border-orange-400"
            />
          </div>
        {/if}

        <div class="flex gap-2 pl-8">
          <button
            on:click={creaFornitoreRapido}
            disabled={loadingCreaFornitore || (!fornitoreTAQuery.trim() && !ragioneSocialeNuova.trim())}
            class="btn-primary text-xs px-3 py-1.5 disabled:opacity-50"
          >
            {loadingCreaFornitore ? 'Creazione…' : 'Crea fornitore e continua'}
          </button>
          <button
            on:click={() => { showBannerCreaFornitore = false }}
            class="btn-secondary text-xs px-2 py-1.5 text-gray-400"
            title="Ignora, seleziona manualmente"
          >Ignora</button>
        </div>
      </div>
    {/if}

    <!-- Header: fornitore typeahead + numero + data + stats -->
    <div class="bg-gray-800 rounded-xl p-4 flex flex-wrap gap-4 items-end">

      <!-- Fornitore typeahead -->
      <div class="space-y-1 min-w-52 relative">
        <label class="text-xs text-gray-400">
          Fornitore
          {#if !fornitoreId}<span class="text-orange-400 ml-1">*richiesto</span>{/if}
        </label>
        <div class="relative">
          <input
            type="text"
            bind:value={fornitoreTAQuery}
            on:input={onFornitoreInput}
            on:focus={() => { if (fornitoreTAQuery.trim()) fornitoreTAOpen = true }}
            on:blur={() => setTimeout(() => { fornitoreTAOpen = false }, 200)}
            on:keydown={e => { if (e.key === 'Escape') fornitoreTAOpen = false }}
            placeholder="Cerca fornitore…"
            class="w-full bg-gray-700 border {fornitoreId ? 'border-gray-600' : 'border-orange-500'} rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 pr-7"
          />
          {#if fornitoreTAQuery}
            <button
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 leading-none px-0.5"
              tabindex="-1"
              on:mousedown|preventDefault={clearFornitore}
            >✕</button>
          {/if}
          {#if fornitoreTAOpen}
            {@const risultati = cercaFornitore(fornitoreTAQuery)}
            {#if risultati.length > 0}
              <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-48 overflow-y-auto">
                {#each risultati as f (f.id)}
                  <button
                    class="w-full text-left px-3 py-2 hover:bg-gray-700 flex flex-col gap-0.5"
                    on:mousedown|preventDefault={() => selectFornitore(f)}
                  >
                    <span class="text-gray-200 text-xs font-medium">{f.ragione_sociale}</span>
                    {#if f.partita_iva}
                      <span class="text-gray-500 text-xs">P.IVA: {f.partita_iva}</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      </div>

      <div class="space-y-1">
        <label class="text-xs text-gray-400">Numero DDT</label>
        <input
          type="text"
          bind:value={numeroDdt}
          placeholder="es. 12345"
          class="bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white text-sm w-36 focus:outline-none focus:border-blue-500"
        />
      </div>

      <div class="space-y-1">
        <label class="text-xs text-gray-400">Data DDT</label>
        <input
          type="date"
          bind:value={dataDdt}
          class="bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
        />
      </div>

      <div class="ml-auto text-sm text-gray-400 self-center text-right">
        <span class="text-white font-medium">{righeInclude.length}</span> righe,
        <span class="text-green-400 font-medium">{righeMatchate.length}</span> matchate,
        <span class="text-blue-400 font-medium">{righeDaCaricare.length}</span> da caricare
      </div>
    </div>

    <!-- Tabella righe -->
    <div class="bg-gray-800 rounded-xl overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-gray-700">
            <th class="px-3 py-3 text-center text-xs text-gray-400 w-8" title="Includi nel documento">Doc</th>
            <th class="px-3 py-3 text-center text-xs text-gray-400 w-8" title="Aggiungi a magazzino">Mag</th>
            <th class="px-3 py-3 text-left text-xs text-gray-400">Codice Forn.</th>
            <th class="px-3 py-3 text-left text-xs text-gray-400">Descrizione PDF</th>
            <th class="px-3 py-3 text-left text-xs text-gray-400 w-12">UM</th>
            <th class="px-3 py-3 text-right text-xs text-gray-400 w-16">Qtà</th>
            <th class="px-3 py-3 text-left text-xs text-gray-400">Articolo Magazzino</th>
          </tr>
        </thead>
        <tbody>
          {#each righe as riga, idx}
            <tr class="border-b border-gray-700/50 {riga.includi ? '' : 'opacity-40'}">
              <td class="px-3 py-2 text-center">
                <input type="checkbox" bind:checked={riga.includi} class="w-4 h-4 accent-blue-500" />
              </td>
              <td class="px-3 py-2 text-center">
                {#if riga.ricambio_id_edit !== null}
                  <input
                    type="checkbox"
                    bind:checked={riga.carica_magazzino}
                    disabled={!riga.includi}
                    class="w-4 h-4 accent-green-500 disabled:opacity-40"
                    title="Aggiungi a magazzino"
                  />
                {:else}
                  <span class="text-gray-600 text-xs" title="Nessun articolo abbinato">—</span>
                {/if}
              </td>
              <td class="px-3 py-2 text-gray-300 font-mono text-xs">{riga.codice_fornitore}</td>
              <td class="px-3 py-2 text-gray-200 text-xs max-w-xs truncate" title={riga.descrizione}>
                {riga.descrizione}
              </td>
              <td class="px-3 py-2 text-gray-400 text-xs">{riga.um}</td>
              <td class="px-3 py-2 text-right text-white font-medium text-xs">
                {riga.quantita.toLocaleString('it-IT', { minimumFractionDigits: 0, maximumFractionDigits: 2 })}
              </td>
              <td class="px-3 py-2 min-w-52">
                {#if riga.ricambio_id_edit !== null}
                  <div class="flex items-center gap-1.5">
                    <span class="text-green-400 text-xs font-mono shrink-0">{riga.codice_interno_match}</span>
                    <span class="text-gray-400 text-xs truncate" title={riga.descrizione_match ?? ''}>
                      {riga.descrizione_match}
                    </span>
                    <button
                      on:click={() => deselezionaRicambio(idx)}
                      class="ml-auto shrink-0 text-gray-500 hover:text-red-400 text-xs px-1"
                      title="Rimuovi match"
                    >✕</button>
                  </div>
                {:else}
                  <div class="relative">
                    <input
                      type="text"
                      placeholder="Cerca articolo…"
                      value={searchQueries[idx] ?? ''}
                      on:input={(e) => onSearchInput(idx, e)}
                      on:focus={() => { if ((searchQueries[idx] ?? '').length > 1) { searchOpen[idx] = true; searchOpen = [...searchOpen] } }}
                      on:blur={() => setTimeout(() => { searchOpen[idx] = false; searchOpen = [...searchOpen] }, 200)}
                      class="w-full bg-gray-700 border border-gray-600 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-blue-500"
                    />
                    {#if searchOpen[idx]}
                      {@const risultati = cercaRicambio(searchQueries[idx] ?? '')}
                      {#if risultati.length > 0}
                        <div class="absolute z-10 left-0 right-0 top-full mt-1 bg-gray-700 border border-gray-600 rounded shadow-lg max-h-40 overflow-y-auto">
                          {#each risultati as r}
                            <button
                              class="w-full text-left px-2 py-1.5 hover:bg-gray-600 text-xs"
                              on:click={() => selezionaRicambio(idx, r.id, r.codice_interno, r.descrizione)}
                            >
                              <span class="text-blue-400 font-mono">{r.codice_interno}</span>
                              <span class="text-gray-300 ml-2">{r.descrizione}</span>
                            </button>
                          {/each}
                        </div>
                      {/if}
                    {/if}
                  </div>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Azioni -->
    <div class="flex gap-3 items-center">
      <button on:click={() => { fase = 'selezione'; righe = [] }} class="btn-secondary">
        ← Indietro
      </button>
      <button
        on:click={importa}
        disabled={loading || !puoImportare}
        class="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
        title={!fornitoreId ? 'Seleziona un fornitore' : !numeroDdt.trim() ? 'Inserisci il numero DDT' : ''}
      >
        {#if loading}
          <span class="animate-pulse">Importazione…</span>
        {:else}
          Importa DDT ({righeInclude.length} righe, {righeDaCaricare.length} in magazzino)
        {/if}
      </button>
      {#if !fornitoreId}
        <span class="text-xs text-orange-400">Seleziona un fornitore per procedere</span>
      {:else if !numeroDdt.trim()}
        <span class="text-xs text-orange-400">Inserisci il numero DDT per procedere</span>
      {/if}
    </div>
  {/if}

  <!-- ── FASE 3: Completato ── -->
  {#if fase === 'completato'}
    <div class="bg-gray-800 rounded-xl p-8 text-center space-y-4 max-w-lg">
      <div class="text-4xl">✓</div>
      <h2 class="text-lg font-semibold text-white">DDT importato</h2>
      <p class="text-gray-400 text-sm">
        DDT <span class="text-white font-medium">{numeroDdt}</span> creato.
        {nCaricati} articol{nCaricati === 1 ? 'o caricato' : 'i caricati'} in magazzino.
      </p>
      <div class="flex gap-3 justify-center pt-2">
        <button on:click={reset} class="btn-secondary">Nuovo import</button>
        <button on:click={() => currentView.set('documenti')} class="btn-primary">
          Vai ai Documenti
        </button>
      </div>
    </div>
  {/if}
</div>
