-- Clienti
CREATE TABLE IF NOT EXISTS clienti (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    ragione_sociale TEXT NOT NULL,
    partita_iva TEXT,
    codice_fiscale TEXT,
    indirizzo   TEXT,
    citta       TEXT,
    cap         TEXT,
    provincia   TEXT,
    telefono    TEXT,
    email       TEXT,
    note        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Fornitori
CREATE TABLE IF NOT EXISTS fornitori (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    ragione_sociale TEXT NOT NULL,
    partita_iva TEXT,
    codice_fiscale TEXT,
    indirizzo   TEXT,
    citta       TEXT,
    cap         TEXT,
    provincia   TEXT,
    telefono    TEXT,
    email       TEXT,
    note        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Ricambi (magazzino)
CREATE TABLE IF NOT EXISTS ricambi (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    codice_interno  TEXT NOT NULL UNIQUE,
    codice_oem      TEXT,
    descrizione     TEXT NOT NULL,
    marca           TEXT,
    modello_auto    TEXT,
    anno_da         INTEGER,
    anno_a          INTEGER,
    categoria       TEXT,
    fornitore_id    INTEGER REFERENCES fornitori(id) ON DELETE SET NULL,
    giacenza        INTEGER NOT NULL DEFAULT 0,
    giacenza_minima INTEGER NOT NULL DEFAULT 1,
    prezzo_acquisto REAL NOT NULL DEFAULT 0.0,
    prezzo_vendita  REAL NOT NULL DEFAULT 0.0,
    iva_percentuale REAL NOT NULL DEFAULT 22.0,
    posizione       TEXT,
    note            TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Testata documenti (fatture, DDT, preventivi)
CREATE TABLE IF NOT EXISTS documenti (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tipo_documento  TEXT NOT NULL CHECK(tipo_documento IN ('fattura','preventivo','ddt','nota_credito')),
    numero          TEXT NOT NULL,
    data            TEXT NOT NULL,
    cliente_id      INTEGER REFERENCES clienti(id) ON DELETE RESTRICT,
    fornitore_id    INTEGER REFERENCES fornitori(id) ON DELETE RESTRICT,
    stato           TEXT NOT NULL DEFAULT 'bozza' CHECK(stato IN ('bozza','confermato','pagato','annullato')),
    note            TEXT,
    totale_imponibile REAL NOT NULL DEFAULT 0.0,
    totale_iva      REAL NOT NULL DEFAULT 0.0,
    totale_documento REAL NOT NULL DEFAULT 0.0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tipo_documento, numero)
);

-- Righe documenti
CREATE TABLE IF NOT EXISTS righe_documento (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    documento_id    INTEGER NOT NULL REFERENCES documenti(id) ON DELETE CASCADE,
    ricambio_id     INTEGER REFERENCES ricambi(id) ON DELETE RESTRICT,
    descrizione     TEXT NOT NULL,
    quantita        REAL NOT NULL DEFAULT 1.0,
    prezzo_unitario REAL NOT NULL DEFAULT 0.0,
    sconto_percentuale REAL NOT NULL DEFAULT 0.0,
    iva_percentuale REAL NOT NULL DEFAULT 22.0,
    imponibile      REAL NOT NULL DEFAULT 0.0,
    totale_iva      REAL NOT NULL DEFAULT 0.0,
    totale_riga     REAL NOT NULL DEFAULT 0.0,
    ordine          INTEGER NOT NULL DEFAULT 0
);

-- Movimenti magazzino (traccia ogni entrata/uscita)
CREATE TABLE IF NOT EXISTS movimenti_magazzino (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ricambio_id     INTEGER NOT NULL REFERENCES ricambi(id) ON DELETE CASCADE,
    tipo_movimento  TEXT NOT NULL CHECK(tipo_movimento IN ('carico','scarico','rettifica')),
    quantita        REAL NOT NULL,
    documento_id    INTEGER REFERENCES documenti(id) ON DELETE SET NULL,
    note            TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indici
CREATE INDEX IF NOT EXISTS idx_ricambi_codice_interno ON ricambi(codice_interno);
CREATE INDEX IF NOT EXISTS idx_ricambi_codice_oem ON ricambi(codice_oem);
CREATE INDEX IF NOT EXISTS idx_documenti_cliente ON documenti(cliente_id);
CREATE INDEX IF NOT EXISTS idx_documenti_data ON documenti(data);
CREATE INDEX IF NOT EXISTS idx_righe_documento ON righe_documento(documento_id);
