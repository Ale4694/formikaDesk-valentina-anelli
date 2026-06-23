<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { formatCurrency } from '../lib/stores'
  import type { ReportMensile } from '../lib/types'

  export let open = false
  export let report: ReportMensile | null = null

  const dispatch = createEventDispatcher<{ close: void }>()

  const mesi = ['', 'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
                 'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre']

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) dispatch('close')
  }
</script>

{#if open && report}
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="report-modal-overlay fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
       on:click={handleBackdrop} on:keydown={() => {}}>
    <div class="report-modal-content bg-gray-900 border border-gray-800 rounded-xl shadow-2xl w-full max-w-3xl max-h-[90vh] flex flex-col">

      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-800">
        <h2 class="text-base font-semibold text-white">
          {report.mese > 0 ? `Report ${mesi[report.mese]} ${report.anno}` : report.anno > 0 ? `Report ${report.anno}` : 'Report periodo'}
        </h2>
        <div class="flex items-center gap-2">
          <button on:click={() => window.print()} class="btn-secondary text-xs flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"/>
            </svg>
            Stampa
          </button>
          <button on:click={() => dispatch('close')} class="text-gray-500 hover:text-gray-300">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto px-6 py-5 space-y-6">

        <!-- Riepilogo numerico -->
        <div class="grid grid-cols-3 gap-4">
          <div class="card p-4">
            <p class="text-xs text-gray-500 mb-1">Entrate (fatture)</p>
            <p class="text-xl font-bold text-green-400">{formatCurrency(report.totale_entrate)}</p>
          </div>
          <div class="card p-4">
            <p class="text-xs text-gray-500 mb-1">Uscite (note credito)</p>
            <p class="text-xl font-bold text-red-400">{formatCurrency(report.totale_uscite)}</p>
          </div>
          <div class="card p-4">
            <p class="text-xs text-gray-500 mb-1">Saldo</p>
            <p class="text-xl font-bold {report.saldo >= 0 ? 'text-brand-400' : 'text-red-400'}">
              {formatCurrency(report.saldo)}
            </p>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3 text-sm">
          <div class="card p-3 flex items-center justify-between">
            <span class="text-gray-400">Fatture emesse</span>
            <span class="text-white font-medium">{report.lista_fatture.length}</span>
          </div>
          <div class="card p-3 flex items-center justify-between">
            <span class="text-gray-400">Ordini in attesa</span>
            <span class="text-yellow-400 font-medium">{report.ordini_in_attesa}</span>
          </div>
        </div>

        <!-- Tabella fatture -->
        {#if report.lista_fatture.length > 0}
          <div>
            <h3 class="text-sm font-semibold text-gray-300 mb-2">Fatture del periodo</h3>
            <div class="card overflow-hidden">
              <table class="w-full text-sm">
                <thead class="bg-gray-800/60">
                  <tr class="text-xs text-gray-400">
                    <th class="text-left px-3 py-2 font-medium">N°</th>
                    <th class="text-left px-3 py-2 font-medium">Cliente</th>
                    <th class="text-right px-3 py-2 font-medium">Totale</th>
                    <th class="text-left px-3 py-2 font-medium">Stato</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-800">
                  {#each report.lista_fatture as f}
                    <tr>
                      <td class="px-3 py-2 text-brand-400 font-medium">{f.numero}</td>
                      <td class="px-3 py-2 text-gray-300">{f.cliente_ragione_sociale ?? '—'}</td>
                      <td class="px-3 py-2 text-right text-gray-200">{formatCurrency(f.totale_documento)}</td>
                      <td class="px-3 py-2 text-gray-500 capitalize text-xs">{f.stato}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}

      </div>
    </div>
  </div>
{/if}

<style>
  @media print {
    .report-modal-overlay {
      position: static !important;
      background: white !important;
      backdrop-filter: none !important;
      padding: 0 !important;
    }
    .report-modal-content {
      position: static !important;
      max-height: none !important;
      overflow: visible !important;
      background: white !important;
      color: black !important;
      box-shadow: none !important;
      border: none !important;
      border-radius: 0 !important;
    }
  }
</style>
