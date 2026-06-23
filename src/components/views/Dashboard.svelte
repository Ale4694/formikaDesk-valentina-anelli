<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { dashboardStats, formatCurrency, currentView } from '../../lib/stores'
  import { api } from '../../lib/api'
  import ReportModal from '../ReportModal.svelte'
  import type { ReportMensile } from '../../lib/types'

  const dispatch = createEventDispatcher()

  const now = new Date()
  let reportAnno = now.getFullYear()
  let reportMese = now.getMonth() + 1
  let reportData: ReportMensile | null = null
  let reportOpen = false
  let reportLoading = false

  const mesi = ['', 'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
                 'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre']

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

  // Report periodo
  let periodoFrom: string = ''
  let periodoTo: string = ''
  let periodoLoading = false

  function fmtDate(d: Date): string {
    const pad = (n: number) => String(n).padStart(2, '0')
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
  }

  async function generaReportPeriodo() {
    if (!periodoFrom || !periodoTo) return
    periodoLoading = true
    try {
      reportData = await api.report.getPeriodo(periodoFrom, periodoTo)
      reportOpen = true
    } catch (e: any) {
      console.error(e)
    } finally {
      periodoLoading = false
    }
  }

  async function reportOggi() {
    const s = fmtDate(new Date())
    periodoFrom = s; periodoTo = s
    await generaReportPeriodo()
  }

  async function reportSettimana() {
    const oggi = new Date()
    const lun = new Date(oggi)
    lun.setDate(oggi.getDate() - ((oggi.getDay() + 6) % 7))
    periodoFrom = fmtDate(lun); periodoTo = fmtDate(oggi)
    await generaReportPeriodo()
  }

  async function reportAnnoCorrente() {
    const oggi = new Date()
    periodoFrom = `${oggi.getFullYear()}-01-01`; periodoTo = fmtDate(oggi)
    await generaReportPeriodo()
  }

  async function generaReport() {
    reportLoading = true
    try {
      reportData = await api.report.getMensile(reportAnno, reportMese)
      reportOpen = true
    } catch (e: any) {
      console.error(e)
    } finally {
      reportLoading = false
    }
  }
</script>

<div class="p-6 space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Dashboard</h1>
    <button class="btn-secondary text-xs" on:click={() => dispatch('refresh')}>
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

    <!-- Report mensile -->
    <div class="card p-4">
      <p class="text-sm font-medium text-gray-300 mb-3">Report mensile</p>
      <div class="flex items-center gap-3">
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
        <button on:click={generaReport} disabled={reportLoading} class="btn-primary text-sm">
          {reportLoading ? 'Generazione…' : 'Genera report'}
        </button>
      </div>
    </div>

    <!-- Report per periodo -->
    <div class="card p-4">
      <p class="text-sm font-medium text-gray-300 mb-3">Report per periodo</p>

      <!-- Pulsanti rapidi -->
      <div class="flex gap-2 mb-3 flex-wrap">
        <button on:click={reportOggi} disabled={periodoLoading} class="btn-secondary text-xs">
          Oggi
        </button>
        <button on:click={reportSettimana} disabled={periodoLoading} class="btn-secondary text-xs">
          Questa settimana
        </button>
        <button on:click={reportAnnoCorrente} disabled={periodoLoading} class="btn-secondary text-xs">
          Quest'anno
        </button>
      </div>

      <!-- Date personalizzate + Genera -->
      <div class="flex items-end gap-3 flex-wrap">
        <div class="flex flex-col gap-1">
          <span class="text-xs text-gray-400">Dal</span>
          <input type="date" bind:value={periodoFrom}
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-brand-500" />
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-xs text-gray-400">Al</span>
          <input type="date" bind:value={periodoTo}
            class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-brand-500" />
        </div>
        <button
          on:click={generaReportPeriodo}
          disabled={periodoLoading || !periodoFrom || !periodoTo}
          class="btn-primary text-sm"
        >
          {periodoLoading ? 'Generazione…' : 'Genera report'}
        </button>
      </div>
    </div>
  {/if}
</div>

<ReportModal bind:open={reportOpen} report={reportData} on:close={() => reportOpen = false} />
