<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { clienti, documenti, formatCurrency, setError, currentView } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovaRigaDocumento, TipoDocumento, Ricambio, ArticoloStorico } from '../../lib/types'

  const dispatch = createEventDispatcher()

  let tipoDocumento: TipoDocumento = 'fattura'
  let numero = ''
  let data = new Date().toISOString().split('T')[0]
  let clienteId: number | null = null
  let note = ''
  let saving = false
  let touched = false

  // Fattura differita — DDT selezionati
  let ddtSelezionati: number[] = []
  let caricandoDdt = false

  interface RigaUI extends NuovaRigaDocumento { _id: number }
  let righe: RigaUI[] = []
  let nextId = 0

  // Typeahead per selezione articolo per riga
  interface TAState { query: string; results: Ricambio[]; open: boolean }
  let righeTA: Record<number, TAState> = {}
  let taTimers: Record<number, ReturnType<typeof setTimeout>> = {}
  let righeDescTA: Record<number, TAState> = {}
  let descTaTimers: Record<number, ReturnType<typeof setTimeout>> = {}

  function getTa(id: number): TAState {
    return righeTA[id] ?? { query: '', results: [], open: false }
  }

  function setTa(id: number, patch: Partial<TAState>) {
    righeTA = { ...righeTA, [id]: { ...getTa(id), ...patch } }
  }

  function getDescTa(id: number): TAState {
    return righeDescTA[id] ?? { query: '', results: [], open: false }
  }

  function setDescTa(id: number, patch: Partial<TAState>) {
    righeDescTA = { ...righeDescTA, [id]: { ...getDescTa(id), ...patch } }
  }

  async function onTaInput(riga: RigaUI, value: string) {
    setTa(riga._id, { query: value, open: !!value.trim() })
    clearTimeout(taTimers[riga._id])
    if (!value.trim()) { setTa(riga._id, { results: [] }); return }
    taTimers[riga._id] = setTimeout(async () => {
      try {
        const results = (await api.ricambi.search(value)).slice(0, 10)
        setTa(riga._id, { results, open: true })
      } catch {}
    }, 300)
  }

  async function onDescTaInput(riga: RigaUI, value: string) {
    setDescTa(riga._id, { query: value, open: !!value.trim() })
    riga.descrizione = value
    righe = [...righe]
    clearTimeout(descTaTimers[riga._id])
    if (!value.trim()) { setDescTa(riga._id, { results: [] }); return }
    descTaTimers[riga._id] = setTimeout(async () => {
      try {
        const results = (await api.ricambi.search(value)).slice(0, 10)
        setDescTa(riga._id, { results, open: true })
      } catch {}
    }, 300)
  }

  function selectTa(riga: RigaUI, r: Ricambio) {
    riga.ricambio_id = r.id
    riga.descrizione = r.descrizione
    riga.prezzo_unitario = r.prezzo_vendita
    riga.iva_percentuale = r.iva_percentuale
    righe = [...righe]
    setTa(riga._id, { query: `${r.codice_interno} — ${r.descrizione}`, open: false, results: [] })
    setDescTa(riga._id, { query: r.descrizione, open: false, results: [] })
  }

  function selectDescTa(riga: RigaUI, r: Ricambio) {
    riga.ricambio_id = r.id
    riga.descrizione = r.descrizione
    riga.prezzo_unitario = r.prezzo_vendita
    riga.iva_percentuale = r.iva_percentuale
    righe = [...righe]
    setDescTa(riga._id, { query: r.descrizione, open: false, results: [] })
    setTa(riga._id, { query: `${r.codice_interno} — ${r.descrizione}`, open: false, results: [] })
  }

  function clearTa(riga: RigaUI) {
    riga.ricambio_id = null
    riga.descrizione = ''
    riga.prezzo_unitario = 0
    riga.iva_percentuale = 22
    righe = [...righe]
    setTa(riga._id, { query: '', results: [], open: false })
    setDescTa(riga._id, { query: '', results: [], open: false })
  }

  function clearDescTa(riga: RigaUI) {
    riga.ricambio_id = null
    riga.descrizione = ''
    riga.prezzo_unitario = 0
    riga.iva_percentuale = 22
    righe = [...righe]
    setDescTa(riga._id, { query: '', results: [], open: false })
    setTa(riga._id, { query: '', results: [], open: false })
  }

  // Storico acquisti cliente
  let storicoCliente: ArticoloStorico[] = []
  let loadingStorico = false

  $: {
    if (clienteId && tipoDocumento !== 'fattura_differita') {
      caricaStorico(clienteId)
    } else {
      storicoCliente = []
    }
  }

  async function caricaStorico(id: number) {
    loadingStorico = true
    try {
      storicoCliente = await api.documenti.getStoricoCliente(id)
    } catch {
      storicoCliente = []
    } finally {
      loadingStorico = false
    }
  }

  function aggiungiDaStorico(art: ArticoloStorico) {
    const id = nextId++
    righeTA = { ...righeTA, [id]: { query: `${art.codice_interno} — ${art.descrizione}`, results: [], open: false } }
    righeDescTA = { ...righeDescTA, [id]: { query: art.descrizione, results: [], open: false } }
    righe = [...righe, {
      _id: id,
      ricambio_id: art.ricambio_id,
      descrizione: art.descrizione,
      quantita: 1,
      prezzo_unitario: art.prezzo_unitario,
      sconto_percentuale: 0,
      iva_percentuale: 22,
      ordine: righe.length,
    }]
  }

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
    const anno = new Date().getFullYear()
    if (tipo === 'fattura') {
      const n = $documenti.filter(d => d.tipo_documento === 'fattura' && d.numero.startsWith(`${anno}/`)).length
      return `${anno}/${String(n + 1).padStart(3, '0')}`
    }
    const prefissi: Partial<Record<TipoDocumento, string>> = {
      ddt:               `DDT-${anno}-`,
      preventivo:        `PREV-${anno}-`,
      nota_credito:      `NC-${anno}-`,
      vendita_banco:     `VB-${anno}-`,
      buono:             `BUO-${anno}-`,
      fattura_differita: `FAT-${anno}-`,
    }
    const pref = prefissi[tipo] ?? `${anno}-`
    const n = $documenti.filter(d => d.tipo_documento === tipo && d.numero.startsWith(pref)).length
    return `${pref}${String(n + 1).padStart(3, '0')}`
  }

  onMount(() => { numero = computeNumero(tipoDocumento) })

  function onTipoChange() {
    numero = computeNumero(tipoDocumento)
    ddtSelezionati = []
    righe = []
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
  $: errCliente = touched && tipoDocumento !== 'preventivo' && tipoDocumento !== 'buono' && !clienteId
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
    righeTA = { ...righeTA, [id]: { query: '', results: [], open: false } }
    righeDescTA = { ...righeDescTA, [id]: { query: '', results: [], open: false } }
    righe = [...righe, {
      _id: id, ricambio_id: null, descrizione: '',
      quantita: 1, prezzo_unitario: 0, sconto_percentuale: 0,
      iva_percentuale: 22, ordine: righe.length
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
      const result = await api.documenti.create({
        tipo_documento: tipoDocumento,
        numero,
        data,
        cliente_id: clienteId,
        note: note || null,
        ddt_collegati: tipoDocumento === 'fattura_differita' && ddtSelezionati.length > 0
          ? ddtSelezionati
          : null,
        righe: righe.map(({ _id, ...r }) => r),
      })
      documenti.update(list => [result.documento, ...list])
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
    <h1 class="text-xl font-semibold text-white">Nuovo documento</h1>
    <button class="btn-secondary" on:click={() => currentView.set('documenti')}>← Torna</button>
  </div>

  <!-- Tipo documento -->
  <div class="card p-4">
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

      <!-- Cliente -->
      <div class="col-span-2">
        <label class="label">
          Cliente
          {#if tipoDocumento !== 'preventivo' && tipoDocumento !== 'buono'}*{/if}
        </label>
        {#if tipoDocumento === 'vendita_banco'}
          <input
            class="input bg-gray-900 text-gray-500 cursor-not-allowed"
            value="CLIENTE AL BANCO"
            disabled
          />
        {:else}
          <select
            class="input {errCliente ? 'border-red-500 focus:ring-red-500' : ''}"
            bind:value={clienteId}
          >
            <option value={null}>— Seleziona cliente —</option>
            {#each $clienti as c}<option value={c.id}>{c.ragione_sociale}</option>{/each}
          </select>
          {#if errCliente}<p class="text-xs text-red-400 mt-1">{errCliente}</p>{/if}
        {/if}
      </div>
    </div>

    <div>
      <label class="label">Note</label>
      <textarea class="input resize-none" rows="2" bind:value={note}></textarea>
    </div>
  </div>

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
  <div class="card overflow-hidden">
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
                  {#if getTa(riga._id).open && getTa(riga._id).results.length > 0}
                    <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-48 overflow-y-auto">
                      {#each getTa(riga._id).results as r (r.id)}
                        <button
                          class="w-full text-left px-3 py-2 hover:bg-gray-700 flex flex-col gap-0.5"
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
                  {#if getDescTa(riga._id).open && getDescTa(riga._id).results.length > 0}
                    <div class="absolute z-50 top-full left-0 right-0 mt-0.5 bg-gray-800 border border-gray-700 rounded-lg shadow-xl max-h-48 overflow-y-auto">
                      {#each getDescTa(riga._id).results as r (r.id)}
                        <button
                          class="w-full text-left px-3 py-2 hover:bg-gray-700 flex flex-col gap-0.5"
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
            <!-- IVA -->
            <div class="col-span-1">
              <label class="label">IVA %</label>
              <input class="input text-xs" type="number" min="0"
                bind:value={riga.iva_percentuale}
                readonly={tipoDocumento === 'fattura_differita'}
              />
            </div>
            <!-- Totale + rimuovi -->
            <div class="col-span-1 flex flex-col items-end gap-1 pt-5">
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
        {#if tipoDocumento !== 'buono' && tipoDocumento !== 'vendita_banco'}
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

  <!-- Storico acquisti cliente -->
  {#if storicoCliente.length > 0 && tipoDocumento !== 'fattura_differita'}
    <div class="card p-4 space-y-2">
      <h2 class="text-sm font-semibold text-white">Acquisti precedenti di questo cliente</h2>
      <div class="divide-y divide-gray-800">
        {#each storicoCliente as art}
          <div class="py-2 flex items-center gap-3">
            <div class="flex-1 min-w-0">
              <span class="font-mono text-brand-400 text-xs">{art.codice_interno}</span>
              <span class="text-gray-300 text-xs ml-2 truncate">{art.descrizione}</span>
            </div>
            <div class="flex items-center gap-3 text-xs text-gray-500 shrink-0">
              <span title="Volte acquistato">×{art.frequenza}</span>
              <span>{formatCurrency(art.prezzo_unitario)}</span>
            </div>
            <button
              class="btn-secondary text-xs shrink-0"
              on:click={() => aggiungiDaStorico(art)}
            >+ Aggiungi</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

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
