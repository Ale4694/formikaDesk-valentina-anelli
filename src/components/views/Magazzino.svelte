<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte'
  import { ricambi, fornitori, formatCurrency, setError, setSuccess, appConfig } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovoRicambio, Ricambio } from '../../lib/types'
  import ConfirmModal from '../ConfirmModal.svelte'

  type SortDir = 'asc' | 'desc'
  let sortCol = 'descrizione'
  let sortDir: SortDir = 'asc'

  function toggleSort(col: string) {
    if (sortCol === col) { sortDir = sortDir === 'asc' ? 'desc' : 'asc' }
    else { sortCol = col; sortDir = 'asc' }
  }

  function sortVal(r: (typeof $ricambi)[0], col: string): string | number {
    const v = (r as any)[col]
    return v === null || v === undefined ? (sortDir === 'asc' ? '￿' : '') : v
  }

  function downloadCsv(content: string, filename: string) {
    const blob = new Blob(['﻿' + content], { type: 'text/csv;charset=utf-8;' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = filename; a.click()
    URL.revokeObjectURL(url)
  }

  async function esportaCsv() {
    try {
      const csv = await api.backup.exportRicambiCsv()
      const today = new Date().toISOString().slice(0, 10)
      downloadCsv(csv, `ricambi_${today}.csv`)
      setSuccess('CSV ricambi esportato')
    } catch (e: any) { setError(e?.message ?? 'Errore export') }
  }

  const dispatch = createEventDispatcher()

  let search = ''
  let showForm = false
  let saving = false
  let editId: number | null = null

  // Modal conferma eliminazione
  let confirmOpen = false
  let confirmMessage = ''
  let pendingDeleteId: number | null = null

  // Carico inline form state
  let caricoId: number | null = null
  let caricoQty = 1
  let caricoNote = ''
  let caricoSaving = false

  // Carico rapido barcode
  let showCaricoRapido = false
  let barcodeInput = ''
  let barcodeFound: Ricambio | null = null
  let barcodeNotFound = false
  let caricoRapidoQty = 1
  let confirmMsg: string | null = null
  let barcodeLoading = false
  let barcodeEl: HTMLInputElement
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  const emptyForm = (): NuovoRicambio => ({
    codice_interno: '', codice_oem: '', descrizione: '', marca: '',
    modello_auto: '', anno_da: null, anno_a: null, categoria: '',
    fornitore_id: null, giacenza: 0, giacenza_minima: 1,
    prezzo_acquisto: 0, prezzo_vendita: 0, iva_percentuale: 22, posizione: '', note: ''
  })

  let form = emptyForm()

  $: filtered = search.trim()
    ? $ricambi.filter(r =>
        r.descrizione.toLowerCase().includes(search.toLowerCase()) ||
        r.codice_interno.toLowerCase().includes(search.toLowerCase()) ||
        (r.codice_oem ?? '').toLowerCase().includes(search.toLowerCase())
      )
    : $ricambi

  $: vocRicambio   = $appConfig?.vocabolario?.ricambio    ?? 'Ricambio'
  $: vocRicambi    = $appConfig?.vocabolario?.ricambi     ?? 'Ricambi'
  $: vocCodiceOem  = $appConfig?.vocabolario?.codice_oem  ?? 'Codice OEM'
  $: vocModelloAuto= $appConfig?.vocabolario?.modello_auto ?? 'Modello auto'
  $: vocMarca      = $appConfig?.vocabolario?.marca        ?? 'Marca'
  $: vocCategoria  = $appConfig?.vocabolario?.categoria    ?? 'Categoria'
  $: vocPosizione  = $appConfig?.vocabolario?.posizione    ?? 'Posizione'

  $: columns = [
    { col: 'codice_interno',  label: 'Codice',        align: 'left'  },
    { col: 'descrizione',     label: 'Descrizione',   align: 'left'  },
    { col: 'codice_oem',      label: vocCodiceOem,    align: 'left'  },
    { col: 'giacenza',        label: 'Giacenza',      align: 'right' },
    { col: 'prezzo_acquisto', label: 'P. Acquisto',   align: 'right' },
    { col: 'prezzo_vendita',  label: 'P. Vendita',    align: 'right' },
    { col: 'posizione',       label: vocPosizione,    align: 'left'  },
  ]

  $: sorted = [...filtered].sort((a, b) => {
    const av = sortVal(a, sortCol)
    const bv = sortVal(b, sortCol)
    if (typeof av === 'number' && typeof bv === 'number') return sortDir === 'asc' ? av - bv : bv - av
    const cmp = String(av).localeCompare(String(bv), 'it')
    return sortDir === 'asc' ? cmp : -cmp
  })

  async function openCaricoRapido() {
    showCaricoRapido = true
    barcodeInput = ''
    barcodeFound = null
    barcodeNotFound = false
    confirmMsg = null
    caricoRapidoQty = 1
    await tick()
    barcodeEl?.focus()
  }

  function closeCaricoRapido() {
    if (debounceTimer) clearTimeout(debounceTimer)
    showCaricoRapido = false
    barcodeInput = ''
    barcodeFound = null
    barcodeNotFound = false
    confirmMsg = null
  }

  function cercaBarcodeLocale() {
    const codice = barcodeInput.trim().toLowerCase()
    if (!codice) { barcodeFound = null; barcodeNotFound = false; return }
    barcodeFound = $ricambi.find(r =>
      r.codice_interno.toLowerCase() === codice ||
      (r.codice_oem ?? '').toLowerCase() === codice
    ) ?? null
    barcodeNotFound = barcodeFound === null
    caricoRapidoQty = 1
    confirmMsg = null
  }

  function onBarcodeKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (debounceTimer) clearTimeout(debounceTimer)
      cercaBarcodeLocale()
    }
  }

  function onBarcodeInput() {
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(cercaBarcodeLocale, 300)
  }

  async function eseguiCaricoRapido() {
    if (!barcodeFound) return
    barcodeLoading = true
    try {
      const result = await api.magazzino.caricoRapido(barcodeInput.trim(), caricoRapidoQty)
      if (result.trovato && result.ricambio) {
        ricambi.update(list => list.map(r => r.id === result.ricambio!.id ? result.ricambio! : r))
        confirmMsg = `✅ Aggiunto: ${result.ricambio.descrizione} — nuova giacenza: ${result.nuova_giacenza}`
        dispatch('refresh')
      }
      barcodeFound = null
      barcodeNotFound = false
      barcodeInput = ''
      await tick()
      barcodeEl?.focus()
    } catch (e: any) {
      setError(e?.message ?? 'Errore carico rapido')
    } finally {
      barcodeLoading = false
    }
  }

  function apriCreazioneDaBarcode() {
    const codice = barcodeInput.trim()
    closeCaricoRapido()
    cancelForm()
    form.codice_interno = codice
    showForm = true
  }

  function startEdit(r: (typeof $ricambi)[0]) {
    editId = r.id
    form = { ...r }
    showForm = true
    caricoId = null
  }

  function cancelForm() {
    showForm = false
    editId = null
    form = emptyForm()
  }

  function openCarico(id: number) {
    caricoId = caricoId === id ? null : id
    caricoQty = 1
    caricoNote = ''
  }

  async function confermaCarico(ricambioId: number) {
    if (caricoQty <= 0) { setError('Quantità deve essere maggiore di 0'); return }
    caricoSaving = true
    try {
      const updated = await api.ricambi.aggiornaGiacenza(ricambioId, 'carico', caricoQty, null, caricoNote || null)
      ricambi.update(list => list.map(r => r.id === ricambioId ? updated : r))
      caricoId = null
      caricoQty = 1
      caricoNote = ''
      dispatch('refresh')
    } catch (e: any) {
      setError(e?.message ?? 'Errore carico')
    } finally {
      caricoSaving = false
    }
  }

  async function save() {
    if (!form.codice_interno.trim() || !form.descrizione.trim()) return
    saving = true
    try {
      if (editId) {
        const updated = await api.ricambi.update(editId, form)
        ricambi.update(list => list.map(r => r.id === editId ? updated : r))
      } else {
        const nuovo = await api.ricambi.create(form)
        ricambi.update(list => [...list, nuovo])
      }
      cancelForm()
      dispatch('refresh')
    } catch (e: any) {
      setError(e?.message ?? 'Errore salvataggio')
    } finally {
      saving = false
    }
  }

  function richiediElimina(id: number, descrizione: string) {
    pendingDeleteId = id
    confirmMessage = `Eliminare "${descrizione}"? L'operazione non può essere annullata.`
    confirmOpen = true
  }

  async function eseguiElimina() {
    if (pendingDeleteId === null) return
    const id = pendingDeleteId
    confirmOpen = false
    pendingDeleteId = null
    try {
      await api.ricambi.delete(id)
      ricambi.update(list => list.filter(r => r.id !== id))
      dispatch('refresh')
    } catch (e: any) {
      setError(e?.message ?? 'Errore eliminazione')
    }
  }
</script>

<ConfirmModal
  bind:open={confirmOpen}
  title="Elimina ricambio"
  message={confirmMessage}
  onConfirm={eseguiElimina}
  onCancel={() => { confirmOpen = false; pendingDeleteId = null }}
/>

{#if showCaricoRapido}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    on:click|self={closeCaricoRapido}
    role="dialog"
    aria-modal="true"
  >
    <div class="card w-full max-w-md max-h-[90vh] overflow-y-auto p-6 space-y-4 shadow-2xl">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-white">📦 Carico rapido barcode</h2>
        <button class="text-gray-500 hover:text-gray-300 text-lg leading-none" on:click={closeCaricoRapido}>✕</button>
      </div>

      <div>
        <label class="label">Scansiona o digita codice</label>
        <input
          bind:this={barcodeEl}
          bind:value={barcodeInput}
          class="input font-mono"
          placeholder="Codice interno o OEM..."
          on:input={onBarcodeInput}
          on:keydown={onBarcodeKeydown}
          autocomplete="off"
          spellcheck="false"
        />
      </div>

      {#if confirmMsg}
        <div class="rounded-lg bg-green-950/40 border border-green-800/50 px-4 py-3 text-green-300 text-sm">
          {confirmMsg}
        </div>
      {/if}

      {#if barcodeFound}
        <div class="rounded-lg bg-gray-800/60 border border-gray-700 px-4 py-3 space-y-3">
          <div>
            <p class="text-white font-medium text-sm">{barcodeFound.descrizione}</p>
            {#if barcodeFound.marca}
              <p class="text-gray-400 text-xs">{barcodeFound.marca}</p>
            {/if}
            <p class="text-xs text-gray-500 font-mono mt-1">{barcodeFound.codice_interno}{barcodeFound.codice_oem ? ` · ${barcodeFound.codice_oem}` : ''}</p>
          </div>
          <div class="flex items-center gap-3">
            <span class="text-xs text-gray-400">Giacenza attuale:</span>
            <span class="text-sm font-semibold {barcodeFound.giacenza < barcodeFound.giacenza_minima ? 'text-red-400' : 'text-gray-200'}">{barcodeFound.giacenza}</span>
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs text-gray-400 shrink-0">Quantità da aggiungere</label>
            <input
              class="input w-24 text-sm py-1"
              type="number"
              min="1"
              bind:value={caricoRapidoQty}
            />
            <button
              class="btn-primary text-sm px-4 py-1.5 shrink-0"
              on:click={eseguiCaricoRapido}
              disabled={barcodeLoading || caricoRapidoQty < 1}
            >
              {barcodeLoading ? '...' : 'Aggiungi al magazzino'}
            </button>
          </div>
        </div>
      {:else if barcodeNotFound}
        <div class="rounded-lg bg-yellow-950/30 border border-yellow-800/40 px-4 py-3 space-y-2">
          <p class="text-yellow-300 text-sm">Ricambio non trovato per "<span class="font-mono">{barcodeInput.trim()}</span>"</p>
          <button class="btn-secondary text-xs" on:click={apriCreazioneDaBarcode}>+ Crea nuovo ricambio</button>
        </div>
      {/if}

      <div class="pt-1 flex justify-end">
        <button class="btn-secondary text-xs" on:click={closeCaricoRapido}>Chiudi</button>
      </div>
    </div>
  </div>
{/if}

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Magazzino</h1>
    <div class="flex gap-2">
      <button class="btn-secondary text-xs" on:click={esportaCsv}>
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
        </svg>
        Esporta CSV
      </button>
      <button class="btn-secondary" on:click={openCaricoRapido}>📦 Carico rapido</button>
      <button class="btn-primary" on:click={() => { cancelForm(); showForm = true }}>+ Nuovo {vocRicambio.toLowerCase()}</button>
    </div>
  </div>

  {#if showForm}
    <div class="card p-5 space-y-4">
      <h2 class="text-sm font-semibold text-white">{editId ? 'Modifica' : 'Nuovo'} {vocRicambio.toLowerCase()}</h2>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div><label class="label">Codice interno *</label><input class="input" bind:value={form.codice_interno} placeholder="RI-0001"/></div>
        <div><label class="label">{vocCodiceOem}</label><input class="input" bind:value={form.codice_oem} placeholder="1234567890"/></div>
        <div class="col-span-2"><label class="label">Descrizione *</label><input class="input" bind:value={form.descrizione} placeholder="Filtro olio..."/></div>
        <div><label class="label">{vocMarca}</label><input class="input" bind:value={form.marca}/></div>
        <div><label class="label">{vocModelloAuto}</label><input class="input" bind:value={form.modello_auto}/></div>
        <div><label class="label">Anno da</label><input class="input" type="number" bind:value={form.anno_da}/></div>
        <div><label class="label">Anno a</label><input class="input" type="number" bind:value={form.anno_a}/></div>
        <div><label class="label">Giacenza</label><input class="input" type="number" bind:value={form.giacenza}/></div>
        <div><label class="label">Scorta minima</label><input class="input" type="number" bind:value={form.giacenza_minima}/></div>
        <div><label class="label">Prezzo acquisto (€)</label><input class="input" type="number" step="0.01" bind:value={form.prezzo_acquisto}/></div>
        <div><label class="label">Prezzo vendita (€)</label><input class="input" type="number" step="0.01" bind:value={form.prezzo_vendita}/></div>
        <div><label class="label">IVA %</label><input class="input" type="number" bind:value={form.iva_percentuale}/></div>
        <div><label class="label">{vocPosizione}</label><input class="input" bind:value={form.posizione} placeholder="Scaffale A1"/></div>
        <div><label class="label">{vocCategoria}</label><input class="input" bind:value={form.categoria}/></div>
        <div>
          <label class="label">Fornitore</label>
          <select class="input" bind:value={form.fornitore_id}>
            <option value={null}>— Nessuno —</option>
            {#each $fornitori as f}<option value={f.id}>{f.ragione_sociale}</option>{/each}
          </select>
        </div>
      </div>
      <div class="flex gap-2 pt-1">
        <button class="btn-primary" on:click={save} disabled={saving}>{saving ? 'Salvataggio...' : 'Salva'}</button>
        <button class="btn-secondary" on:click={cancelForm}>Annulla</button>
      </div>
    </div>
  {/if}

  <input class="input max-w-sm" placeholder="Cerca per descrizione, codice..." bind:value={search}/>

  <div class="card overflow-hidden">
    <table class="w-full text-sm">
      <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
        <tr>
          {#each columns as h}
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
        {#each sorted as r (r.id)}
          <tr class="transition-colors duration-100
            {r.giacenza < r.giacenza_minima
              ? 'bg-red-950/20 hover:bg-red-950/30'
              : 'hover:bg-gray-800/50'}">
            <td class="px-4 py-3 font-mono text-xs text-brand-400">{r.codice_interno}</td>
            <td class="px-4 py-3 text-gray-200">
              {r.descrizione}
              {#if r.marca}<span class="text-gray-500 text-xs ml-1">({r.marca})</span>{/if}
            </td>
            <td class="px-4 py-3 font-mono text-xs text-gray-400">{r.codice_oem ?? '—'}</td>
            <td class="px-4 py-3 text-right">
              {#if r.giacenza < r.giacenza_minima}
                <span class="badge-red">{r.giacenza}</span>
                <span class="text-red-500 text-xs ml-1">/ {r.giacenza_minima}</span>
              {:else}
                <span class="text-gray-200">{r.giacenza}</span>
                <span class="text-gray-600 text-xs ml-1">/ {r.giacenza_minima}</span>
              {/if}
            </td>
            <td class="px-4 py-3 text-right text-gray-400">{formatCurrency(r.prezzo_acquisto)}</td>
            <td class="px-4 py-3 text-right text-green-400">{formatCurrency(r.prezzo_vendita)}</td>
            <td class="px-4 py-3 text-gray-400 text-xs">{r.posizione ?? '—'}</td>
            <td class="px-4 py-3">
              <div class="flex gap-1 justify-end">
                <button
                  class="text-xs px-2 py-1 rounded-md font-medium transition-colors
                    {caricoId === r.id
                      ? 'bg-green-700 text-green-100'
                      : 'bg-green-900/40 hover:bg-green-800/60 text-green-400 border border-green-800/50'}"
                  on:click={() => openCarico(r.id)}
                >
                  ↑ Carico
                </button>
                <button class="btn-secondary text-xs px-2 py-1" on:click={() => startEdit(r)}>Modifica</button>
                <button class="btn-danger text-xs px-2 py-1" on:click={() => richiediElimina(r.id, r.descrizione)}>Elimina</button>
              </div>
            </td>
          </tr>
          {#if caricoId === r.id}
            <tr class="bg-green-950/20 border-b border-green-900/30">
              <td colspan="8" class="px-4 py-3">
                <div class="flex items-center gap-3">
                  <svg class="w-4 h-4 text-green-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"/>
                  </svg>
                  <span class="text-xs text-green-300 font-medium">Carico merce:</span>
                  <div class="flex items-center gap-1">
                    <label class="text-xs text-gray-400">Quantità</label>
                    <input
                      class="input w-20 text-xs py-1"
                      type="number"
                      min="1"
                      bind:value={caricoQty}
                    />
                  </div>
                  <div class="flex items-center gap-1 flex-1">
                    <label class="text-xs text-gray-400 shrink-0">Nota</label>
                    <input
                      class="input text-xs py-1"
                      placeholder="Opzionale..."
                      bind:value={caricoNote}
                    />
                  </div>
                  <button
                    class="btn-primary text-xs py-1 px-3 shrink-0"
                    on:click={() => confermaCarico(r.id)}
                    disabled={caricoSaving}
                  >
                    {caricoSaving ? '...' : 'Conferma'}
                  </button>
                  <button class="text-xs text-gray-500 hover:text-gray-300" on:click={() => caricoId = null}>✕</button>
                </div>
              </td>
            </tr>
          {/if}
        {:else}
          <tr>
            <td colspan="8" class="py-16 text-center">
              <div class="flex flex-col items-center gap-3 text-gray-600">
                <svg class="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                    d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
                </svg>
                <p class="text-sm">
                  {#if search.trim()}
                    Nessun {vocRicambio.toLowerCase()} corrisponde a "<span class="text-gray-400">{search}</span>"
                  {:else}
                    Magazzino vuoto — aggiungi il primo {vocRicambio.toLowerCase()}
                  {/if}
                </p>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
