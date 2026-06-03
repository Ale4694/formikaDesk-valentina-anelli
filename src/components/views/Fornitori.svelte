<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { fornitori, setError } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovoFornitore } from '../../lib/types'

  const dispatch = createEventDispatcher()
  let showForm = false
  let saving = false

  const emptyForm = (): NuovoFornitore => ({
    ragione_sociale: '', partita_iva: '', codice_fiscale: '',
    indirizzo: '', citta: '', cap: '', provincia: '', telefono: '', email: '', note: ''
  })
  let form = emptyForm()

  async function save() {
    if (!form.ragione_sociale.trim()) return
    saving = true
    try {
      const nuovo = await api.fornitori.create(form)
      fornitori.update(list => [...list, nuovo])
      form = emptyForm(); showForm = false; dispatch('refresh')
    } catch (e: any) { setError(e?.message ?? 'Errore') } finally { saving = false }
  }
</script>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Fornitori</h1>
    <button class="btn-primary" on:click={() => showForm = !showForm}>+ Nuovo fornitore</button>
  </div>

  {#if showForm}
    <div class="card p-5 space-y-4">
      <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
        <div class="col-span-2 md:col-span-3"><label class="label">Ragione sociale *</label><input class="input" bind:value={form.ragione_sociale}/></div>
        <div><label class="label">Partita IVA</label><input class="input" bind:value={form.partita_iva}/></div>
        <div><label class="label">Telefono</label><input class="input" bind:value={form.telefono}/></div>
        <div><label class="label">Email</label><input class="input" bind:value={form.email}/></div>
        <div><label class="label">Città</label><input class="input" bind:value={form.citta}/></div>
        <div><label class="label">Provincia</label><input class="input" bind:value={form.provincia} maxlength="2"/></div>
      </div>
      <div class="flex gap-2"><button class="btn-primary" on:click={save} disabled={saving}>{saving ? 'Salvo...' : 'Salva'}</button><button class="btn-secondary" on:click={() => showForm = false}>Annulla</button></div>
    </div>
  {/if}

  <div class="card overflow-hidden">
    <table class="w-full text-sm">
      <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
        <tr>
          <th class="px-4 py-3 text-left">Ragione sociale</th>
          <th class="px-4 py-3 text-left">P.IVA</th>
          <th class="px-4 py-3 text-left">Città</th>
          <th class="px-4 py-3 text-left">Telefono</th>
          <th class="px-4 py-3 text-left">Email</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-800">
        {#each $fornitori as f}
          <tr class="table-row-hover">
            <td class="px-4 py-3 font-medium text-gray-200">{f.ragione_sociale}</td>
            <td class="px-4 py-3 font-mono text-xs text-gray-400">{f.partita_iva ?? '—'}</td>
            <td class="px-4 py-3 text-gray-400">{f.citta ?? '—'}{#if f.provincia} ({f.provincia}){/if}</td>
            <td class="px-4 py-3 text-gray-400">{f.telefono ?? '—'}</td>
            <td class="px-4 py-3 text-gray-400">{f.email ?? '—'}</td>
          </tr>
        {:else}
          <tr>
            <td colspan="5" class="py-16 text-center">
              <div class="flex flex-col items-center gap-3 text-gray-600">
                <svg class="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                    d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                </svg>
                <p class="text-sm">Nessun fornitore — aggiungi il primo fornitore</p>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
