<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../../lib/api'
  import { formatCurrency, setSuccess, setError } from '../../lib/stores'
  import type { ScontrinoCompleto } from '../../lib/types'
  import PrintScontrino from '../PrintScontrino.svelte'
  import ConfirmModal from '../ConfirmModal.svelte'

  let tutti: ScontrinoCompleto[] = []
  let loading = false
  let filtro: 'oggi' | 'settimana' | 'mese' = 'oggi'

  let printComp: PrintScontrino | undefined
  let printData: ScontrinoCompleto | null = null

  let annullaId: number | null = null
  let confirmOpen = false

  onMount(load)

  async function load() {
    loading = true
    try { tutti = await api.cassa.getAll() }
    catch (e: any) { setError(e?.message ?? 'Errore') }
    finally { loading = false }
  }

  $: filtered = tutti.filter(sc => {
    const data = new Date(sc.scontrino.data)
    const now = new Date()
    if (filtro === 'oggi') {
      return sc.scontrino.data === now.toISOString().slice(0, 10)
    } else if (filtro === 'settimana') {
      const sett = new Date(now); sett.setDate(now.getDate() - 7)
      return data >= sett
    } else {
      const mese = new Date(now); mese.setDate(now.getDate() - 30)
      return data >= mese
    }
  })

  $: totaleFiltered = filtered
    .filter(sc => sc.scontrino.stato === 'chiuso')
    .reduce((s, sc) => s + sc.scontrino.totale, 0)

  function ristampa(sc: ScontrinoCompleto) {
    printData = sc
    setTimeout(() => printComp?.stampa(), 50)
  }

  function apriAnnulla(id: number) {
    annullaId = id
    confirmOpen = true
  }

  async function confermaAnnulla() {
    if (annullaId === null) return
    try {
      const updated = await api.cassa.annulla(annullaId)
      tutti = tutti.map(sc => sc.scontrino.id === annullaId ? updated : sc)
      setSuccess('Scontrino annullato — giacenza ripristinata')
    } catch (e: any) {
      setError(e?.message ?? 'Errore annullamento')
    } finally {
      annullaId = null
      confirmOpen = false
    }
  }

  function labelStato(s: string) {
    return { chiuso: 'Chiuso', annullato: 'Annullato', aperto: 'Aperto' }[s] ?? s
  }
  function colorStato(s: string) {
    return { chiuso: 'badge-green', annullato: 'badge-red', aperto: 'badge-yellow' }[s] ?? 'badge-gray'
  }
  function metodoLabel(m: string) {
    return { contanti: 'Contanti', carta: 'Carta', satispay: 'Satispay', bonifico: 'Bonifico' }[m] ?? m
  }
</script>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Storico Cassa</h1>
    <button class="btn-secondary text-xs" on:click={load}>Aggiorna</button>
  </div>

  <!-- Filtri -->
  <div class="flex items-center gap-3">
    <div class="flex gap-1">
      {#each [['oggi','Oggi'],['settimana','7 giorni'],['mese','30 giorni']] as [val, label]}
        <button
          class="px-3 py-1 rounded text-xs transition-colors {filtro === val ? 'bg-brand-600 text-white' : 'bg-gray-800 text-gray-400 hover:text-gray-200'}"
          on:click={() => filtro = val as typeof filtro}
        >{label}</button>
      {/each}
    </div>
    <span class="text-xs text-gray-500 ml-auto">
      {filtered.length} scontrini · Incasso: <span class="text-white font-medium">{formatCurrency(totaleFiltered)}</span>
    </span>
  </div>

  {#if loading}
    <div class="flex justify-center py-12"><div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else if filtered.length === 0}
    <div class="card p-8 text-center text-gray-500 text-sm">Nessuno scontrino nel periodo selezionato.</div>
  {:else}
    <div class="card overflow-hidden">
      <table class="w-full text-sm">
        <thead class="bg-gray-800/60">
          <tr class="text-xs text-gray-400 border-b border-gray-800">
            <th class="text-left px-4 py-3 font-medium">N° Scontrino</th>
            <th class="text-left px-4 py-3 font-medium">Data</th>
            <th class="text-left px-4 py-3 font-medium">Operatore</th>
            <th class="text-left px-4 py-3 font-medium">Pagamento</th>
            <th class="text-right px-4 py-3 font-medium">Totale</th>
            <th class="text-left px-4 py-3 font-medium">Stato</th>
            <th class="px-4 py-3"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800">
          {#each filtered as sc}
            {@const s = sc.scontrino}
            <tr class="hover:bg-gray-800/30 {s.stato === 'annullato' ? 'opacity-50' : ''}">
              <td class="px-4 py-3 font-mono text-brand-400">{s.numero}</td>
              <td class="px-4 py-3 text-gray-400">{new Date(s.data).toLocaleDateString('it-IT')}</td>
              <td class="px-4 py-3 text-gray-300">{s.operatore ?? '—'}</td>
              <td class="px-4 py-3 text-gray-400">{metodoLabel(s.metodo_pagamento)}</td>
              <td class="px-4 py-3 text-right text-gray-200 font-medium">{formatCurrency(s.totale)}</td>
              <td class="px-4 py-3"><span class="badge {colorStato(s.stato)}">{labelStato(s.stato)}</span></td>
              <td class="px-4 py-3">
                <div class="flex items-center gap-2 justify-end">
                  <button
                    on:click={() => ristampa(sc)}
                    class="text-xs text-gray-400 hover:text-gray-200 transition-colors"
                  >Ristampa</button>
                  {#if s.stato === 'chiuso'}
                    <button
                      on:click={() => apriAnnulla(s.id)}
                      class="text-xs text-red-400 hover:text-red-300 transition-colors"
                    >Annulla</button>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<PrintScontrino bind:this={printComp} scontrino={printData} />

<ConfirmModal
  bind:open={confirmOpen}
  title="Annulla scontrino"
  message="L'annullamento ripristina la giacenza dei prodotti venduti. Questa operazione non è reversibile."
  confirmLabel="Annulla scontrino"
  confirmClass="btn-danger"
  onConfirm={confermaAnnulla}
  onCancel={() => { confirmOpen = false; annullaId = null }}
/>
