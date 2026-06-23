<script lang="ts">
  import { printReportData, formatCurrency } from '../lib/stores'

  const mesi = ['', 'Gennaio', 'Febbraio', 'Marzo', 'Aprile', 'Maggio', 'Giugno',
                 'Luglio', 'Agosto', 'Settembre', 'Ottobre', 'Novembre', 'Dicembre']

  $: report = $printReportData

  $: titolo = report
    ? report.mese > 0
      ? `Report ${mesi[report.mese]} ${report.anno}`
      : report.anno > 0
        ? `Report ${report.anno}`
        : 'Report periodo'
    : ''
</script>

<div id="print-report-overlay">
  {#if report}
    <h1 class="titolo">{titolo}</h1>

    <div class="stats">
      <div class="stat-box">
        <div class="stat-label">Entrate (fatture)</div>
        <div class="stat-value entrate">{formatCurrency(report.totale_entrate)}</div>
      </div>
      <div class="stat-box">
        <div class="stat-label">Uscite (note credito)</div>
        <div class="stat-value uscite">{formatCurrency(report.totale_uscite)}</div>
      </div>
      <div class="stat-box">
        <div class="stat-label">Saldo</div>
        <div class="stat-value {report.saldo >= 0 ? 'entrate' : 'uscite'}">{formatCurrency(report.saldo)}</div>
      </div>
    </div>

    <div class="info-row">
      <div class="info-box">
        <span class="info-label">Fatture emesse</span>
        <span class="info-val">{report.lista_fatture.length}</span>
      </div>
      <div class="info-box">
        <span class="info-label">Ordini in attesa</span>
        <span class="info-val">{report.ordini_in_attesa}</span>
      </div>
    </div>

    {#if report.lista_fatture.length > 0}
      <h2 class="sezione">Fatture del periodo</h2>
      <table>
        <thead>
          <tr>
            <th>N°</th>
            <th>Cliente</th>
            <th class="text-right">Totale</th>
            <th>Stato</th>
          </tr>
        </thead>
        <tbody>
          {#each report.lista_fatture as f}
            <tr>
              <td>{f.numero}</td>
              <td>{f.cliente_ragione_sociale ?? '—'}</td>
              <td class="text-right">{formatCurrency(f.totale_documento)}</td>
              <td class="capitalize">{f.stato}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  #print-report-overlay {
    display: none;
  }

  @media print {
    #print-report-overlay {
      color: #000;
      font-family: Arial, Helvetica, sans-serif;
      padding: 15mm 20mm;
      box-sizing: border-box;
    }
  }

  .titolo {
    font-size: 18pt;
    font-weight: bold;
    margin-bottom: 8mm;
    padding-bottom: 3mm;
    border-bottom: 2pt solid #000;
  }

  .stats {
    display: flex;
    gap: 6mm;
    margin-bottom: 6mm;
  }

  .stat-box {
    flex: 1;
    border: 0.5pt solid #999;
    padding: 3mm 4mm;
  }

  .stat-label {
    font-size: 8pt;
    color: #666;
    margin-bottom: 1.5mm;
  }

  .stat-value {
    font-size: 14pt;
    font-weight: bold;
  }

  .entrate { color: #166534; }
  .uscite  { color: #991b1b; }

  .info-row {
    display: flex;
    gap: 6mm;
    margin-bottom: 6mm;
  }

  .info-box {
    flex: 1;
    border: 0.5pt solid #999;
    padding: 2.5mm 4mm;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 10pt;
  }

  .info-label {
    color: #444;
  }

  .info-val {
    font-weight: bold;
  }

  .sezione {
    font-size: 12pt;
    font-weight: bold;
    margin-top: 5mm;
    margin-bottom: 3mm;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 10pt;
  }

  th {
    background: #e8e8e8;
    padding: 2mm 3mm;
    text-align: left;
    border: 0.5pt solid #999;
    font-weight: bold;
    font-size: 9pt;
  }

  td {
    padding: 2mm 3mm;
    border: 0.5pt solid #ccc;
  }

  .text-right {
    text-align: right;
  }

  .capitalize {
    text-transform: capitalize;
  }
</style>
