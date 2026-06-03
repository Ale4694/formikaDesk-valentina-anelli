<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'
  import { fornitori, ricambi, setError, setSuccess, formatCurrency } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { OrdineCompleto, NuovoOrdineFornitore, NuovaRigaOrdine } from '../../lib/types'

  const dispatch = createEventDispatcher()

  let ordini: OrdineCompleto[] = []
  let loading = false
  let filtroStato = 'tutti'
  let showForm = false

  // Form nuovo ordine
  let formFornitoreId = 0
  let formData = new Date().toISOString().slice(0, 10)
  let formNote = ''
  let formRighe: (NuovaRigaOrdine & { _ricambioQuery: string })[] = []
  let formLoading = false
  let formError = ''

  const statiOrdine = ['tutti', 'bozza', 'inviato', 'ricevuto', 'annullato']

  $: ordiniFiltered = filtroStato === 'tutti'
    ? ordini
    : ordini.filter(o => o.ordine.stato === filtroStato)

  onMount(loadOrdini)

  async function loadOrdini() {
    loading = true
    try {
      ordini = await api.ordini.getAll()
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento ordini')
    } finally {
      loading = false
    }
  }

  function nuovoOrdine() {
    formFornitoreId = $fornitori[0]?.id ?? 0
    formData = new Date().toISOString().slice(0, 10)
    formNote = ''
    formRighe = [{ ricambio_id: null, descrizione: '', quantita: 1, prezzo_unitario: 0, _ricambioQuery: '' }]
    formError = ''
    showForm = true
  }

  function addRiga() {
    formRighe = [...formRighe, { ricambio_id: null, descrizione: '', quantita: 1, prezzo_unitario: 0, _ricambioQuery: '' }]
  }

  function removeRiga(i: number) {
    formRighe = formRighe.filter((_, idx) => idx !== i)
  }

  function selectRicambio(i: number, ricambioId: number) {
    const r = $ricambi.find(x => x.id === ricambioId)
    if (r) {
      formRighe[i] = { ...formRighe[i], ricambio_id: r.id, descrizione: r.descrizione, prezzo_unitario: r.prezzo_acquisto, _ricambioQuery: r.descrizione }
      formRighe = [...formRighe]
    }
  }

  $: totaleForm = formRighe.reduce((s, r) => s + r.quantita * r.prezzo_unitario, 0)

  async function creaOrdine() {
    if (!formFornitoreId) { formError = 'Seleziona un fornitore'; return }
    if (formRighe.some(r => !r.descrizione.trim())) { formError = 'Descrizione obbligatoria per ogni riga'; return }
    formError = ''
    formLoading = true
    try {
      const payload: NuovoOrdineFornitore = {
        fornitore_id: formFornitoreId,
        data: formData,
        note: formNote || null,
        righe: formRighe.map(({ _ricambioQuery: _, ...r }) => r),
      }
      const created = await api.ordini.create(payload)
      ordini = [created, ...ordini]
      showForm = false
      setSuccess('Ordine creato')
      dispatch('refresh')
    } catch (e: any) {
      formError = e?.message ?? 'Errore creazione ordine'
    } finally {
      formLoading = false
    }
  }

  async function segnaRicevuto(id: number) {
    try {
      const updated = await api.ordini.updateStato(id, 'ricevuto')
      ordini = ordini.map(o => o.ordine.id === id ? updated : o)
      setSuccess('Ordine segnato come ricevuto — giacenza aggiornata')
      dispatch('refresh')
    } catch (e: any) {
      setError(e?.message ?? 'Errore aggiornamento stato')
    }
  }

  async function aggiornaSato(id: number, stato: string) {
    try {
      const updated = await api.ordini.updateStato(id, stato)
      ordini = ordini.map(o => o.ordine.id === id ? updated : o)
    } catch (e: any) {
      setError(e?.message ?? 'Errore aggiornamento stato')
    }
  }

  function labelStato(s: string) {
    const map: Record<string, string> = { bozza: 'Bozza', inviato: 'Inviato', ricevuto: 'Ricevuto', annullato: 'Annullato' }
    return map[s] ?? s
  }

  function colorStato(s: string) {
    const map: Record<string, string> = {
      bozza: 'text-gray-400 bg-gray-800',
      inviato: 'text-yellow-400 bg-yellow-950/40',
      ricevuto: 'text-green-400 bg-green-950/40',
      annullato: 'text-red-400 bg-red-950/40',
    }
    return map[s] ?? 'text-gray-400 bg-gray-800'
  }

  function fornitoreName(id: number) {
    return $fornitori.find(f => f.id === id)?.ragione_sociale ?? `#${id}`
  }
</script>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Ordini Fornitore</h1>
    <button class="btn-primary text-sm" on:click={nuovoOrdine}>+ Nuovo ordine</button>
  </div>

  <!-- Filtro stato -->
  <div class="flex gap-1">
    {#each statiOrdine as s}
      <button
        class="px-3 py-1 rounded text-xs transition-colors {filtroStato === s ? 'bg-brand-600 text-white' : 'bg-gray-800 text-gray-400 hover:text-gray-200'}"
        on:click={() => filtroStato = s}
      >
        {s === 'tutti' ? 'Tutti' : labelStato(s)}
      </button>
    {/each}
  </div>

  {#if loading}
    <div class="flex justify-center py-12"><div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else if ordiniFiltered.length === 0}
    <div class="card p-8 text-center text-gray-500 text-sm">Nessun ordine trovato.</div>
  {:else}
    <div class="space-y-2">
      {#each ordiniFiltered as oc}
        {@const o = oc.ordine}
        <div class="card p-4">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-sm font-medium text-white">{fornitoreName(o.fornitore_id)}</span>
                <span class="text-xs px-2 py-0.5 rounded-full font-medium {colorStato(o.stato)}">{labelStato(o.stato)}</span>
              </div>
              <p class="text-xs text-gray-500">{new Date(o.data).toLocaleDateString('it-IT')} · {oc.righe.length} articoli · {formatCurrency(o.totale)}</p>
              {#if o.note}<p class="text-xs text-gray-600 mt-1 truncate">{o.note}</p>{/if}
              <!-- Righe -->
              {#if oc.righe.length > 0}
                <div class="mt-2 space-y-0.5">
                  {#each oc.righe as r}
                    <div class="text-xs text-gray-500 flex gap-2">
                      <span class="text-gray-400">{r.descrizione}</span>
                      <span>×{r.quantita}</span>
                      <span>{formatCurrency(r.prezzo_unitario)}</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
            <div class="flex items-center gap-2 shrink-0">
              {#if o.stato === 'bozza'}
                <button class="btn-xs" on:click={() => aggiornaSato(o.id, 'inviato')}>Segna inviato</button>
              {/if}
              {#if o.stato === 'bozza' || o.stato === 'inviato'}
                <button class="btn-xs btn-success" on:click={() => segnaRicevuto(o.id)}>Segna ricevuto</button>
                <button class="btn-xs btn-danger" on:click={() => aggiornaSato(o.id, 'annullato')}>Annulla</button>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Modal nuovo ordine -->
{#if showForm}
  <div class="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
    <div class="bg-gray-900 border border-gray-800 rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col">
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-800">
        <h2 class="text-base font-semibold text-white">Nuovo ordine fornitore</h2>
        <button on:click={() => showForm = false} class="text-gray-500 hover:text-gray-300">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="label">Fornitore *</label>
            <select bind:value={formFornitoreId} class="input w-full">
              {#each $fornitori as f}
                <option value={f.id}>{f.ragione_sociale}</option>
              {/each}
            </select>
          </div>
          <div>
            <label class="label">Data *</label>
            <input type="date" bind:value={formData} class="input w-full"/>
          </div>
        </div>
        <div>
          <label class="label">Note</label>
          <input bind:value={formNote} class="input w-full" placeholder="Opzionale"/>
        </div>

        <!-- Righe -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm font-medium text-gray-300">Articoli</p>
            <button on:click={addRiga} class="text-xs text-brand-400 hover:text-brand-300">+ Aggiungi riga</button>
          </div>
          <div class="space-y-2">
            {#each formRighe as riga, i}
              <div class="bg-gray-800/50 rounded-lg p-3 space-y-2">
                <div class="grid grid-cols-4 gap-2 items-end">
                  <div class="col-span-2">
                    <label class="label-xs">Ricambio / Descrizione *</label>
                    <div class="relative">
                      <select
                        class="input w-full text-xs"
                        on:change={(e) => { const v = parseInt((e.target as HTMLSelectElement).value); if (v) selectRicambio(i, v) }}
                      >
                        <option value="">— Seleziona ricambio —</option>
                        {#each $ricambi as r}
                          <option value={r.id} selected={riga.ricambio_id === r.id}>{r.codice_interno} — {r.descrizione}</option>
                        {/each}
                      </select>
                    </div>
                    <input bind:value={riga.descrizione} class="input w-full text-xs mt-1" placeholder="Descrizione libera"/>
                  </div>
                  <div>
                    <label class="label-xs">Qtà</label>
                    <input bind:value={riga.quantita} type="number" min="0.01" step="1" class="input w-full text-xs"/>
                  </div>
                  <div>
                    <label class="label-xs">Prezzo acquisto</label>
                    <input bind:value={riga.prezzo_unitario} type="number" min="0" step="0.01" class="input w-full text-xs"/>
                  </div>
                </div>
                {#if formRighe.length > 1}
                  <button on:click={() => removeRiga(i)} class="text-xs text-red-400 hover:text-red-300">Rimuovi riga</button>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="px-5 py-4 border-t border-gray-800">
        <div class="flex items-center justify-between">
          <div class="text-sm text-gray-400">Totale: <span class="text-white font-semibold">{formatCurrency(totaleForm)}</span></div>
          <div class="flex items-center gap-2">
            {#if formError}<p class="text-xs text-red-400">{formError}</p>{/if}
            <button on:click={() => showForm = false} class="btn-secondary text-sm">Annulla</button>
            <button on:click={creaOrdine} disabled={formLoading} class="btn-primary text-sm">
              {formLoading ? 'Salvataggio…' : 'Crea ordine'}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .label { @apply block text-xs font-medium text-gray-400 mb-1; }
  .label-xs { @apply block text-xs text-gray-500 mb-0.5; }
  .input { @apply bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-brand-500; }
  .btn-xs { @apply text-xs px-2.5 py-1 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 transition-colors; }
  .btn-xs.btn-success { @apply bg-green-700 hover:bg-green-600 text-white; }
  .btn-xs.btn-danger { @apply bg-red-900 hover:bg-red-800 text-red-200; }
</style>
