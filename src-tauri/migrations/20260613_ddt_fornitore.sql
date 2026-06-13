-- Colonna per allegato PDF sui documenti (usata da ddt_fornitore)
ALTER TABLE documenti ADD COLUMN pdf_allegato TEXT DEFAULT NULL;
