use crate::{AppError, AppState};
use crate::models::{Cliente, DocumentoCompleto, RigaDocumento};
use std::collections::HashMap;
use tauri::State;

#[derive(sqlx::FromRow)]
struct ImpostazioneRow {
    chiave: String,
    valore: String,
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn opt_str<'a>(map: &'a HashMap<String, String>, key: &str) -> &'a str {
    map.get(key).map(|s| s.as_str()).unwrap_or("")
}

fn metodo_pagamento_code(metodo: Option<&str>) -> &str {
    match metodo {
        Some("contanti") => "MP01",
        Some("carta") => "MP08",
        Some("satispay") => "MP08",
        Some("bonifico") => "MP05",
        _ => "MP05",
    }
}

struct RiepilogoIva {
    aliquota: f64,
    imponibile: f64,
    imposta: f64,
}

fn raggruppa_iva(righe: &[RigaDocumento]) -> Vec<RiepilogoIva> {
    let mut map: HashMap<String, (f64, f64)> = HashMap::new();
    for riga in righe {
        let key = format!("{:.2}", riga.iva_percentuale);
        let entry = map.entry(key).or_insert((0.0, 0.0));
        entry.0 += riga.imponibile;
        entry.1 += riga.totale_iva;
    }
    let mut result: Vec<RiepilogoIva> = map
        .into_iter()
        .map(|(k, (imponibile, imposta))| RiepilogoIva {
            aliquota: k.parse().unwrap_or(0.0),
            imponibile,
            imposta,
        })
        .collect();
    result.sort_by(|a, b| a.aliquota.partial_cmp(&b.aliquota).unwrap());
    result
}

fn build_xml(
    doc: &DocumentoCompleto,
    cliente: Option<&Cliente>,
    imp: &HashMap<String, String>,
) -> String {
    let d = &doc.documento;

    let anno = d.data.get(..4).unwrap_or("0000");
    let progressivo = format!("{:05}", d.id);

    let cedente_piva = opt_str(imp, "partita_iva");
    let cedente_cf = opt_str(imp, "codice_fiscale");
    let cedente_rs = xml_escape(opt_str(imp, "ragione_sociale"));
    let cedente_ind = xml_escape(opt_str(imp, "indirizzo"));
    let cedente_cap = opt_str(imp, "cap");
    let cedente_citta = xml_escape(opt_str(imp, "citta"));
    let cedente_prov = opt_str(imp, "provincia");
    let regime = opt_str(imp, "regime_fiscale");
    let cod_dest = opt_str(imp, "codice_destinatario");

    let tipo_doc = match d.tipo_documento.as_str() {
        "nota_credito" => "TD04",
        _ => "TD01",
    };

    // DatiPagamento
    let scadenza_xml = d.scadenza_pagamento.as_deref().unwrap_or(&d.data);
    let modalita = metodo_pagamento_code(d.metodo_pagamento.as_deref());

    // Righe
    let mut linee_xml = String::new();
    for (i, riga) in doc.righe.iter().enumerate() {
        let lordo = riga.quantita * riga.prezzo_unitario;
        let sconto = lordo * (riga.sconto_percentuale / 100.0);
        let imponibile = lordo - sconto;
        linee_xml.push_str(&format!(
            "      <DettaglioLinee>\n\
                     <NumeroLinea>{}</NumeroLinea>\n\
                     <Descrizione>{}</Descrizione>\n\
                     <Quantita>{:.2}</Quantita>\n\
                     <UnitaMisura>PZ</UnitaMisura>\n\
                     <PrezzoUnitario>{:.2}</PrezzoUnitario>\n\
             {}\
             <PrezzoTotale>{:.2}</PrezzoTotale>\n\
             <AliquotaIVA>{:.2}</AliquotaIVA>\n\
             </DettaglioLinee>\n",
            i + 1,
            xml_escape(&riga.descrizione),
            riga.quantita,
            riga.prezzo_unitario,
            if riga.sconto_percentuale > 0.0 {
                format!("        <ScontoMaggiorazione><Tipo>SC</Tipo><Percentuale>{:.2}</Percentuale></ScontoMaggiorazione>\n", riga.sconto_percentuale)
            } else {
                String::new()
            },
            imponibile,
            riga.iva_percentuale,
        ));
    }

    // Riepilogo IVA
    let riepilogo = raggruppa_iva(&doc.righe);
    let mut riepilogo_xml = String::new();
    for r in &riepilogo {
        riepilogo_xml.push_str(&format!(
            "      <DatiRiepilogo>\n\
             <AliquotaIVA>{:.2}</AliquotaIVA>\n\
             <ImponibileImporto>{:.2}</ImponibileImporto>\n\
             <Imposta>{:.2}</Imposta>\n\
             <EsigibilitaIVA>I</EsigibilitaIVA>\n\
             </DatiRiepilogo>\n",
            r.aliquota, r.imponibile, r.imposta,
        ));
    }

    // CessionarioCommittente
    let committente_xml = if let Some(c) = cliente {
        let nome = xml_escape(&c.ragione_sociale);
        let ind = xml_escape(c.indirizzo.as_deref().unwrap_or("ND"));
        let cap = c.cap.as_deref().unwrap_or("00000");
        let citta = xml_escape(c.citta.as_deref().unwrap_or("ND"));
        let prov = c.provincia.as_deref().unwrap_or("");

        let id_fiscale_xml = if let Some(piva) = &c.partita_iva {
            if !piva.is_empty() {
                format!(
                    "        <IdFiscaleIVA><IdPaese>IT</IdPaese><IdCodice>{}</IdCodice></IdFiscaleIVA>\n",
                    xml_escape(piva)
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let cf_xml = if let Some(cf) = &c.codice_fiscale {
            if !cf.is_empty() {
                format!("        <CodiceFiscale>{}</CodiceFiscale>\n", xml_escape(cf))
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let prov_xml = if !prov.is_empty() {
            format!("        <Provincia>{}</Provincia>\n", prov)
        } else {
            String::new()
        };

        format!(
            "    <CessionarioCommittente>\n\
             <DatiAnagrafici>\n\
             {id_fiscale_xml}{cf_xml}\
             <Anagrafica><Denominazione>{nome}</Denominazione></Anagrafica>\n\
             </DatiAnagrafici>\n\
             <Sede>\n\
             <Indirizzo>{ind}</Indirizzo>\n\
             <CAP>{cap}</CAP>\n\
             <Comune>{citta}</Comune>\n\
             {prov_xml}\
             <Nazione>IT</Nazione>\n\
             </Sede>\n\
             </CessionarioCommittente>\n"
        )
    } else {
        String::from(
            "    <CessionarioCommittente>\n\
             <DatiAnagrafici><Anagrafica><Denominazione>Cliente generico</Denominazione></Anagrafica></DatiAnagrafici>\n\
             <Sede><Indirizzo>ND</Indirizzo><CAP>00000</CAP><Comune>ND</Comune><Nazione>IT</Nazione></Sede>\n\
             </CessionarioCommittente>\n",
        )
    };

    let _ = anno;

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<p:FatturaElettronica xmlns:p="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2" versione="FPR12">
  <FatturaElettronicaHeader>
    <DatiTrasmissione>
      <IdTrasmittente>
        <IdPaese>IT</IdPaese>
        <IdCodice>{cedente_piva}</IdCodice>
      </IdTrasmittente>
      <ProgressivoInvio>{progressivo}</ProgressivoInvio>
      <FormatoTrasmissione>FPR12</FormatoTrasmissione>
      <CodiceDestinatario>{cod_dest}</CodiceDestinatario>
    </DatiTrasmissione>
    <CedentePrestatore>
      <DatiAnagrafici>
        <IdFiscaleIVA>
          <IdPaese>IT</IdPaese>
          <IdCodice>{cedente_piva}</IdCodice>
        </IdFiscaleIVA>
        <CodiceFiscale>{cedente_cf}</CodiceFiscale>
        <Anagrafica>
          <Denominazione>{cedente_rs}</Denominazione>
        </Anagrafica>
        <RegimeFiscale>{regime}</RegimeFiscale>
      </DatiAnagrafici>
      <Sede>
        <Indirizzo>{cedente_ind}</Indirizzo>
        <CAP>{cedente_cap}</CAP>
        <Comune>{cedente_citta}</Comune>
        <Provincia>{cedente_prov}</Provincia>
        <Nazione>IT</Nazione>
      </Sede>
    </CedentePrestatore>
{committente_xml}  </FatturaElettronicaHeader>
  <FatturaElettronicaBody>
    <DatiGenerali>
      <DatiGeneraliDocumento>
        <TipoDocumento>{tipo_doc}</TipoDocumento>
        <Divisa>EUR</Divisa>
        <Data>{data}</Data>
        <Numero>{numero}</Numero>
        <ImportoTotaleDocumento>{totale:.2}</ImportoTotaleDocumento>
      </DatiGeneraliDocumento>
    </DatiGenerali>
    <DatiBeniServizi>
{linee_xml}{riepilogo_xml}    </DatiBeniServizi>
    <DatiPagamento>
      <CondizioniPagamento>TP02</CondizioniPagamento>
      <DettaglioPagamento>
        <ModalitaPagamento>{modalita}</ModalitaPagamento>
        <DataScadenzaPagamento>{scadenza}</DataScadenzaPagamento>
        <ImportoPagamento>{totale:.2}</ImportoPagamento>
      </DettaglioPagamento>
    </DatiPagamento>
  </FatturaElettronicaBody>
</p:FatturaElettronica>
"#,
        cedente_piva = cedente_piva,
        progressivo = progressivo,
        cod_dest = cod_dest,
        cedente_cf = cedente_cf,
        cedente_rs = cedente_rs,
        cedente_ind = cedente_ind,
        cedente_cap = cedente_cap,
        cedente_citta = cedente_citta,
        cedente_prov = cedente_prov,
        regime = regime,
        committente_xml = committente_xml,
        tipo_doc = tipo_doc,
        data = d.data,
        numero = xml_escape(&d.numero),
        totale = d.totale_documento,
        linee_xml = linee_xml,
        riepilogo_xml = riepilogo_xml,
        modalita = modalita,
        scadenza = scadenza_xml,
    )
}

#[tauri::command]
pub async fn genera_fattura_pa(
    documento_id: i64,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let documento = sqlx::query_as::<_, crate::models::Documento>(
        "SELECT * FROM documenti WHERE id=?",
    )
    .bind(documento_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("documento id={documento_id} non trovato")))?;

    let righe = sqlx::query_as::<_, RigaDocumento>(
        "SELECT * FROM righe_documento WHERE documento_id=? ORDER BY ordine ASC",
    )
    .bind(documento_id)
    .fetch_all(&state.db)
    .await?;

    let doc_completo = DocumentoCompleto { documento, righe };

    let cliente: Option<Cliente> = if let Some(cid) = doc_completo.documento.cliente_id {
        sqlx::query_as::<_, Cliente>("SELECT * FROM clienti WHERE id=?")
            .bind(cid)
            .fetch_optional(&state.db)
            .await?
    } else {
        None
    };

    let rows = sqlx::query_as::<_, ImpostazioneRow>("SELECT chiave, valore FROM impostazioni")
        .fetch_all(&state.db)
        .await?;
    let imp: HashMap<String, String> = rows.into_iter().map(|r| (r.chiave, r.valore)).collect();

    let piva = imp.get("partita_iva").map(|s| s.as_str()).unwrap_or("");
    if piva.is_empty() {
        return Err(AppError::Validation(
            "Partita IVA azienda non configurata. Vai in Impostazioni per compilarla.".into(),
        ));
    }

    let xml = build_xml(&doc_completo, cliente.as_ref(), &imp);

    let fatture_dir = state.fatture_dir.clone();

    let anno = doc_completo.documento.data.get(..4).unwrap_or("0000");
    let numero_safe = doc_completo
        .documento
        .numero
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', ' '], "_");
    let filename = format!("FPA_{}_{}.xml", anno, numero_safe);
    let file_path = fatture_dir.join(&filename);

    std::fs::write(&file_path, xml.as_bytes())
        .map_err(|e| AppError::Internal(format!("Errore scrittura file: {e}")))?;

    Ok(file_path.display().to_string())
}
