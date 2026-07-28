-- Permette di nascondere l'IBAN in stampa sul singolo documento (richiesta Valentina):
-- alcuni DDT/Fatture non devono mostrarlo anche se l'azienda ha un IBAN configurato.
ALTER TABLE documenti ADD COLUMN mostra_iban INTEGER NOT NULL DEFAULT 1;
