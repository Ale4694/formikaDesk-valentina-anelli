<script lang="ts">
  import { onMount } from 'svelte'
  import { currentView, ricambiSottoScorta, searchOpen, setError, setSuccess, appConfig } from '../lib/stores'
  import { api } from '../lib/api'
  import { getVersion } from '@tauri-apps/api/app'
  import type { View } from '../lib/types'

  let appVersion = ''
  onMount(async () => {
    try { appVersion = await getVersion() } catch {}
  })
  import ConfirmModal from './ConfirmModal.svelte'
  import AboutModal from './AboutModal.svelte'

  let confirmRestoreOpen = false
  let aboutOpen = false

  async function eseguiBackup() {
    try {
      const path = await api.backup.backupDatabase()
      if (path) setSuccess(`Backup salvato: ${path}`)
    } catch (e: any) {
      setError(e?.message ?? 'Errore backup')
    }
  }

  async function eseguiRestore() {
    confirmRestoreOpen = false
    try {
      const result = await api.backup.restoreDatabase()
      if (result === 'ripristinato') {
        setSuccess('Database ripristinato. Riavvia l\'applicazione per applicare le modifiche.')
      }
    } catch (e: any) {
      setError(e?.message ?? 'Errore ripristino')
    }
  }

  const navItems: { id: View; modulo: string | null; label: string; icon: string }[] = [
    { id: 'dashboard',        modulo: null,            label: 'Dashboard',      icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
    { id: 'magazzino',        modulo: 'magazzino',     label: 'Magazzino',      icon: 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4' },
    { id: 'clienti',          modulo: 'clienti',       label: 'Clienti',        icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z' },
    { id: 'veicoli',          modulo: 'veicoli',       label: 'Veicoli',        icon: 'M9 17H7m10 0h-2M3 10l2-5h14l2 5M3 10h18v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6z' },
    { id: 'fornitori',        modulo: 'fornitori',     label: 'Fornitori',      icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4' },
    { id: 'ordini-fornitore', modulo: 'ordini',        label: 'Ordini Forn.',   icon: 'M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 11-4 0 2 2 0 014 0z' },
    { id: 'importa-ddt-forn', modulo: 'ordini',        label: 'Importa DDT',    icon: 'M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12' },
    { id: 'documenti',        modulo: 'documenti',     label: 'Documenti',      icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z' },
    { id: 'nuova-fattura',    modulo: 'documenti',     label: 'Nuovo Documento', icon: 'M12 4v16m8-8H4' },
    { id: 'scadenzario',      modulo: 'scadenzario',   label: 'Scadenzario',    icon: 'M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z' },
    { id: 'cassa',            modulo: 'cassa',         label: 'Cassa',          icon: 'M9 7H6a2 2 0 00-2 2v9a2 2 0 002 2h12a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4' },
    { id: 'storico-cassa',    modulo: 'storico-cassa', label: 'Storico Cassa',  icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01' },
    { id: 'impostazioni',     modulo: 'impostazioni',  label: 'Impostazioni',   icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' },
  ]

  $: visibleItems = $appConfig
    ? navItems.filter(item => item.modulo === null || $appConfig!.moduli.includes(item.modulo))
    : navItems

  // Stato collapse con persistenza localStorage
  let collapsed: boolean = (() => {
    try { return localStorage.getItem('sidebar_collapsed') === 'true' } catch { return false }
  })()

  function toggleCollapsed() {
    collapsed = !collapsed
    try { localStorage.setItem('sidebar_collapsed', String(collapsed)) } catch {}
  }
</script>

<aside class="bg-gray-900 border-r border-gray-800 flex flex-col shrink-0 transition-all duration-200 {collapsed ? 'w-14' : 'w-60'}">

  <!-- Logo -->
  <div class="p-3 border-b border-gray-800 flex items-center {collapsed ? 'justify-center' : 'gap-3 px-5'}">
    <div class="w-8 h-8 bg-brand-600 rounded-lg flex items-center justify-center shrink-0">
      <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
      </svg>
    </div>
    {#if !collapsed}
      <div>
        <p class="text-sm font-semibold text-white">{$appConfig?.nome_attivita ?? 'FormikaDesk'}</p>
      </div>
    {/if}
  </div>

  <!-- Ricerca -->
  <div class="px-2 py-1.5 border-b border-gray-800">
    <button
      class="w-full flex items-center rounded-lg text-sm text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors duration-100
        {collapsed ? 'justify-center px-0 py-2.5' : 'gap-2 px-3 py-2'}"
      on:click={() => searchOpen.set(true)}
      title="Cerca (Ctrl+K)"
    >
      <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
      </svg>
      {#if !collapsed}
        <span class="flex-1 text-left text-xs">Cerca…</span>
        <kbd class="text-xs bg-gray-800 px-1 py-0.5 rounded border border-gray-700">⌃K</kbd>
      {/if}
    </button>
  </div>

  <!-- Navigazione -->
  <nav class="flex-1 p-2 space-y-0.5">
    {#each visibleItems as item}
      <button
        class="w-full flex items-center rounded-lg text-sm transition-colors duration-100
          {collapsed ? 'justify-center px-0 py-2.5' : 'gap-3 px-3 py-2'}
          {$currentView === item.id
            ? 'bg-brand-600/20 text-brand-400 font-medium'
            : 'text-gray-400 hover:text-gray-200 hover:bg-gray-800'}"
        on:click={() => currentView.set(item.id)}
        title={collapsed ? item.label : undefined}
      >
        <div class="relative shrink-0">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={item.icon}/>
          </svg>
          {#if item.id === 'magazzino' && $ricambiSottoScorta.length > 0 && collapsed}
            <span class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-red-500"></span>
          {/if}
        </div>

        {#if !collapsed}
          <span class="flex-1 text-left">{item.label}</span>
          {#if item.id === 'magazzino' && $ricambiSottoScorta.length > 0}
            <span class="badge-red text-xs">{$ricambiSottoScorta.length}</span>
          {/if}
        {/if}
      </button>
    {/each}
  </nav>

  <!-- Footer: backup + toggle + versione -->
  <div class="p-2 border-t border-gray-800 space-y-0.5">
    <button
      class="w-full flex items-center rounded-lg text-xs text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors duration-100
        {collapsed ? 'justify-center px-0 py-2' : 'gap-2 px-3 py-2'}"
      on:click={eseguiBackup}
      title="Backup database"
    >
      <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>
      </svg>
      {#if !collapsed}<span>Backup DB</span>{/if}
    </button>

    <button
      class="w-full flex items-center rounded-lg text-xs text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors duration-100
        {collapsed ? 'justify-center px-0 py-2' : 'gap-2 px-3 py-2'}"
      on:click={() => confirmRestoreOpen = true}
      title="Ripristina database"
    >
      <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
      </svg>
      {#if !collapsed}<span>Ripristina DB</span>{/if}
    </button>

    <button
      class="w-full flex items-center rounded-lg text-xs text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors duration-100
        {collapsed ? 'justify-center px-0 py-2' : 'gap-2 px-3 py-2'}"
      on:click={() => aboutOpen = true}
      title="Informazioni"
    >
      <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
      {#if !collapsed}<span>Informazioni</span>{/if}
    </button>

    <button
      class="w-full flex items-center rounded-lg px-3 py-2 text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors duration-100 {collapsed ? 'justify-center px-0' : 'gap-2'}"
      on:click={toggleCollapsed}
      title={collapsed ? 'Espandi sidebar' : 'Comprimi sidebar'}
    >
      <svg class="w-4 h-4 shrink-0 transition-transform duration-200 {collapsed ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7"/>
      </svg>
      {#if !collapsed}
        <span class="text-xs">Comprimi</span>
      {/if}
    </button>
    {#if !collapsed}
      <p class="text-xs text-gray-700 px-3">{appVersion ? 'v' + appVersion : ''}</p>
    {/if}
  </div>
</aside>

<ConfirmModal
  bind:open={confirmRestoreOpen}
  title="Ripristina database"
  message="Questa operazione sovrascrive il database corrente con il file selezionato. Tutti i dati non presenti nel backup andranno persi. Riavvia l'app dopo il ripristino."
  confirmLabel="Ripristina"
  confirmClass="btn-danger"
  onConfirm={eseguiRestore}
  onCancel={() => confirmRestoreOpen = false}
/>

<AboutModal bind:open={aboutOpen} on:close={() => aboutOpen = false} />
