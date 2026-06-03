<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  export let show = false
  export let numeroDocumento = ''

  const dispatch = createEventDispatcher<{
    conferma: {
      data_pagamento: string
      metodo_pagamento: string
      riferimento_pagamento: string | null
      note_pagamento: string | null
    }
    annulla: void
  }>()

  const metodi = ['Bonifico', 'Contanti', 'Carta', 'Assegno', 'Satispay']

  let dataPagamento = new Date().toISOString().slice(0, 10)
  let metodoPagamento = 'Bonifico'
  let riferimento = ''
  let note = ''

  function reset() {
    dataPagamento = new Date().toISOString().slice(0, 10)
    metodoPagamento = 'Bonifico'
    riferimento = ''
    note = ''
  }

  function conferma() {
    dispatch('conferma', {
      data_pagamento: dataPagamento,
      metodo_pagamento: metodoPagamento,
      riferimento_pagamento: riferimento.trim() || null,
      note_pagamento: note.trim() || null,
    })
    reset()
  }

  function annulla() {
    dispatch('annulla')
    reset()
  }
</script>

{#if show}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
    on:click|self={annulla}
    on:keydown={e => e.key === 'Escape' && annulla()}
  >
    <div class="bg-gray-900 border border-gray-700 rounded-xl shadow-2xl w-full max-w-md p-6 space-y-5">
      <div>
        <h2 class="text-lg font-semibold text-white">Registra pagamento</h2>
        {#if numeroDocumento}
          <p class="text-sm text-gray-400 mt-0.5">Documento <span class="font-mono text-brand-400">{numeroDocumento}</span></p>
        {/if}
      </div>

      <div class="space-y-4">
        <div>
          <label class="label" for="data-pag">Data pagamento</label>
          <input
            id="data-pag"
            type="date"
            class="input"
            bind:value={dataPagamento}
          />
        </div>

        <div>
          <label class="label" for="metodo-pag">Metodo pagamento</label>
          <select id="metodo-pag" class="input" bind:value={metodoPagamento}>
            {#each metodi as m}
              <option value={m}>{m}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="label" for="rif-pag">Riferimento <span class="text-gray-600 font-normal">(es. numero bonifico)</span></label>
          <input
            id="rif-pag"
            type="text"
            class="input"
            placeholder="Opzionale"
            bind:value={riferimento}
          />
        </div>

        <div>
          <label class="label" for="note-pag">Note</label>
          <textarea
            id="note-pag"
            class="input resize-none"
            rows="2"
            placeholder="Opzionale"
            bind:value={note}
          ></textarea>
        </div>
      </div>

      <div class="flex gap-3 pt-1">
        <button class="btn-secondary flex-1" on:click={annulla}>Annulla</button>
        <button class="btn-primary flex-1" on:click={conferma}>Conferma pagamento</button>
      </div>
    </div>
  </div>
{/if}
