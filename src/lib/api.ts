import { invoke } from '@tauri-apps/api/core'
import type {
  Cliente, NuovoCliente,
  Fornitore, NuovoFornitore,
  Ricambio, NuovoRicambio, RicambioResult,
  Documento, DocumentoCompleto, NuovoDocumento,
  DashboardStats, ResocontoPeriodo, RisultatoRicerca,
  Veicolo, NuovoVeicolo,
  OrdineCompleto, NuovoOrdineFornitore,
  ReportMensile, ScadenzaDocumento,
  ScontrinoCompleto, NuovoScontrino,
  Impostazioni, LicenseInfo, AppConfig, ConfigIntestazione,
  PaginatedResult, ArticoloStorico,
  RigaDdtParsed, RigaDdtImport, ParseDdtResult, ImportaDdtResult,
} from './types'

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e: unknown) {
    if (typeof e === 'object' && e !== null && 'message' in e) throw e
    throw { type: 'Internal', message: String(e) }
  }
}

export const api = {
  dashboard: {
    getStats: () => call<DashboardStats>('get_dashboard_stats'),
    getResoconto: (dataFrom: string, dateTo: string, tipo?: string) =>
      call<ResocontoPeriodo>('get_resoconto_periodo', {
        dataFrom,
        dateTo,
        tipoDocumento: tipo ?? 'tutti'
      }),
  },
  clienti: {
    getAll: () => call<Cliente[]>('get_all_clienti'),
    getPaginated: (page: number, pageSize: number, search: string) =>
      call<PaginatedResult<Cliente>>('get_clienti_paginated', { page, pageSize, search }),
    get: (id: number) => call<Cliente>('get_cliente', { id }),
    create: (cliente: NuovoCliente) => call<Cliente>('create_cliente', { cliente }),
    update: (id: number, cliente: NuovoCliente) => call<Cliente>('update_cliente', { id, cliente }),
    delete: (id: number) => call<void>('delete_cliente', { id }),
  },
  fornitori: {
    getAll: () => call<Fornitore[]>('get_all_fornitori'),
    create: (fornitore: NuovoFornitore) => call<Fornitore>('create_fornitore', { fornitore }),
    update: (id: number, fornitore: NuovoFornitore) => call<Fornitore>('update_fornitore', { id, fornitore }),
    delete: (id: number) => call<void>('delete_fornitore', { id }),
  },
  ricambi: {
    getAll: () => call<Ricambio[]>('get_all_ricambi'),
    getPaginated: (page: number, pageSize: number, search: string, sortCol: string, sortDir: string) =>
      call<PaginatedResult<Ricambio>>('get_ricambi_paginated', { page, pageSize, search, sortCol, sortDir }),
    get: (id: number) => call<Ricambio>('get_ricambio', { id }),
    search: (query: string) => call<Ricambio[]>('search_ricambi', { query }),
    create: (ricambio: NuovoRicambio) => call<Ricambio>('create_ricambio', { ricambio }),
    update: (id: number, ricambio: NuovoRicambio) => call<Ricambio>('update_ricambio', { id, ricambio }),
    delete: (id: number) => call<void>('delete_ricambio', { id }),
    aggiornaGiacenza: (ricambioId: number, tipoMovimento: string, quantita: number, documentoId?: number | null, note?: string | null) =>
      call<Ricambio>('aggiorna_giacenza', { ricambioId, tipoMovimento, quantita, documentoId, note }),
  },
  documenti: {
    getAll: () => call<Documento[]>('get_all_documenti'),
    getPaginated: (page: number, pageSize: number, stato: string, tipo: string, search: string, sortCol: string, sortDir: string) =>
      call<PaginatedResult<Documento>>('get_documenti_paginated', { page, pageSize, stato, tipo, search, sortCol, sortDir }),
    get: (id: number) => call<DocumentoCompleto>('get_documento', { id }),
    create: (doc: NuovoDocumento) => call<DocumentoCompleto>('create_documento', { doc }),
    update: (id: number, doc: NuovoDocumento) => call<DocumentoCompleto>('update_documento', { id, doc }),
    delete: (id: number) => call<void>('delete_documento', { id }),
    updateStato: (
      id: number,
      stato: string,
      pagamento?: {
        data_pagamento?: string | null
        metodo_pagamento?: string | null
        riferimento_pagamento?: string | null
        note_pagamento?: string | null
      }
    ) => call<Documento>('update_stato_documento', { id, stato, ...pagamento }),
    getScadenzario: () => call<ScadenzaDocumento[]>('get_scadenzario'),
    getStoricoCliente: (clienteId: number) =>
      call<ArticoloStorico[]>('get_storico_articoli_cliente', { clienteId }),
  },
  ricerca: {
    searchGlobal: (query: string) => call<RisultatoRicerca[]>('search_global', { query }),
  },
  backup: {
    exportRicambiCsv: () => call<string>('export_ricambi_csv'),
    exportClientiCsv: () => call<string>('export_clienti_csv'),
    backupDatabase: () => call<string>('backup_database'),
    restoreDatabase: () => call<string>('restore_database'),
  },
  license: {
    getMachineId: () => call<string>('get_machine_id'),
    checkLicense: () => call<LicenseInfo>('check_license'),
    activateLicense: (chiave: string) => call<LicenseInfo>('activate_license', { chiave }),
    deactivateLicense: () => call<void>('deactivate_license'),
  },
  veicoli: {
    getByCliente: (clienteId: number) => call<Veicolo[]>('get_veicoli_cliente', { clienteId }),
    create: (veicolo: NuovoVeicolo) => call<Veicolo>('create_veicolo', { veicolo }),
    update: (id: number, veicolo: NuovoVeicolo) => call<Veicolo>('update_veicolo', { id, veicolo }),
    delete: (id: number) => call<void>('delete_veicolo', { id }),
  },
  ordini: {
    getAll: () => call<OrdineCompleto[]>('get_all_ordini'),
    create: (ordine: NuovoOrdineFornitore) => call<OrdineCompleto>('create_ordine_fornitore', { ordine }),
    updateStato: (id: number, stato: string) => call<OrdineCompleto>('update_stato_ordine', { id, stato }),
  },
  report: {
    getMensile: (anno: number, mese: number) => call<ReportMensile>('get_report_mensile', { anno, mese }),
  },
  cassa: {
    getAll: () => call<ScontrinoCompleto[]>('get_all_scontrini'),
    get: (id: number) => call<ScontrinoCompleto>('get_scontrino', { id }),
    cercaBarcode: (codice: string) => call<Ricambio | null>('cerca_ricambio_barcode', { codice }),
    crea: (scontrino: NuovoScontrino) => call<ScontrinoCompleto>('crea_scontrino', { scontrino }),
    annulla: (id: number) => call<ScontrinoCompleto>('annulla_scontrino', { id }),
  },
  impostazioni: {
    get: () => call<Impostazioni>('get_impostazioni'),
    save: (impostazioni: Impostazioni) => call<void>('save_impostazioni', { impostazioni }),
  },
  fatturaPa: {
    genera: (documentoId: number) => call<string>('genera_fattura_pa', { documentoId }),
  },
  config: {
    get: () => call<AppConfig>('get_config'),
    salvaIntestazione: (intestazione: ConfigIntestazione) =>
      call<AppConfig>('salva_intestazione', { intestazione }),
  },
  magazzino: {
    caricoRapido: (codice: string, quantita: number) =>
      call<RicambioResult>('carico_rapido_ricambio', { codice, quantita }),
  },
  ddtFornitore: {
    selezionaPdf: () =>
      call<string | null>('seleziona_pdf_ddt'),
    parse: (pdfPath: string) =>
      call<ParseDdtResult>('parse_ddt_fornitore_pdf', { pdfPath }),
    importa: (
      fornitoreId: number,
      numero: string,
      data: string,
      righe: RigaDdtImport[],
      pdfPathSorgente: string,
    ) => call<ImportaDdtResult>('importa_ddt_fornitore', { fornitoreId, numero, data, righe, pdfPathSorgente }),
    apriPdf: (documentoId: number) =>
      call<void>('apri_pdf_allegato', { documentoId }),
    creaFornitoreRapido: (ragioneSociale: string, partitaIva: string | null) =>
      call<Fornitore>('crea_fornitore_rapido', { ragioneSociale, partitaIva }),
  },
}
