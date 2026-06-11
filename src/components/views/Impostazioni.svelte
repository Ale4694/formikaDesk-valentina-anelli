<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../../lib/api'
  import { setError, setSuccess, appConfig } from '../../lib/stores'
  import type { Impostazioni, ConfigIntestazione } from '../../lib/types'

  let loading = true
  let saving = false
  let savingAzienda = false

  // Form dati azienda (intestazione PDF/stampe)
  let formAzienda: ConfigIntestazione = {
    ragione_sociale: '',
    sottotitolo: '',
    indirizzo: '',
    cap_citta: '',
    piva: '',
    cf: '',
    telefono: '',
    email: '',
  }

  let form: Impostazioni = {
    ragione_sociale: '',
    partita_iva: '',
    codice_fiscale: '',
    indirizzo: '',
    cap: '',
    citta: '',
    provincia: '',
    codice_destinatario: '0000000',
    regime_fiscale: 'RF01',
  }

  const regimiFiscali = [
    { value: 'RF01', label: 'RF01 — Ordinario' },
    { value: 'RF02', label: 'RF02 — Contribuenti minimi' },
    { value: 'RF04', label: 'RF04 — Agricoltura e attività connesse' },
    { value: 'RF05', label: 'RF05 — Vendita sali e tabacchi' },
    { value: 'RF06', label: 'RF06 — Commercio fiammiferi' },
    { value: 'RF07', label: 'RF07 — Editoria' },
    { value: 'RF08', label: 'RF08 — Gestione servizi telefonia' },
    { value: 'RF09', label: 'RF09 — Rivendita documenti di trasporto' },
    { value: 'RF10', label: 'RF10 — Intrattenimenti, giochi' },
    { value: 'RF11', label: 'RF11 — Agenzie viaggi e turismo' },
    { value: 'RF12', label: 'RF12 — Agriturismo' },
    { value: 'RF13', label: 'RF13 — Vendite a domicilio' },
    { value: 'RF14', label: 'RF14 — Rivendita beni usati' },
    { value: 'RF15', label: 'RF15 — Agenzie di vendite all\'asta' },
    { value: 'RF16', label: 'RF16 — IVA per cassa P.A.' },
    { value: 'RF17', label: 'RF17 — IVA per cassa (art. 32-bis D.L. 83/2012)' },
    { value: 'RF18', label: 'RF18 — Altro' },
    { value: 'RF19', label: 'RF19 — Regime forfettario' },
  ]

  onMount(async () => {
    try {
      const data = await api.impostazioni.get()
      form = { ...form, ...data }
      // Inizializza form azienda dall'appConfig già caricato
      const cfg = await api.config.get()
      if (cfg.intestazione) {
        formAzienda = {
          ragione_sociale: cfg.intestazione.ragione_sociale ?? '',
          sottotitolo:     cfg.intestazione.sottotitolo     ?? '',
          indirizzo:       cfg.intestazione.indirizzo       ?? '',
          cap_citta:       cfg.intestazione.cap_citta       ?? '',
          piva:            cfg.intestazione.piva            ?? '',
          cf:              cfg.intestazione.cf              ?? '',
          telefono:        cfg.intestazione.telefono         ?? '',
          email:           cfg.intestazione.email           ?? '',
        }
      }
    } catch (e: any) {
      setError(e?.message ?? 'Errore caricamento impostazioni')
    } finally {
      loading = false
    }
  })

  async function salvaAzienda() {
    if (savingAzienda) return
    savingAzienda = true
    try {
      const updatedConfig = await api.config.salvaIntestazione({
        ragione_sociale: formAzienda.ragione_sociale,
        sottotitolo:     formAzienda.sottotitolo     || null,
        indirizzo:       formAzienda.indirizzo       || null,
        cap_citta:       formAzienda.cap_citta       || null,
        piva:            formAzienda.piva            || null,
        cf:              formAzienda.cf              || null,
        telefono:        formAzienda.telefono        || null,
        email:           formAzienda.email           || null,
      })
      appConfig.set(updatedConfig)
      setSuccess('Dati azienda salvati')
    } catch (e: any) {
      setError(e?.message ?? 'Errore salvataggio dati azienda')
    } finally {
      savingAzienda = false
    }
  }

  async function salva() {
    if (saving) return
    saving = true
    try {
      await api.impostazioni.save(form)
      setSuccess('Impostazioni salvate')
    } catch (e: any) {
      setError(e?.message ?? 'Errore salvataggio')
    } finally {
      saving = false
    }
  }

  function apriPortaleAde() {
    import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
      openUrl('https://ivaservizi.agenziaentrate.gov.it')
    })
  }
</script>

<div class="p-6 max-w-2xl space-y-6">
  <h1 class="text-xl font-semibold text-white">Impostazioni azienda</h1>

  {#if loading}
    <div class="flex items-center justify-center py-16">
      <div class="w-7 h-7 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <!-- Sezione Dati Azienda -->
    <form on:submit|preventDefault={salvaAzienda} class="card p-6 space-y-5">
      <div>
        <h2 class="text-sm font-semibold text-white">Dati Azienda</h2>
        <p class="text-xs text-gray-500 mt-1">
          Questi dati appaiono nell'intestazione delle fatture e dei documenti stampati.
        </p>
      </div>

      <div class="grid grid-cols-1 gap-4">
        <div>
          <label class="label" for="az_ragione_sociale">Ragione Sociale *</label>
          <input
            id="az_ragione_sociale"
            class="input"
            type="text"
            bind:value={formAzienda.ragione_sociale}
            placeholder="Es. ANELLI RICAMBI"
          />
        </div>

        <div>
          <label class="label" for="az_sottotitolo">Sottotitolo</label>
          <input
            id="az_sottotitolo"
            class="input"
            type="text"
            bind:value={formAzienda.sottotitolo}
            placeholder="Es. di Rossi Mario"
          />
        </div>

        <div>
          <label class="label" for="az_indirizzo">Indirizzo</label>
          <input
            id="az_indirizzo"
            class="input"
            type="text"
            bind:value={formAzienda.indirizzo}
            placeholder="Es. Via Roma, 1"
          />
        </div>

        <div>
          <label class="label" for="az_cap_citta">CAP e Città</label>
          <input
            id="az_cap_citta"
            class="input"
            type="text"
            bind:value={formAzienda.cap_citta}
            placeholder="Es. 92025 Casteltermini (AG)"
          />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="az_piva">Partita IVA</label>
            <input
              id="az_piva"
              class="input font-mono"
              type="text"
              maxlength="11"
              bind:value={formAzienda.piva}
              placeholder="01234567890"
            />
          </div>
          <div>
            <label class="label" for="az_cf">Codice Fiscale</label>
            <input
              id="az_cf"
              class="input font-mono uppercase"
              type="text"
              maxlength="16"
              bind:value={formAzienda.cf}
              placeholder="RSSMRA80A01H501Z"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="az_telefono">Telefono</label>
            <input
              id="az_telefono"
              class="input"
              type="text"
              bind:value={formAzienda.telefono}
              placeholder="Es. 0922 123456"
            />
          </div>
          <div>
            <label class="label" for="az_email">Email</label>
            <input
              id="az_email"
              class="input"
              type="email"
              bind:value={formAzienda.email}
              placeholder="info@esempio.it"
            />
          </div>
        </div>
      </div>

      <div class="flex justify-end pt-2">
        <button type="submit" class="btn-primary" disabled={savingAzienda}>
          {savingAzienda ? 'Salvataggio…' : 'Salva dati azienda'}
        </button>
      </div>
    </form>

    <!-- Sezione FatturaPA -->
    <form on:submit|preventDefault={salva} class="card p-6 space-y-5">
      <div>
        <h2 class="text-sm font-semibold text-white">Dati FatturaPA</h2>
        <p class="text-xs text-gray-500 mt-1">
          Questi dati vengono usati per generare il file XML FatturaPA. Compilali correttamente prima di esportare fatture elettroniche.
        </p>
      </div>

      <div class="grid grid-cols-1 gap-4">
        <div>
          <label class="label" for="ragione_sociale">Ragione sociale *</label>
          <input
            id="ragione_sociale"
            class="input"
            type="text"
            bind:value={form.ragione_sociale}
            placeholder="Es. Ricambi Rossi S.r.l."
          />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="partita_iva">Partita IVA *</label>
            <input
              id="partita_iva"
              class="input font-mono"
              type="text"
              maxlength="11"
              bind:value={form.partita_iva}
              placeholder="12345678901"
            />
          </div>
          <div>
            <label class="label" for="codice_fiscale">Codice fiscale</label>
            <input
              id="codice_fiscale"
              class="input font-mono"
              type="text"
              maxlength="16"
              bind:value={form.codice_fiscale}
              placeholder="RSSMRA80A01H501Z"
            />
          </div>
        </div>

        <div>
          <label class="label" for="indirizzo">Indirizzo sede legale</label>
          <input
            id="indirizzo"
            class="input"
            type="text"
            bind:value={form.indirizzo}
            placeholder="Via Roma, 1"
          />
        </div>

        <div class="grid grid-cols-3 gap-4">
          <div>
            <label class="label" for="cap">CAP</label>
            <input
              id="cap"
              class="input font-mono"
              type="text"
              maxlength="5"
              bind:value={form.cap}
              placeholder="00100"
            />
          </div>
          <div>
            <label class="label" for="citta">Città</label>
            <input
              id="citta"
              class="input"
              type="text"
              bind:value={form.citta}
              placeholder="Roma"
            />
          </div>
          <div>
            <label class="label" for="provincia">Provincia</label>
            <input
              id="provincia"
              class="input font-mono uppercase"
              type="text"
              maxlength="2"
              bind:value={form.provincia}
              placeholder="RM"
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="regime_fiscale">Regime fiscale</label>
            <select id="regime_fiscale" class="input" bind:value={form.regime_fiscale}>
              {#each regimiFiscali as r}
                <option value={r.value}>{r.label}</option>
              {/each}
            </select>
          </div>
          <div>
            <label class="label" for="codice_destinatario">Codice destinatario SDI</label>
            <input
              id="codice_destinatario"
              class="input font-mono uppercase"
              type="text"
              maxlength="7"
              bind:value={form.codice_destinatario}
              placeholder="0000000"
            />
            <p class="text-xs text-gray-600 mt-1">7 caratteri. Usa 0000000 per privati/consumatori.</p>
          </div>
        </div>
      </div>

      <div class="flex justify-end pt-2">
        <button type="submit" class="btn-primary" disabled={saving}>
          {saving ? 'Salvataggio…' : 'Salva impostazioni'}
        </button>
      </div>
    </form>

    <!-- Link portale AdE -->
    <div class="card p-4 flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-white">Portale Agenzia delle Entrate</p>
        <p class="text-xs text-gray-500 mt-0.5">Carica i file XML FatturaPA sul portale SDI dell'AdE</p>
      </div>
      <button
        class="btn-secondary text-xs flex items-center gap-1.5"
        on:click={apriPortaleAde}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
        </svg>
        Carica su portale AdE
      </button>
    </div>
  {/if}
</div>
