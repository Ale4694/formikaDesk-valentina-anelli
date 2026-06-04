<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'
  import { api } from '../lib/api'
  import { licenseValid, licenseInfo, setSuccess, setError } from '../lib/stores'

  const dispatch = createEventDispatcher<{ close: void; deactivated: void }>()

  export let open = false

  let machineId = ''
  let licensed = false
  let copiato = false
  let confirmDisattiva = false

  $: if (open) {
    licensed = $licenseValid ?? false
    loadMachineId()
  }

  async function loadMachineId() {
    try {
      machineId = await api.license.getMachineId()
    } catch {}
  }

  async function disattiva() {
    try {
      await api.license.deactivateLicense()
      licenseValid.set(false)
      licenseInfo.set(null)
      setSuccess('Licenza disattivata')
      dispatch('deactivated')
      dispatch('close')
    } catch (e: any) {
      setError(e?.message ?? 'Errore durante la disattivazione')
    }
    confirmDisattiva = false
  }

  async function copiaMachineId() {
    if (!machineId) return
    try {
      await navigator.clipboard.writeText(machineId)
      copiato = true
      setTimeout(() => { copiato = false }, 2000)
    } catch {}
  }

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) dispatch('close')
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') dispatch('close')
  }
</script>

{#if open}
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    on:click={handleBackdrop}
    on:keydown={handleKeydown}
  >
    <div class="bg-gray-900 border border-gray-800 rounded-xl shadow-2xl w-full max-w-sm mx-4">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-800">
        <h2 class="text-base font-semibold text-white">Informazioni</h2>
        <button
          on:click={() => dispatch('close')}
          class="text-gray-500 hover:text-gray-300 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-5 py-5 space-y-5">

        <!-- Logo + nome -->
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 bg-brand-600 rounded-xl flex items-center justify-center shrink-0">
            <svg class="w-7 h-7 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M13 10V3L4 14h7v7l9-11h-7z"/>
            </svg>
          </div>
          <div>
            <p class="text-white font-semibold">AutoParts Gestionale</p>
            <p class="text-xs text-gray-500">Versione 0.1.0</p>
          </div>
        </div>

        <!-- Stato licenza -->
        <div class="bg-gray-800 rounded-lg px-4 py-3 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-400">Stato licenza</span>
            {#if licensed}
              <span class="flex items-center gap-1.5 text-xs text-green-400 font-medium">
                <span class="w-1.5 h-1.5 rounded-full bg-green-400"></span>
                Attivata
              </span>
            {:else}
              <span class="flex items-center gap-1.5 text-xs text-red-400 font-medium">
                <span class="w-1.5 h-1.5 rounded-full bg-red-400"></span>
                Non attivata
              </span>
            {/if}
          </div>
          {#if $licenseInfo?.tipo && $licenseInfo.tipo !== 'none'}
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-400">Tipo</span>
              <span class="text-xs text-gray-300 font-medium">
                {$licenseInfo.tipo === 'permanent' ? 'Permanente' : 'Demo'}
              </span>
            </div>
          {/if}
          {#if $licenseInfo?.tipo === 'demo' && $licenseInfo.scadenza}
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-400">Scadenza</span>
              <span class="text-xs text-yellow-400 font-medium">
                {$licenseInfo.scadenza} ({$licenseInfo.giorni_rimanenti} giorni rimanenti)
              </span>
            </div>
          {/if}

          <div>
            <p class="text-xs text-gray-400 mb-1">ID dispositivo</p>
            <div class="flex items-center gap-2">
              <code class="flex-1 text-xs font-mono text-gray-300 truncate select-all">
                {machineId || '…'}
              </code>
              <button
                on:click={copiaMachineId}
                disabled={!machineId}
                class="shrink-0 text-xs px-2 py-1 rounded bg-gray-700 hover:bg-gray-600
                  disabled:opacity-40 text-gray-300 transition-colors"
              >
                {copiato ? 'Copiato!' : 'Copia'}
              </button>
            </div>
          </div>
        </div>

        <!-- Disattiva licenza -->
        {#if licensed}
          {#if confirmDisattiva}
            <div class="bg-red-950 border border-red-800 rounded-lg px-4 py-3 space-y-3">
              <p class="text-xs text-red-300">
                Sei sicuro? L'app richiederà nuovamente la chiave di licenza.
              </p>
              <div class="flex gap-2">
                <button
                  on:click={() => confirmDisattiva = false}
                  class="flex-1 text-xs py-1.5 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 transition-colors"
                >
                  Annulla
                </button>
                <button
                  on:click={disattiva}
                  class="flex-1 text-xs py-1.5 rounded bg-red-700 hover:bg-red-600 text-white transition-colors font-medium"
                >
                  Disattiva
                </button>
              </div>
            </div>
          {:else}
            <button
              on:click={() => confirmDisattiva = true}
              class="w-full text-xs py-2 rounded-lg border border-red-800 text-red-400
                hover:bg-red-950 transition-colors"
            >
              Disattiva licenza
            </button>
          {/if}
        {/if}

      </div>
    </div>
  </div>
{/if}
