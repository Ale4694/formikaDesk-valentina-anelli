CREATE TABLE IF NOT EXISTS impostazioni (
    chiave TEXT PRIMARY KEY,
    valore TEXT NOT NULL DEFAULT ''
);

INSERT OR IGNORE INTO impostazioni (chiave, valore) VALUES
    ('ragione_sociale', ''),
    ('partita_iva', ''),
    ('codice_fiscale', ''),
    ('indirizzo', ''),
    ('cap', ''),
    ('citta', ''),
    ('provincia', ''),
    ('codice_destinatario', '0000000'),
    ('regime_fiscale', 'RF01');
