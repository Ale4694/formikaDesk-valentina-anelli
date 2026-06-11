-- Indici aggiuntivi per performance con dataset grandi (post-migrazione)
CREATE INDEX IF NOT EXISTS idx_ricambi_descrizione        ON ricambi(descrizione);
CREATE INDEX IF NOT EXISTS idx_ricambi_marca              ON ricambi(marca);
CREATE INDEX IF NOT EXISTS idx_documenti_tipo             ON documenti(tipo_documento);
CREATE INDEX IF NOT EXISTS idx_documenti_stato            ON documenti(stato);
CREATE INDEX IF NOT EXISTS idx_documenti_numero           ON documenti(numero);
CREATE INDEX IF NOT EXISTS idx_clienti_ragione_sociale    ON clienti(ragione_sociale);
