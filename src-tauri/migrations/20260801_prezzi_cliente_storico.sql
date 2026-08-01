-- Scheda cliente: storico prezzi/vendite e prezzi dedicati.
-- Additiva e idempotente.
-- Solo CREATE TABLE/INDEX IF NOT EXISTS e CREATE VIEW IF NOT EXISTS.

CREATE TABLE IF NOT EXISTS prezzi_cliente (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    cliente_id    INTEGER NOT NULL
                  REFERENCES clienti(id) ON DELETE CASCADE,
    ricambio_id   INTEGER NOT NULL
                  REFERENCES ricambi(id) ON DELETE CASCADE,
    prezzo        REAL NOT NULL,
    sconto_perc   REAL,
    note          TEXT,
    aggiornato_il TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(cliente_id, ricambio_id)
);

CREATE INDEX IF NOT EXISTS idx_prezzi_cliente_cliente
    ON prezzi_cliente(cliente_id);

-- Query storiche: documenti di un cliente ordinati per data.
CREATE INDEX IF NOT EXISTS idx_documenti_cliente_data
    ON documenti(cliente_id, data);

-- Query storiche: righe per articolo.
CREATE INDEX IF NOT EXISTS idx_righe_documento_ricambio
    ON righe_documento(ricambio_id);

-- Punto unico di verita per lo storico vendite cliente/articolo.
--
-- Tipi inclusi: fattura, ddt, vendita_banco, buono.
-- Sono vendite concluse verso il cliente.
-- Il buono e' incluso di proposito: e' il documento che la
-- cliente consulta piu' spesso per il prezzo gia' praticato.
--
-- Escluso: preventivo.
-- E' una trattativa non conclusa, falserebbe medie e min/max.
--
-- Escluso: nota_credito.
-- E' un reso, non una vendita. Le righe non hanno segno negativo
-- in questo schema, quindi non si possono nettare riga per riga.
--
-- Escluso: ddt_fornitore.
-- E' lato fornitore (fornitore_id), non riguarda i clienti.
--
-- Escluso: fattura_differita.
-- In questa app nasce SEMPRE copiando le righe di uno o piu' DDT
-- gia' emessi (vedi NuovaFattura.svelte: per questo tipo il
-- pulsante "+ Aggiungi riga" e' nascosto, le righe arrivano solo
-- da importaRigheDdt() sui DDT selezionati).
-- Includerla insieme al DDT duplicherebbe la stessa vendita
-- due volte (data DDT e data fattura), gonfiando quantita' e
-- frequenza. Il DDT resta il dato canonico, con la data reale
-- di consegna.
--
-- Esclusi: stato bozza/annullato (non sono vendite valide).
-- Escluse: righe con ricambio_id NULL (voci a testo libero).
CREATE VIEW IF NOT EXISTS v_storico_vendite AS
SELECT
    d.cliente_id         AS cliente_id,
    r.id                  AS articolo_id,
    r.codice_interno      AS codice_articolo,
    r.descrizione         AS descrizione,
    d.data                AS data,
    d.tipo_documento      AS tipo_documento,
    d.numero              AS numero_documento,
    d.id                   AS documento_id,
    rd.quantita           AS quantita,
    rd.prezzo_unitario    AS prezzo_unitario,
    rd.sconto_percentuale AS sconto_perc,
    rd.totale_riga         AS totale_riga
FROM righe_documento rd
JOIN documenti d
    ON d.id = rd.documento_id
JOIN ricambi r
    ON r.id = rd.ricambio_id
WHERE rd.ricambio_id IS NOT NULL
  AND d.cliente_id IS NOT NULL
  AND d.stato NOT IN ('annullato', 'bozza')
  AND d.tipo_documento IN ('fattura', 'ddt', 'vendita_banco', 'buono');
