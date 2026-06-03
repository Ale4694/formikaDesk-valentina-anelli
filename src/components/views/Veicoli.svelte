<script lang="ts">
  import { clienti } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { Veicolo, NuovoVeicolo } from '../../lib/types'

  let expandedId: number | null = null
  let veicoliMap: Record<number, Veicolo[]> = {}
  let loadingId: number | null = null

  const emptyForm = (): NuovoVeicolo => ({
    cliente_id: 0,
    targa: '',
    marca: '',
    modello: '',
    anno: null,
    cilindrata: '',
    carburante: '',
    km_attuali: null,
    note: '',
  })

  let form: NuovoVeicolo = emptyForm()
  let editingId: number | null = null
  let formError = ''
  let formLoading = false

  async function toggleCliente(id: number) {
    if (expandedId === id) {
      expandedId = null
      return
    }
    expandedId = id
    if (!veicoliMap[id]) {
      loadingId = id
      try {
        veicoliMap[id] = await api.veicoli.getByCliente(id)
        veicoliMap = { ...veicoliMap }
      } finally {
        loadingId = null
      }
    }
  }

  function startAdd(clienteId: number) {
    form = { ...emptyForm(), cliente_id: clienteId }
    editingId = null
    formError = ''
  }

  function startEdit(v: Veicolo) {
    form = {
      cliente_id: v.cliente_id,
      targa: v.targa,
      marca: v.marca ?? '',
      modello: v.modello ?? '',
      anno: v.anno,
      cilindrata: v.cilindrata ?? '',
      carburante: v.carburante ?? '',
      km_attuali: v.km_attuali,
      note: v.note ?? '',
    }
    editingId = v.id
    formError = ''
  }

  function cancelForm() {
    form = emptyForm()
    editingId = null
    formError = ''
  }

  async function salva(clienteId: number) {
    if (!form.targa.trim()) { formError = 'Targa obbligatoria'; return }
    formError = ''
    formLoading = true
    try {
      if (editingId !== null) {
        const updated = await api.veicoli.update(editingId, form)
        veicoliMap[clienteId] = (veicoliMap[clienteId] ?? []).map(v => v.id === editingId ? updated : v)
      } else {
        const created = await api.veicoli.create(form)
        veicoliMap[clienteId] = [...(veicoliMap[clienteId] ?? []), created]
      }
      veicoliMap = { ...veicoliMap }
      cancelForm()
    } catch (e: any) {
      formError = e?.message ?? 'Errore salvataggio'
    } finally {
      formLoading = false
    }
  }

  async function elimina(v: Veicolo) {
    if (!confirm(`Eliminare il veicolo ${v.targa}?`)) return
    try {
      await api.veicoli.delete(v.id)
      veicoliMap[v.cliente_id] = (veicoliMap[v.cliente_id] ?? []).filter(x => x.id !== v.id)
      veicoliMap = { ...veicoliMap }
    } catch (e: any) {
      alert(e?.message ?? 'Errore eliminazione')
    }
  }
</script>

<div class="p-6 space-y-4">
  <h1 class="text-xl font-semibold text-white">Veicoli</h1>
  <p class="text-sm text-gray-500">Espandi un cliente per vedere e gestire i suoi veicoli.</p>

  {#if $clienti.length === 0}
    <div class="card p-8 text-center text-gray-500 text-sm">Nessun cliente presente.</div>
  {:else}
    <div class="card divide-y divide-gray-800">
      {#each $clienti as cliente}
        {@const expanded = expandedId === cliente.id}
        {@const isAdding = form.cliente_id === cliente.id && editingId === null && form.targa === '' && expanded}
        <div>
          <!-- Riga cliente -->
          <button
            class="w-full flex items-center justify-between px-4 py-3 text-left hover:bg-gray-800/50 transition-colors"
            on:click={() => toggleCliente(cliente.id)}
          >
            <div class="flex items-center gap-3">
              <svg class="w-4 h-4 text-gray-500 transition-transform {expanded ? 'rotate-90' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
              </svg>
              <span class="text-sm font-medium text-gray-200">{cliente.ragione_sociale}</span>
              {#if cliente.citta}<span class="text-xs text-gray-500">{cliente.citta}</span>{/if}
            </div>
            {#if loadingId === cliente.id}
              <div class="w-4 h-4 border border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            {:else if veicoliMap[cliente.id]}
              <span class="text-xs text-gray-500">{veicoliMap[cliente.id].length} veicoli</span>
            {/if}
          </button>

          {#if expanded}
            <div class="bg-gray-900/40 px-4 py-3 space-y-3">
              <!-- Tabella veicoli -->
              {#if (veicoliMap[cliente.id] ?? []).length > 0}
                <table class="w-full text-xs">
                  <thead>
                    <tr class="text-gray-500 border-b border-gray-800">
                      <th class="text-left pb-2 font-medium">Targa</th>
                      <th class="text-left pb-2 font-medium">Marca / Modello</th>
                      <th class="text-left pb-2 font-medium">Anno</th>
                      <th class="text-left pb-2 font-medium">KM</th>
                      <th class="text-left pb-2 font-medium">Carburante</th>
                      <th class="pb-2"></th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-gray-800/50">
                    {#each veicoliMap[cliente.id] as v}
                      {#if editingId === v.id}
                        <!-- Form modifica inline -->
                        <tr class="bg-gray-800/40">
                          <td class="py-2 pr-2"><input bind:value={form.targa} class="input-sm w-24" placeholder="Targa"/></td>
                          <td class="py-2 pr-2 flex gap-1">
                            <input bind:value={form.marca} class="input-sm w-20" placeholder="Marca"/>
                            <input bind:value={form.modello} class="input-sm w-24" placeholder="Modello"/>
                          </td>
                          <td class="py-2 pr-2"><input bind:value={form.anno} type="number" class="input-sm w-16" placeholder="Anno"/></td>
                          <td class="py-2 pr-2"><input bind:value={form.km_attuali} type="number" class="input-sm w-20" placeholder="KM"/></td>
                          <td class="py-2 pr-2"><input bind:value={form.carburante} class="input-sm w-20" placeholder="Carb."/></td>
                          <td class="py-2">
                            <div class="flex gap-1">
                              <button on:click={() => salva(cliente.id)} disabled={formLoading} class="btn-xs btn-primary">Salva</button>
                              <button on:click={cancelForm} class="btn-xs">Annulla</button>
                            </div>
                          </td>
                        </tr>
                      {:else}
                        <tr class="hover:bg-gray-800/30 group">
                          <td class="py-2 pr-3 font-mono text-brand-400">{v.targa}</td>
                          <td class="py-2 pr-3 text-gray-300">{[v.marca, v.modello].filter(Boolean).join(' ') || '—'}</td>
                          <td class="py-2 pr-3 text-gray-400">{v.anno ?? '—'}</td>
                          <td class="py-2 pr-3 text-gray-400">{v.km_attuali ? v.km_attuali.toLocaleString('it-IT') : '—'}</td>
                          <td class="py-2 pr-3 text-gray-400">{v.carburante ?? '—'}</td>
                          <td class="py-2">
                            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                              <button on:click={() => startEdit(v)} class="btn-xs">Modifica</button>
                              <button on:click={() => elimina(v)} class="btn-xs text-red-400 hover:text-red-300">Elimina</button>
                            </div>
                          </td>
                        </tr>
                      {/if}
                    {/each}
                  </tbody>
                </table>
              {:else}
                <p class="text-xs text-gray-600 italic">Nessun veicolo registrato.</p>
              {/if}

              <!-- Form aggiunta nuovo -->
              {#if form.cliente_id === cliente.id && editingId === null && form.targa !== ''}
                <div class="border border-gray-700 rounded-lg p-3 space-y-2">
                  <p class="text-xs font-medium text-gray-400">Nuovo veicolo</p>
                  <div class="grid grid-cols-3 gap-2">
                    <div>
                      <label class="label-xs">Targa *</label>
                      <input bind:value={form.targa} class="input-sm w-full" placeholder="AA000BB"/>
                    </div>
                    <div>
                      <label class="label-xs">Marca</label>
                      <input bind:value={form.marca} class="input-sm w-full" placeholder="Fiat"/>
                    </div>
                    <div>
                      <label class="label-xs">Modello</label>
                      <input bind:value={form.modello} class="input-sm w-full" placeholder="Panda"/>
                    </div>
                    <div>
                      <label class="label-xs">Anno</label>
                      <input bind:value={form.anno} type="number" class="input-sm w-full" placeholder="2020"/>
                    </div>
                    <div>
                      <label class="label-xs">Carburante</label>
                      <input bind:value={form.carburante} class="input-sm w-full" placeholder="Diesel"/>
                    </div>
                    <div>
                      <label class="label-xs">KM attuali</label>
                      <input bind:value={form.km_attuali} type="number" class="input-sm w-full" placeholder="50000"/>
                    </div>
                  </div>
                  {#if formError}<p class="text-xs text-red-400">{formError}</p>{/if}
                  <div class="flex gap-2">
                    <button on:click={() => salva(cliente.id)} disabled={formLoading} class="btn-sm btn-primary">
                      {formLoading ? 'Salvataggio…' : 'Aggiungi'}
                    </button>
                    <button on:click={cancelForm} class="btn-sm">Annulla</button>
                  </div>
                </div>
              {:else if form.cliente_id !== cliente.id || editingId !== null}
                <button
                  on:click={() => { form = { ...emptyForm(), cliente_id: cliente.id, targa: ' ' }; editingId = null }}
                  class="text-xs text-brand-400 hover:text-brand-300 flex items-center gap-1"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                  </svg>
                  Aggiungi veicolo
                </button>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .input-sm { @apply bg-gray-800 border border-gray-700 rounded px-2 py-1 text-xs text-white focus:outline-none focus:border-brand-500; }
  .label-xs { @apply block text-xs text-gray-500 mb-0.5; }
  .btn-xs { @apply text-xs px-2 py-1 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 transition-colors; }
  .btn-xs.btn-primary { @apply bg-brand-600 hover:bg-brand-500 text-white; }
  .btn-sm { @apply text-xs px-3 py-1.5 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 transition-colors; }
  .btn-sm.btn-primary { @apply bg-brand-600 hover:bg-brand-500 text-white; }
</style>
