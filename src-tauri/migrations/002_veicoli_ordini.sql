-- Veicoli (associati a clienti)
CREATE TABLE IF NOT EXISTS veicoli (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    cliente_id  INTEGER NOT NULL REFERENCES clienti(id) ON DELETE CASCADE,
    targa       TEXT NOT NULL UNIQUE,
    marca       TEXT,
    modello     TEXT,
    anno        INTEGER,
    cilindrata  TEXT,
    carburante  TEXT,
    km_attuali  INTEGER,
    note        TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Ordini fornitore
CREATE TABLE IF NOT EXISTS ordini_fornitore (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    fornitore_id INTEGER NOT NULL REFERENCES fornitori(id) ON DELETE RESTRICT,
    data        TEXT NOT NULL,
    stato       TEXT NOT NULL DEFAULT 'bozza'
                    CHECK(stato IN ('bozza','inviato','ricevuto','annullato')),
    note        TEXT,
    totale      REAL NOT NULL DEFAULT 0.0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Righe ordine fornitore
CREATE TABLE IF NOT EXISTS righe_ordine (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ordine_id       INTEGER NOT NULL REFERENCES ordini_fornitore(id) ON DELETE CASCADE,
    ricambio_id     INTEGER REFERENCES ricambi(id) ON DELETE RESTRICT,
    descrizione     TEXT NOT NULL,
    quantita        REAL NOT NULL DEFAULT 1.0,
    prezzo_unitario REAL NOT NULL DEFAULT 0.0,
    totale_riga     REAL NOT NULL DEFAULT 0.0
);

-- Aggiunta colonne scadenza a documenti
ALTER TABLE documenti ADD COLUMN scadenza_pagamento TEXT;
ALTER TABLE documenti ADD COLUMN giorni_pagamento INTEGER DEFAULT 30;

-- Indici
CREATE INDEX IF NOT EXISTS idx_veicoli_cliente ON veicoli(cliente_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_veicoli_targa ON veicoli(targa);
CREATE INDEX IF NOT EXISTS idx_ordini_fornitore_id ON ordini_fornitore(fornitore_id);
CREATE INDEX IF NOT EXISTS idx_ordini_stato ON ordini_fornitore(stato);
