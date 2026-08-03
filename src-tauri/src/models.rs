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
    pub mostra_iban: bool,
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
    pub mostra_iban: Option<bool>,
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

/// Riga della card "Clienti più attivi" in dashboard. Fatturato e
/// numero_documenti condividono lo stesso filtro WHERE della query in
/// get_riepilogo_cliente (tipo_documento='fattura', anno corrente,
/// stato diverso da annullato/bozza): il totale mostrato qui deve
/// coincidere con quello che l'utente legge aprendo la scheda del
/// cliente, altrimenti perde fiducia in entrambi i numeri.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ClienteAttivo {
    pub cliente_id: i64,
    pub ragione_sociale: String,
    pub numero_documenti: i64,
    pub fatturato: f64,
}

/// Top clienti per numero di documenti dell'anno corrente (non per
/// fatturato: e' un ordinamento diverso da ClienteAttivo, pensato per
/// il pannello F2 sul campo cliente di NuovaFattura.svelte, dove serve
/// "chi torna spesso" per compilare in fretta, non "chi vale di piu'").
/// Conta tutti i tipi di documento (non solo fattura): qui l'obiettivo
/// e' la frequenza di contatto col cliente, non il fatturato fiscale.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ClienteTopDocumenti {
    pub cliente_id: i64,
    pub ragione_sociale: String,
    pub citta: Option<String>,
    pub numero_documenti: i64,
    pub ultimo_documento: String,
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

// Scheda cliente: prezzi dedicati e storico vendite

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PrezzoCliente {
    pub id: i64,
    pub cliente_id: i64,
    pub ricambio_id: i64,
    pub codice_interno: String,
    pub descrizione: String,
    pub prezzo: f64,
    pub sconto_perc: Option<f64>,
    pub note: Option<String>,
    pub aggiornato_il: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuovoPrezzoCliente {
    pub cliente_id: i64,
    pub ricambio_id: i64,
    pub prezzo: f64,
    pub sconto_perc: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ArticoloVendutoCliente {
    pub articolo_id: i64,
    pub codice_articolo: String,
    pub descrizione: String,
    pub prezzo_ultimo: f64,
    pub prezzo_min: f64,
    pub prezzo_max: f64,
    pub prezzo_medio: f64,
    pub quantita_totale: f64,
    pub numero_vendite: i64,
    pub ultima_vendita: String,
}

/// Tab "Documenti" della scheda cliente: elenco completo, non filtrato,
/// di tutti i documenti intestati al cliente. Legge direttamente da
/// `documenti` (non da v_storico_vendite): qui servono anche preventivi,
/// note di credito, fatture differite, bozze e annullati.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentoCliente {
    pub documento_id: i64,
    pub tipo_documento: String,
    pub numero: String,
    pub data: String,
    pub totale: f64,
    pub stato: String,
}

/// Riepilogo sintetico in cima alla scheda cliente, sempre visibile
/// sopra i tab. Fatturato calcolato solo su tipo_documento='fattura'
/// (mai sulla view storico): deve restare un numero verificabile
/// dall'utente confrontandolo col resoconto esistente.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RiepilogoCliente {
    pub ultima_vendita: Option<String>,
    pub numero_documenti: i64,
    pub fatturato_anno_corrente: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MovimentoVenditaCliente {
    pub documento_id: i64,
    pub tipo_documento: String,
    pub numero_documento: String,
    pub data: String,
    pub articolo_id: i64,
    pub codice_articolo: String,
    pub descrizione: String,
    pub quantita: f64,
    pub prezzo_unitario: f64,
    pub sconto_perc: f64,
    pub totale_riga: f64,
}

/// Metadati del documento di vendita piu' recente di un cliente, letto
/// da v_storico_vendite (solo tipi di vendita conclusa). Alimenta il
/// pulsante "Ricarica ultimo documento" nel pannello F2 di NuovaFattura:
/// le righe vere e proprie si recuperano poi con get_documento(documento_id),
/// cosi' si riusa la stessa lettura completa gia' usata per la modifica.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UltimoDocumentoVendita {
    pub documento_id: i64,
    pub tipo_documento: String,
    pub numero: String,
    pub data: String,
}

/// Aggregato per cliente di un singolo articolo: speculare a
/// ArticoloVendutoCliente ma raggruppato per cliente invece che per
/// articolo. Alimenta il pannello F2 di Magazzino ("a chi ho venduto
/// questo articolo e a che prezzo").
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ClienteVendutoArticolo {
    pub cliente_id: i64,
    pub ragione_sociale: String,
    pub prezzo_ultimo: f64,
    pub numero_vendite: i64,
    pub ultima_vendita: String,
}

/// Cascata di risoluzione prezzo per un cliente+articolo.
/// - Dedicato: riga in prezzi_cliente, va applicata in automatico.
/// - Storico: ultimo prezzo praticato, solo suggerimento (badge cliccabile).
/// - Listino: nessun dato, il chiamante usa il prezzo di listino attuale.
#[derive(Debug, Serialize)]
#[serde(tag = "livello", rename_all = "snake_case")]
pub enum PrezzoSuggerito {
    Dedicato {
        prezzo: f64,
        sconto_perc: Option<f64>,
        note: Option<String>,
    },
    Storico {
        prezzo: f64,
        data: String,
        tipo_documento: String,
        numero_documento: String,
    },
    Listino,
}
