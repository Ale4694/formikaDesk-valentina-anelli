ALTER TABLE documenti ADD COLUMN scadenza_pagamento TEXT;
ALTER TABLE documenti ADD COLUMN giorni_pagamento INTEGER DEFAULT 30;
ALTER TABLE documenti ADD COLUMN data_pagamento TEXT;
ALTER TABLE documenti ADD COLUMN metodo_pagamento TEXT;
ALTER TABLE documenti ADD COLUMN riferimento_pagamento TEXT;
ALTER TABLE documenti ADD COLUMN note_pagamento TEXT;
