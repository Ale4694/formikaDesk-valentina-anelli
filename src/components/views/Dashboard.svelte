<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'
  import { dashboardStats, formatCurrency, currentView, schedaClienteId } from '../../lib/stores'
  import { api } from '../../lib/api'
  import ReportModal from '../ReportModal.svelte'
  import type { ReportMensile, ClienteAttivo } from '../../lib/types'

  const dispatch = createEventDispatcher()

  const now = new Date()
  const annoCorrente = now.getFullYear()
  let reportAnno = now.getFullYear()
  let reportMese = now.getMonth() + 1
  let reportData: ReportMensile | null = null
  let reportOpen = false
  let reportLoading = false

  let reportTipo: 'mensile' | 'giornaliero' | 'annuale' | 'personalizzato' = 'mensile'
  let reportDataFrom: string = ''
  let reportDataTo: string = ''

  const tipiFilter = ['tutti', 'fattura', 'ddt', 'preventivo', 'nota_credito', 'vendita_banco', 'buono', 'fattura_differita', 'ddt_fornitore'] as const
  const tipoLabel: Record<string, string> = {
    fattura: 'Fattura',
    preventivo: 'Preventivo',
    ddt: 'DDT',
    nota_credito: 'Nota credito',
    vendita_banco: 'Vendita Banco',
    buono: 'Buono',
    fattura_differita: 'Fattura Differita',
    ddt_fornitore: 'DDT Fornitore',
  }
  let tipoAttivo: typeof tipiFilter[number] = 'tutti'

  const mesi = ['', 'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
                 'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre']

  let clientiAttivi: ClienteAttivo[] = []
  let clientiAttiviLoading = true

  async function caricaClientiAttivi() {
    clientiAttiviLoading = true
    try {
      clientiAttivi = await api.dashboard.getClientiPiuAttivi()
    } catch (e) {
      console.error(e)
    } finally {
      clientiAttiviLoading = false
    }
  }

  function apriSchedaCliente(clienteId: number) {
    schedaClienteId.set(clienteId)
    currentView.set('scheda-cliente')
  }

  onMount(caricaClientiAttivi)

  interface StatCard { label: string; value: string; color: string; alert?: boolean }

  $: cards = $dashboardStats ? ([
    { label: 'Clienti',            value: String($dashboardStats.totale_clienti),            color: 'text-blue-400' },
    { label: 'Fornitori',          value: String($dashboardStats.totale_fornitori),           color: 'text-purple-400' },
    { label: 'Ricambi a magazzino',value: String($dashboardStats.totale_ricambi),            color: 'text-cyan-400' },
    { label: 'Sotto scorta',       value: String($dashboardStats.ricambi_sotto_scorta),      color: 'text-red-400',    alert: $dashboardStats.ricambi_sotto_scorta > 0 },
    { label: 'Fatture del mese',   value: String($dashboardStats.fatture_mese_corrente),     color: 'text-green-400' },
    { label: 'Fatturato mese',     value: formatCurrency($dashboardStats.fatturato_mese_corrente), color: 'text-green-400' },
    { label: 'Da incassare',       value: String($dashboardStats.fatture_non_pagate),        color: 'text-yellow-400', alert: $dashboardStats.fatture_non_pagate > 0 },
    { label: 'Valore magazzino',   value: formatCurrency($dashboardStats.valore_magazzino),  color: 'text-brand-400' },
    { label: 'Scontrini oggi',     value: String($dashboardStats.scontrini_oggi),            color: 'text-orange-400' },
    { label: 'Incasso oggi',       value: formatCurrency($dashboardStats.incasso_oggi),      color: 'text-orange-400' },
  ] as StatCard[]) : []

  async function generaReport() {
    reportLoading = true
    try {
      if (reportTipo === 'mensile') {
        reportData = await api.report.getMensile(reportAnno, reportMese, tipoAttivo)
      } else if (reportTipo === 'giornaliero') {
        reportData = await api.report.getGiornaliero(reportDataFrom, tipoAttivo)
      } else if (reportTipo === 'annuale') {
        reportData = await api.report.getAnnuale(reportAnno, tipoAttivo)
      } else if (reportTipo === 'personalizzato') {
        reportData = await api.report.getPersonalizzato(reportDataFrom, reportDataTo, tipoAttivo)
      }
      reportOpen = true
    } catch (e: any) {
      console.error(e)
    } finally {
      reportLoading = false
    }
  }

  $: generaDisabled = reportLoading ||
    (reportTipo === 'giornaliero' && !reportDataFrom) ||
    (reportTipo === 'personalizzato' && (!reportDataFrom || !reportDataTo))
</script>

<div class="p-6 space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Dashboard</h1>
    <button class="btn-secondary text-xs" on:click={() => { dispatch('refresh'); caricaClientiAttivi() }}>
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
      </svg>
      Aggiorna
    </button>
  </div>

  {#if !$dashboardStats}
    <div class="flex justify-center py-20">
      <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      {#each cards as card}
        <div class="card p-4 {card.alert ? 'border-red-800/70 bg-red-950/20' : ''}">
          <p class="text-xs text-gray-500 mb-1">{card.label}</p>
          <p class="text-2xl font-bold {card.color}">{card.value}</p>
        </div>
      {/each}
    </div>

    <!-- Alert scorta -->
    {#if $dashboardStats.ricambi_sotto_scorta > 0}
      <div class="card p-4 border-red-800/70 bg-red-950/20">
        <div class="flex items-center gap-2 text-red-400">
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/></svg>
          <span class="text-sm font-medium">{$dashboardStats.ricambi_sotto_scorta} ricambi sotto scorta minima</span>
          <button class="ml-auto text-xs underline" on:click={() => currentView.set('magazzino')}>Vai al magazzino →</button>
        </div>
      </div>
    {/if}

    <!-- Alert fatture scadute -->
    {#if $dashboardStats.fatture_scadute > 0}
      <div class="card p-4 border-orange-800/70 bg-orange-950/20">
        <div class="flex items-center gap-2 text-orange-400">
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/></svg>
          <span class="text-sm font-medium">{$dashboardStats.fatture_scadute} fattura/e scaduta/e non pagate</span>
          <button class="ml-auto text-xs underline" on:click={() => currentView.set('scadenzario')}>Vai allo scadenzario →</button>
        </div>
      </div>
    {/if}

    <!-- Clienti più attivi -->
    <div class="card p-4">
      <p class="text-sm font-medium text-gray-300 mb-3">Clienti più attivi {annoCorrente}</p>
      {#if clientiAttiviLoading}
        <div class="flex justify-center py-6">
          <div class="w-5 h-5 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if clientiAttivi.length === 0}
        <p class="text-sm text-gray-500 text-center py-6">Nessuna fattura emessa nel {annoCorrente}.</p>
      {:else}
        <div class="divide-y divide-gray-800">
          {#each clientiAttivi as c (c.cliente_id)}
            <button
              class="w-full flex items-center justify-between gap-4 py-2.5 px-2 -mx-2 rounded text-left hover:bg-gray-800/40 transition-colors"
              on:click={() => apriSchedaCliente(c.cliente_id)}
            >
              <span class="text-sm text-gray-200 truncate">{c.ragione_sociale}</span>
              <span class="flex items-center gap-4 shrink-0">
                <span class="text-xs text-gray-500 whitespace-nowrap">{c.numero_documenti} doc.</span>
                <span class="text-sm font-mono font-medium text-green-400 text-right w-24">
                  {formatCurrency(c.fatturato)}
                </span>
              </span>
            </button>
          {/each}
        </div>
      {/if}
    </div>

     <!-- Report -->
     <div class="card p-4">
       <p class="text-sm font-medium text-gray-300 mb-3">Report</p>
 
       <!-- Filtro tipo documento -->
       <div class="flex gap-1.5 flex-wrap mb-4">
         {#each tipiFilter as t}
           <button
             class="px-3 py-1 rounded-full text-xs font-medium transition-colors duration-100
               {tipoAttivo === t
                 ? 'bg-gray-600 text-white'
                 : 'bg-gray-800/60 text-gray-500 hover:text-gray-300 hover:bg-gray-800'}"
             on:click={() => { tipoAttivo = t; if (reportOpen) generaReport() }}
           >
             {t === 'tutti' ? 'Tutti i tipi' : (tipoLabel[t] ?? t)}
           </button>
         {/each}
       </div>
 
       <!-- Selezione tipo -->
       <div class="flex gap-2 mb-3 flex-wrap">

        {#each [['giornaliero', 'Giornaliero'], ['mensile', 'Mensile'], ['annuale', 'Annuale'], ['personalizzato', 'Personalizzato']] as [tipo, label]}
          <button
            on:click={() => reportTipo = tipo as typeof reportTipo}
            class="text-xs px-3 py-1.5 rounded border transition-colors {reportTipo === tipo ? 'bg-brand-600 border-brand-500 text-white' : 'bg-gray-800 border-gray-700 text-gray-400 hover:border-gray-600'}"
          >
            {label}
          </button>
        {/each}
      </div>

      <!-- Controlli in base al tipo -->
      <div class="flex items-end gap-3 flex-wrap">
        {#if reportTipo === 'giornaliero'}
          <input type="date" bind:value={reportDataFrom}
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-brand-500" />
        {:else if reportTipo === 'mensile'}
          <select bind:value={reportMese} class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-brand-500">
            {#each mesi.slice(1) as m, i}
              <option value={i + 1}>{m}</option>
            {/each}
          </select>
          <input
            type="number"
            bind:value={reportAnno}
            min="2020" max="2099"
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white w-20 focus:outline-none focus:border-brand-500"
          />
        {:else if reportTipo === 'annuale'}
          <input
            type="number"
            bind:value={reportAnno}
            min="2020" max="2099"
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white w-20 focus:outline-none focus:border-brand-500"
          />
        {:else if reportTipo === 'personalizzato'}
          <div class="flex flex-col gap-1">
            <span class="text-xs text-gray-400">Dal</span>
            <input type="date" bind:value={reportDataFrom}
              class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-brand-500" />
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-gray-400">Al</span>
            <input type="date" bind:value={reportDataTo}
              class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-brand-500" />
          </div>
        {/if}
        <button on:click={generaReport} disabled={generaDisabled} class="btn-primary text-sm">
          {reportLoading ? 'Generazione…' : 'Genera report'}
        </button>
      </div>
    </div>
  {/if}
</div>

<ReportModal bind:open={reportOpen} report={reportData} on:close={() => reportOpen = false} />
