import { invoke } from '@tauri-apps/api/core'
import type {
  Cliente, NuovoCliente,
  Fornitore, NuovoFornitore,
  Ricambio, NuovoRicambio,
  Documento, DocumentoCompleto, NuovoDocumento,
  DashboardStats, RisultatoRicerca,
  Veicolo, NuovoVeicolo,
  OrdineCompleto, NuovoOrdineFornitore,
  ReportMensile, ScadenzaDocumento,
  ScontrinoCompleto, NuovoScontrino,
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
  },
  clienti: {
    getAll: () => call<Cliente[]>('get_all_clienti'),
    get: (id: number) => call<Cliente>('get_cliente', { id }),
    create: (cliente: NuovoCliente) => call<Cliente>('create_cliente', { cliente }),
    update: (id: number, cliente: NuovoCliente) => call<Cliente>('update_cliente', { id, cliente }),
    delete: (id: number) => call<void>('delete_cliente', { id }),
  },
  fornitori: {
    getAll: () => call<Fornitore[]>('get_all_fornitori'),
    create: (fornitore: NuovoFornitore) => call<Fornitore>('create_fornitore', { fornitore }),
  },
  ricambi: {
    getAll: () => call<Ricambio[]>('get_all_ricambi'),
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
    get: (id: number) => call<DocumentoCompleto>('get_documento', { id }),
    create: (doc: NuovoDocumento) => call<DocumentoCompleto>('create_documento', { doc }),
    updateStato: (id: number, stato: string) => call<Documento>('update_stato_documento', { id, stato }),
    getScadenzario: () => call<ScadenzaDocumento[]>('get_scadenzario'),
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
    checkLicense: () => call<boolean>('check_license'),
    activateLicense: (chiave: string) => call<boolean>('activate_license', { chiave }),
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
}
