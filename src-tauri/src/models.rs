use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PaginatedResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Cliente {
    pub id: i64,
    pub ragione_sociale: String,
    pub partita_iva: Option<String>,
    pub codice_fiscale: Option<String>,
    pub indirizzo: Option<String>,
    pub citta: Option<String>,
    pub cap: Option<String>,
    pub provincia: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoCliente {
    pub ragione_sociale: String,
    pub partita_iva: Option<String>,
    pub codice_fiscale: Option<String>,
    pub indirizzo: Option<String>,
    pub citta: Option<String>,
    pub cap: Option<String>,
    pub provincia: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Fornitore {
    pub id: i64,
    pub ragione_sociale: String,
    pub partita_iva: Option<String>,
    pub codice_fiscale: Option<String>,
    pub indirizzo: Option<String>,
    pub citta: Option<String>,
    pub cap: Option<String>,
    pub provincia: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoFornitore {
    pub ragione_sociale: String,
    pub partita_iva: Option<String>,
    pub codice_fiscale: Option<String>,
    pub indirizzo: Option<String>,
    pub citta: Option<String>,
    pub cap: Option<String>,
    pub provincia: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Ricambio {
    pub id: i64,
    pub codice_interno: String,
    pub codice_oem: Option<String>,
    pub descrizione: String,
    pub marca: Option<String>,
    pub modello_auto: Option<String>,
    pub anno_da: Option<i64>,
    pub anno_a: Option<i64>,
    pub categoria: Option<String>,
    pub fornitore_id: Option<i64>,
    pub giacenza: i64,
    pub giacenza_minima: i64,
    pub prezzo_acquisto: f64,
    pub prezzo_vendita: f64,
    pub iva_percentuale: f64,
    pub posizione: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoRicambio {
    pub codice_interno: String,
    pub codice_oem: Option<String>,
    pub descrizione: String,
    pub marca: Option<String>,
    pub modello_auto: Option<String>,
    pub anno_da: Option<i64>,
    pub anno_a: Option<i64>,
    pub categoria: Option<String>,
    pub fornitore_id: Option<i64>,
    pub giacenza: i64,
    pub giacenza_minima: i64,
    pub prezzo_acquisto: f64,
    pub prezzo_vendita: f64,
    pub iva_percentuale: f64,
    pub posizione: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Documento {
    pub id: i64,
    pub tipo_documento: String,
    pub numero: String,
    pub data: String,
    pub cliente_id: Option<i64>,
    pub fornitore_id: Option<i64>,
    pub stato: String,
    pub note: Option<String>,
    pub totale_imponibile: f64,
    pub totale_iva: f64,
    pub totale_documento: f64,
    pub created_at: String,
    pub updated_at: String,
    pub scadenza_pagamento: Option<String>,
    pub giorni_pagamento: Option<i64>,
    pub data_pagamento: Option<String>,
    pub metodo_pagamento: Option<String>,
    pub riferimento_pagamento: Option<String>,
    pub note_pagamento: Option<String>,
    pub ddt_collegati: Option<String>,
    pub is_fattura_differita: i64,
    pub fatturato: i64,
    pub pdf_allegato: Option<String>,
    pub vettore: Option<String>,
    pub data_ora_ritiro: Option<String>,
    pub n_colli: Option<i64>,
    pub aspetto_esteriore_beni: Option<String>,
    pub porto: Option<String>,
    pub causale_trasporto: Option<String>,
    pub trasporto_a_cura: Option<String>,
    pub banca_appoggio: Option<String>,
    pub agente: Option<String>,
    pub bolli_art15: Option<String>,
    pub spese_varie: Option<f64>,
    pub spese_incasso: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoDocumento {
    pub tipo_documento: String,
    pub numero: String,
    pub data: String,
    pub cliente_id: Option<i64>,
    pub fornitore_id: Option<i64>,
    pub note: Option<String>,
    pub giorni_pagamento: Option<i64>,
    pub ddt_collegati: Option<Vec<i64>>,
    pub vettore: Option<String>,
    pub data_ora_ritiro: Option<String>,
    pub n_colli: Option<i64>,
    pub aspetto_esteriore_beni: Option<String>,
    pub porto: Option<String>,
    pub causale_trasporto: Option<String>,
    pub trasporto_a_cura: Option<String>,
    pub banca_appoggio: Option<String>,
    pub agente: Option<String>,
    pub bolli_art15: Option<String>,
    pub spese_varie: Option<f64>,
    pub spese_incasso: Option<f64>,
    pub righe: Vec<NuovaRigaDocumento>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RigaDocumento {
    pub id: i64,
    pub documento_id: i64,
    pub ricambio_id: Option<i64>,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
    pub sconto_percentuale: f64,
    pub iva_percentuale: f64,
    pub imponibile: f64,
    pub totale_iva: f64,
    pub totale_riga: f64,
    pub ordine: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovaRigaDocumento {
    pub ricambio_id: Option<i64>,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
    pub sconto_percentuale: f64,
    pub iva_percentuale: f64,
    pub ordine: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentoCompleto {
    pub documento: Documento,
    pub righe: Vec<RigaDocumento>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub totale_clienti: i64,
    pub totale_fornitori: i64,
    pub totale_ricambi: i64,
    pub ricambi_sotto_scorta: i64,
    pub fatture_mese_corrente: i64,
    pub fatturato_mese_corrente: f64,
    pub fatture_non_pagate: i64,
    pub valore_magazzino: f64,
    pub fatture_scadute: i64,
    pub scontrini_oggi: i64,
    pub incasso_oggi: f64,
}

// Carico rapido barcode

#[derive(Debug, Serialize, Deserialize)]
pub struct RicambioResult {
    pub trovato: bool,
    pub ricambio: Option<Ricambio>,
    pub nuova_giacenza: Option<i64>,
}

// Cassa / scontrini

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Scontrino {
    pub id: i64,
    pub numero: String,
    pub data: String,
    pub operatore: Option<String>,
    pub totale: f64,
    pub sconto_totale: f64,
    pub metodo_pagamento: String,
    pub stato: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoScontrino {
    pub righe: Vec<NuovaRigaScontrino>,
    pub metodo_pagamento: String,
    pub sconto_totale: f64,
    pub operatore: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RigaScontrino {
    pub id: i64,
    pub scontrino_id: i64,
    pub ricambio_id: Option<i64>,
    pub codice: Option<String>,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
    pub sconto_percentuale: f64,
    pub totale_riga: f64,
    pub iva_percentuale: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovaRigaScontrino {
    pub ricambio_id: Option<i64>,
    pub codice: Option<String>,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
    pub sconto_percentuale: f64,
    pub iva_percentuale: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScontrinoCompleto {
    pub scontrino: Scontrino,
    pub righe: Vec<RigaScontrino>,
}

// Veicoli

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Veicolo {
    pub id: i64,
    pub cliente_id: i64,
    pub targa: String,
    pub marca: Option<String>,
    pub modello: Option<String>,
    pub anno: Option<i64>,
    pub cilindrata: Option<String>,
    pub carburante: Option<String>,
    pub km_attuali: Option<i64>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoVeicolo {
    pub cliente_id: i64,
    pub targa: String,
    pub marca: Option<String>,
    pub modello: Option<String>,
    pub anno: Option<i64>,
    pub cilindrata: Option<String>,
    pub carburante: Option<String>,
    pub km_attuali: Option<i64>,
    pub note: Option<String>,
}

// Ordini fornitore

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct OrdineFornitore {
    pub id: i64,
    pub fornitore_id: i64,
    pub data: String,
    pub stato: String,
    pub note: Option<String>,
    pub totale: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoOrdineFornitore {
    pub fornitore_id: i64,
    pub data: String,
    pub stato: Option<String>,
    pub note: Option<String>,
    pub righe: Vec<NuovaRigaOrdine>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RigaOrdine {
    pub id: i64,
    pub ordine_id: i64,
    pub ricambio_id: Option<i64>,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
    pub totale_riga: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovaRigaOrdine {
    pub ricambio_id: Option<i64>,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrdineCompleto {
    pub ordine: OrdineFornitore,
    pub righe: Vec<RigaOrdine>,
}

// Report

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FatturaReport {
    pub id: i64,
    pub numero: String,
    pub cliente_ragione_sociale: Option<String>,
    pub totale_documento: f64,
    pub stato: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMensile {
    pub anno: i32,
    pub mese: i32,
    pub totale_entrate: f64,
    pub totale_uscite: f64,
    pub saldo: f64,
    pub lista_fatture: Vec<FatturaReport>,
    pub ricambi_sotto_scorta: Vec<Ricambio>,
    pub ordini_in_attesa: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScadenzaDocumento {
    pub id: i64,
    pub numero: String,
    pub tipo_documento: String,
    pub data: String,
    pub scadenza_pagamento: Option<String>,
    pub giorni_alla_scadenza: Option<f64>,
    pub totale_documento: f64,
    pub stato: String,
    pub cliente_id: Option<i64>,
    pub cliente_ragione_sociale: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ArticoloStorico {
    pub ricambio_id: i64,
    pub codice_interno: String,
    pub descrizione: String,
    pub prezzo_unitario: f64,
    pub quantita_totale: f64,
    pub frequenza: i64,
}
