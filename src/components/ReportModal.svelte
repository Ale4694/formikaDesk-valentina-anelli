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

  function stampaReport() {
    if (!report) return
    const w = window.open('', '_blank', 'width=800,height=900')
    if (!w) return

    const titolo = report.mese > 0
      ? `Report ${mesi[report.mese]} ${report.anno}`
      : report.anno > 0
        ? `Report ${report.anno}`
        : 'Report periodo'

    const html = `
      <!DOCTYPE html>
      <html>
      <head>
        <meta charset="utf-8">
        <title>${titolo}</title>
        <style>
          body { font-family: Arial, sans-serif; padding: 20mm; color: #000; }
          h1 { font-size: 18pt; margin-bottom: 10mm; border-bottom: 2pt solid #000; padding-bottom: 3mm; }
          .stats { display: flex; gap: 10mm; margin-bottom: 8mm; }
          .stat-box { flex: 1; border: 0.5pt solid #999; padding: 4mm; }
          .stat-label { font-size: 9pt; color: #666; margin-bottom: 2mm; }
          .stat-value { font-size: 14pt; font-weight: bold; }
          .info-row { display: flex; gap: 10mm; margin-bottom: 8mm; font-size: 10pt; }
          .info-row > div { flex: 1; border: 0.5pt solid #999; padding: 3mm; }
          h2 { font-size: 12pt; margin-top: 6mm; margin-bottom: 3mm; }
          table { width: 100%; border-collapse: collapse; font-size: 10pt; }
          th { background: #e8e8e8; padding: 2mm; text-align: left; border: 0.5pt solid #999; font-weight: bold; }
          td { padding: 2mm; border: 0.5pt solid #ccc; }
          .text-right { text-align: right; }
          @media print { body { padding: 10mm; } }
        </style>
      </head>
      <body>
        <h1>${titolo}</h1>

        <div class="stats">
          <div class="stat-box">
            <div class="stat-label">Entrate (fatture)</div>
            <div class="stat-value">${formatCurrency(report.totale_entrate)}</div>
          </div>
          <div class="stat-box">
            <div class="stat-label">Uscite (note credito)</div>
            <div class="stat-value">${formatCurrency(report.totale_uscite)}</div>
          </div>
          <div class="stat-box">
            <div class="stat-label">Saldo</div>
            <div class="stat-value">${formatCurrency(report.saldo)}</div>
          </div>
        </div>

        <div class="info-row">
          <div><strong>Fatture emesse:</strong> ${report.lista_fatture.length}</div>
          <div><strong>Ordini in attesa:</strong> ${report.ordini_in_attesa}</div>
        </div>

        ${report.lista_fatture.length > 0 ? `
          <h2>Fatture del periodo</h2>
          <table>
            <thead>
              <tr>
                <th>N°</th>
                <th>Cliente</th>
                <th class="text-right">Totale</th>
                <th>Stato</th>
              </tr>
            </thead>
            <tbody>
              ${report.lista_fatture.map(f => `
                <tr>
                  <td>${f.numero}</td>
                  <td>${f.cliente_ragione_sociale ?? ''}</td>
                  <td class="text-right">${formatCurrency(f.totale_documento)}</td>
                  <td>${f.stato}</td>
                </tr>
              `).join('')}
            </tbody>
          </table>
        ` : ''}
      </body>
      </html>
    `

    w.document.write(html)
    w.document.close()
    w.focus()
    setTimeout(() => {
      w.print()
      w.close()
    }, 250)
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
          <button on:click={stampaReport} class="btn-secondary text-xs flex items-center gap-1.5">
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

