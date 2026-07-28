<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { clienti, fornitori, ricambi, documenti, formatCurrency, setError, currentView, editDocumentoId } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovaRigaDocumento, TipoDocumento, Ricambio, ArticoloStorico, Cliente } from '../../lib/types'

  const dispatch = createEventDispatcher()

  let tipoDocumento: TipoDocumento = 'fattura'
  let numero = ''
  let data = new Date().toISOString().split('T')[0]
  let clienteId: number | null = null
  let note = ''
  let saving = false
  let touched = false
  let editMode = false
  let editId: number | null = null

  // DDT Fornitore: creato solo tramite import PDF, qui è modificabile ma non riclassificabile
  let fornitoreIdDdt: number | null = null
  $: fornitoreNomeDdt = fornitoreIdDdt != null
    ? ($fornitori.find(f => f.id === fornitoreIdDdt)?.ragione_sociale ?? '—')
    : '—'

  // Dati trasporto / pagamento
  let giorniPagamento = 30
  let vettore = ''
  let dataOraRitiro = ''
  let nColli = 0
  let aspettoEsterioreBeni = ''
  let porto = ''
  let causaleTrasporto = ''
  let trasportoACura = 'Destinatario'
  let bancaAppoggio = ''
  let agente = ''
  let bolliArt15 = ''
  let speseVarie = 0
  let speseIncasso = 0
  let mostraIban = true

  $: mostraTrasportoCore = ['ddt', 'fattura', 'nota_credito', 'vendita_banco'].includes(tipoDocumento)
  $: mostraTrasportoGruppoA = tipoDocumento === 'buono' || tipoDocumento === 'preventivo'
  $: mostraGiorniPagamento = tipoDocumento !== 'vendita_banco' && tipoDocumento !== 'buono' && tipoDocumento !== 'fattura_differita'

  function resetTrasporto() {
    vettore = ''
    dataOraRitiro = ''
    nColli = 0
    aspettoEsterioreBeni = ''
    porto = ''
    causaleTrasporto = ''
    trasportoACura = 'Destinatario'
    bancaAppoggio = ''
    agente = ''
    bolliArt15 = ''
    speseVarie = 0
    speseIncasso = 0
    mostraIban = true
  }

  // Fattura differita — DDT selezionati
  let ddtSelezionati: number[] = []
  let caricandoDdt = false

  interface RigaUI extends NuovaRigaDocumento { _id: number }
  let righe: RigaUI[] = []
  let nextId = 0

  // Typeahead per selezione articolo per riga
  interface TAState { query: string; results: Ricambio[]; storicoHits: ArticoloStorico[]; open: boolean }
  let righeTA: Record<number, TAState> = {}
  let taTimers: Record<number, ReturnType<typeof setTimeout>> = {}
  let righeDescTA: Record<number, TAState> = {}
  let descTaTimers: Record<number, ReturnType<typeof setTimeout>> = {}

  const emptyTA = (): TAState => ({ query: '', results: [], storicoHits: [], open: false })

  function getTa(id: number): TAState {
    return righeTA[id] ?? emptyTA()
  }

  function setTa(id: number, patch: Partial<TAState>) {
    righeTA = { ...righeTA, [id]: { ...getTa(id), ...patch } }
  }

  function getDescTa(id: number): TAState {
    return righeDescTA[id] ?? emptyTA()
  }

  function setDescTa(id: number, patch: Partial<TAState>) {
    righeDescTA = { ...righeDescTA, [id]: { ...getDescTa(id), ...patch } }
  }

  function buildStoricoHits(query: string): ArticoloStorico[] {
    if (!clienteId || tipoDocumento === 'fattura_differita' || !query.trim()) return []
    const q = query.trim().toLowerCase()
    return storicoCliente
      .filter(a => a.codice_interno.toLowerCase().includes(q) || a.descrizione.toLowerCase().includes(q))
      .slice(0, 10)
  }

  async function onTaInput(riga: RigaUI, value: string) {
    const hits = buildStoricoHits(value)
    setTa(riga._id, { query: value, open: !!value.trim(), storicoHits: hits })
    clearTimeout(taTimers[riga._id])
    if (!value.trim()) { setTa(riga._id, { results: [], storicoHits: [] }); return }
    taTimers[riga._id] = setTimeout(async () => {
      const q = getTa(riga._id).query
      if (!q.trim()) return
      try {
        const currentHits = buildStoricoHits(q)
        const hitIds = new Set(currentHits.map(a => a.ricambio_id))
        const all = await api.ricambi.search(q)
        const results = all.filter(r => !hitIds.has(r.id)).slice(0, Math.max(0, 10 - currentHits.length))
        setTa(riga._id, { results, storicoHits: currentHits, open: true })
      } catch {}
    }, 300)
  }

  async function onDescTaInput(riga: RigaUI, value: string) {
    const hits = buildStoricoHits(value)
    setDescTa(riga._id, { query: value, open: !!value.trim(), storicoHits: hits })
    riga.descrizione = value
    righe = [...righe]
    clearTimeout(descTaTimers[riga._id])
    if (!value.trim()) { setDescTa(riga._id, { results: [], storicoHits: [] }); return }
    descTaTimers[riga._id] = setTimeout(async () => {
      const q = getDescTa(riga._id).query
      if (!q.trim()) return
      try {
        const currentHits = buildStoricoHits(q)
        const hitIds = new Set(currentHits.map(a => a.ricambio_id))
        const all = await api.ricambi.search(q)
        const results = all.filter(r => !hitIds.has(r.id)).slice(0, Math.max(0, 10 - currentHits.length))
        setDescTa(riga._id, { results, storicoHits: currentHits, open: true })
      } catch {}
    }, 300)
  }

  function selectTa(riga: RigaUI, r: Ricambio) {
    riga.ricambio_id = r.id
    riga.descrizione = r.descrizione
    riga.prezzo_unitario = r.prezzo_vendita
    riga.iva_percentuale = senzaIva ? 0 : r.iva_percentuale
    righe = [...righe]
    setTa(riga._id, { query: `${r.codice_interno} — ${r.descrizione}`, open: false, results: [], storicoHits: [] })
    setDescTa(riga._id, { query: r.descrizione, open: false, results: [], storicoHits: [] })
  }

  function selectDescTa(riga: RigaUI, r: Ricambio) {
    riga.ricambio_id = r.id
    riga.descrizione = r.descrizione
    riga.prezzo_unitario = r.prezzo_vendita
    riga.iva_percentuale = senzaIva ? 0 : r.iva_percentuale
    righe = [...righe]
    setDescTa(riga._id, { query: r.descrizione, open: false, results: [], storicoHits: [] })
    setTa(riga._id, { query: `${r.codice_interno} — ${r.descrizione}`, open: false, results: [], storicoHits: [] })
  }

  function selectTaFromStorico(riga: RigaUI, art: ArticoloStorico) {
    riga.ricambio_id = art.ricambio_id
    riga.descrizione = art.descrizione
    riga.prezzo_unitario = art.prezzo_unitario
    riga.iva_percentuale = senzaIva ? 0 : 22
    righe = [...righe]
    setTa(riga._id, { query: `${art.codice_interno} — ${art.descrizione}`, open: false, results: [], storicoHits: [] })
    setDescTa(riga._id, { query: art.descrizione, open: false, results: [], storicoHits: [] })
  }

  function selectDescTaFromStorico(riga: RigaUI, art: ArticoloStorico) {
    riga.ricambio_id = art.ricambio_id
    riga.descrizione = art.descrizione
    riga.prezzo_unitario = art.prezzo_unitario
    riga.iva_percentuale = senzaIva ? 0 : 22
    righe = [...righe]
    setDescTa(riga._id, { query: art.descrizione, open: false, results: [], storicoHits: [] })
    setTa(riga._id, { query: `${art.codice_interno} — ${art.descrizione}`, open: false, results: [], storicoHits: [] })
  }

  function clearTa(riga: RigaUI) {
    riga.ricambio_id = null
    riga.descrizione = ''
    riga.prezzo_unitario = 0
    riga.iva_percentuale = senzaIva ? 0 : 22
    righe = [...righe]
    setTa(riga._id, { query: '', results: [], storicoHits: [], open: false })
    setDescTa(riga._id, { query: '', results: [], storicoHits: [], open: false })
  }

  function clearDescTa(riga: RigaUI) {
    riga.ricambio_id = null
    riga.descrizione = ''
    riga.prezzo_unitario = 0
    riga.iva_percentuale = senzaIva ? 0 : 22
    righe = [...righe]
    setDescTa(riga._id, { query: '', results: [], storicoHits: [], open: false })
    setTa(riga._id, { query: '', results: [], storicoHits: [], open: false })
  }

  // Typeahead selezione cliente
  interface ClienteTAState { query: string; results: Cliente[]; open: boolean }
  let clienteTA: ClienteTAState = { query: '', results: [], open: false }
  let clienteTATimer: ReturnType<typeof setTimeout>

  async function onClienteInput(value: string) {
    clienteTA = { query: value, results: clienteTA.results, open: !!value.trim() }
    clienteId = null
    clearTimeout(clienteTATimer)
    if (!value.trim()) { clienteTA = { query: '', results: [], open: false }; return }
    clienteTATimer = setTimeout(async () => {
      try {
        const res = await api.clienti.getPaginated(0, 10, value)
        clienteTA = { query: value, results: res.items, open: true }
      } catch {}
    }, 300)
  }

  function selectCliente(c: Cliente) {
    clienteId = c.id
    clienteTA = { query: c.ragione_sociale, results: [], open: false }
  }

  function clearCliente() {
    clienteId = null
    clienteTA = { query: '', results: [], open: false }
  }

  // Storico acquisti cliente (alimenta i dropdown di ricambio/descrizione)
  let storicoCliente: ArticoloStorico[] = []

  $: if (clienteId && tipoDocumento !== 'fattura_differita') {
    caricaStorico(clienteId)
  } else {
    storicoCliente = []
  }

  async function caricaStorico(id: number) {
    try {
      storicoCliente = await api.documenti.getStoricoCliente(id)
    } catch {
      storicoCliente = []
    }
  }

  $: senzaIva = tipoDocumento === 'buono' || tipoDocumento === 'preventivo'

  const tipiDisponibili: { value: TipoDocumento; label: string }[] = [
    { value: 'fattura',           label: 'Fattura' },
    { value: 'ddt',               label: 'DDT' },
    { value: 'preventivo',        label: 'Preventivo' },
    { value: 'nota_credito',      label: 'Nota credito' },
    { value: 'vendita_banco',     label: 'Vendita Banco' },
    { value: 'buono',             label: 'Buono' },
    { value: 'fattura_differita', label: 'Fattura Differita' },
  ]

  function computeNumero(tipo: TipoDocumento): string {
    const minimi: Partial<Record<TipoDocumento, number>> = {
      fattura:           16,
      ddt:               16,
      fattura_differita: 16,
      preventivo:        3,
      nota_credito:      1,
      vendita_banco:     1,
      buono:             7,
    }
    const min = minimi[tipo] ?? 1
    const count = $documenti.filter(d =>
      d.tipo_documento === tipo &&
      Number.isInteger(Number(d.numero)) &&
      Number(d.numero) >= min
    ).length
    return (min + count).toString()
  }

  onMount(async () => {
    const eid = $editDocumentoId
    editDocumentoId.set(null)
    if (eid !== null) {
      editMode = true
      editId = eid
      try {
        const docCompleto = await api.documenti.get(eid)
        const d = docCompleto.documento
        tipoDocumento = d.tipo_documento as TipoDocumento
        numero = d.numero
        data = d.data
        clienteId = d.cliente_id
        fornitoreIdDdt = d.fornitore_id
        note = d.note ?? ''
        giorniPagamento = d.giorni_pagamento ?? 30
        vettore = d.vettore ?? ''
        dataOraRitiro = d.data_ora_ritiro ?? ''
        nColli = d.n_colli ?? 0
        aspettoEsterioreBeni = d.aspetto_esteriore_beni ?? ''
        porto = d.porto ?? ''
        causaleTrasporto = d.causale_trasporto ?? ''
        trasportoACura = d.trasporto_a_cura ?? 'Destinatario'
        bancaAppoggio = d.banca_appoggio ?? ''
        agente = d.agente ?? ''
        bolliArt15 = d.bolli_art15 ?? ''
        speseVarie = d.spese_varie ?? 0
        speseIncasso = d.spese_incasso ?? 0
        mostraIban = d.mostra_iban ?? true
        if (d.cliente_id) {
          const c = $clienti.find(c => c.id === d.cliente_id)
          if (c) clienteTA = { query: c.ragione_sociale, results: [], open: false }
        }
        righe = docCompleto.righe.map(r => {
          const _id = nextId++
          const rcMatch = r.ricambio_id != null ? $ricambi.find(rc => rc.id === r.ricambio_id) : null
          const taQuery = rcMatch ? `${rcMatch.codice_interno} — ${rcMatch.descrizione}` : r.descrizione
          righeTA = { ...righeTA, [_id]: { query: taQuery, results: [], storicoHits: [], open: false } }
          righeDescTA = { ...righeDescTA, [_id]: { query: r.descrizione, results: [], storicoHits: [], open: false } }
          return {
            _id,
            ricambio_id: r.ricambio_id,
            descrizione: r.descrizione,
            quantita: r.quantita,
            prezzo_unitario: r.prezzo_unitario,
            sconto_percentuale: r.sconto_percentuale,
            iva_percentuale: r.iva_percentuale,
            ordine: r.ordine,
          }
        })
      } catch (e: any) {
        setError(e?.message ?? 'Errore caricamento documento')
        currentView.set('documenti')
      }
    } else {
      numero = computeNumero(tipoDocumento)
    }
  })

  function onTipoChange() {
    numero = computeNumero(tipoDocumento)
    ddtSelezionati = []
    righe = []
    resetTrasporto()
    giorniPagamento = 30
    // Per vendita_banco: cerca o lascia null il cliente generico
    if (tipoDocumento === 'vendita_banco') {
      const banco = $clienti.find(c => c.ragione_sociale === 'CLIENTE AL BANCO')
      clienteId = banco?.id ?? null
    }
  }

  $: ddtDisponibili = tipoDocumento === 'fattura_differita' && clienteId
    ? $documenti.filter(d => d.tipo_documento === 'ddt' && d.fatturato === 0 && d.cliente_id === clienteId)
    : []

  async function toggleDdt(id: number) {
    if (ddtSelezionati.includes(id)) {
      ddtSelezionati = ddtSelezionati.filter(x => x !== id)
    } else {
      ddtSelezionati = [...ddtSelezionati, id]
    }
    await importaRigheDdt()
  }

  async function importaRigheDdt() {
    if (ddtSelezionati.length === 0) { righe = []; return }
    caricandoDdt = true
    try {
      const nuoveRighe: RigaUI[] = []
      for (const id of ddtSelezionati) {
        const doc = await api.documenti.get(id)
        for (const r of doc.righe) {
          nuoveRighe.push({
            _id: nextId++,
            ricambio_id: r.ricambio_id,
            descrizione: r.descrizione,
            quantita: r.quantita,
            prezzo_unitario: r.prezzo_unitario,
            sconto_percentuale: r.sconto_percentuale,
            iva_percentuale: r.iva_percentuale,
            ordine: nuoveRighe.length,
          })
        }
      }
      righe = nuoveRighe
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento righe DDT')
    } finally {
      caricandoDdt = false
    }
  }

  // --- Validazione reattiva ---
  $: errNumero  = touched && !numero.trim() ? 'Numero obbligatorio' : ''
  $: errCliente = touched && tipoDocumento !== 'preventivo' && tipoDocumento !== 'buono' && tipoDocumento !== 'ddt_fornitore' && !clienteId
    ? 'Seleziona un cliente'
    : ''
  $: errRighe   = touched
    ? righe.reduce((acc, r) => {
        const e: { descrizione?: string; quantita?: string; prezzo?: string } = {}
        if (!r.descrizione.trim()) e.descrizione = 'Descrizione obbligatoria'
        if (r.quantita <= 0)       e.quantita    = 'Deve essere > 0'
        if (r.prezzo_unitario < 0) e.prezzo      = 'Non può essere negativo'
        if (Object.keys(e).length) acc[r._id] = e
        return acc
      }, {} as Record<number, { descrizione?: string; quantita?: string; prezzo?: string }>)
    : ({} as Record<number, { descrizione?: string; quantita?: string; prezzo?: string }>)

  $: hasErrors = !!errNumero || !!errCliente || Object.keys(errRighe).length > 0 || (touched && righe.length === 0)

  function addRiga() {
    const id = nextId++
    righeTA = { ...righeTA, [id]: emptyTA() }
    righeDescTA = { ...righeDescTA, [id]: emptyTA() }
    righe = [...righe, {
      _id: id, ricambio_id: null, descrizione: '',
      quantita: 1, prezzo_unitario: 0, sconto_percentuale: 0,
      iva_percentuale: senzaIva ? 0 : 22, ordine: righe.length
    }]
  }

  function removeRiga(id: number) {
    const { [id]: _, ...rest } = righeTA
    righeTA = rest
    const { [id]: _d, ...restD } = righeDescTA
    righeDescTA = restD
    righe = righe.filter(r => r._id !== id).map((r, i) => ({ ...r, ordine: i }))
  }

  function imponibileRiga(r: RigaUI) {
    const lordo = r.quantita * r.prezzo_unitario
    return lordo - lordo * (r.sconto_percentuale / 100)
  }

  $: totaleImponibile = righe.reduce((s, r) => s + imponibileRiga(r), 0)
  $: totaleIva        = righe.reduce((s, r) => s + imponibileRiga(r) * (r.iva_percentuale / 100), 0)
  $: totaleDoc        = totaleImponibile + totaleIva

  async function salva() {
    touched = true
    await new Promise(r => setTimeout(r, 0))
    if (righe.length === 0) { setError('Aggiungi almeno una riga'); return }
    if (hasErrors) return
    saving = true
    try {
      const payload = {
        tipo_documento: tipoDocumento,
        numero,
        data,
        cliente_id: clienteId,
        fornitore_id: tipoDocumento === 'ddt_fornitore' ? fornitoreIdDdt : null,
        note: note || null,
        giorni_pagamento: mostraGiorniPagamento ? giorniPagamento : null,
        ddt_collegati: tipoDocumento === 'fattura_differita' && ddtSelezionati.length > 0
          ? ddtSelezionati
          : null,
        vettore: mostraTrasportoCore ? (vettore || null) : null,
        data_ora_ritiro: (mostraTrasportoCore || mostraTrasportoGruppoA) ? (dataOraRitiro || null) : null,
        n_colli: (mostraTrasportoCore || mostraTrasportoGruppoA) ? nColli : null,
        aspetto_esteriore_beni: (mostraTrasportoCore || mostraTrasportoGruppoA) ? (aspettoEsterioreBeni || null) : null,
        porto: mostraTrasportoCore ? (porto || null) : null,
        causale_trasporto: mostraTrasportoCore ? (causaleTrasporto || null) : null,
        trasporto_a_cura: mostraTrasportoCore ? (trasportoACura || null) : null,
        banca_appoggio: mostraTrasportoCore ? (bancaAppoggio || null) : null,
        agente: mostraTrasportoCore ? (agente || null) : null,
        bolli_art15: mostraTrasportoCore ? (bolliArt15 || null) : null,
        spese_varie: mostraTrasportoCore ? speseVarie : null,
        spese_incasso: mostraTrasportoCore ? speseIncasso : null,
        mostra_iban: mostraTrasportoCore ? mostraIban : null,
        righe: righe.map(({ _id, ...r }) => r),
      }
      if (editMode && editId !== null) {
        const result = await api.documenti.update(editId, payload)
        documenti.update(list => list.map(d => d.id === result.documento.id ? result.documento : d))
      } else {
        const result = await api.documenti.create(payload)
        documenti.update(list => [result.documento, ...list])
      }
      dispatch('refresh')
      currentView.set('documenti')
    } catch (e: any) {
      setError(e?.message ?? 'Errore salvataggio')
    } finally {
      saving = false
    }
  }
</script>

<div class="p-6 space-y-5 max-w-5xl">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">{editMode ? 'Modifica documento' : 'Nuovo documento'}</h1>
    <button class="btn-secondary" on:click={() => currentView.set('documenti')}>← Torna</button>
  </div>

  <!-- Tipo documento -->
  <div class="card p-4">
    {#if tipoDocumento === 'ddt_fornitore'}
      <div class="flex items-center gap-2 text-xs">
        <span class="px-2 py-1 rounded bg-gray-800 border border-gray-700 text-gray-300 font-medium">DDT Fornitore</span>
        <span class="text-gray-500">Documento creato tramite import PDF — il tipo non è modificabile da qui.</span>
      </div>
    {:else}
      <label class="label mb-2 block">Tipo documento</label>
      <div class="flex flex-wrap gap-2">
        {#each tipiDisponibili as t}
          <button
            class="px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors
              {tipoDocumento === t.value
                ? 'bg-brand-600 border-brand-500 text-white'
                : 'bg-gray-800 border-gray-700 text-gray-400 hover:text-gray-200 hover:border-gray-500'}"
            on:click={() => { tipoDocumento = t.value; onTipoChange() }}
          >
            {t.label}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Testata -->
  <div class="card p-5 space-y-4">
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
      <!-- Numero -->
      <div>
        <label class="label">Numero *</label>
        <input
          class="input {errNumero ? 'border-red-500 focus:ring-red-500' : ''}"
          bind:value={numero}
        />
        {#if errNumero}<p class="text-xs text-red-400 mt-1">{errNumero}</p>{/if}
      </div>

      <!-- Data -->
      <div>
        <label class="label">Data *</label>
        <input class="input" type="date" bind:value={data}/>
      </div>

      <!-- Cliente / Fornitore -->
      <div class="col-span-2">
        <label class="label">
          {tipoDocumento === 'ddt_fornitore' ? 'Fornitore' : 'Cliente'}
          {#if tipoDocumento !== 'preventivo' && tipoDocumento !== 'buono' && tipoDocumento !== 'ddt_fornitore'}*{/if}
        </label>
        {#if tipoDocumento === 'vendita_banco'}
          <input
            class="input bg-gray-900 text-gray-500 cursor-not-allowed"
            value="CLIENTE AL BANCO"
            disabled
          />
        {:else if tipoDocumento === 'ddt_fornitore'}
          <input
            class="input bg-gray-900 text-gray-500 cursor-not-allowed"
            value={fornitoreNomeDdt}
            disabled
          />
        {:else}
          <div class="relative">
            <input
              class="input pr-7 {errCliente ? 'border-red-500 focus:ring-red-500' : ''}"
              placeholder="Cerca cliente per nome, P.IVA..."
              value={clienteTA.query}
              on:input={e => onClienteInput((e.target as HTMLInputElement).value)}
              on:keydown={e => { if (e.key === 'Escape') clienteTA = { ...clienteTA, open: false } }}
              on:blur={() => setTimeout(() => clienteTA = { ...clienteTA, open: false }, 150)}
            />
            {#if clienteTA.query}
              <button
                class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 leading-none px-0.5"
                tabindex="-1"
                on:mousedown|preventDefault={clearCliente}
              >✕</button>
            {/if}
            {#if clienteTA.open && clienteTA.results.length > 0}
              <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-60 overflow-y-auto">
                {#each clienteTA.results as c (c.id)}
                  <button
                    class="w-full text-left px-3 py-2 hover:bg-gray-700 flex flex-col gap-0.5"
                    on:mousedown|preventDefault={() => selectCliente(c)}
                  >
                    <span class="text-gray-200 text-xs font-medium">{c.ragione_sociale}</span>
                    {#if c.partita_iva}<span class="text-gray-500 text-xs">P.IVA: {c.partita_iva}</span>{/if}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
          {#if errCliente}<p class="text-xs text-red-400 mt-1">{errCliente}</p>{/if}
        {/if}
      </div>
    </div>

    {#if mostraGiorniPagamento}
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div>
          <label class="label">Giorni pagamento</label>
          <input class="input" type="number" min="0" step="1" bind:value={giorniPagamento} />
        </div>
      </div>
    {/if}

    <div>
      <label class="label">Note</label>
      <textarea class="input resize-none" rows="2" bind:value={note}></textarea>
    </div>
  </div>

  <!-- Dati trasporto -->
  {#if mostraTrasportoCore || mostraTrasportoGruppoA}
    <div class="card p-5 space-y-4">
      <h2 class="text-sm font-semibold text-white">Dati trasporto</h2>

      <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div>
          <label class="label">N° Colli</label>
          <input class="input" type="number" min="0" step="1" bind:value={nColli} />
        </div>
        <div class="col-span-2">
          <label class="label">Aspetto esteriore beni</label>
          <input class="input" bind:value={aspettoEsterioreBeni} placeholder="es. Scatole integre" />
        </div>
        <div>
          <label class="label">{mostraTrasportoGruppoA ? 'Alle ore' : 'Data e ora ritiro'}</label>
          <input
            class="input"
            type={mostraTrasportoGruppoA ? 'time' : 'datetime-local'}
            bind:value={dataOraRitiro}
          />
        </div>
      </div>

      {#if mostraTrasportoCore}
        <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div>
            <label class="label">Vettore</label>
            <input class="input" bind:value={vettore} placeholder="es. Corriere / mezzo proprio" />
          </div>
          <div>
            <label class="label">Porto</label>
            <select class="input" bind:value={porto}>
              <option value="">—</option>
              <option value="Franco">Franco</option>
              <option value="Assegnato">Assegnato</option>
            </select>
          </div>
          <div>
            <label class="label">Causale trasporto</label>
            <input class="input" bind:value={causaleTrasporto} placeholder="Vendita" />
          </div>
          <div>
            <label class="label">Trasporto a cura</label>
            <select class="input" bind:value={trasportoACura}>
              <option value="Destinatario">Destinatario</option>
              <option value="Mittente">Mittente</option>
              <option value="Vettore">Vettore</option>
            </select>
          </div>
        </div>

        <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div>
            <label class="label">Banca d'appoggio</label>
            <input class="input" bind:value={bancaAppoggio} />
          </div>
          <div>
            <label class="label">Agente</label>
            <input class="input" bind:value={agente} />
          </div>
          <div>
            <label class="label">Bolli es. art.15</label>
            <input class="input" bind:value={bolliArt15} />
          </div>
          <div class="flex items-end pb-1.5">
            <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
              <input type="checkbox" class="rounded border-gray-600" bind:checked={mostraIban} />
              Mostra IBAN in stampa
            </label>
          </div>
        </div>

        <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div>
            <label class="label">Spese varie €</label>
            <input class="input" type="number" min="0" step="0.01" bind:value={speseVarie} />
          </div>
          <div>
            <label class="label">Spese incasso €</label>
            <input class="input" type="number" min="0" step="0.01" bind:value={speseIncasso} />
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Selettore DDT per Fattura Differita -->
  {#if tipoDocumento === 'fattura_differita'}
    <div class="card p-5 space-y-3">
      <h2 class="text-sm font-semibold text-white">DDT da fatturare</h2>
      {#if !clienteId}
        <p class="text-xs text-gray-500">Seleziona prima un cliente per vedere i DDT disponibili.</p>
      {:else if ddtDisponibili.length === 0}
        <p class="text-xs text-gray-500">Nessun DDT non fatturato per questo cliente.</p>
      {:else}
        <div class="space-y-2">
          {#each ddtDisponibili as ddt}
            <label class="flex items-center gap-3 p-2 rounded-lg bg-gray-800/50 hover:bg-gray-800 cursor-pointer">
              <input
                type="checkbox"
                checked={ddtSelezionati.includes(ddt.id)}
                on:change={() => toggleDdt(ddt.id)}
                class="rounded border-gray-600"
              />
              <span class="font-mono text-brand-400 text-xs">{ddt.numero}</span>
              <span class="text-gray-400 text-xs">{ddt.data}</span>
              <span class="text-green-400 text-xs ml-auto">{formatCurrency(ddt.totale_documento)}</span>
            </label>
          {/each}
        </div>
        {#if caricandoDdt}
          <p class="text-xs text-gray-500">Caricamento righe DDT…</p>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- Righe documento -->
  <div class="card">
    <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
      <h2 class="text-sm font-semibold text-white">Righe documento</h2>
      {#if tipoDocumento !== 'fattura_differita'}
        <button class="btn-secondary text-xs" on:click={addRiga}>+ Aggiungi riga</button>
      {/if}
    </div>

    {#if righe.length === 0}
      <div class="flex flex-col items-center gap-3 py-12 text-gray-600">
        <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
            d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <p class="text-sm {touched ? 'text-red-400' : ''}">
          {#if touched}
            Aggiungi almeno una riga
          {:else if tipoDocumento === 'fattura_differita'}
            Seleziona uno o più DDT qui sopra per importare le righe
          {:else}
            Nessuna riga — clicca "+ Aggiungi riga"
          {/if}
        </p>
      </div>
    {:else}
      <div class="divide-y divide-gray-800">
        {#each righe as riga (riga._id)}
          {@const re = errRighe[riga._id] ?? {}}
          <div class="p-4 grid grid-cols-12 gap-2 items-start">
            <!-- Ricambio typeahead -->
            <div class="col-span-3">
              <label class="label">Ricambio</label>
              {#if tipoDocumento !== 'fattura_differita'}
                <div class="relative">
                  <input
                    class="input text-xs pr-6"
                    placeholder="Cerca codice o descrizione..."
                    value={getTa(riga._id).query}
                    on:input={e => onTaInput(riga, (e.target as HTMLInputElement).value)}
                    on:keydown={e => { if (e.key === 'Escape') setTa(riga._id, { open: false }) }}
                    on:blur={() => setTimeout(() => setTa(riga._id, { open: false }), 150)}
                  />
                  {#if getTa(riga._id).query}
                    <button
                      class="absolute right-1.5 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 leading-none px-0.5"
                      tabindex="-1"
                      on:mousedown|preventDefault={() => clearTa(riga)}
                    >✕</button>
                  {/if}
                  {#if getTa(riga._id).open && (getTa(riga._id).storicoHits.length > 0 || getTa(riga._id).results.length > 0)}
                    <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-80 overflow-y-auto">
                      {#each getTa(riga._id).storicoHits as art (art.ricambio_id)}
                        <button
                          class="w-full text-left px-3 py-2.5 hover:bg-gray-700 flex items-center gap-1.5"
                          on:mousedown|preventDefault={() => selectTaFromStorico(riga, art)}
                        >
                          <span class="text-yellow-400 text-xs shrink-0">★</span>
                          <span class="font-mono text-brand-400 text-xs shrink-0">{art.codice_interno}</span>
                          <span class="text-gray-300 text-xs truncate flex-1">{art.descrizione}</span>
                          <span class="text-gray-500 text-xs shrink-0">×{art.frequenza}</span>
                        </button>
                      {/each}
                      {#if getTa(riga._id).storicoHits.length > 0 && getTa(riga._id).results.length > 0}
                        <div class="px-3 py-0.5 border-t border-gray-700 text-gray-600 text-xs">Altri risultati</div>
                      {/if}
                      {#each getTa(riga._id).results as r (r.id)}
                        <button
                          class="w-full text-left px-3 py-2.5 hover:bg-gray-700 flex flex-col gap-0.5"
                          on:mousedown|preventDefault={() => selectTa(riga, r)}
                        >
                          <span class="font-mono text-brand-400 text-xs">{r.codice_interno}</span>
                          <span class="text-gray-300 text-xs truncate">{r.descrizione}</span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {:else}
                <input class="input text-xs bg-gray-900 text-gray-500" value={getTa(riga._id).query || riga.descrizione} disabled />
              {/if}
            </div>
            <!-- Descrizione typeahead -->
            <div class="col-span-3">
              <label class="label">Descrizione</label>
              {#if tipoDocumento !== 'fattura_differita'}
                <div class="relative">
                  <input
                    class="input text-xs pr-6 {re.descrizione ? 'border-red-500' : ''}"
                    placeholder="Cerca descrizione..."
                    value={getDescTa(riga._id).query}
                    on:input={e => onDescTaInput(riga, (e.target as HTMLInputElement).value)}
                    on:keydown={e => { if (e.key === 'Escape') setDescTa(riga._id, { open: false }) }}
                    on:blur={() => setTimeout(() => setDescTa(riga._id, { open: false }), 150)}
                  />
                  {#if getDescTa(riga._id).query}
                    <button
                      class="absolute right-1.5 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300 leading-none px-0.5"
                      tabindex="-1"
                      on:mousedown|preventDefault={() => clearDescTa(riga)}
                    >✕</button>
                  {/if}
                  {#if getDescTa(riga._id).open && (getDescTa(riga._id).storicoHits.length > 0 || getDescTa(riga._id).results.length > 0)}
                    <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-80 overflow-y-auto">
                      {#each getDescTa(riga._id).storicoHits as art (art.ricambio_id)}
                        <button
                          class="w-full text-left px-3 py-2.5 hover:bg-gray-700 flex items-center gap-1.5"
                          on:mousedown|preventDefault={() => selectDescTaFromStorico(riga, art)}
                        >
                          <span class="text-yellow-400 text-xs shrink-0">★</span>
                          <span class="text-gray-300 text-xs truncate flex-1">{art.descrizione}</span>
                          <span class="font-mono text-brand-400 text-xs shrink-0">{art.codice_interno}</span>
                          <span class="text-gray-500 text-xs shrink-0">×{art.frequenza}</span>
                        </button>
                      {/each}
                      {#if getDescTa(riga._id).storicoHits.length > 0 && getDescTa(riga._id).results.length > 0}
                        <div class="px-3 py-0.5 border-t border-gray-700 text-gray-600 text-xs">Altri risultati</div>
                      {/if}
                      {#each getDescTa(riga._id).results as r (r.id)}
                        <button
                          class="w-full text-left px-3 py-2.5 hover:bg-gray-700 flex flex-col gap-0.5"
                          on:mousedown|preventDefault={() => selectDescTa(riga, r)}
                        >
                          <span class="text-gray-300 text-xs truncate">{r.descrizione}</span>
                          <span class="font-mono text-brand-400 text-xs">{r.codice_interno}</span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>
              {:else}
                <input class="input text-xs bg-gray-900 text-gray-500" value={riga.descrizione} disabled />
              {/if}
              {#if re.descrizione}<p class="text-xs text-red-400 mt-0.5">{re.descrizione}</p>{/if}
            </div>
            <!-- Quantità -->
            <div class="col-span-1">
              <label class="label">Qtà</label>
              <input
                class="input text-xs {re.quantita ? 'border-red-500' : ''}"
                type="number" min="0.01" step="0.01"
                bind:value={riga.quantita}
                readonly={tipoDocumento === 'fattura_differita'}
              />
              {#if re.quantita}<p class="text-xs text-red-400 mt-0.5">{re.quantita}</p>{/if}
            </div>
            <!-- Prezzo -->
            <div class="col-span-2">
              <label class="label">Prezzo €</label>
              <input
                class="input text-xs {re.prezzo ? 'border-red-500' : ''}"
                type="number" min="0" step="0.01"
                bind:value={riga.prezzo_unitario}
                readonly={tipoDocumento === 'fattura_differita'}
              />
              {#if re.prezzo}<p class="text-xs text-red-400 mt-0.5">{re.prezzo}</p>{/if}
            </div>
            <!-- Sconto -->
            <div class="col-span-1">
              <label class="label">Sc. %</label>
              <input class="input text-xs" type="number" min="0" max="100"
                bind:value={riga.sconto_percentuale}
                readonly={tipoDocumento === 'fattura_differita'}
              />
            </div>
            <!-- IVA — nascosta per buono/preventivo -->
            {#if !senzaIva}
              <div class="col-span-1">
                <label class="label">IVA %</label>
                <input class="input text-xs" type="number" min="0"
                  bind:value={riga.iva_percentuale}
                  readonly={tipoDocumento === 'fattura_differita'}
                />
              </div>
            {/if}
            <!-- Totale + rimuovi -->
            <div class="{senzaIva ? 'col-span-2' : 'col-span-1'} flex flex-col items-end gap-1 pt-5">
              <p class="text-sm font-medium text-green-400">
                {formatCurrency(imponibileRiga(riga) * (1 + riga.iva_percentuale / 100))}
              </p>
              {#if tipoDocumento !== 'fattura_differita'}
                <button
                  class="text-red-400 hover:text-red-300 text-xs leading-none"
                  on:click={() => removeRiga(riga._id)}
                  title="Rimuovi riga"
                >✕</button>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <!-- Totali -->
      <div class="px-4 py-3 border-t border-gray-800 bg-gray-800/30 flex justify-end gap-6 text-sm">
        {#if !senzaIva && tipoDocumento !== 'vendita_banco'}
          <span class="text-gray-400">
            Imponibile: <span class="text-white font-medium">{formatCurrency(totaleImponibile)}</span>
          </span>
          <span class="text-gray-400">
            IVA: <span class="text-white font-medium">{formatCurrency(totaleIva)}</span>
          </span>
        {/if}
        <span class="text-gray-300 font-semibold">
          Totale: <span class="text-green-400 text-base font-bold">{formatCurrency(totaleDoc)}</span>
        </span>
      </div>
    {/if}
  </div>

  <!-- Nota informativa per tipi non fiscali -->
  {#if tipoDocumento === 'buono'}
    <p class="text-xs text-gray-500 px-1">
      Il Buono è un documento interno senza valore fiscale. Non genera scadenzario.
    </p>
  {:else if tipoDocumento === 'vendita_banco'}
    <p class="text-xs text-gray-500 px-1">
      Vendita Banco: documento fiscale immediato, nessuno scadenzario generato.
    </p>
  {/if}

  <!-- Azioni -->
  <div class="flex gap-2 items-center">
    <button class="btn-primary" on:click={salva} disabled={saving}>
      {saving ? 'Salvataggio...' : 'Salva documento'}
    </button>
    <button class="btn-secondary" on:click={() => currentView.set('documenti')}>Annulla</button>
    {#if touched && hasErrors}
      <p class="text-xs text-red-400">Correggi i campi evidenziati prima di procedere</p>
    {/if}
  </div>
</div>
