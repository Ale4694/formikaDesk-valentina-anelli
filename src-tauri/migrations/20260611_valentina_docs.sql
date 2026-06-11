-- Rimuove il CHECK constraint su tipo_documento per supportare i nuovi tipi:
-- vendita_banco, buono, fattura_differita
-- Aggiunge: ddt_collegati (JSON array ID DDT), is_fattura_differita, fatturato

CREATE TABLE documenti_new (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    tipo_documento  TEXT NOT NULL,
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
    scadenza_pagamento TEXT,
    giorni_pagamento INTEGER DEFAULT 30,
    data_pagamento  TEXT,
    metodo_pagamento TEXT,
    riferimento_pagamento TEXT,
    note_pagamento  TEXT,
    ddt_collegati   TEXT DEFAULT NULL,
    is_fattura_differita INTEGER NOT NULL DEFAULT 0,
    fatturato       INTEGER NOT NULL DEFAULT 0,
    UNIQUE(tipo_documento, numero)
);

INSERT INTO documenti_new (
    id, tipo_documento, numero, data, cliente_id, fornitore_id, stato, note,
    totale_imponibile, totale_iva, totale_documento, created_at, updated_at,
    scadenza_pagamento, giorni_pagamento, data_pagamento, metodo_pagamento,
    riferimento_pagamento, note_pagamento,
    ddt_collegati, is_fattura_differita, fatturato
)
SELECT
    id, tipo_documento, numero, data, cliente_id, fornitore_id, stato, note,
    totale_imponibile, totale_iva, totale_documento, created_at, updated_at,
    scadenza_pagamento, giorni_pagamento, data_pagamento, metodo_pagamento,
    riferimento_pagamento, note_pagamento,
    NULL, 0, 0
FROM documenti;

DROP TABLE documenti;
ALTER TABLE documenti_new RENAME TO documenti;

CREATE INDEX IF NOT EXISTS idx_documenti_cliente ON documenti(cliente_id);
CREATE INDEX IF NOT EXISTS idx_documenti_data ON documenti(data);
