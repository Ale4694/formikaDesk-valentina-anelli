<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'
  import { api } from '../lib/api'

  const dispatch = createEventDispatcher<{ activated: void }>()

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
</script>

<div class="min-h-screen bg-gray-950 flex items-center justify-center p-6">
  <div class="w-full max-w-md">

    <!-- Logo -->
    <div class="flex flex-col items-center mb-8">
      <div class="w-16 h-16 bg-brand-600 rounded-2xl flex items-center justify-center mb-4 shadow-lg">
        <svg class="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
      </div>
      <h1 class="text-2xl font-bold text-white">AutoParts</h1>
      <p class="text-sm text-gray-500 mt-1">Gestionale</p>
    </div>

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
            placeholder="XXXX-XXXX-XXXX-XXXX"
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

    <!-- Machine ID -->
    <div class="mt-4 bg-gray-900 border border-gray-800 rounded-xl p-4">
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
