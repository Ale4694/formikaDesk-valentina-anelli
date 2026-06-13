<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { fornitori, setError, setSuccess } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovoFornitore } from '../../lib/types'
  import ConfirmModal from '../ConfirmModal.svelte'

  const dispatch = createEventDispatcher()
  let showForm = false
  let saving = false
  let editId: number | null = null
  let search = ''

  let confirmOpen = false
  let confirmMessage = ''
  let pendingDeleteId: number | null = null

  const emptyForm = (): NuovoFornitore => ({
    ragione_sociale: '', partita_iva: '', codice_fiscale: '',
    indirizzo: '', citta: '', cap: '', provincia: '', telefono: '', email: '', note: ''
  })
  let form = emptyForm()

  $: filtered = search.trim()
    ? $fornitori.filter(f =>
        f.ragione_sociale.toLowerCase().includes(search.toLowerCase()) ||
        (f.partita_iva ?? '').includes(search) ||
        (f.citta ?? '').toLowerCase().includes(search.toLowerCase())
      )
    : $fornitori

  function startEdit(f: (typeof $fornitori)[0]) {
    editId = f.id
    form = {
      ragione_sociale: f.ragione_sociale,
      partita_iva: f.partita_iva ?? '',
      codice_fiscale: f.codice_fiscale ?? '',
      indirizzo: f.indirizzo ?? '',
      citta: f.citta ?? '',
      cap: f.cap ?? '',
      provincia: f.provincia ?? '',
      telefono: f.telefono ?? '',
      email: f.email ?? '',
      note: f.note ?? '',
    }
    showForm = true
  }

  function cancelForm() { showForm = false; editId = null; form = emptyForm() }

  async function save() {
    if (!form.ragione_sociale.trim()) return
    saving = true
    try {
      if (editId) {
        const updated = await api.fornitori.update(editId, form)
        fornitori.update(list => list.map(f => f.id === editId ? updated : f))
        setSuccess('Fornitore aggiornato')
      } else {
        const nuovo = await api.fornitori.create(form)
        fornitori.update(list => [...list, nuovo])
        setSuccess('Fornitore creato')
      }
      cancelForm(); dispatch('refresh')
    } catch (e: any) { setError(e?.message ?? 'Errore') } finally { saving = false }
  }

  function richiediElimina(id: number, nome: string) {
    pendingDeleteId = id
    confirmMessage = `Eliminare "${nome}"? L'operazione non può essere annullata.`
    confirmOpen = true
  }

  async function eseguiElimina() {
    if (pendingDeleteId === null) return
    const id = pendingDeleteId
    confirmOpen = false
    pendingDeleteId = null
    try {
      await api.fornitori.delete(id)
      fornitori.update(list => list.filter(f => f.id !== id))
      dispatch('refresh')
    } catch (e: any) { setError(e?.message ?? 'Errore eliminazione') }
  }
</script>

<ConfirmModal
  bind:open={confirmOpen}
  title="Elimina fornitore"
  message={confirmMessage}
  onConfirm={eseguiElimina}
  onCancel={() => { confirmOpen = false; pendingDeleteId = null }}
/>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Fornitori</h1>
    <button class="btn-primary" on:click={() => { cancelForm(); showForm = true }}>+ Nuovo fornitore</button>
  </div>

  {#if showForm}
    <div class="card p-5 space-y-4">
      <h2 class="text-sm font-semibold text-white">{editId ? 'Modifica fornitore' : 'Nuovo fornitore'}</h2>
      <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
        <div class="col-span-2 md:col-span-3"><label class="label">Ragione sociale *</label><input class="input" bind:value={form.ragione_sociale}/></div>
        <div><label class="label">Partita IVA</label><input class="input" bind:value={form.partita_iva}/></div>
        <div><label class="label">Codice fiscale</label><input class="input" bind:value={form.codice_fiscale}/></div>
        <div><label class="label">Telefono</label><input class="input" bind:value={form.telefono}/></div>
        <div class="col-span-2"><label class="label">Indirizzo</label><input class="input" bind:value={form.indirizzo}/></div>
        <div><label class="label">Città</label><input class="input" bind:value={form.citta}/></div>
        <div><label class="label">CAP</label><input class="input" bind:value={form.cap}/></div>
        <div><label class="label">Provincia</label><input class="input" bind:value={form.provincia} maxlength="2" placeholder="CL"/></div>
        <div class="col-span-2 md:col-span-3"><label class="label">Email</label><input class="input" type="email" bind:value={form.email}/></div>
        <div class="col-span-2 md:col-span-3"><label class="label">Note</label><input class="input" bind:value={form.note}/></div>
      </div>
      <div class="flex gap-2">
        <button class="btn-primary" on:click={save} disabled={saving}>{saving ? 'Salvo...' : 'Salva'}</button>
        <button class="btn-secondary" on:click={cancelForm}>Annulla</button>
      </div>
    </div>
  {/if}

  <input class="input max-w-sm" placeholder="Cerca fornitore..." bind:value={search}/>

  <div class="card overflow-hidden">
    <table class="w-full text-sm">
      <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
        <tr>
          <th class="px-4 py-3 text-left">Ragione sociale</th>
          <th class="px-4 py-3 text-left">P.IVA</th>
          <th class="px-4 py-3 text-left">Città</th>
          <th class="px-4 py-3 text-left">Telefono</th>
          <th class="px-4 py-3 text-left">Email</th>
          <th class="px-4 py-3"></th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-800">
        {#each filtered as f}
          <tr class="table-row-hover">
            <td class="px-4 py-3 font-medium text-gray-200">{f.ragione_sociale}</td>
            <td class="px-4 py-3 font-mono text-xs text-gray-400">{f.partita_iva ?? '—'}</td>
            <td class="px-4 py-3 text-gray-400">{f.citta ?? '—'}{#if f.provincia} ({f.provincia}){/if}</td>
            <td class="px-4 py-3 text-gray-400">{f.telefono ?? '—'}</td>
            <td class="px-4 py-3 text-gray-400">{f.email ?? '—'}</td>
            <td class="px-4 py-3">
              <div class="flex gap-1 justify-end">
                <button class="btn-secondary text-xs px-2 py-1" on:click={() => startEdit(f)}>Modifica</button>
                <button class="btn-danger text-xs px-2 py-1" on:click={() => richiediElimina(f.id, f.ragione_sociale)}>Elimina</button>
              </div>
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="6" class="py-16 text-center">
              <div class="flex flex-col items-center gap-3 text-gray-600">
                <svg class="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                    d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"/>
                </svg>
                <p class="text-sm">
                  {#if search.trim()}
                    Nessun fornitore corrisponde a "<span class="text-gray-400">{search}</span>"
                  {:else}
                    Nessun fornitore — aggiungi il primo fornitore
                  {/if}
                </p>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
