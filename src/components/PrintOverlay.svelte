<script lang="ts">
  import { onMount } from 'svelte'
  import { printData, clienti, fornitori, formatCurrency, formatDate, appConfig } from '../lib/stores'

  const tipoLabel: Record<string, string> = {
    fattura:           'FATTURA',
    preventivo:        'PREVENTIVO',
    ddt:               'DOCUMENTO DI TRASPORTO',
    nota_credito:      'NOTA DI CREDITO',
    vendita_banco:     'VENDITA BANCO',
    buono:             'BUONO',
    fattura_differita: 'FATTURA DIFFERITA',
  }

  $: doc = $printData?.documento
  $: righe = $printData?.righe ?? []
  $: cliente = doc?.cliente_id != null
    ? $clienti.find(c => c.id === doc!.cliente_id) ?? null
    : null
  $: fornitore = doc?.fornitore_id != null
    ? $fornitori.find(f => f.id === doc!.fornitore_id) ?? null
    : null

  onMount(() => {
    window.addEventListener('afterprint', () => printData.set(null))
  })
</script>

<div id="print-overlay">
  <div class="pg-header">{$appConfig?.intestazione?.ragione_sociale ?? $appConfig?.nome_attivita ?? 'FormikaDesk'}</div>
  <div class="pg-footer">{$appConfig?.intestazione?.ragione_sociale ?? $appConfig?.nome_attivita ?? 'FormikaDesk'} — P.IVA {$appConfig?.intestazione?.piva ?? ''}</div>

  {#if $printData && doc}
    <div class="inv">
      <!-- Intestazione -->
      <div class="inv-head">
        <div class="inv-company">
          <div class="inv-company-name">{$appConfig?.intestazione?.ragione_sociale ?? $appConfig?.nome_attivita ?? 'FormikaDesk'}</div>
          {#if $appConfig?.intestazione?.sottotitolo}
            <div class="inv-company-sub">{$appConfig.intestazione.sottotitolo}</div>
          {/if}
          {#if $appConfig?.intestazione?.indirizzo}
            <div class="inv-company-detail">{$appConfig.intestazione.indirizzo}</div>
          {/if}
          {#if $appConfig?.intestazione?.cap_citta}
            <div class="inv-company-detail">{$appConfig.intestazione.cap_citta}</div>
          {/if}
          {#if $appConfig?.intestazione?.piva}
            <div class="inv-company-detail">P.IVA: {$appConfig.intestazione.piva}</div>
          {/if}
          {#if $appConfig?.intestazione?.cf}
            <div class="inv-company-detail">C.F.: {$appConfig.intestazione.cf}</div>
          {/if}
          {#if $appConfig?.intestazione?.telefono}
            <div class="inv-company-detail">Tel: {$appConfig.intestazione.telefono}</div>
          {/if}
        </div>
        <div class="inv-meta">
          <div class="inv-tipo">{tipoLabel[doc.tipo_documento] ?? doc.tipo_documento.toUpperCase()}</div>
          <div>N. <strong>{doc.numero}</strong> del {formatDate(doc.data)}</div>
          <div class="inv-stato">Stato: {doc.stato}</div>
        </div>
      </div>

      <hr class="inv-divider" />

      <!-- Destinatario -->
      {#if cliente || fornitore}
        {@const soggetto = cliente ?? fornitore}
        <div class="inv-dest">
          <div class="inv-dest-label">Intestato a</div>
          <div class="inv-dest-name">{soggetto!.ragione_sociale}</div>
          {#if soggetto!.partita_iva}<div>P.IVA: {soggetto!.partita_iva}</div>{/if}
          {#if soggetto!.codice_fiscale}<div>C.F.: {soggetto!.codice_fiscale}</div>{/if}
          {#if soggetto!.indirizzo}<div>{soggetto!.indirizzo}</div>{/if}
          {#if soggetto!.citta}
            <div>{soggetto!.citta}{soggetto!.cap ? ' ' + soggetto!.cap : ''}{soggetto!.provincia ? ' (' + soggetto!.provincia + ')' : ''}</div>
          {/if}
          {#if soggetto!.email}<div>{soggetto!.email}</div>{/if}
        </div>
      {/if}

      <!-- Righe -->
      <table class="inv-table">
        <thead>
          <tr>
            <th class="inv-th inv-th-left">Descrizione</th>
            <th class="inv-th inv-th-right">Qtà</th>
            <th class="inv-th inv-th-right">Prezzo unit.</th>
            <th class="inv-th inv-th-right">Sconto</th>
            <th class="inv-th inv-th-right">IVA</th>
            <th class="inv-th inv-th-right">Totale riga</th>
          </tr>
        </thead>
        <tbody>
          {#each righe as r}
            <tr class="inv-tr">
              <td class="inv-td">{r.descrizione}</td>
              <td class="inv-td inv-td-right">{r.quantita}</td>
              <td class="inv-td inv-td-right">{formatCurrency(r.prezzo_unitario)}</td>
              <td class="inv-td inv-td-right">{r.sconto_percentuale > 0 ? r.sconto_percentuale + '%' : '—'}</td>
              <td class="inv-td inv-td-right">{r.iva_percentuale}%</td>
              <td class="inv-td inv-td-right inv-td-bold">{formatCurrency(r.totale_riga)}</td>
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr class="inv-tfoot-row">
            <td colspan="5" class="inv-tfoot-label">Imponibile</td>
            <td class="inv-tfoot-val">{formatCurrency(doc.totale_imponibile)}</td>
          </tr>
          <tr class="inv-tfoot-row">
            <td colspan="5" class="inv-tfoot-label">IVA</td>
            <td class="inv-tfoot-val">{formatCurrency(doc.totale_iva)}</td>
          </tr>
          <tr class="inv-tfoot-total">
            <td colspan="5" class="inv-tfoot-total-label">TOTALE DOCUMENTO</td>
            <td class="inv-tfoot-total-val">{formatCurrency(doc.totale_documento)}</td>
          </tr>
        </tfoot>
      </table>

      {#if doc.data_pagamento || doc.metodo_pagamento}
        <div class="inv-pag">
          <div class="inv-pag-title">Pagamento registrato</div>
          <div class="inv-pag-row">
            {#if doc.data_pagamento}<span><strong>Data:</strong> {formatDate(doc.data_pagamento)}</span>{/if}
            {#if doc.metodo_pagamento}<span><strong>Metodo:</strong> {doc.metodo_pagamento}</span>{/if}
            {#if doc.riferimento_pagamento}<span><strong>Riferimento:</strong> {doc.riferimento_pagamento}</span>{/if}
          </div>
          {#if doc.note_pagamento}<div class="inv-pag-note">{doc.note_pagamento}</div>{/if}
        </div>
      {/if}

      {#if doc.note}
        <div class="inv-note">
          <span class="inv-note-label">Note:</span> {doc.note}
        </div>
      {/if}

      <div class="inv-footer">
        Documento generato il {new Date().toLocaleDateString('it-IT')}
      </div>
    </div>
  {/if}
</div>

<style lang="css">
  #print-overlay {
    display: none;
  }

  /* Stili dell'anteprima di stampa — visibili solo a @media print via app.css */
  :global(#print-overlay) .inv {
    font-family: Arial, Helvetica, sans-serif;
    font-size: 11pt;
    color: #111;
    padding: 15mm 20mm;
    max-width: 190mm;
  }

  :global(#print-overlay) .inv-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 6mm;
  }

  :global(#print-overlay) .inv-company-name {
    font-size: 18pt;
    font-weight: 700;
    color: #1a3a6b;
  }

  :global(#print-overlay) .inv-company-sub {
    font-size: 9pt;
    color: #444;
    margin-top: 1mm;
    font-style: italic;
  }

  :global(#print-overlay) .inv-company-detail {
    font-size: 8.5pt;
    color: #555;
    margin-top: 0.5mm;
    line-height: 1.4;
  }

  :global(#print-overlay) .inv-meta {
    text-align: right;
    font-size: 10pt;
  }

  :global(#print-overlay) .inv-tipo {
    font-size: 14pt;
    font-weight: 700;
    color: #1a3a6b;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .inv-stato {
    margin-top: 1mm;
    font-size: 9pt;
    color: #666;
    text-transform: capitalize;
  }

  :global(#print-overlay) .inv-divider {
    border: none;
    border-top: 1.5pt solid #1a3a6b;
    margin: 4mm 0;
  }

  :global(#print-overlay) .inv-dest {
    margin-bottom: 8mm;
    font-size: 10pt;
    line-height: 1.5;
  }

  :global(#print-overlay) .inv-dest-label {
    font-size: 8pt;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .inv-dest-name {
    font-weight: 700;
    font-size: 12pt;
  }

  :global(#print-overlay) .inv-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 5mm;
    font-size: 10pt;
  }

  :global(#print-overlay) .inv-th {
    background: #1a3a6b;
    color: #fff;
    padding: 2mm 3mm;
    font-size: 9pt;
    font-weight: 600;
  }

  :global(#print-overlay) .inv-th-left { text-align: left; }
  :global(#print-overlay) .inv-th-right { text-align: right; }

  :global(#print-overlay) .inv-tr:nth-child(even) td {
    background: #f4f7fb;
  }

  :global(#print-overlay) .inv-td {
    padding: 2mm 3mm;
    border-bottom: 0.5pt solid #ddd;
    vertical-align: top;
  }

  :global(#print-overlay) .inv-td-right { text-align: right; }
  :global(#print-overlay) .inv-td-bold { font-weight: 600; }

  :global(#print-overlay) .inv-tfoot-row td {
    padding: 1.5mm 3mm;
    font-size: 10pt;
    border-top: 0.5pt solid #ddd;
  }

  :global(#print-overlay) .inv-tfoot-label { text-align: right; color: #555; }
  :global(#print-overlay) .inv-tfoot-val { text-align: right; }

  :global(#print-overlay) .inv-tfoot-total td {
    padding: 2mm 3mm;
    border-top: 1.5pt solid #1a3a6b;
    border-bottom: 1.5pt solid #1a3a6b;
    background: #f0f4ff;
  }

  :global(#print-overlay) .inv-tfoot-total-label {
    text-align: right;
    font-weight: 700;
    font-size: 11pt;
  }

  :global(#print-overlay) .inv-tfoot-total-val {
    text-align: right;
    font-weight: 700;
    font-size: 13pt;
    color: #1a3a6b;
  }

  :global(#print-overlay) .inv-pag {
    margin-top: 5mm;
    padding: 2.5mm 3mm;
    border: 0.5pt solid #22c55e;
    border-left: 3pt solid #22c55e;
    background: #f0fdf4;
    border-radius: 1mm;
    font-size: 9.5pt;
  }

  :global(#print-overlay) .inv-pag-title {
    font-weight: 700;
    font-size: 9pt;
    color: #15803d;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .inv-pag-row {
    display: flex;
    gap: 6mm;
    flex-wrap: wrap;
    color: #222;
  }

  :global(#print-overlay) .inv-pag-note {
    margin-top: 1mm;
    color: #555;
    font-size: 9pt;
  }

  :global(#print-overlay) .inv-note {
    margin-top: 5mm;
    padding: 2mm 3mm;
    border-left: 2pt solid #1a3a6b;
    background: #f4f7fb;
    font-size: 9.5pt;
  }

  :global(#print-overlay) .inv-note-label {
    font-weight: 600;
  }

  :global(#print-overlay) .inv-footer {
    margin-top: 10mm;
    text-align: center;
    font-size: 8pt;
    color: #999;
  }

  :global(#print-overlay) .pg-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    padding: 2.5mm 1cm;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 9pt;
    font-weight: 700;
    color: #1a3a6b;
    background: #fff;
    border-bottom: 0.5pt solid #d0d8e8;
  }

  :global(#print-overlay) .pg-footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 2mm 1cm;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 8pt;
    color: #999;
    background: #fff;
    border-top: 0.5pt solid #d0d8e8;
    text-align: center;
  }
</style>
