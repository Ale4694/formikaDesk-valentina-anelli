<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../../lib/api'
  import { formatCurrency, currentView } from '../../lib/stores'
  import type { ScadenzaDocumento } from '../../lib/types'

  let scadenze: ScadenzaDocumento[] = []
  let loading = false

  onMount(load)

  async function load() {
    loading = true
    try {
      scadenze = await api.documenti.getScadenzario()
    } catch (e: any) {
      console.error(e)
    } finally {
      loading = false
    }
  }

  function badgeGiorni(giorni: number | null) {
    if (giorni === null) return { label: 'N/D', cls: 'text-gray-400 bg-gray-800' }
    if (giorni < 0) return { label: `Scaduta ${Math.abs(Math.round(giorni))}gg fa`, cls: 'text-red-400 bg-red-950/50 border border-red-800' }
    if (giorni <= 30) return { label: `${Math.round(giorni)}gg`, cls: 'text-yellow-400 bg-yellow-950/40' }
    return { label: `${Math.round(giorni)}gg`, cls: 'text-green-400 bg-green-950/30' }
  }

  function formatData(s: string | null) {
    if (!s) return '—'
    return new Date(s).toLocaleDateString('it-IT')
  }

  $: scadute = scadenze.filter(s => (s.giorni_alla_scadenza ?? 0) < 0).length
  $: inScadenza = scadenze.filter(s => s.giorni_alla_scadenza !== null && s.giorni_alla_scadenza >= 0 && s.giorni_alla_scadenza <= 30).length
</script>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Scadenzario</h1>
    <button class="btn-secondary text-xs" on:click={load}>Aggiorna</button>
  </div>

  <!-- Riepilogo alert -->
  {#if scadute > 0}
    <div class="card p-3 border-red-800/60 bg-red-950/20 flex items-center gap-2 text-sm text-red-400">
      <svg class="w-4 h-4 shrink-0" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/></svg>
      {scadute} fattura/e scaduta/e — pagamento in ritardo
    </div>
  {/if}
  {#if inScadenza > 0}
    <div class="card p-3 border-yellow-800/60 bg-yellow-950/10 flex items-center gap-2 text-sm text-yellow-400">
      <svg class="w-4 h-4 shrink-0" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/></svg>
      {inScadenza} fattura/e in scadenza entro 30 giorni
    </div>
  {/if}

  {#if loading}
    <div class="flex justify-center py-12"><div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else if scadenze.length === 0}
    <div class="card p-8 text-center text-gray-500 text-sm">
      <p class="text-lg mb-1">Nessuna fattura in scadenza</p>
      <p class="text-xs">Le fatture con scadenza verranno mostrate qui.</p>
    </div>
  {:else}
    <div class="card overflow-hidden">
      <table class="w-full text-sm">
        <thead class="bg-gray-800/60">
          <tr class="text-xs text-gray-400 border-b border-gray-800">
            <th class="text-left px-4 py-3 font-medium">N° Fattura</th>
            <th class="text-left px-4 py-3 font-medium">Cliente</th>
            <th class="text-left px-4 py-3 font-medium">Data doc.</th>
            <th class="text-left px-4 py-3 font-medium">Scadenza</th>
            <th class="text-left px-4 py-3 font-medium">Giorni</th>
            <th class="text-right px-4 py-3 font-medium">Totale</th>
            <th class="text-left px-4 py-3 font-medium">Stato</th>
            <th class="px-4 py-3"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800">
          {#each scadenze as s}
            {@const badge = badgeGiorni(s.giorni_alla_scadenza)}
            <tr class="hover:bg-gray-800/30 {s.giorni_alla_scadenza !== null && s.giorni_alla_scadenza < 0 ? 'bg-red-950/10' : ''}">
              <td class="px-4 py-3 font-medium text-brand-400">{s.numero}</td>
              <td class="px-4 py-3 text-gray-300">{s.cliente_ragione_sociale ?? '—'}</td>
              <td class="px-4 py-3 text-gray-400">{formatData(s.data)}</td>
              <td class="px-4 py-3 text-gray-300">{formatData(s.scadenza_pagamento)}</td>
              <td class="px-4 py-3">
                <span class="text-xs px-2 py-0.5 rounded-full font-medium {badge.cls}">{badge.label}</span>
              </td>
              <td class="px-4 py-3 text-right text-gray-200">{formatCurrency(s.totale_documento)}</td>
              <td class="px-4 py-3">
                <span class="text-xs text-gray-500 capitalize">{s.stato}</span>
              </td>
              <td class="px-4 py-3">
                <button
                  class="text-xs text-gray-500 hover:text-gray-300"
                  on:click={() => currentView.set('documenti')}
                >Vai →</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
