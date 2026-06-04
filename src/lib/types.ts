export interface Cliente {
  id: number
  ragione_sociale: string
  partita_iva: string | null
  codice_fiscale: string | null
  indirizzo: string | null
  citta: string | null
  cap: string | null
  provincia: string | null
  telefono: string | null
  email: string | null
  note: string | null
  created_at: string
  updated_at: string
}

export interface NuovoCliente {
  ragione_sociale: string
  partita_iva?: string | null
  codice_fiscale?: string | null
  indirizzo?: string | null
  citta?: string | null
  cap?: string | null
  provincia?: string | null
  telefono?: string | null
  email?: string | null
  note?: string | null
}

export interface Fornitore {
  id: number
  ragione_sociale: string
  partita_iva: string | null
  codice_fiscale: string | null
  indirizzo: string | null
  citta: string | null
  cap: string | null
  provincia: string | null
  telefono: string | null
  email: string | null
  note: string | null
  created_at: string
  updated_at: string
}

export interface NuovoFornitore {
  ragione_sociale: string
  partita_iva?: string | null
  codice_fiscale?: string | null
  indirizzo?: string | null
  citta?: string | null
  cap?: string | null
  provincia?: string | null
  telefono?: string | null
  email?: string | null
  note?: string | null
}

export interface Ricambio {
  id: number
  codice_interno: string
  codice_oem: string | null
  descrizione: string
  marca: string | null
  modello_auto: string | null
  anno_da: number | null
  anno_a: number | null
  categoria: string | null
  fornitore_id: number | null
  giacenza: number
  giacenza_minima: number
  prezzo_acquisto: number
  prezzo_vendita: number
  iva_percentuale: number
  posizione: string | null
  note: string | null
  created_at: string
  updated_at: string
}

export interface NuovoRicambio {
  codice_interno: string
  codice_oem?: string | null
  descrizione: string
  marca?: string | null
  modello_auto?: string | null
  anno_da?: number | null
  anno_a?: number | null
  categoria?: string | null
  fornitore_id?: number | null
  giacenza: number
  giacenza_minima: number
  prezzo_acquisto: number
  prezzo_vendita: number
  iva_percentuale: number
  posizione?: string | null
  note?: string | null
}

export interface Documento {
  id: number
  tipo_documento: 'fattura' | 'preventivo' | 'ddt' | 'nota_credito'
  numero: string
  data: string
  cliente_id: number | null
  fornitore_id: number | null
  stato: 'bozza' | 'confermato' | 'pagato' | 'annullato'
  note: string | null
  totale_imponibile: number
  totale_iva: number
  totale_documento: number
  created_at: string
  updated_at: string
  scadenza_pagamento: string | null
  giorni_pagamento: number | null
  data_pagamento: string | null
  metodo_pagamento: string | null
  riferimento_pagamento: string | null
  note_pagamento: string | null
}

export interface RigaDocumento {
  id: number
  documento_id: number
  ricambio_id: number | null
  descrizione: string
  quantita: number
  prezzo_unitario: number
  sconto_percentuale: number
  iva_percentuale: number
  imponibile: number
  totale_iva: number
  totale_riga: number
  ordine: number
}

export interface NuovaRigaDocumento {
  ricambio_id?: number | null
  descrizione: string
  quantita: number
  prezzo_unitario: number
  sconto_percentuale: number
  iva_percentuale: number
  ordine: number
}

export interface NuovoDocumento {
  tipo_documento: string
  numero: string
  data: string
  cliente_id?: number | null
  fornitore_id?: number | null
  note?: string | null
  giorni_pagamento?: number | null
  righe: NuovaRigaDocumento[]
}

export interface DocumentoCompleto {
  documento: Documento
  righe: RigaDocumento[]
}

export interface DashboardStats {
  totale_clienti: number
  totale_fornitori: number
  totale_ricambi: number
  ricambi_sotto_scorta: number
  fatture_mese_corrente: number
  fatturato_mese_corrente: number
  fatture_non_pagate: number
  valore_magazzino: number
  fatture_scadute: number
  scontrini_oggi: number
  incasso_oggi: number
}

export interface AppError {
  type: 'Database' | 'NotFound' | 'Validation' | 'Internal'
  message: string
}

export type View =
  | 'dashboard'
  | 'magazzino'
  | 'clienti'
  | 'veicoli'
  | 'fornitori'
  | 'ordini-fornitore'
  | 'documenti'
  | 'nuova-fattura'
  | 'scadenzario'
  | 'cassa'
  | 'storico-cassa'
  | 'impostazioni'

export interface Impostazioni {
  ragione_sociale: string
  partita_iva: string
  codice_fiscale: string
  indirizzo: string
  cap: string
  citta: string
  provincia: string
  codice_destinatario: string
  regime_fiscale: string
  [key: string]: string
}

export interface RisultatoRicerca {
  tipo: 'cliente' | 'ricambio' | 'documento'
  id: number
  titolo: string
  sottotitolo: string | null
}

// Veicoli

export interface Veicolo {
  id: number
  cliente_id: number
  targa: string
  marca: string | null
  modello: string | null
  anno: number | null
  cilindrata: string | null
  carburante: string | null
  km_attuali: number | null
  note: string | null
  created_at: string
  updated_at: string
}

export interface NuovoVeicolo {
  cliente_id: number
  targa: string
  marca?: string | null
  modello?: string | null
  anno?: number | null
  cilindrata?: string | null
  carburante?: string | null
  km_attuali?: number | null
  note?: string | null
}

// Ordini fornitore

export interface OrdineFornitore {
  id: number
  fornitore_id: number
  data: string
  stato: 'bozza' | 'inviato' | 'ricevuto' | 'annullato'
  note: string | null
  totale: number
  created_at: string
  updated_at: string
}

export interface RigaOrdine {
  id: number
  ordine_id: number
  ricambio_id: number | null
  descrizione: string
  quantita: number
  prezzo_unitario: number
  totale_riga: number
}

export interface NuovaRigaOrdine {
  ricambio_id?: number | null
  descrizione: string
  quantita: number
  prezzo_unitario: number
}

export interface NuovoOrdineFornitore {
  fornitore_id: number
  data: string
  stato?: string | null
  note?: string | null
  righe: NuovaRigaOrdine[]
}

export interface OrdineCompleto {
  ordine: OrdineFornitore
  righe: RigaOrdine[]
}

// Report

export interface FatturaReport {
  id: number
  numero: string
  cliente_ragione_sociale: string | null
  totale_documento: number
  stato: string
}

export interface ReportMensile {
  anno: number
  mese: number
  totale_entrate: number
  totale_uscite: number
  saldo: number
  lista_fatture: FatturaReport[]
  ricambi_sotto_scorta: Ricambio[]
  ordini_in_attesa: number
}

export interface ScadenzaDocumento {
  id: number
  numero: string
  tipo_documento: string
  data: string
  scadenza_pagamento: string | null
  giorni_alla_scadenza: number | null
  totale_documento: number
  stato: string
  cliente_id: number | null
  cliente_ragione_sociale: string | null
}

// Cassa / scontrini

export interface Scontrino {
  id: number
  numero: string
  data: string
  operatore: string | null
  totale: number
  sconto_totale: number
  metodo_pagamento: 'contanti' | 'carta' | 'satispay' | 'bonifico'
  stato: 'aperto' | 'chiuso' | 'annullato'
  note: string | null
  created_at: string
}

export interface RigaScontrino {
  id: number
  scontrino_id: number
  ricambio_id: number | null
  codice: string | null
  descrizione: string
  quantita: number
  prezzo_unitario: number
  sconto_percentuale: number
  totale_riga: number
  iva_percentuale: number
}

export interface NuovaRigaScontrino {
  ricambio_id?: number | null
  codice?: string | null
  descrizione: string
  quantita: number
  prezzo_unitario: number
  sconto_percentuale: number
  iva_percentuale: number
}

export interface NuovoScontrino {
  righe: NuovaRigaScontrino[]
  metodo_pagamento: string
  sconto_totale: number
  operatore?: string | null
  note?: string | null
}

export interface ScontrinoCompleto {
  scontrino: Scontrino
  righe: RigaScontrino[]
}
