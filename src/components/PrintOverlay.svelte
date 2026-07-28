<script lang="ts">
  import { onMount } from 'svelte'
  import { printData, clienti, ricambi, formatCurrency, formatDate, appConfig } from '../lib/stores'

  const tipoLabelMap: Record<string, string> = {
    fattura:           'FATTURA IMMEDIATA',
    preventivo:        'PREVENTIVO',
    ddt:               'DDT',
    nota_credito:      'NOTA DI CREDITO',
    vendita_banco:     'VENDITA BANCO',
    buono:             'BUONO',
    fattura_differita: 'FATTURA DIFFERITA',
    ddt_fornitore:     'DDT FORNITORE',
  }

  $: doc = $printData?.documento
  $: righe = $printData?.righe ?? []
  $: isGruppoA = doc?.tipo_documento === 'buono' || doc?.tipo_documento === 'preventivo'

  $: cliente = doc?.cliente_id != null
    ? $clienti.find(c => c.id === doc!.cliente_id) ?? null
    : null

  $: righeConInfo = righe.map(r => ({
    ...r,
    codice: r.ricambio_id != null ? ($ricambi.find(rc => rc.id === r.ricambio_id)?.codice_interno ?? '') : '',
    um: 'PZ',
  }))

  $: righeVuoteA = Array.from({ length: Math.max(0, 18 - righeConInfo.length) })
  $: righeVuoteB = Array.from({ length: Math.max(0, 10 - righeConInfo.length) })

  $: riepilogoIvaMap = righe.reduce((acc, r) => {
    const k = r.iva_percentuale
    if (!acc[k]) acc[k] = { aliquota: k, imponibile: 0, imposta: 0 }
    acc[k].imponibile += r.imponibile
    acc[k].imposta += r.totale_iva
    return acc
  }, {} as Record<number, { aliquota: number; imponibile: number; imposta: number }>)
  $: riepilogoIvaRows = Object.values(riepilogoIvaMap).sort((a, b) => a.aliquota - b.aliquota)

  $: oraCorrente = new Date().toLocaleTimeString('it-IT', { hour: '2-digit', minute: '2-digit' })
  $: tipoLabel = doc ? (tipoLabelMap[doc.tipo_documento] ?? doc.tipo_documento.toUpperCase()) : ''

  function formatDataOra(v: string | null | undefined): string | null {
    if (!v) return null
    const d = new Date(v)
    if (isNaN(d.getTime())) return null
    return d.toLocaleString('it-IT', { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' })
  }

  onMount(() => {
    window.addEventListener('afterprint', () => printData.set(null))
  })
</script>

<div id="print-overlay">
  <div class="pg-footer">
    {$appConfig?.intestazione?.ragione_sociale ?? $appConfig?.nome_attivita ?? ''} — P.IVA {$appConfig?.intestazione?.piva ?? ''}
  </div>

  {#if $printData && doc}
    {#if isGruppoA}

      <!-- ══════ GRUPPO A: BUONI E PREVENTIVI ══════ -->
      <div class="doc-buono">

        <!-- SEZIONE 1: Intestazione affiancata -->
        <div class="row-header-a">
          <div class="box-a box-azienda">
            <div class="azienda-nome">{$appConfig?.intestazione?.ragione_sociale ?? $appConfig?.nome_attivita ?? ''}</div>
            {#if $appConfig?.intestazione?.indirizzo}
              <div class="azienda-detail">{$appConfig.intestazione.indirizzo}</div>
            {/if}
            {#if $appConfig?.intestazione?.cap_citta}
              <div class="azienda-detail">{$appConfig.intestazione.cap_citta}</div>
            {/if}
            {#if $appConfig?.intestazione?.piva || $appConfig?.intestazione?.cf}
              <div class="azienda-detail">P.IVA: {$appConfig?.intestazione?.piva ?? '—'} · C.F.: {$appConfig?.intestazione?.cf ?? '—'}</div>
            {/if}
            {#if $appConfig?.intestazione?.telefono}
              <div class="azienda-detail">Tel: {$appConfig.intestazione.telefono}</div>
            {/if}
          </div>
          <div class="box-a box-cliente-a">
            <div class="cell-label">INTESTATARIO</div>
            <div class="cell-body">
              {#if cliente}
                <div class="cliente-nome">{cliente.ragione_sociale}</div>
                {#if cliente.indirizzo}<div class="cliente-detail">{cliente.indirizzo}</div>{/if}
                {#if cliente.citta}
                  <div class="cliente-detail">{cliente.citta}{cliente.provincia ? ' (' + cliente.provincia + ')' : ''}</div>
                {/if}
                {#if cliente.partita_iva}<div class="cliente-detail">P.IVA: {cliente.partita_iva}</div>{/if}
                {#if cliente.telefono}<div class="cliente-detail">Tel: {cliente.telefono}</div>{/if}
              {:else}
                <div class="cliente-nome">—</div>
              {/if}
            </div>
          </div>
        </div>

        <!-- SEZIONE 2: Tipo documento + Numero + Data -->
        <div class="row-tipo">
          <div class="tipo-badge">{tipoLabel}</div>
          <div class="tipo-meta">N° {doc.numero} del {formatDate(doc.data)}</div>
        </div>

        <!-- SEZIONE 3: Modalità pagamento + Causale -->
        <div class="row-pag-a">
          <div class="box-a box-pag-cell">
            <div class="cell-label">MODALITÀ PAGAMENTO</div>
            <div class="cell-body"><span class="cell-val">{doc.metodo_pagamento || 'RIMESSA DIRETTA'}</span></div>
          </div>
          <div class="box-a box-pag-cell">
            <div class="cell-label">CAUSALE</div>
            <div class="cell-body"><span class="cell-val">{doc.causale_trasporto || 'CONSEGNA'}</span></div>
          </div>
        </div>

        <!-- SEZIONE 4: Tabella articoli -->
        <table class="doc-table">
          <thead>
            <tr>
              <th class="th-codice">CODICE</th>
              <th class="th-desc">DESCRIZIONE</th>
              <th class="th-um">UM</th>
              <th class="th-qty">QTÀ</th>
              <th class="th-price">PREZZO</th>
              <th class="th-sconto">SCONTO</th>
              <th class="th-importo">IMPORTO</th>
            </tr>
          </thead>
          <tbody>
            {#each righeConInfo as r, i}
              <tr class:tr-alt={i % 2 === 1}>
                <td class="td-codice">{r.codice}</td>
                <td class="td-desc">{r.descrizione}</td>
                <td class="td-um">{r.um}</td>
                <td class="td-qty">{r.quantita}</td>
                <td class="td-price">{formatCurrency(r.prezzo_unitario)}</td>
                <td class="td-sconto">{r.sconto_percentuale > 0 ? r.sconto_percentuale + '%' : ''}</td>
                <td class="td-importo">{formatCurrency(r.totale_riga)}</td>
              </tr>
            {/each}
            {#each righeVuoteA as _, i}
              <tr class:tr-alt={(righeConInfo.length + i) % 2 === 1}>
                <td class="td-codice td-empty">&nbsp;</td>
                <td class="td-desc td-empty">&nbsp;</td>
                <td class="td-um td-empty">&nbsp;</td>
                <td class="td-qty td-empty">&nbsp;</td>
                <td class="td-price td-empty">&nbsp;</td>
                <td class="td-sconto td-empty">&nbsp;</td>
                <td class="td-importo td-empty">&nbsp;</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <!-- SEZIONE 5: Footer documento -->
        <div class="row-footer1">
          <div class="footer1-trasp">
            <div class="cell-label">TRASP. A CURA</div>
            <div class="cell-body"><span class="cell-val">{doc.trasporto_a_cura || 'Destinatario'}</span></div>
          </div>
          <div class="footer1-totale-label">TOTALE</div>
          <div class="footer1-totale-val">{formatCurrency(doc.totale_documento)}</div>
        </div>
        <div class="row-footer2">
          <div class="footer2-aspetto">
            <div class="cell-label">ASPETTO ESTERIORE</div>
            <div class="cell-body">{#if doc.aspetto_esteriore_beni}<span class="cell-val">{doc.aspetto_esteriore_beni}</span>{:else}&nbsp;{/if}</div>
          </div>
          <div class="footer2-colli">
            <div class="cell-label">N° COLLI</div>
            <div class="cell-body"><span class="cell-val">{doc.n_colli ?? 0}</span></div>
          </div>
          <div class="footer2-ore">
            <div class="cell-label">ALLE ORE</div>
            <div class="cell-body"><span class="cell-val">{doc.data_ora_ritiro || oraCorrente}</span></div>
          </div>
        </div>

        <!-- SEZIONE 6: Firma -->
        <div class="row-firma-a">
          <div class="firma-box-a">
            <div class="firma-label-a">Firma cliente per ricevuta</div>
            <div class="firma-linea-a">X</div>
          </div>
        </div>

      </div>

    {:else}

      <!-- ══════ GRUPPO B: FATTURE E DDT ══════ -->
      <div class="doc-fattura">

        <!-- SEZIONE 1: Header a 3 colonne -->
        <div class="row-header3">
          <div class="box-b box-mittente">
            <div class="cell-label">MITTENTE</div>
            <div class="cell-body">
              <div class="mit-nome">{$appConfig?.intestazione?.ragione_sociale ?? $appConfig?.nome_attivita ?? ''}</div>
              {#if $appConfig?.intestazione?.sottotitolo}
                <div class="mit-sottotitolo">{$appConfig.intestazione.sottotitolo}</div>
              {/if}
              {#if $appConfig?.intestazione?.indirizzo}
                <div class="mit-detail">{$appConfig.intestazione.indirizzo}</div>
              {/if}
              {#if $appConfig?.intestazione?.cap_citta}
                <div class="mit-detail">{$appConfig.intestazione.cap_citta}</div>
              {/if}
              {#if $appConfig?.intestazione?.piva}
                <div class="mit-detail">P.IVA {$appConfig.intestazione.piva}</div>
              {/if}
              {#if $appConfig?.intestazione?.cf}
                <div class="mit-detail">C.F. {$appConfig.intestazione.cf}</div>
              {/if}
              {#if $appConfig?.intestazione?.telefono}
                <div class="mit-detail">Tel. {$appConfig.intestazione.telefono}</div>
              {/if}
            </div>
          </div>

          <div class="box-b box-cliente-dest">
            <div class="cell-label">CLIENTE</div>
            <div class="cell-body">
              {#if cliente}
                <div class="cli-nome">{cliente.ragione_sociale}</div>
                {#if cliente.indirizzo}<div class="cli-detail">{cliente.indirizzo}</div>{/if}
                {#if cliente.citta}
                  <div class="cli-detail">{cliente.citta}{cliente.cap ? ' ' + cliente.cap : ''}{cliente.provincia ? ' (' + cliente.provincia + ')' : ''}</div>
                {/if}
                {#if cliente.partita_iva}<div class="cli-detail">P.IVA {cliente.partita_iva}</div>{/if}
                {#if cliente.codice_fiscale}<div class="cli-detail">C.F. {cliente.codice_fiscale}</div>{/if}
              {:else}
                <div class="cli-nome">—</div>
              {/if}
            </div>
            <div class="cell-label dest-sep">DESTINAZIONE</div>
            <div class="cell-body"><span class="cli-dest">IDEM</span></div>
          </div>

          <div class="box-b box-documento">
            <div class="doc-tipo-badge">{tipoLabel}</div>
            <div class="cell-body doc-body">
              <div class="doc-num-label">NUMERO</div>
              <div class="doc-num">{doc.numero}</div>
              <div class="doc-data-label">DEL</div>
              <div class="doc-data">{formatDate(doc.data)}</div>
              <div class="doc-subrow">
                <div>
                  <div class="doc-sub-label">COD. CLIENTE</div>
                  <div class="doc-sub-val">{doc.cliente_id ?? ''}</div>
                </div>
                <div>
                  <div class="doc-sub-label">AGENTE</div>
                  <div class="doc-sub-val">{#if doc.agente}{doc.agente}{:else}&nbsp;{/if}</div>
                </div>
                <div class="doc-pagina">Pag. 1 di 1</div>
              </div>
            </div>
          </div>
        </div>

        <!-- SEZIONE 2: Causale + Modalità + Banca + Scadenze -->
        <div class="row-causale">
          <div class="caus-cell" style="width:28%">
            <div class="cell-label">CAUSALE DEL TRASPORTO</div>
            <div class="cell-body"><span class="cell-val">{doc.causale_trasporto || 'Vendita'}</span></div>
          </div>
          <div class="caus-cell" style="width:25%">
            <div class="cell-label">MODALITÀ PAGAMENTO</div>
            <div class="cell-body"><span class="cell-val">{doc.metodo_pagamento || 'RIMESSA DIRETTA'}</span></div>
          </div>
          <div class="caus-cell" style="width:27%">
            <div class="cell-label">BANCA D'APPOGGIO</div>
            <div class="cell-body">{#if doc.banca_appoggio}<span class="cell-val">{doc.banca_appoggio}</span>{:else}&nbsp;{/if}</div>
          </div>
          <div class="caus-cell" style="width:20%; border-right:none">
            <div class="cell-label">SCADENZE</div>
            <div class="cell-body">{#if doc.scadenza_pagamento}<span class="cell-val">{formatDate(doc.scadenza_pagamento)}</span>{:else}&nbsp;{/if}</div>
          </div>
        </div>

        <!-- SEZIONE 3: Tabella articoli -->
        <table class="doc-table">
          <thead>
            <tr>
              <th class="th-codice">COD. ART.</th>
              <th class="th-desc">DESCRIZIONE</th>
              <th class="th-um">UM</th>
              <th class="th-qty">QTÀ</th>
              <th class="th-price">PREZZO</th>
              <th class="th-sconto">SCONTO</th>
              <th class="th-importo">IMPORTO</th>
              <th class="th-iva">IVA</th>
            </tr>
          </thead>
          <tbody>
            {#each righeConInfo as r, i}
              <tr class:tr-alt={i % 2 === 1}>
                <td class="td-codice">{r.codice}</td>
                <td class="td-desc">{r.descrizione}</td>
                <td class="td-um">{r.um}</td>
                <td class="td-qty">{r.quantita}</td>
                <td class="td-price">{formatCurrency(r.prezzo_unitario)}</td>
                <td class="td-sconto">{r.sconto_percentuale > 0 ? r.sconto_percentuale + '%' : ''}</td>
                <td class="td-importo">{formatCurrency(r.totale_riga)}</td>
                <td class="td-iva">{r.iva_percentuale}%</td>
              </tr>
            {/each}
            {#each righeVuoteB as _, i}
              <tr class:tr-alt={(righeConInfo.length + i) % 2 === 1}>
                <td class="td-codice td-empty">&nbsp;</td>
                <td class="td-desc td-empty">&nbsp;</td>
                <td class="td-um td-empty">&nbsp;</td>
                <td class="td-qty td-empty">&nbsp;</td>
                <td class="td-price td-empty">&nbsp;</td>
                <td class="td-sconto td-empty">&nbsp;</td>
                <td class="td-importo td-empty">&nbsp;</td>
                <td class="td-iva td-empty">&nbsp;</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <!-- SEZIONE 4: Riga spese -->
        <div class="row-spese">
          <div class="spese-cell">
            <div class="spese-label">TOTALE MERCE</div>
            <div class="spese-val">{formatCurrency(doc.totale_imponibile)}</div>
          </div>
          <div class="spese-cell">
            <div class="spese-label">BOLLI ES. ART.15</div>
            <div class="spese-val">{#if doc.bolli_art15}{doc.bolli_art15}{:else}&nbsp;{/if}</div>
          </div>
          <div class="spese-cell">
            <div class="spese-label">SPESE VARIE</div>
            <div class="spese-val">{#if doc.spese_varie}{formatCurrency(doc.spese_varie)}{:else}&nbsp;{/if}</div>
          </div>
          <div class="spese-cell">
            <div class="spese-label">SCONTO</div>
            <div class="spese-val">&nbsp;</div>
          </div>
          <div class="spese-cell">
            <div class="spese-label">SPESE INCASSO</div>
            <div class="spese-val">{#if doc.spese_incasso}{formatCurrency(doc.spese_incasso)}{:else}&nbsp;{/if}</div>
          </div>
          <div class="spese-cell" style="border-right:none">
            <div class="spese-label">TOTALE NETTO</div>
            <div class="spese-val">{formatCurrency(doc.totale_imponibile)}</div>
          </div>
        </div>

        <!-- SEZIONE 5: Riepilogo IVA + Totali -->
         <div class="row-iva-totali">
           <div class="iva-sinistra">
             <div class="cell-label">RIEPILOGO IVA</div>
             <table class="iva-table">
              <thead>
                <tr>
                  <th>ALIQUOTA</th>
                  <th class="th-r">IMPONIBILE</th>
                  <th class="th-r">IMPOSTA</th>
                </tr>
              </thead>
              <tbody>
                {#each riepilogoIvaRows as row}
                  <tr>
                    <td>{row.aliquota}%</td>
                    <td class="td-r">{formatCurrency(row.imponibile)}</td>
                    <td class="td-r">{formatCurrency(row.imposta)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
          <div class="iva-destra">
            <div class="tot-row">
              <div class="tot-label">Totale Imponibile</div>
              <div class="tot-val">{formatCurrency(doc.totale_imponibile)}</div>
            </div>
            <div class="tot-row">
              <div class="tot-label">Totale Imposta</div>
              <div class="tot-val">{formatCurrency(doc.totale_iva)}</div>
            </div>
            <div class="tot-finale">
              <div class="tot-finale-label">TOTALE DOCUMENTO</div>
              <div class="tot-finale-val">{formatCurrency(doc.totale_documento)}</div>
            </div>
          </div>
        </div>

        <!-- SEZIONE 6: Trasporto -->
        <div class="row-trasporto">
          <div class="trasp-cell">
            <div class="cell-label">TRASPORTO A CURA</div>
            <div class="cell-body"><span class="cell-val">{doc.trasporto_a_cura || 'Destinatario'}</span></div>
          </div>
          <div class="trasp-cell">
            <div class="cell-label">VETTORE</div>
            <div class="cell-body">{#if doc.vettore}<span class="cell-val">{doc.vettore}</span>{:else}&nbsp;{/if}</div>
          </div>
          <div class="trasp-cell">
            <div class="cell-label">DATA E ORA RITIRO</div>
            <div class="cell-body"><span class="cell-val">{formatDataOra(doc.data_ora_ritiro) ?? oraCorrente}</span></div>
          </div>
          <div class="trasp-cell">
            <div class="cell-label">PORTO</div>
            <div class="cell-body">{#if doc.porto}<span class="cell-val">{doc.porto}</span>{:else}&nbsp;{/if}</div>
          </div>
          <div class="trasp-cell">
            <div class="cell-label">N. COLLI</div>
            <div class="cell-body"><span class="cell-val">{doc.n_colli ?? 0}</span></div>
          </div>
          <div class="trasp-cell" style="border-right:none">
            <div class="cell-label">ASPETTO ESTERIORE</div>
            <div class="cell-body">{#if doc.aspetto_esteriore_beni}<span class="cell-val">{doc.aspetto_esteriore_beni}</span>{:else}&nbsp;{/if}</div>
          </div>
        </div>

        <!-- SEZIONE 7: Firme -->
        <div class="row-firme">
          <div class="firma-cell">
            <div class="cell-label">FIRMA DEL CONDUCENTE</div>
            <div class="firma-spazio">&nbsp;</div>
          </div>
          <div class="firma-cell" style="border-left:none">
            <div class="cell-label">FIRMA DEL DESTINATARIO</div>
            <div class="firma-spazio">&nbsp;</div>
          </div>
        </div>

        <!-- SEZIONE 8: IBAN -->
         <div class="row-iban">
           <div class="cell-label">IBAN PER BONIFICO</div>
           <div class="cell-body iban-val">{#if doc.mostra_iban !== false}{$appConfig?.intestazione?.iban ?? ''}{:else}&nbsp;{/if}</div>
         </div>

      </div>
    {/if}
  {/if}
</div>

<style lang="css">
  #print-overlay {
    display: none;
  }

  /* ── Footer pagina ──────────────────────────────────── */

  :global(#print-overlay) .pg-footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 2mm 1cm;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 8pt;
    color: #555;
    background: #fff;
    border-top: 0.8pt solid #1a1a1a;
    text-align: center;
  }

  /* ── Base documento ─────────────────────────────────── */

  :global(#print-overlay) .doc-buono,
  :global(#print-overlay) .doc-fattura {
    font-family: Arial, Helvetica, sans-serif;
    font-size: 10pt;
    color: #1a1a1a;
    padding: 2mm 8mm 2mm 8mm;
  }

  /* ── Celle generiche con label ──────────────────────── */

  :global(#print-overlay) .cell-label {
    font-size: 7pt;
    font-weight: bold;
    background: #e8e8e8;
    padding: 1mm 1.5mm;
    display: block;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: #444;
    line-height: 1.3;
  }

  :global(#print-overlay) .cell-body {
    padding: 1.5mm 1.5mm;
  }

  :global(#print-overlay) .cell-val {
    font-size: 10pt;
    font-weight: bold;
  }

  /* ── Box con bordo pieno (Gruppo A e Mittente/Cliente B) */

  :global(#print-overlay) .box-a,
  :global(#print-overlay) .box-b {
    border: 0.8pt solid #1a1a1a;
    overflow: hidden;
  }

  /* ── GRUPPO A ───────────────────────────────────────── */

  :global(#print-overlay) .row-header-a {
    display: flex;
    gap: 3mm;
    margin-bottom: 3mm;
  }

  :global(#print-overlay) .box-azienda {
    flex: 1;
    padding: 2mm 2.5mm;
  }

  :global(#print-overlay) .azienda-nome {
    font-size: 13pt;
    font-weight: bold;
    color: #B91C1C;
    margin-bottom: 1.5mm;
  }

  :global(#print-overlay) .azienda-detail {
    font-size: 8.5pt;
    line-height: 1.5;
    color: #333;
  }

  :global(#print-overlay) .box-cliente-a {
    flex: 1;
  }

  :global(#print-overlay) .cliente-nome {
    font-size: 11pt;
    font-weight: bold;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .cliente-detail {
    font-size: 9pt;
    line-height: 1.5;
  }

  :global(#print-overlay) .row-tipo {
    display: flex;
    align-items: center;
    gap: 5mm;
    margin-bottom: 3mm;
    padding: 2mm 3mm;
    border: 0.8pt solid #1a1a1a;
    background: #fef2f2;
  }

  :global(#print-overlay) .tipo-badge {
    font-size: 16pt;
    font-weight: bold;
    color: #B91C1C;
  }

  :global(#print-overlay) .tipo-meta {
    font-size: 11pt;
    font-weight: bold;
    color: #333;
  }

  :global(#print-overlay) .row-pag-a {
    display: flex;
    gap: 3mm;
    margin-bottom: 3mm;
  }

  :global(#print-overlay) .box-pag-cell {
    flex: 1;
  }

  :global(#print-overlay) .row-footer1 {
    display: flex;
    align-items: stretch;
    border: 0.8pt solid #1a1a1a;
    border-top: none;
  }

  :global(#print-overlay) .footer1-trasp {
    flex: 1;
    border-right: 0.8pt solid #1a1a1a;
    overflow: hidden;
  }

  :global(#print-overlay) .footer1-totale-label {
    padding: 1.5mm 3mm;
    font-size: 9pt;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    background: #e8e8e8;
    border-right: 0.8pt solid #1a1a1a;
    white-space: nowrap;
  }

  :global(#print-overlay) .footer1-totale-val {
    padding: 2mm 4mm;
    font-size: 14pt;
    font-weight: bold;
    color: #B91C1C;
    display: flex;
    align-items: center;
    background: #fef2f2;
    min-width: 38mm;
    justify-content: flex-end;
  }

  :global(#print-overlay) .row-footer2 {
    display: flex;
    border: 0.8pt solid #1a1a1a;
    border-top: none;
  }

  :global(#print-overlay) .footer2-aspetto {
    flex: 2;
    border-right: 0.8pt solid #1a1a1a;
    min-height: 9mm;
    overflow: hidden;
  }

  :global(#print-overlay) .footer2-colli {
    flex: 1;
    border-right: 0.8pt solid #1a1a1a;
    overflow: hidden;
  }

  :global(#print-overlay) .footer2-ore {
    flex: 1;
    overflow: hidden;
  }

  :global(#print-overlay) .row-firma-a {
    margin-top: 6mm;
    display: flex;
  }

  :global(#print-overlay) .firma-box-a {
    border: 0.8pt solid #1a1a1a;
    padding: 2mm 3mm;
    min-width: 70mm;
    min-height: 18mm;
  }

  :global(#print-overlay) .firma-label-a {
    font-size: 7.5pt;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #555;
    margin-bottom: 8mm;
  }

  :global(#print-overlay) .firma-linea-a {
    font-size: 11pt;
    color: #ccc;
    border-top: 0.8pt solid #555;
    padding-top: 1.5mm;
  }

  /* ── Tabella articoli (comune A e B) ─────────────────── */

  :global(#print-overlay) .doc-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 0;
    border: 0.8pt solid #1a1a1a;
  }


  :global(#print-overlay) .doc-table th {
    background: #B91C1C;
    color: #fff;
    font-size: 8.5pt;
    font-weight: bold;
    padding: 1.5mm 2mm;
    text-align: left;
    border: 0.5pt solid #ccc;
    border-bottom: 0.8pt solid #8b0000;
  }


  :global(#print-overlay) .doc-table td {
    padding: 1.5mm 2mm;
    font-size: 9.5pt;
    border: 0.5pt solid #ccc;
    height: 6mm;
  }

  :global(#print-overlay) .tr-alt td {
    background: #fafafa;
  }

  :global(#print-overlay) .td-empty {
    height: 4mm;
  }

  :global(#print-overlay) .th-codice { width: 11%; }
  :global(#print-overlay) .td-codice { font-family: monospace; font-size: 8.5pt; }
  :global(#print-overlay) .th-desc   { width: auto; }
  :global(#print-overlay) .td-desc   { word-break: break-word; }
  :global(#print-overlay) .th-um     { width: 6%; text-align: center; }
  :global(#print-overlay) .td-um     { text-align: center; }
  :global(#print-overlay) .th-qty    { width: 7%; text-align: right; }
  :global(#print-overlay) .td-qty    { text-align: right; }
  :global(#print-overlay) .th-price  { width: 12%; text-align: right; }
  :global(#print-overlay) .td-price  { text-align: right; }
  :global(#print-overlay) .th-sconto { width: 8%; text-align: right; }
  :global(#print-overlay) .td-sconto { text-align: right; }
  :global(#print-overlay) .th-importo { width: 12%; text-align: right; }
  :global(#print-overlay) .td-importo { text-align: right; font-weight: bold; }
  :global(#print-overlay) .th-iva    { width: 6%; text-align: right; }
  :global(#print-overlay) .td-iva    { text-align: right; }
  :global(#print-overlay) .th-r      { text-align: right; }
  :global(#print-overlay) .td-r      { text-align: right; }

  /* ── GRUPPO B ───────────────────────────────────────── */

  :global(#print-overlay) .row-header3 {
    display: flex;
    gap: 3mm;
    margin-bottom: 1.5mm;
  }

  :global(#print-overlay) .box-mittente {
    width: 34%;
    flex-shrink: 0;
  }

  :global(#print-overlay) .mit-nome {
    font-size: 12pt;
    font-weight: bold;
    color: #B91C1C;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .mit-detail {
    font-size: 8.5pt;
    line-height: 1.5;
    color: #333;
  }

  :global(#print-overlay) .box-cliente-dest {
    flex: 1;
  }

  :global(#print-overlay) .cli-nome {
    font-size: 11pt;
    font-weight: bold;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .cli-detail {
    font-size: 8.5pt;
    line-height: 1.5;
  }

  :global(#print-overlay) .dest-sep {
    margin-top: 2mm;
  }

  :global(#print-overlay) .cli-dest {
    font-size: 9pt;
    font-weight: bold;
    color: #444;
  }

  :global(#print-overlay) .box-documento {
    width: 30%;
    flex-shrink: 0;
  }

  :global(#print-overlay) .doc-tipo-badge {
    font-size: 10pt;
    font-weight: bold;
    color: #fff;
    text-align: center;
    padding: 1.5mm 2mm;
    background: #B91C1C;
    border-bottom: 0.8pt solid #1a1a1a;
  }

  .mit-sottotitolo {
    font-size: 8pt;
    color: #666;
    margin-bottom: 1mm;
  }

  :global(#print-overlay) .doc-body {
    padding: 1.5mm 2mm;
  }

  :global(#print-overlay) .doc-num-label {
    font-size: 7pt;
    font-weight: bold;
    text-transform: uppercase;
    color: #666;
    letter-spacing: 0.04em;
  }

  :global(#print-overlay) .doc-num {
    font-size: 20pt;
    font-weight: bold;
    color: #1a1a1a;
    line-height: 1;
    margin-bottom: 1.5mm;
  }

  :global(#print-overlay) .doc-data-label {
    font-size: 7pt;
    font-weight: bold;
    text-transform: uppercase;
    color: #666;
    letter-spacing: 0.04em;
  }

  :global(#print-overlay) .doc-data {
    font-size: 11pt;
    font-weight: bold;
    margin-bottom: 2mm;
  }

  :global(#print-overlay) .doc-subrow {
    display: flex;
    gap: 2mm;
    border-top: 0.5pt solid #ccc;
    padding-top: 1.5mm;
    font-size: 8pt;
    align-items: flex-end;
    justify-content: space-between;
  }

  :global(#print-overlay) .doc-sub-label {
    font-size: 6.5pt;
    font-weight: bold;
    text-transform: uppercase;
    color: #666;
    letter-spacing: 0.03em;
  }

  :global(#print-overlay) .doc-sub-val {
    font-size: 9pt;
    font-weight: bold;
  }

  :global(#print-overlay) .doc-pagina {
    font-size: 7.5pt;
    color: #555;
    align-self: flex-end;
    white-space: nowrap;
  }

  :global(#print-overlay) .row-causale {
    display: flex;
    border: 0.8pt solid #1a1a1a;
    border-bottom: none;
    margin-bottom: 1.5mm;
    overflow: hidden;
  }

  :global(#print-overlay) .caus-cell {
    border-right: 0.5pt solid #ccc;
    min-height: 7mm;
    overflow: hidden;
  }

  :global(#print-overlay) .row-spese {
    display: flex;
    border: 0.8pt solid #1a1a1a;
    border-top: none;
    overflow: hidden;
  }

  :global(#print-overlay) .spese-cell {
    flex: 1;
    border-right: 0.5pt solid #ccc;
    min-height: 8mm;
    overflow: hidden;
  }

  :global(#print-overlay) .spese-label {
    font-size: 6.5pt;
    font-weight: bold;
    text-transform: uppercase;
    background: #e8e8e8;
    padding: 1mm 1.5mm;
    display: block;
    color: #444;
  }

  :global(#print-overlay) .spese-val {
    font-size: 9.5pt;
    font-weight: bold;
    text-align: right;
    padding: 1mm 1.5mm;
  }

  :global(#print-overlay) .row-iva-totali {
    display: flex;
    margin-top: 1.5mm;
    border: 0.8pt solid #1a1a1a;
    overflow: hidden;
  }

  :global(#print-overlay) .iva-sinistra {
    width: 55%;
    border-right: 0.8pt solid #1a1a1a;
  }

  :global(#print-overlay) .iva-table {
    width: 100%;
    border-collapse: collapse;
  }

  :global(#print-overlay) .iva-table th {
    background: #e8e8e8;
    font-size: 7.5pt;
    font-weight: bold;
    padding: 1.5mm 2mm;
    border-bottom: 0.5pt solid #ccc;
    text-align: left;
  }

  :global(#print-overlay) .iva-table td {
    font-size: 9pt;
    padding: 1.5mm 2mm;
    border-bottom: 0.5pt solid #eee;
  }

  :global(#print-overlay) .iva-destra {
    flex: 1;
    padding: 2mm 3mm;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  :global(#print-overlay) .tot-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5mm 0;
    border-bottom: 0.5pt solid #eee;
  }

  :global(#print-overlay) .tot-label {
    font-size: 8.5pt;
    font-weight: bold;
    color: #555;
    text-transform: uppercase;
  }

  :global(#print-overlay) .tot-val {
    font-size: 10pt;
    font-weight: bold;
    text-align: right;
  }

  :global(#print-overlay) .tot-finale {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 2mm;
    padding: 2mm 2mm;
    border-top: 1.5pt solid #B91C1C;
    border-bottom: 1.5pt solid #B91C1C;
    background: #fef2f2;
  }

  :global(#print-overlay) .tot-finale-label {
    font-size: 9pt;
    font-weight: bold;
    text-transform: uppercase;
    color: #B91C1C;
  }

  :global(#print-overlay) .tot-finale-val {
    font-size: 14pt;
    font-weight: bold;
    color: #B91C1C;
    text-align: right;
  }

  :global(#print-overlay) .row-trasporto {
    display: flex;
    margin-top: 1mm;
    border: 0.8pt solid #1a1a1a;
    overflow: hidden;
  }

  :global(#print-overlay) .trasp-cell {
    flex: 1;
    border-right: 0.5pt solid #ccc;
    min-height: 9mm;
    overflow: hidden;
  }

  :global(#print-overlay) .row-firme {
    display: flex;
    margin-top: 1mm;
    border: 0.8pt solid #1a1a1a;
    overflow: hidden;
  }

  :global(#print-overlay) .firma-cell {
    flex: 1;
    border-right: 0.5pt solid #ccc;
    min-height: 9mm;
    overflow: hidden;
  }

  :global(#print-overlay) .firma-spazio {
    min-height: 11mm;
    padding: 2mm;
  }

  :global(#print-overlay) .row-iban {
    margin-top: 1mm;
    border: 0.8pt solid #1a1a1a;
    overflow: hidden;
    min-height: 8mm;
  }

  :global(#print-overlay) .row-iva-totali,
  :global(#print-overlay) .row-trasporto,
  :global(#print-overlay) .row-firme,
  :global(#print-overlay) .row-iban {
    page-break-inside: avoid;
    break-inside: avoid;
  }

  :global(#print-overlay) .iban-val {
    font-size: 11pt;
    font-weight: bold;
    font-family: monospace;
    letter-spacing: 0.1em;
  }

  @media print {
    :global(#print-overlay) * {
      -webkit-print-color-adjust: exact !important;
      print-color-adjust: exact !important;
      color-adjust: exact !important;
    }

    :global(#print-overlay) .doc-fattura {
      transform: scale(0.92);
      transform-origin: top left;
      width: 108.7%;
    }
  }
</style>
