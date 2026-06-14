use crate::{AppError, AppState};
use crate::models::Fornitore;
use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tauri::{Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

// ── Strutture pubbliche ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RigaDdtParsed {
    pub codice_fornitore: String,
    pub descrizione: String,
    pub um: String,
    pub quantita: f64,
    pub ricambio_id: Option<i64>,
    pub codice_interno_match: Option<String>,
    pub descrizione_match: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseDdtResult {
    pub righe: Vec<RigaDdtParsed>,
    pub fornitore_rilevato_nome: Option<String>,
    pub fornitore_rilevato_piva: Option<String>,
    pub fornitore_id_match: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RigaDdtImport {
    pub codice_fornitore: String,
    pub descrizione: String,
    pub um: String,
    pub quantita: f64,
    pub ricambio_id: Option<i64>,
    pub carica_magazzino: bool,
}

// ── Strutture interne DB ──────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct RicambioMatchRow {
    id: i64,
    codice_interno: String,
    descrizione: String,
}

#[derive(sqlx::FromRow)]
struct FornitoreIdRow {
    id: i64,
}

// ── Regex statiche ────────────────────────────────────────────────────────────

static ROW_RE: OnceLock<Regex> = OnceLock::new();
static PIVA_RE: OnceLock<Regex> = OnceLock::new();
static ADDR_RE: OnceLock<Regex> = OnceLock::new();
static SKIP_HDR_RE: OnceLock<Regex> = OnceLock::new();

fn row_re() -> &'static Regex {
    ROW_RE.get_or_init(|| {
        // Formato CORI: BRAND PARTNUMBER  descrizione  UM  quantita
        // es testo nativo: "LPR F2081P  Disco freno ant DX  PZ  2,00"
        // es OCR:          "LER F2081P ABARTH ...500C|pz| 2,00"   → |UM| con pipe da bordi tabella
        //                  "LPR 05P483 FIAT CINQUECENTO Pzl 1,00" → 'l' è OCR di '|'
        Regex::new(
            r"(?i)^([A-Z]{2,6}\s+[A-Z0-9][A-Z0-9\-\/\.]*)\s+(.+?)[\s|]+(PZ|KG|LT|MT|SC|CF|NR|SET|GR|ML|CL|NP|PC)[l|]?\s+([\d]+[,\.][\d]{1,3})"
        ).expect("regex righe DDT non valida")
    })
}

fn piva_re() -> &'static Regex {
    PIVA_RE.get_or_init(|| {
        Regex::new(r"(?i)(?:partita\s+iva|p\.?\s*iva)\s*:?\s*(\d{11})")
            .expect("regex PIVA non valida")
    })
}

fn addr_re() -> &'static Regex {
    ADDR_RE.get_or_init(|| {
        Regex::new(r"(?i)^(?:via|viale|piazza|corso|strada|loc\.|fraz\.|contrada|largo|vicolo|p\.za)")
            .expect("regex indirizzo non valida")
    })
}

fn skip_hdr_re() -> &'static Regex {
    SKIP_HDR_RE.get_or_init(|| {
        Regex::new(r"(?i)^(?:documento\s+di\s+trasporto|d\.d\.t\.|ddt\b|bolla|consegna|packing|invoice|foglio|pagina\s+\d)")
            .expect("regex skip header non valida")
    })
}

// ── Helper: parsing righe articolo ───────────────────────────────────────────

fn parse_righe_from_text(testo: &str) -> Vec<RigaDdtParsed> {
    let re = row_re();
    let mut righe = Vec::new();

    for line in testo.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(caps) = re.captures(line) {
            let codice_fornitore = caps[1].trim().to_uppercase();
            let descrizione = caps[2].trim().to_string();
            let um = caps[3].trim().to_uppercase();
            let qty_str = caps[4].replace(',', ".");
            let quantita = qty_str.parse::<f64>().unwrap_or(0.0);

            if quantita > 0.0 && !codice_fornitore.is_empty() {
                righe.push(RigaDdtParsed {
                    codice_fornitore,
                    descrizione,
                    um,
                    quantita,
                    ricambio_id: None,
                    codice_interno_match: None,
                    descrizione_match: None,
                });
            }
        }
    }
    righe
}

// ── Helper: rilevamento fornitore dall'header del PDF ────────────────────────

fn rileva_fornitore_da_header(testo: &str) -> (Option<String>, Option<String>) {
    let piva = piva_re();
    let addr = addr_re();
    let skip = skip_hdr_re();

    let mut piva_trovata: Option<String> = None;
    let mut nome_trovato: Option<String> = None;

    for line in testo.lines().take(30).map(|l| l.trim()).filter(|l| !l.is_empty()) {
        // Cerca P.IVA (11 cifre)
        if piva_trovata.is_none() {
            if let Some(caps) = piva.captures(line) {
                piva_trovata = Some(caps[1].to_string());
            }
        }

        // Candidato nome azienda: ignora indirizzi, titoli documento, righe solo numeri
        if nome_trovato.is_none()
            && !addr.is_match(line)
            && !skip.is_match(line)
            && !piva.is_match(line)
        {
            let len = line.len();
            let alpha = line.chars().filter(|c| c.is_alphabetic()).count();
            // Prima parola di un singolo carattere maiuscolo = probabile artefatto OCR di logo
            let first_word = line.split_whitespace().next().unwrap_or("");
            let ocr_logo_noise = first_word.len() == 1
                && first_word.chars().next().map_or(false, |c| c.is_uppercase());
            if len >= 3 && len <= 60 && alpha >= 3 && !ocr_logo_noise {
                nome_trovato = Some(line.to_string());
            }
        }

        if piva_trovata.is_some() && nome_trovato.is_some() {
            break;
        }
    }

    (nome_trovato, piva_trovata)
}

// ── Fallback OCR (pdftoppm + tesseract) per PDF basati su immagini ──────────

// Risolve i path dei binari: su Windows prova più candidate paths in ordine,
// altrimenti (o su Linux/Mac) si affida ai binari di sistema sul PATH.
fn resolve_ocr_tools(_resource_dir: Option<&std::path::Path>) -> (String, String, Option<std::path::PathBuf>) {
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            // 1. resource_dir (Tauri bundle resources)
            _resource_dir.map(|r| r.join("bin-windows")),
            // 2. accanto all'exe (AppData\Local\AutoParts Gestionale\)
            std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("bin-windows"))),
            // 3. un livello sopra l'exe
            std::env::current_exe().ok().and_then(|e| e.parent().and_then(|p| p.parent()).map(|p| p.join("bin-windows"))),
        ];
        for base in candidates.iter().flatten() {
            let pdftoppm = base.join("poppler/pdftoppm.exe");
            let tesseract = base.join("tesseract/tesseract.exe");
            eprintln!("[OCR DEBUG] candidato: {:?} exists={}", base, pdftoppm.exists());
            if pdftoppm.exists() && tesseract.exists() {
                let tessdata = base.join("tesseract/tessdata");
                return (
                    pdftoppm.to_string_lossy().into_owned(),
                    tesseract.to_string_lossy().into_owned(),
                    Some(tessdata),
                );
            }
        }
    }
    // Fallback: binari di sistema
    ("pdftoppm".to_owned(), "tesseract".to_owned(), None)
}

fn extract_text_ocr(pdf_path: &str, resource_dir: Option<&std::path::Path>) -> Result<String, AppError> {
    use std::process::Command;

    let (pdftoppm_bin, tesseract_bin, tessdata_dir) = resolve_ocr_tools(resource_dir);

    let tmp_prefix = std::env::temp_dir().join(format!(
        "ddt_ocr_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let prefix_str = tmp_prefix.to_string_lossy().to_string();

    // Renderizza ogni pagina come PPM a 200 dpi
    let pdftoppm = Command::new(&pdftoppm_bin)
        .args(["-r", "200", pdf_path, &prefix_str])
        .output()
        .map_err(|e| AppError::Internal(format!(
            "pdftoppm non avviato (bin={:?}): {}",
            pdftoppm_bin, e
        )))?;

    if !pdftoppm.status.success() {
        return Err(AppError::Internal(format!(
            "pdftoppm fallito (exit {}): stdout={} stderr={}",
            pdftoppm.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&pdftoppm.stdout),
            String::from_utf8_lossy(&pdftoppm.stderr)
        )));
    }

    // Raccogli e ordina i file PPM generati
    let tmp_dir = tmp_prefix
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/tmp"));
    let prefix_name = tmp_prefix
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut ppm_files: Vec<std::path::PathBuf> = std::fs::read_dir(tmp_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(&prefix_name) && name.ends_with(".ppm")
        })
        .map(|e| e.path())
        .collect();
    ppm_files.sort();

    let mut full_text = String::new();

    for ppm in &ppm_files {
        let ppm_str = ppm.to_string_lossy().to_string();

        // Prova ita+eng; se la lingua italiana non è installata tesseract esce con
        // stato non-zero e stdout vuoto — in quel caso riprova con eng soltanto.
        // Se tessdata_dir è Some, imposta TESSDATA_PREFIX per i binari bundlati su Windows.
        if let Some(ref td) = tessdata_dir {
            eprintln!("[TESS DEBUG] TESSDATA_PREFIX: {:?}", td);
            eprintln!("[TESS DEBUG] eng.traineddata exists: {}", td.join("eng.traineddata").exists());
        }

        let mut cmd_ita = Command::new(&tesseract_bin);
        cmd_ita.args([&ppm_str, "stdout", "-l", "ita+eng"]);
        if let Some(ref td) = tessdata_dir {
            cmd_ita.env("TESSDATA_PREFIX", td);
        }

        let tess = match cmd_ita.output() {
            Err(_) => return Err(AppError::Internal(
                "tesseract non trovato. Installa tesseract-ocr per il supporto OCR su PDF immagine."
                    .into(),
            )),
            Ok(o) if o.status.success() && !o.stdout.is_empty() => o,
            Ok(_) => {
                let mut cmd_eng = Command::new(&tesseract_bin);
                cmd_eng.args([&ppm_str, "stdout", "-l", "eng"]);
                if let Some(ref td) = tessdata_dir {
                    cmd_eng.env("TESSDATA_PREFIX", td);
                }
                cmd_eng.output().map_err(|_| AppError::Internal(
                    "tesseract non trovato. Installa tesseract-ocr per il supporto OCR su PDF immagine."
                        .into(),
                ))?
            }
        };

        eprintln!("[TESS DEBUG] stdout len: {}", tess.stdout.len());
        eprintln!("[TESS DEBUG] stderr: {}", String::from_utf8_lossy(&tess.stderr));
        eprintln!("[TESS DEBUG] status: {}", tess.status);

        let debug_content = format!(
            "pdftoppm ok\ntesseract stdout len: {}\ntesseract stderr: {}\ntesseract status: {}",
            tess.stdout.len(),
            String::from_utf8_lossy(&tess.stderr),
            tess.status
        );
        let _ = std::fs::write(std::env::temp_dir().join("ddt_ocr_debug.txt"), debug_content);

        if !tess.stdout.is_empty() {
            full_text.push_str(&String::from_utf8_lossy(&tess.stdout));
            full_text.push('\n');
        }

        let _ = std::fs::remove_file(ppm);
    }

    eprintln!("[OCR DEBUG] testo finale lunghezza: {} chars", full_text.len());

    Ok(full_text)
}

// ── Comandi Tauri ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn seleziona_pdf_ddt(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    let result = app
        .dialog()
        .file()
        .add_filter("PDF", &["pdf"])
        .blocking_pick_file();

    match result {
        Some(FilePath::Path(p)) => Ok(Some(p.to_string_lossy().to_string())),
        _ => Ok(None),
    }
}

#[tauri::command]
pub async fn parse_ddt_fornitore_pdf(
    pdf_path: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<ParseDdtResult, AppError> {
    let bytes = std::fs::read(&pdf_path)
        .map_err(|e| AppError::Internal(format!("Impossibile leggere il PDF: {e}")))?;

    let resource_dir = app.path().resource_dir().ok();
    eprintln!("[OCR DEBUG] resource_dir: {:?}", resource_dir);
    // Prova prima l'estrazione nativa; se il PDF è basato su immagini (testo vuoto) usa OCR
    let testo = match pdf_extract::extract_text_from_mem(&bytes) {
        Ok(t) if !t.trim().is_empty() => t,
        _ => extract_text_ocr(&pdf_path, resource_dir.as_deref())?,
    };

    eprintln!("[OCR DEBUG] testo finale ({} chars): {}", testo.len(), &testo[..testo.len().min(500)]);
    // Parsing righe articolo
    let mut righe = parse_righe_from_text(&testo);

    // Match righe → ricambi (4 strategie in ordine di priorità)
    for riga in &mut righe {
        let codice = riga.codice_fornitore.clone();
        let like_pat = format!("%{codice}%");

        // 1. codice_oem esatto
        let m = sqlx::query_as::<_, RicambioMatchRow>(
            "SELECT id, codice_interno, descrizione FROM ricambi \
             WHERE LOWER(codice_oem) = LOWER(?) LIMIT 1",
        ).bind(&codice).fetch_optional(&state.db).await?;

        // 2. codice_oem LIKE %codice%
        let m = if m.is_none() {
            sqlx::query_as::<_, RicambioMatchRow>(
                "SELECT id, codice_interno, descrizione FROM ricambi \
                 WHERE codice_oem LIKE ? LIMIT 1",
            ).bind(&like_pat).fetch_optional(&state.db).await?
        } else { m };

        // 3. codice_interno esatto
        let m = if m.is_none() {
            sqlx::query_as::<_, RicambioMatchRow>(
                "SELECT id, codice_interno, descrizione FROM ricambi \
                 WHERE LOWER(codice_interno) = LOWER(?) LIMIT 1",
            ).bind(&codice).fetch_optional(&state.db).await?
        } else { m };

        // 4. codice_oem o codice_interno è sottostringa del codice DDT
        //    es: "F2081P" (codice_interno) trovato dentro "LPR F2081P" (codice DDT)
        let m = if m.is_none() {
            sqlx::query_as::<_, RicambioMatchRow>(
                "SELECT id, codice_interno, descrizione FROM ricambi \
                 WHERE (LENGTH(codice_oem) >= 4 AND INSTR(UPPER(?), UPPER(codice_oem)) > 0) \
                    OR (LENGTH(codice_interno) >= 4 AND INSTR(UPPER(?), UPPER(codice_interno)) > 0) \
                 LIMIT 1",
            ).bind(&codice).bind(&codice).fetch_optional(&state.db).await?
        } else { m };

        if let Some(r) = m {
            riga.ricambio_id = Some(r.id);
            riga.codice_interno_match = Some(r.codice_interno);
            riga.descrizione_match = Some(r.descrizione);
        }
    }

    // Rilevamento fornitore dall'header
    let (fornitore_rilevato_nome, fornitore_rilevato_piva) = rileva_fornitore_da_header(&testo);
    let mut fornitore_id_match: Option<i64> = None;

    // 1. Match esatto per P.IVA
    if let Some(ref piva) = fornitore_rilevato_piva {
        let hit = sqlx::query_as::<_, FornitoreIdRow>(
            "SELECT id FROM fornitori WHERE partita_iva = ? LIMIT 1",
        )
        .bind(piva)
        .fetch_optional(&state.db)
        .await?;

        if let Some(f) = hit {
            fornitore_id_match = Some(f.id);
        }
    }

    // 2. Contains match per ragione_sociale (solo se P.IVA non ha trovato nulla)
    if fornitore_id_match.is_none() {
        if let Some(ref nome) = fornitore_rilevato_nome {
            let like_pattern = format!("%{nome}%");
            let hit = sqlx::query_as::<_, FornitoreIdRow>(
                "SELECT id FROM fornitori WHERE LOWER(ragione_sociale) LIKE LOWER(?) LIMIT 1",
            )
            .bind(&like_pattern)
            .fetch_optional(&state.db)
            .await?;

            if let Some(f) = hit {
                fornitore_id_match = Some(f.id);
            }
        }
    }

    Ok(ParseDdtResult {
        righe,
        fornitore_rilevato_nome,
        fornitore_rilevato_piva,
        fornitore_id_match,
    })
}

#[tauri::command]
pub async fn importa_ddt_fornitore(
    fornitore_id: i64,
    numero: String,
    data: String,
    righe: Vec<RigaDdtImport>,
    pdf_path_sorgente: String,
    state: State<'_, AppState>,
) -> Result<i64, AppError> {
    if numero.trim().is_empty() {
        return Err(AppError::Validation("numero DDT obbligatorio".into()));
    }
    if righe.is_empty() {
        return Err(AppError::Validation("almeno una riga richiesta".into()));
    }

    let ddt_dir = state.fatture_dir.join("ddt_fornitori");
    std::fs::create_dir_all(&ddt_dir)
        .map_err(|e| AppError::Internal(format!("Impossibile creare cartella ddt_fornitori: {e}")))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let safe_numero = numero.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', ' '], "_");
    let nome_file = format!("DDT_{safe_numero}_{timestamp}.pdf");
    let dest_path = ddt_dir.join(&nome_file);
    let pdf_allegato = format!("ddt_fornitori/{nome_file}");

    std::fs::copy(&pdf_path_sorgente, &dest_path)
        .map_err(|e| AppError::Internal(format!("Impossibile copiare il PDF: {e}")))?;

    let mut tx = state.db.begin().await?;

    let doc_id = sqlx::query(
        "INSERT INTO documenti (tipo_documento, numero, data, fornitore_id, stato,
         totale_imponibile, totale_iva, totale_documento, pdf_allegato)
         VALUES ('ddt_fornitore', ?, ?, ?, 'confermato', 0, 0, 0, ?)",
    )
    .bind(&numero)
    .bind(&data)
    .bind(fornitore_id)
    .bind(&pdf_allegato)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for (i, riga) in righe.iter().enumerate() {
        sqlx::query(
            "INSERT INTO righe_documento (documento_id, ricambio_id, descrizione, quantita,
             prezzo_unitario, sconto_percentuale, iva_percentuale, imponibile, totale_iva,
             totale_riga, ordine)
             VALUES (?, ?, ?, ?, 0, 0, 0, 0, 0, 0, ?)",
        )
        .bind(doc_id)
        .bind(riga.ricambio_id)
        .bind(&riga.descrizione)
        .bind(riga.quantita)
        .bind(i as i64)
        .execute(&mut *tx)
        .await?;

        // Aggiorna giacenza solo se la riga ha un ricambio abbinato E carica_magazzino=true
        if let Some(rid) = riga.ricambio_id {
            if riga.carica_magazzino {
                sqlx::query(
                    "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?",
                )
                .bind(riga.quantita as i64)
                .bind(rid)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO movimenti_magazzino \
                     (ricambio_id, tipo_movimento, quantita, documento_id, note)
                     VALUES (?, 'carico', ?, ?, 'DDT Fornitore')",
                )
                .bind(rid)
                .bind(riga.quantita)
                .bind(doc_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(doc_id)
}

#[tauri::command]
pub async fn crea_fornitore_rapido(
    ragione_sociale: String,
    partita_iva: Option<String>,
    state: State<'_, AppState>,
) -> Result<Fornitore, AppError> {
    if ragione_sociale.trim().is_empty() {
        return Err(AppError::Validation("ragione_sociale obbligatoria".into()));
    }

    let id = sqlx::query(
        "INSERT INTO fornitori (ragione_sociale, partita_iva) VALUES (?, ?)",
    )
    .bind(ragione_sociale.trim())
    .bind(&partita_iva)
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    let fornitore = sqlx::query_as::<_, Fornitore>("SELECT * FROM fornitori WHERE id=?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    Ok(fornitore)
}

#[tauri::command]
pub async fn apri_pdf_allegato(
    documento_id: i64,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    let pdf_allegato: Option<String> = sqlx::query_scalar(
        "SELECT pdf_allegato FROM documenti WHERE id=?",
    )
    .bind(documento_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    let relativo = pdf_allegato
        .ok_or_else(|| AppError::NotFound("Nessun PDF allegato a questo documento".into()))?;

    let assoluto = state.fatture_dir.join(&relativo);

    if !assoluto.exists() {
        return Err(AppError::NotFound(format!(
            "File PDF non trovato: {}",
            assoluto.display()
        )));
    }

    use tauri_plugin_shell::ShellExt;
    app.shell()
        .open(assoluto.to_string_lossy().as_ref(), None)
        .map_err(|e| AppError::Internal(format!("Impossibile aprire il PDF: {e}")))?;

    Ok(())
}
