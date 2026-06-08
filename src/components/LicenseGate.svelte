<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'
  import { api } from '../lib/api'
  import { appConfig } from '../lib/stores'
  import type { LicenseInfo } from '../lib/types'

  const dispatch = createEventDispatcher<{ activated: void }>()

  export let licenseInfoData: LicenseInfo | null = null

  let chiave = ''
  let machineId = ''
  let errore = ''
  let loading = false
  let copiato = false

  onMount(async () => {
    try {
      machineId = await api.license.getMachineId()
    } catch {}
  })

  function formatKey(raw: string): string {
    const clean = raw.replace(/[^A-Za-z0-9]/g, '').toUpperCase().slice(0, 16)
    const parts: string[] = []
    for (let i = 0; i < clean.length; i += 4) {
      parts.push(clean.slice(i, i + 4))
    }
    return parts.join('-')
  }

  function handleInput(e: Event) {
    const input = e.target as HTMLInputElement
    chiave = formatKey(input.value)
  }

  async function attiva() {
    if (chiave.length < 19) {
      errore = 'Inserisci una chiave completa (XXXX-XXXX-XXXX-XXXX)'
      return
    }
    errore = ''
    loading = true
    try {
      await api.license.activateLicense(chiave)
      dispatch('activated')
    } catch (e: any) {
      errore = e?.message ?? 'Errore durante l\'attivazione'
    } finally {
      loading = false
    }
  }

  async function copiaMachineId() {
    if (!machineId) return
    try {
      await navigator.clipboard.writeText(machineId)
      copiato = true
      setTimeout(() => { copiato = false }, 2000)
    } catch {}
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') attiva()
  }

  $: isPermExpired = licenseInfoData?.tipo === 'permanent' && licenseInfoData?.scaduto
  $: isDemoExpired = licenseInfoData?.tipo === 'demo' && licenseInfoData?.scaduto
</script>

<div class="min-h-screen bg-gray-950 flex items-center justify-center p-6">
  <div class="w-full max-w-md space-y-4">

    <!-- Logo -->
    <div class="flex flex-col items-center mb-8">
      <div class="w-16 h-16 bg-brand-600 rounded-2xl flex items-center justify-center mb-4 shadow-lg">
        <svg class="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
      </div>
      <h1 class="text-2xl font-bold text-white">{$appConfig?.nome_attivita ?? 'FormikaDesk'}</h1>
    </div>

    {#if isPermExpired}
      <!-- Licenza permanente scaduta -->
      <div class="bg-red-950 border border-red-700 rounded-xl p-6 shadow-xl space-y-3">
        <div class="flex items-center gap-2 text-red-400">
          <svg class="w-5 h-5 shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
          <h2 class="text-base font-semibold">Licenza scaduta il {licenseInfoData?.scadenza}</h2>
        </div>
        <p class="text-sm text-red-300 leading-relaxed">
          Rinnova per continuare a usare il software. Contatta Alessandro Formica:
        </p>
        <p class="text-sm font-semibold text-red-200">+39 320 456 2042</p>
      </div>

      <!-- Card rinnovo: inserisci nuova chiave -->
      <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-xl">
        <h2 class="text-lg font-semibold text-white mb-1">Inserisci chiave di rinnovo</h2>
        <p class="text-sm text-gray-400 mb-5">
          Hai ricevuto una nuova chiave? Inseriscila qui sotto per riattivare il software.
        </p>
        <div class="space-y-4">
          <div>
            <label for="license-key-renew" class="block text-xs font-medium text-gray-400 mb-1.5">
              Chiave di licenza
            </label>
            <input
              id="license-key-renew"
              type="text"
              value={chiave}
              on:input={handleInput}
              on:keydown={handleKeydown}
              placeholder="PERM-XXXX-XXXX-XXXX"
              maxlength="19"
              spellcheck="false"
              autocomplete="off"
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-white
                placeholder-gray-600 text-sm font-mono tracking-widest
                focus:outline-none focus:border-brand-500 focus:ring-1 focus:ring-brand-500
                {errore ? 'border-red-600 focus:border-red-500 focus:ring-red-500' : ''}"
            />
            {#if errore}
              <p class="mt-1.5 text-xs text-red-400">{errore}</p>
            {/if}
          </div>
          <button
            on:click={attiva}
            disabled={loading}
            class="w-full bg-brand-600 hover:bg-brand-500 disabled:opacity-50 disabled:cursor-not-allowed
              text-white font-medium py-2.5 px-4 rounded-lg text-sm transition-colors duration-100"
          >
            {#if loading}
              <span class="flex items-center justify-center gap-2">
                <span class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                Verifica in corso…
              </span>
            {:else}
              Attiva licenza
            {/if}
          </button>
        </div>
      </div>

    {:else if isDemoExpired}
      <!-- Demo scaduta -->
      <div class="bg-red-950 border border-red-700 rounded-xl p-6 shadow-xl space-y-3">
        <div class="flex items-center gap-2 text-red-400">
          <svg class="w-5 h-5 shrink-0" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
          <h2 class="text-base font-semibold">Licenza demo scaduta</h2>
        </div>
        <p class="text-sm text-red-300 leading-relaxed">
          La tua licenza demo è scaduta il {licenseInfoData?.scadenza}. Contatta Alessandro Formica per acquistare la licenza completa:
        </p>
        <p class="text-sm font-semibold text-red-200">+39 320 456 2042</p>
      </div>

    {:else}
      <!-- Card attivazione -->
      <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 shadow-xl">
        <h2 class="text-lg font-semibold text-white mb-1">Attivazione licenza</h2>
        <p class="text-sm text-gray-400 mb-5">
          Inserisci la chiave di licenza per attivare il software.
        </p>

        <div class="space-y-4">
          <div>
            <label for="license-key" class="block text-xs font-medium text-gray-400 mb-1.5">
              Chiave di licenza
            </label>
            <input
              id="license-key"
              type="text"
              value={chiave}
              on:input={handleInput}
              on:keydown={handleKeydown}
              placeholder="DEMO-XXXX-XXXX-XXXX  o  PERM-XXXX-XXXX-XXXX"
              maxlength="19"
              spellcheck="false"
              autocomplete="off"
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-white
                placeholder-gray-600 text-sm font-mono tracking-widest
                focus:outline-none focus:border-brand-500 focus:ring-1 focus:ring-brand-500
                {errore ? 'border-red-600 focus:border-red-500 focus:ring-red-500' : ''}"
            />
            {#if errore}
              <p class="mt-1.5 text-xs text-red-400">{errore}</p>
            {/if}
          </div>

          <button
            on:click={attiva}
            disabled={loading}
            class="w-full bg-brand-600 hover:bg-brand-500 disabled:opacity-50 disabled:cursor-not-allowed
              text-white font-medium py-2.5 px-4 rounded-lg text-sm transition-colors duration-100"
          >
            {#if loading}
              <span class="flex items-center justify-center gap-2">
                <span class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                Verifica in corso…
              </span>
            {:else}
              Attiva licenza
            {/if}
          </button>
        </div>
      </div>
    {/if}

    <!-- Machine ID -->
    <div class="bg-gray-900 border border-gray-800 rounded-xl p-4">
      <p class="text-xs text-gray-500 mb-2">
        Per richiedere una licenza, invia questo ID dispositivo:
      </p>
      <div class="flex items-center gap-2">
        <code class="flex-1 text-xs font-mono text-gray-300 bg-gray-800 rounded px-2 py-1.5 truncate select-all">
          {machineId || '…'}
        </code>
        <button
          on:click={copiaMachineId}
          disabled={!machineId}
          class="shrink-0 text-xs px-2.5 py-1.5 rounded bg-gray-700 hover:bg-gray-600
            disabled:opacity-40 disabled:cursor-not-allowed text-gray-300 transition-colors"
        >
          {copiato ? 'Copiato!' : 'Copia'}
        </button>
      </div>
    </div>

  </div>
</div>
