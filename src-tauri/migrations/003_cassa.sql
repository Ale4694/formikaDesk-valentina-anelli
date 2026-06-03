-- Scontrini (vendite al bancone)
CREATE TABLE IF NOT EXISTS scontrini (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    numero              TEXT NOT NULL UNIQUE,
    data                TEXT NOT NULL,
    operatore           TEXT,
    totale              REAL NOT NULL DEFAULT 0.0,
    sconto_totale       REAL NOT NULL DEFAULT 0.0,
    metodo_pagamento    TEXT NOT NULL DEFAULT 'contanti'
                            CHECK(metodo_pagamento IN ('contanti','carta','satispay','bonifico')),
    stato               TEXT NOT NULL DEFAULT 'chiuso'
                            CHECK(stato IN ('aperto','chiuso','annullato')),
    note                TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Righe scontrino
CREATE TABLE IF NOT EXISTS righe_scontrino (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    scontrino_id        INTEGER NOT NULL REFERENCES scontrini(id) ON DELETE CASCADE,
    ricambio_id         INTEGER REFERENCES ricambi(id) ON DELETE RESTRICT,
    codice              TEXT,
    descrizione         TEXT NOT NULL,
    quantita            REAL NOT NULL DEFAULT 1.0,
    prezzo_unitario     REAL NOT NULL DEFAULT 0.0,
    sconto_percentuale  REAL NOT NULL DEFAULT 0.0,
    totale_riga         REAL NOT NULL DEFAULT 0.0,
    iva_percentuale     REAL NOT NULL DEFAULT 22.0
);

-- Indici
CREATE INDEX IF NOT EXISTS idx_scontrini_data   ON scontrini(data);
CREATE INDEX IF NOT EXISTS idx_scontrini_stato  ON scontrini(stato);
CREATE INDEX IF NOT EXISTS idx_righe_scontrino  ON righe_scontrino(scontrino_id);
