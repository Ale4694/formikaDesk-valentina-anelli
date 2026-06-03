<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { clienti, setError, setSuccess } from '../../lib/stores'
  import { api } from '../../lib/api'
  import type { NuovoCliente } from '../../lib/types'
  import ConfirmModal from '../ConfirmModal.svelte'

  type SortDir = 'asc' | 'desc'
  let sortCol = 'ragione_sociale'
  let sortDir: SortDir = 'asc'

  function toggleSort(col: string) {
    if (sortCol === col) { sortDir = sortDir === 'asc' ? 'desc' : 'asc' }
    else { sortCol = col; sortDir = 'asc' }
  }

  function sortVal(c: (typeof $clienti)[0], col: string): string | number {
    const v = (c as any)[col]
    return v === null || v === undefined ? (sortDir === 'asc' ? '￿' : '') : v
  }

  async function esportaCsv() {
    try {
      const csv = await api.backup.exportClientiCsv()
      const today = new Date().toISOString().slice(0, 10)
      const blob = new Blob(['﻿' + csv], { type: 'text/csv;charset=utf-8;' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url; a.download = `clienti_${today}.csv`; a.click()
      URL.revokeObjectURL(url)
      setSuccess('CSV clienti esportato')
    } catch (e: any) { setError(e?.message ?? 'Errore export') }
  }

  const dispatch = createEventDispatcher()
  let showForm = false
  let saving = false
  let editId: number | null = null
  let search = ''

  // Modal conferma eliminazione
  let confirmOpen = false
  let confirmMessage = ''
  let pendingDeleteId: number | null = null

  const emptyForm = (): NuovoCliente => ({
    ragione_sociale: '', partita_iva: '', codice_fiscale: '',
    indirizzo: '', citta: '', cap: '', provincia: '', telefono: '', email: '', note: ''
  })
  let form = emptyForm()

  $: filtered = search.trim()
    ? $clienti.filter(c => c.ragione_sociale.toLowerCase().includes(search.toLowerCase()) || (c.email ?? '').toLowerCase().includes(search.toLowerCase()))
    : $clienti

  $: sorted = [...filtered].sort((a, b) => {
    const av = sortVal(a, sortCol)
    const bv = sortVal(b, sortCol)
    if (typeof av === 'number' && typeof bv === 'number') return sortDir === 'asc' ? av - bv : bv - av
    const cmp = String(av).localeCompare(String(bv), 'it')
    return sortDir === 'asc' ? cmp : -cmp
  })

  function startEdit(c: (typeof $clienti)[0]) {
    editId = c.id; form = { ...c }; showForm = true
  }
  function cancelForm() { showForm = false; editId = null; form = emptyForm() }

  async function save() {
    if (!form.ragione_sociale.trim()) return
    saving = true
    try {
      if (editId) {
        const updated = await api.clienti.update(editId, form)
        clienti.update(list => list.map(c => c.id === editId ? updated : c))
      } else {
        const nuovo = await api.clienti.create(form)
        clienti.update(list => [...list, nuovo])
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
      await api.clienti.delete(id)
      clienti.update(list => list.filter(c => c.id !== id))
      dispatch('refresh')
    } catch (e: any) { setError(e?.message ?? 'Errore') }
  }
</script>

<ConfirmModal
  bind:open={confirmOpen}
  title="Elimina cliente"
  message={confirmMessage}
  onConfirm={eseguiElimina}
  onCancel={() => { confirmOpen = false; pendingDeleteId = null }}
/>

<div class="p-6 space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="text-xl font-semibold text-white">Clienti</h1>
    <div class="flex gap-2">
      <button class="btn-secondary text-xs" on:click={esportaCsv}>
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
        </svg>
        Esporta CSV
      </button>
      <button class="btn-primary" on:click={() => { cancelForm(); showForm = true }}>+ Nuovo cliente</button>
    </div>
  </div>

  {#if showForm}
    <div class="card p-5 space-y-4">
      <h2 class="text-sm font-semibold text-white">{editId ? 'Modifica cliente' : 'Nuovo cliente'}</h2>
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
      </div>
      <div class="flex gap-2"><button class="btn-primary" on:click={save} disabled={saving}>{saving ? 'Salvo...' : 'Salva'}</button><button class="btn-secondary" on:click={cancelForm}>Annulla</button></div>
    </div>
  {/if}

  <input class="input max-w-sm" placeholder="Cerca cliente..." bind:value={search}/>

  <div class="card overflow-hidden">
    <table class="w-full text-sm">
      <thead class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wide">
        <tr>
          {#each [
            { col: 'ragione_sociale', label: 'Ragione sociale' },
            { col: 'partita_iva',     label: 'P.IVA'           },
            { col: 'citta',           label: 'Città'            },
            { col: 'telefono',        label: 'Telefono'         },
            { col: 'email',           label: 'Email'            },
          ] as h}
            <th
              class="px-4 py-3 text-left cursor-pointer select-none hover:text-gray-200 whitespace-nowrap"
              on:click={() => toggleSort(h.col)}
            >
              {h.label}
              {#if sortCol === h.col}
                <span class="ml-0.5">{sortDir === 'asc' ? '↑' : '↓'}</span>
              {:else}
                <span class="ml-0.5 opacity-25">↕</span>
              {/if}
            </th>
          {/each}
          <th class="px-4 py-3"></th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-800">
        {#each sorted as c}
          <tr class="table-row-hover">
            <td class="px-4 py-3 font-medium text-gray-200">{c.ragione_sociale}</td>
            <td class="px-4 py-3 font-mono text-xs text-gray-400">{c.partita_iva ?? '—'}</td>
            <td class="px-4 py-3 text-gray-400">{c.citta ?? '—'}{#if c.provincia} ({c.provincia}){/if}</td>
            <td class="px-4 py-3 text-gray-400">{c.telefono ?? '—'}</td>
            <td class="px-4 py-3 text-gray-400">{c.email ?? '—'}</td>
            <td class="px-4 py-3"><div class="flex gap-1 justify-end"><button class="btn-secondary text-xs px-2 py-1" on:click={() => startEdit(c)}>Modifica</button><button class="btn-danger text-xs px-2 py-1" on:click={() => richiediElimina(c.id, c.ragione_sociale)}>Elimina</button></div></td>
          </tr>
        {:else}
          <tr>
            <td colspan="6" class="py-16 text-center">
              <div class="flex flex-col items-center gap-3 text-gray-600">
                <svg class="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                    d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"/>
                </svg>
                <p class="text-sm">
                  {#if search.trim()}
                    Nessun cliente corrisponde a "<span class="text-gray-400">{search}</span>"
                  {:else}
                    Nessun cliente — aggiungi il primo cliente
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
