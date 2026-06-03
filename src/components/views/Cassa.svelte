<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { api } from '../../lib/api'
  import { formatCurrency, setError } from '../../lib/stores'
  import type { NuovaRigaScontrino, Scontrino } from '../../lib/types'
  import PrintScontrino from '../PrintScontrino.svelte'

  // --- Carrello ---
  interface RigaCarrello extends NuovaRigaScontrino {
    _id: number
  }
  let carrello: RigaCarrello[] = []
  let nextId = 0

  // --- Barcode ---
  let codiceScan = ''
  let barcodeError = ''
  let barcodeInput: HTMLInputElement | undefined

  // --- Pagamento ---
  type Metodo = 'contanti' | 'carta' | 'satispay' | 'bonifico'
  let metodoPagamento: Metodo | null = null
  let importoRicevuto = 0
  let scontoTotalePerc = 0
  let operatore = ''
  let salvataggio = false

  // --- Stampa ---
  let printComp: PrintScontrino | undefined
  let printScontrino: import('../../lib/types').ScontrinoCompleto | null = null
  let importoRicevutoPrint = 0

  // --- Ultimi scontrini oggi ---
  let ultimiOggi: Scontrino[] = []

  onMount(async () => {
    await caricaUltimi()
    await tick()
    barcodeInput?.focus()
  })

  async function caricaUltimi() {
    try {
      const tutti = await api.cassa.getAll()
      const oggi = new Date().toISOString().slice(0, 10)
      ultimiOggi = tutti
        .filter(sc => sc.scontrino.data === oggi && sc.scontrino.stato === 'chiuso')
        .slice(0, 5)
        .map(sc => sc.scontrino)
    } catch {}
  }

  // --- Calcoli ---
  $: subtotaleRighe = carrello.reduce((s, r) => {
    const imp = r.quantita * r.prezzo_unitario * (1 - r.sconto_percentuale / 100)
    const iva = imp * r.iva_percentuale / 100
    return s + imp + iva
  }, 0)
  $: scontoEuro = subtotaleRighe * scontoTotalePerc / 100
  $: totaleFinale = Math.max(0, subtotaleRighe - scontoEuro)
  $: imponibile = carrello.reduce((s, r) => {
    const imp = r.quantita * r.prezzo_unitario * (1 - r.sconto_percentuale / 100)
    return s + imp
  }, 0) * (1 - scontoTotalePerc / 100)
  $: ivaTotale = totaleFinale - imponibile
  $: resto = metodoPagamento === 'contanti' && importoRicevuto > 0
    ? importoRicevuto - totaleFinale
    : null

  // --- Barcode ---
  async function handleBarcode(e: KeyboardEvent) {
    if (e.key !== 'Enter') return
    const cod = codiceScan.trim()
    if (!cod) return
    barcodeError = ''
    try {
      const r = await api.cassa.cercaBarcode(cod)
      if (!r) {
        barcodeError = `Codice "${cod}" non trovato`
        codiceScan = ''
        return
      }
      aggiungiAlCarrello({
        _id: nextId++,
        ricambio_id: r.id,
        codice: r.codice_interno,
        descrizione: r.descrizione,
        quantita: 1,
        prezzo_unitario: r.prezzo_vendita,
        sconto_percentuale: 0,
        iva_percentuale: r.iva_percentuale,
      })
      codiceScan = ''
    } catch (err: any) {
      barcodeError = err?.message ?? 'Errore ricerca'
    }
  }

  function aggiungiAlCarrello(riga: RigaCarrello) {
    // Incrementa se già presente
    const idx = carrello.findIndex(r => r.ricambio_id === riga.ricambio_id && riga.ricambio_id !== null)
    if (idx >= 0) {
      carrello[idx] = { ...carrello[idx], quantita: carrello[idx].quantita + 1 }
      carrello = [...carrello]
    } else {
      carrello = [...carrello, riga]
    }
    barcodeInput?.focus()
  }

  function rimuoviRiga(id: number) {
    carrello = carrello.filter(r => r._id !== id)
    barcodeInput?.focus()
  }

  function cambiaQuantita(id: number, delta: number) {
    carrello = carrello.map(r => {
      if (r._id !== id) return r
      const q = Math.max(0.1, r.quantita + delta)
      return { ...r, quantita: Math.round(q * 10) / 10 }
    }).filter(r => r.quantita > 0)
    barcodeInput?.focus()
  }

  function svuotaCarrello() {
    carrello = []
    codiceScan = ''
    barcodeError = ''
    scontoTotalePerc = 0
    metodoPagamento = null
    importoRicevuto = 0
    barcodeInput?.focus()
  }

  // --- Chiudi scontrino ---
  async function chiudiScontrino() {
    if (carrello.length === 0) { barcodeError = 'Carrello vuoto'; return }
    if (!metodoPagamento) { barcodeError = 'Seleziona un metodo di pagamento'; return }

    salvataggio = true
    barcodeError = ''
    try {
      const sc = await api.cassa.crea({
        righe: carrello.map(({ _id: _, ...r }) => r),
        metodo_pagamento: metodoPagamento,
        sconto_totale: scontoTotalePerc,
        operatore: operatore || null,
        note: null,
      })
      importoRicevutoPrint = importoRicevuto
      printScontrino = sc
      await tick()
      printComp?.stampa()
      await caricaUltimi()
      svuotaCarrello()
    } catch (err: any) {
      barcodeError = err?.message ?? 'Errore salvataggio scontrino'
    } finally {
      salvataggio = false
    }
  }

  function metodoIcon(m: Metodo) {
    return { contanti: '💵', carta: '💳', satispay: '📱', bonifico: '🏦' }[m]
  }
  function metodoLabel(m: Metodo) {
    return { contanti: 'Contanti', carta: 'Carta', satispay: 'Satispay', bonifico: 'Bonifico' }[m]
  }
</script>

<div class="flex h-full overflow-hidden">

  <!-- COLONNA SINISTRA: carrello -->
  <div class="flex-1 flex flex-col p-4 overflow-hidden border-r border-gray-800">

    <!-- Barcode input -->
    <div class="mb-3">
      <div class="relative">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 3.5V21H4v-5.5"/>
        </svg>
        <input
          bind:this={barcodeInput}
          bind:value={codiceScan}
          on:keydown={handleBarcode}
          placeholder="Scansiona o digita codice — premi Invio"
          class="input pl-9 text-sm font-mono"
          autocomplete="off"
          spellcheck="false"
        />
      </div>
      {#if barcodeError}
        <p class="mt-1 text-xs text-red-400">{barcodeError}</p>
      {/if}
    </div>

    <!-- Lista carrello -->
    <div class="flex-1 overflow-y-auto space-y-1 mb-3">
      {#if carrello.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-gray-600 select-none">
          <svg class="w-12 h-12 mb-2 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 11-4 0 2 2 0 014 0z"/>
          </svg>
          <p class="text-sm">Carrello vuoto</p>
        </div>
      {:else}
        {#each carrello as riga (riga._id)}
          {@const imp = riga.quantita * riga.prezzo_unitario * (1 - riga.sconto_percentuale / 100)}
          {@const tot = imp * (1 + riga.iva_percentuale / 100)}
          <div class="bg-gray-800/60 rounded-lg px-3 py-2 flex items-center gap-3 group">
            <div class="flex-1 min-w-0">
              <p class="text-sm text-gray-200 truncate">{riga.descrizione}</p>
              <p class="text-xs text-gray-500 font-mono">{riga.codice ?? ''} · {formatCurrency(riga.prezzo_unitario)} IVA {riga.iva_percentuale}%</p>
            </div>
            <!-- Quantità +/- -->
            <div class="flex items-center gap-1 shrink-0">
              <button
                on:click={() => cambiaQuantita(riga._id, -1)}
                class="w-6 h-6 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm flex items-center justify-center"
              >−</button>
              <span class="w-8 text-center text-sm text-white">{riga.quantita}</span>
              <button
                on:click={() => cambiaQuantita(riga._id, 1)}
                class="w-6 h-6 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm flex items-center justify-center"
              >+</button>
            </div>
            <span class="text-sm font-semibold text-white w-20 text-right shrink-0">{formatCurrency(tot)}</span>
            <button
              on:click={() => rimuoviRiga(riga._id)}
              class="text-gray-600 hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100"
              title="Rimuovi"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
        {/each}
      {/if}
    </div>

    <!-- Sconto totale -->
    <div class="flex items-center gap-2 mb-3">
      <label class="text-xs text-gray-400 whitespace-nowrap">Sconto %</label>
      <input
        type="number"
        bind:value={scontoTotalePerc}
        min="0" max="100" step="1"
        class="input w-20 text-sm text-center"
      />
      {#if scontoEuro > 0}
        <span class="text-xs text-yellow-400">−{formatCurrency(scontoEuro)}</span>
      {/if}
    </div>

    <!-- Totali -->
    <div class="border-t border-gray-800 pt-3 space-y-1">
      <div class="flex justify-between text-xs text-gray-500">
        <span>Imponibile</span><span>{formatCurrency(imponibile)}</span>
      </div>
      <div class="flex justify-between text-xs text-gray-500">
        <span>IVA</span><span>{formatCurrency(ivaTotale)}</span>
      </div>
      <div class="flex justify-between text-xl font-bold text-white pt-1">
        <span>TOTALE</span><span class="text-brand-400">{formatCurrency(totaleFinale)}</span>
      </div>
    </div>
  </div>

  <!-- COLONNA DESTRA: pagamento + azioni -->
  <div class="w-72 flex flex-col p-4 space-y-4 overflow-y-auto">

    <!-- Metodo pagamento -->
    <div>
      <p class="text-xs font-medium text-gray-400 mb-2">Metodo di pagamento</p>
      <div class="grid grid-cols-2 gap-2">
        {#each (['contanti', 'carta', 'satispay', 'bonifico'] as Metodo[]) as m}
          <button
            on:click={() => { metodoPagamento = m; barcodeInput?.focus() }}
            class="flex flex-col items-center justify-center py-3 rounded-xl border text-sm font-medium transition-all
              {metodoPagamento === m
                ? 'bg-brand-600 border-brand-500 text-white shadow-lg shadow-brand-900/40'
                : 'bg-gray-800 border-gray-700 text-gray-400 hover:border-gray-600 hover:text-gray-200'}"
          >
            <span class="text-xl mb-0.5">{metodoIcon(m)}</span>
            <span class="text-xs">{metodoLabel(m)}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Contanti: importo e resto -->
    {#if metodoPagamento === 'contanti'}
      <div class="bg-gray-800/60 rounded-lg p-3 space-y-2">
        <div>
          <label class="text-xs text-gray-400">Importo ricevuto €</label>
          <input
            type="number"
            bind:value={importoRicevuto}
            min="0" step="0.01"
            class="input text-sm mt-1"
            on:focus={() => {}}
          />
        </div>
        {#if importoRicevuto > 0}
          <div class="flex justify-between items-center">
            <span class="text-sm text-gray-400">Resto</span>
            <span class="text-lg font-bold {resto !== null && resto < 0 ? 'text-red-400' : 'text-green-400'}">
              {formatCurrency(Math.max(0, importoRicevuto - totaleFinale))}
            </span>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Operatore -->
    <div>
      <label class="text-xs text-gray-400">Operatore</label>
      <input bind:value={operatore} placeholder="Nome operatore" class="input text-sm mt-1"/>
    </div>

    <!-- Bottoni azione -->
    <div class="space-y-2">
      <button
        on:click={chiudiScontrino}
        disabled={salvataggio || carrello.length === 0 || !metodoPagamento}
        class="w-full py-3 rounded-xl bg-green-700 hover:bg-green-600 disabled:opacity-40
          disabled:cursor-not-allowed text-white font-bold text-base transition-colors"
      >
        {salvataggio ? 'Salvataggio…' : '✓ CHIUDI SCONTRINO'}
      </button>
      <button
        on:click={svuotaCarrello}
        disabled={carrello.length === 0}
        class="w-full py-2 rounded-xl bg-red-900/50 hover:bg-red-800/70 disabled:opacity-30
          disabled:cursor-not-allowed text-red-400 text-sm font-medium transition-colors border border-red-800"
      >
        Annulla / Svuota
      </button>
    </div>

    <!-- Ultimi scontrini oggi -->
    {#if ultimiOggi.length > 0}
      <div>
        <p class="text-xs font-medium text-gray-500 mb-2">Ultimi scontrini oggi</p>
        <div class="space-y-1">
          {#each ultimiOggi as sc}
            <div class="flex justify-between text-xs text-gray-400 bg-gray-800/40 rounded px-2 py-1.5">
              <span class="font-mono text-gray-300">{sc.numero}</span>
              <span>{formatCurrency(sc.totale)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<PrintScontrino
  bind:this={printComp}
  scontrino={printScontrino}
  importoRicevuto={importoRicevutoPrint}
/>
