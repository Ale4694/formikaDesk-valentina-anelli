#!/usr/bin/env python3
"""
Genera chiavi di licenza per FormikaDesk.

Uso:
  python3 genera_chiave.py <machine_id> demo <YYYY-MM-DD>
  python3 genera_chiave.py <machine_id> perm <YYYY-MM-DD>

Esempi:
  python3 genera_chiave.py 7ed11013abcdef12 demo 2026-06-23
  python3 genera_chiave.py 7ed11013abcdef12 perm 2027-06-08
"""

import sys
import hashlib
from datetime import datetime

BASE36 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ'


def to_base36(n: int, width: int = 8) -> str:
    if n == 0:
        return '0' * width
    digits = []
    while n > 0:
        digits.append(BASE36[n % 36])
        n //= 36
    while len(digits) < width:
        digits.append('0')
    return ''.join(reversed(digits))


def encode_date(date_str: str):
    d = datetime.strptime(date_str, '%Y-%m-%d')
    n = int(d.strftime('%Y%m%d'))
    b36 = to_base36(n, 8)
    return b36[:4], b36[4:]


def compute_machine_hash(machine_id: str) -> str:
    prefix = machine_id[:16]
    return hashlib.sha256(prefix.encode('utf-8')).hexdigest()


def main():
    if len(sys.argv) != 4:
        print(__doc__)
        sys.exit(1)

    machine_id = sys.argv[1]
    tipo = sys.argv[2].lower()
    data_scadenza = sys.argv[3]

    if tipo not in ('demo', 'perm'):
        print(f"Errore: tipo deve essere 'demo' o 'perm', ricevuto '{tipo}'")
        sys.exit(1)

    try:
        datetime.strptime(data_scadenza, '%Y-%m-%d')
    except ValueError:
        print(f"Errore: data non valida '{data_scadenza}', usa formato YYYY-MM-DD")
        sys.exit(1)

    mhash = compute_machine_hash(machine_id)
    xxxx = mhash[:4].upper()
    yyyy, zzzz = encode_date(data_scadenza)
    prefix = 'DEMO' if tipo == 'demo' else 'PERM'
    chiave = f'{prefix}-{xxxx}-{yyyy}-{zzzz}'
    print(chiave)


if __name__ == '__main__':
    main()
