-- Campi dati trasporto (DDT/Fattura/NotaCredito/VenditaBanco) e accessori
-- richiesti da Valentina: erano stampati in PDF come testo fisso/vuoto,
-- non collegati a nessun dato reale del documento.
ALTER TABLE documenti ADD COLUMN vettore TEXT;
ALTER TABLE documenti ADD COLUMN data_ora_ritiro TEXT;
ALTER TABLE documenti ADD COLUMN n_colli INTEGER DEFAULT 0;
ALTER TABLE documenti ADD COLUMN aspetto_esteriore_beni TEXT;
ALTER TABLE documenti ADD COLUMN porto TEXT;
ALTER TABLE documenti ADD COLUMN causale_trasporto TEXT DEFAULT 'Vendita';
ALTER TABLE documenti ADD COLUMN trasporto_a_cura TEXT DEFAULT 'Destinatario';
ALTER TABLE documenti ADD COLUMN banca_appoggio TEXT;
ALTER TABLE documenti ADD COLUMN agente TEXT;
ALTER TABLE documenti ADD COLUMN bolli_art15 TEXT;
ALTER TABLE documenti ADD COLUMN spese_varie REAL DEFAULT 0;
ALTER TABLE documenti ADD COLUMN spese_incasso REAL DEFAULT 0;

-- Buono/Preventivo stampavano storicamente "CONSEGNA" come causale (non "Vendita"):
-- preserva quel testo sui documenti già esistenti per non alterare le stampe già emesse.
UPDATE documenti SET causale_trasporto = 'CONSEGNA' WHERE tipo_documento IN ('buono', 'preventivo');
