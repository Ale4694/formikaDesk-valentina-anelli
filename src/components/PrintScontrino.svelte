<script lang="ts">
  import type { ScontrinoCompleto } from '../lib/types'
  import { formatCurrency, appConfig } from '../lib/stores'

  export let scontrino: ScontrinoCompleto | null = null

  export function stampa() {
    if (!scontrino) return
    document.body.classList.add('print-scontrino')
    window.print()
    document.body.classList.remove('print-scontrino')
  }

  function sep(char = '-', n = 32) { return char.repeat(n) }

  $: s = scontrino?.scontrino
  $: righe = scontrino?.righe ?? []

  $: subtotaleRighe = righe.reduce((acc, r) => acc + r.totale_riga, 0)
  $: imponibileTotale = righe.reduce((acc, r) => {
    const imp = r.totale_riga / (1 + r.iva_percentuale / 100)
    return acc + imp
  }, 0)
  $: ivaTotale = subtotaleRighe - imponibileTotale
  $: resto = (s?.metodo_pagamento === 'contanti' && importoRicevuto > 0)
    ? importoRicevuto - (s?.totale ?? 0)
    : null

  export let importoRicevuto = 0

  function metodoLabel(m: string) {
    const map: Record<string, string> = { contanti: 'Contanti', carta: 'Carta', satispay: 'Satispay', bonifico: 'Bonifico' }
    return map[m] ?? m
  }

  function formatOra(dt: string) {
    try { return new Date(dt).toLocaleTimeString('it-IT', { hour: '2-digit', minute: '2-digit' }) } catch { return '' }
  }
  function formatData(d: string) {
    try { return new Date(d).toLocaleDateString('it-IT') } catch { return d }
  }
</script>

<!-- Hidden in normal view, shown on print when body.print-scontrino -->
<div id="scontrino-print-overlay" style="display:none">
  {#if s}
    <div style="max-width:300px; margin:0 auto; font-family:monospace; font-size:12px; color:#000; padding:8px">

      <!-- Intestazione -->
      <div style="text-align:center; margin-bottom:4px">
        <div style="font-size:16px; font-weight:bold">{$appConfig?.nome_attivita ?? 'FormikaDesk'}</div>
        <div style="font-size:10px">Gestionale Ricambi Auto</div>
      </div>
      <div style="text-align:center; font-size:10px; margin-bottom:4px">
        {formatData(s.data)} — {formatOra(s.created_at)}<br/>
        Scontrino n° <strong>{s.numero}</strong>
        {#if s.operatore}<br/>Operatore: {s.operatore}{/if}
      </div>

      <div>{sep()}</div>

      <!-- Righe -->
      {#each righe as r}
        <div style="margin:2px 0">
          <div style="font-size:10px; color:#444">{r.codice ?? ''}</div>
          <div style="white-space:nowrap; overflow:hidden; text-overflow:ellipsis">{r.descrizione}</div>
          <div style="display:flex; justify-content:space-between">
            <span>{r.quantita} x {formatCurrency(r.prezzo_unitario)}{r.sconto_percentuale > 0 ? ` (-${r.sconto_percentuale}%)` : ''}</span>
            <span><strong>{formatCurrency(r.totale_riga)}</strong></span>
          </div>
        </div>
      {/each}

      <div>{sep()}</div>

      <!-- Totali -->
      <div style="display:flex; justify-content:space-between; font-size:10px">
        <span>Imponibile</span><span>{formatCurrency(imponibileTotale)}</span>
      </div>
      <div style="display:flex; justify-content:space-between; font-size:10px">
        <span>IVA</span><span>{formatCurrency(ivaTotale)}</span>
      </div>
      {#if s.sconto_totale > 0}
        <div style="display:flex; justify-content:space-between; font-size:10px">
          <span>Sconto</span><span>-{formatCurrency(s.sconto_totale)}</span>
        </div>
      {/if}

      <div>{sep('=')}</div>
      <div style="display:flex; justify-content:space-between; font-size:16px; font-weight:bold">
        <span>TOTALE</span><span>{formatCurrency(s.totale)}</span>
      </div>
      <div>{sep('=')}</div>

      <div style="margin-top:4px; font-size:11px">
        Pagamento: <strong>{metodoLabel(s.metodo_pagamento)}</strong>
      </div>
      {#if s.metodo_pagamento === 'contanti' && importoRicevuto > 0}
        <div style="font-size:11px">Ricevuto: {formatCurrency(importoRicevuto)}</div>
        <div style="font-size:11px">Resto: <strong>{formatCurrency(Math.max(0, importoRicevuto - s.totale))}</strong></div>
      {/if}

      <div style="margin-top:8px; text-align:center; font-size:10px">
        {sep()}<br/>
        Grazie per il vostro acquisto!<br/>
        {sep()}
      </div>

    </div>
  {/if}
</div>
