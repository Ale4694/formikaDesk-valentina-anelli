<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { clienti, ricambi, documenti, formatCurrency, setError, currentView } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovaRigaDocumento } from '../../lib/types'

  const dispatch = createEventDispatcher()

  let numero = ''
  let data = new Date().toISOString().split('T')[0]
  let clienteId: number | null = null
  let note = ''
  let saving = false
  let touched = false

  interface RigaUI extends NuovaRigaDocumento { _id: number }
  let righe: RigaUI[] = []
  let nextId = 0

  // --- Numero fattura automatico ---
  onMount(() => {
    const anno = new Date().getFullYear()
    const fattureAnno = $documenti.filter(d =>
      d.tipo_documento === 'fattura' && d.numero.startsWith(`${anno}/`)
    ).length
    numero = `${anno}/${String(fattureAnno + 1).padStart(3, '0')}`
  })

  // --- Validazione reattiva ---
  $: errNumero  = touched && !numero.trim() ? 'Numero obbligatorio' : ''
  $: errCliente = touched && !clienteId ? 'Seleziona un cliente' : ''
  $: errRighe   = touched
    ? righe.reduce((acc, r) => {
        const e: { descrizione?: string; quantita?: string; prezzo?: string } = {}
        if (!r.descrizione.trim()) e.descrizione = 'Descrizione obbligatoria'
        if (r.quantita <= 0)       e.quantita    = 'Deve essere > 0'
        if (r.prezzo_unitario <= 0) e.prezzo     = 'Deve essere > 0'
        if (Object.keys(e).length) acc[r._id] = e
        return acc
      }, {} as Record<number, { descrizione?: string; quantita?: string; prezzo?: string }>)
    : ({} as Record<number, { descrizione?: string; quantita?: string; prezzo?: string }>)

  $: hasErrors = !!errNumero || !!errCliente || Object.keys(errRighe).length > 0 || (touched && righe.length === 0)

  function addRiga() {
    righe = [...righe, {
      _id: nextId++, ricambio_id: null, descrizione: '',
      quantita: 1, prezzo_unitario: 0, sconto_percentuale: 0,
      iva_percentuale: 22, ordine: righe.length
    }]
  }

  function removeRiga(id: number) {
    righe = righe.filter(r => r._id !== id).map((r, i) => ({ ...r, ordine: i }))
  }

  function onRicambioChange(riga: RigaUI) {
    if (!riga.ricambio_id) {
      riga.descrizione = ''
      riga.prezzo_unitario = 0
      riga.iva_percentuale = 22
    } else {
      const r = $ricambi.find(x => x.id === riga.ricambio_id)
      if (r) {
        riga.descrizione    = r.descrizione
        riga.prezzo_unitario = r.prezzo_vendita
        riga.iva_percentuale = r.iva_percentuale
      }
    }
    righe = [...righe]
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
    // Aspetta un tick perché le reactive declarations si aggiornino
    await new Promise(r => setTimeout(r, 0))
    if (righe.length === 0) { setError('Aggiungi almeno una riga'); return }
    if (hasErrors) return
    saving = true
    try {
      const result = await api.documenti.create({
        tipo_documento: 'fattura', numero, data,
        cliente_id: clienteId, note: note || null,
        righe: righe.map(({ _id, ...r }) => r)
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
    <h1 class="text-xl font-semibold text-white">Nuova fattura</h1>
    <button class="btn-secondary" on:click={() => currentView.set('documenti')}>← Torna</button>
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
          placeholder="2026/001"
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
        <label class="label">Cliente *</label>
        <select
          class="input {errCliente ? 'border-red-500 focus:ring-red-500' : ''}"
          bind:value={clienteId}
        >
          <option value={null}>— Seleziona cliente —</option>
          {#each $clienti as c}<option value={c.id}>{c.ragione_sociale}</option>{/each}
        </select>
        {#if errCliente}<p class="text-xs text-red-400 mt-1">{errCliente}</p>{/if}
      </div>
    </div>

    <div>
      <label class="label">Note</label>
      <textarea class="input resize-none" rows="2" bind:value={note}></textarea>
    </div>
  </div>

  <!-- Righe documento -->
  <div class="card overflow-hidden">
    <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
      <h2 class="text-sm font-semibold text-white">Righe documento</h2>
      <button class="btn-secondary text-xs" on:click={addRiga}>+ Aggiungi riga</button>
    </div>

    {#if righe.length === 0}
      <div class="flex flex-col items-center gap-3 py-12 text-gray-600">
        <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
            d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
        </svg>
        <p class="text-sm {touched ? 'text-red-400' : ''}">
          {touched ? 'Aggiungi almeno una riga' : 'Nessuna riga — clicca "+ Aggiungi riga"'}
        </p>
      </div>
    {:else}
      <div class="divide-y divide-gray-800">
        {#each righe as riga (riga._id)}
          {@const re = errRighe[riga._id] ?? {}}
          <div class="p-4 grid grid-cols-12 gap-2 items-start">
            <!-- Ricambio -->
            <div class="col-span-3">
              <label class="label">Ricambio</label>
              <select
                class="input text-xs"
                bind:value={riga.ricambio_id}
                on:change={() => onRicambioChange(riga)}
              >
                <option value={null}>— Descrizione libera —</option>
                {#each $ricambi as r}
                  <option value={r.id}>{r.codice_interno} — {r.descrizione}</option>
                {/each}
              </select>
            </div>
            <!-- Descrizione -->
            <div class="col-span-3">
              <label class="label">Descrizione</label>
              <input
                class="input text-xs {re.descrizione ? 'border-red-500' : ''}"
                bind:value={riga.descrizione}
                placeholder="Descrizione..."
              />
              {#if re.descrizione}<p class="text-xs text-red-400 mt-0.5">{re.descrizione}</p>{/if}
            </div>
            <!-- Quantità -->
            <div class="col-span-1">
              <label class="label">Qtà</label>
              <input
                class="input text-xs {re.quantita ? 'border-red-500' : ''}"
                type="number" min="0.01" step="0.01"
                bind:value={riga.quantita}
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
              />
              {#if re.prezzo}<p class="text-xs text-red-400 mt-0.5">{re.prezzo}</p>{/if}
            </div>
            <!-- Sconto -->
            <div class="col-span-1">
              <label class="label">Sc. %</label>
              <input class="input text-xs" type="number" min="0" max="100" bind:value={riga.sconto_percentuale}/>
            </div>
            <!-- IVA -->
            <div class="col-span-1">
              <label class="label">IVA %</label>
              <input class="input text-xs" type="number" min="0" bind:value={riga.iva_percentuale}/>
            </div>
            <!-- Totale + rimuovi -->
            <div class="col-span-1 flex flex-col items-end gap-1 pt-5">
              <p class="text-sm font-medium text-green-400">
                {formatCurrency(imponibileRiga(riga) * (1 + riga.iva_percentuale / 100))}
              </p>
              <button
                class="text-red-400 hover:text-red-300 text-xs leading-none"
                on:click={() => removeRiga(riga._id)}
                title="Rimuovi riga"
              >✕</button>
            </div>
          </div>
        {/each}
      </div>

      <!-- Totali -->
      <div class="px-4 py-3 border-t border-gray-800 bg-gray-800/30 flex justify-end gap-6 text-sm">
        <span class="text-gray-400">
          Imponibile: <span class="text-white font-medium">{formatCurrency(totaleImponibile)}</span>
        </span>
        <span class="text-gray-400">
          IVA: <span class="text-white font-medium">{formatCurrency(totaleIva)}</span>
        </span>
        <span class="text-gray-300 font-semibold">
          Totale: <span class="text-green-400 text-base font-bold">{formatCurrency(totaleDoc)}</span>
        </span>
      </div>
    {/if}
  </div>

  <!-- Azioni -->
  <div class="flex gap-2 items-center">
    <button class="btn-primary" on:click={salva} disabled={saving}>
      {saving ? 'Salvataggio...' : 'Salva fattura'}
    </button>
    <button class="btn-secondary" on:click={() => currentView.set('documenti')}>Annulla</button>
    {#if touched && hasErrors}
      <p class="text-xs text-red-400">Correggi i campi evidenziati prima di procedere</p>
    {/if}
  </div>
</div>
