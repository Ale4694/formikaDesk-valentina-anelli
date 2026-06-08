<script lang="ts">
  import { onMount } from 'svelte'
  import { currentView, globalError, globalSuccess, isLoading, clienti, fornitori, ricambi, documenti, dashboardStats, setError, searchOpen, licenseValid, licenseInfo, appConfig } from '$lib/stores'
  import { api } from '$lib/api'
  import Sidebar from '../components/Sidebar.svelte'
  import Dashboard from '../components/views/Dashboard.svelte'
  import Magazzino from '../components/views/Magazzino.svelte'
  import Clienti from '../components/views/Clienti.svelte'
  import Fornitori from '../components/views/Fornitori.svelte'
  import Documenti from '../components/views/Documenti.svelte'
  import NuovaFattura from '../components/views/NuovaFattura.svelte'
  import PrintOverlay from '../components/PrintOverlay.svelte'
  import SearchModal from '../components/SearchModal.svelte'
  import LicenseGate from '../components/LicenseGate.svelte'
  import Veicoli from '../components/views/Veicoli.svelte'
  import OrdiniFornitore from '../components/views/OrdiniFornitore.svelte'
  import Scadenzario from '../components/views/Scadenzario.svelte'
  import Cassa from '../components/views/Cassa.svelte'
  import StoricoCassa from '../components/views/StoricoCassa.svelte'
  import Impostazioni from '../components/views/Impostazioni.svelte'
  import UpdateModal from '../components/UpdateModal.svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { getVersion } from '@tauri-apps/api/app'

  let updateInfo: { version: string; notes: string; date: string } | null = null
  let currentVersion = ''

  async function loadAll() {
    isLoading.set(true)
    try {
      const [stats, c, f, r, d] = await Promise.all([
        api.dashboard.getStats(),
        api.clienti.getAll(),
        api.fornitori.getAll(),
        api.ricambi.getAll(),
        api.documenti.getAll(),
      ])
      dashboardStats.set(stats)
      clienti.set(c)
      fornitori.set(f)
      ricambi.set(r)
      documenti.set(d)
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento dati')
    } finally {
      isLoading.set(false)
    }
  }

  async function onLicenseActivated() {
    const info = await api.license.checkLicense()
    licenseInfo.set(info)
    licenseValid.set(info.valid)
    await loadAll()
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault()
      searchOpen.update(v => !v)
    }
  }

  onMount(async () => {
    try {
      const cfg = await api.config.get()
      appConfig.set(cfg)
    } catch {
      // config non critica, usa defaults
    }
    const info = await api.license.checkLicense()
    licenseInfo.set(info)
    licenseValid.set(info.valid)
    if (info.valid) {
      await loadAll()
      try {
        currentVersion = await getVersion()
        const update = await invoke<{ version: string; notes: string; date: string } | null>('check_update')
        if (update) updateInfo = update
      } catch {
        // aggiornamenti non critici: ignora errori di rete
      }
    }
  })
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $licenseValid === null}
  <div class="flex items-center justify-center h-screen bg-gray-950">
    <div class="w-8 h-8 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
  </div>
{:else if !$licenseValid}
  <LicenseGate licenseInfoData={$licenseInfo} on:activated={onLicenseActivated} />
{:else}
  <div class="flex flex-col h-screen overflow-hidden bg-gray-950">
    {#if $licenseInfo?.tipo === 'demo' && $licenseInfo.valid}
      <div class="shrink-0 bg-yellow-900 border-b border-yellow-700 text-yellow-200 px-4 py-1.5 text-xs text-center font-medium">
        Licenza demo — scade il {$licenseInfo.scadenza} ({$licenseInfo.giorni_rimanenti} {$licenseInfo.giorni_rimanenti === 1 ? 'giorno rimanente' : 'giorni rimanenti'})
      </div>
    {:else if $licenseInfo?.tipo === 'permanent' && $licenseInfo.valid && ($licenseInfo.giorni_rimanenti ?? 999) < 30}
      <div class="shrink-0 bg-orange-900 border-b border-orange-700 text-orange-200 px-4 py-1.5 text-xs text-center font-medium">
        Licenza in scadenza il {$licenseInfo.scadenza} — rinnova per continuare. Contatta Alessandro Formica: +39 320 456 2042
      </div>
    {/if}
    <div id="app-wrapper" class="flex flex-1 min-h-0">
    <Sidebar />
    <main class="flex-1 overflow-y-auto">
      {#if $globalError}
        <div class="fixed top-4 right-4 z-50 bg-red-900 border border-red-700 text-red-200 px-4 py-3 rounded-lg shadow-xl text-sm flex items-center gap-2 max-w-sm">
          <svg class="w-4 h-4 shrink-0" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/></svg>
          {$globalError}
        </div>
      {/if}
      {#if $globalSuccess}
        <div class="fixed top-4 right-4 z-50 bg-green-900 border border-green-700 text-green-200 px-4 py-3 rounded-lg shadow-xl text-sm flex items-center gap-2 max-w-sm">
          <svg class="w-4 h-4 shrink-0" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/></svg>
          {$globalSuccess}
        </div>
      {/if}

      {#if $isLoading}
        <div class="flex items-center justify-center h-full">
          <div class="w-8 h-8 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if $currentView === 'dashboard'}
        <Dashboard on:refresh={loadAll} />
      {:else if $currentView === 'magazzino'}
        <Magazzino on:refresh={loadAll} />
      {:else if $currentView === 'clienti'}
        <Clienti on:refresh={loadAll} />
      {:else if $currentView === 'fornitori'}
        <Fornitori on:refresh={loadAll} />
      {:else if $currentView === 'documenti'}
        <Documenti on:refresh={loadAll} />
      {:else if $currentView === 'nuova-fattura'}
        <NuovaFattura on:refresh={loadAll} />
      {:else if $currentView === 'veicoli'}
        <Veicoli />
      {:else if $currentView === 'ordini-fornitore'}
        <OrdiniFornitore on:refresh={loadAll} />
      {:else if $currentView === 'scadenzario'}
        <Scadenzario />
      {:else if $currentView === 'cassa'}
        <Cassa />
      {:else if $currentView === 'storico-cassa'}
        <StoricoCassa />
      {:else if $currentView === 'impostazioni'}
        <Impostazioni />
      {/if}
    </main>
  </div>
  </div>

  <PrintOverlay />
  <SearchModal />
  {#if updateInfo}
    <UpdateModal {updateInfo} {currentVersion} on:dismiss={() => (updateInfo = null)} />
  {/if}
{/if}
