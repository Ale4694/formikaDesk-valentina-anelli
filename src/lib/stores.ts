import { writable, derived } from 'svelte/store'
import type { Cliente, Fornitore, Ricambio, Documento, DashboardStats, View, DocumentoCompleto } from './types'

function getStoredView(): View {
  try {
    return (localStorage.getItem('autoparts_view') as View) ?? 'dashboard'
  } catch {
    return 'dashboard'
  }
}

function createPersistedView() {
  const { subscribe, set, update } = writable<View>(getStoredView())
  return {
    subscribe,
    set: (v: View) => {
      try { localStorage.setItem('autoparts_view', v) } catch {}
      set(v)
    },
    update,
  }
}

export const currentView = createPersistedView()
export const isLoading = writable(false)
export const globalError = writable<string | null>(null)

export const clienti = writable<Cliente[]>([])
export const fornitori = writable<Fornitore[]>([])
export const ricambi = writable<Ricambio[]>([])
export const documenti = writable<Documento[]>([])
export const dashboardStats = writable<DashboardStats | null>(null)

export const ricambiSottoScorta = derived(ricambi, $r =>
  $r.filter(r => r.giacenza < r.giacenza_minima)
)

export const searchOpen = writable(false)
export const printData = writable<DocumentoCompleto | null>(null)
export const licenseValid = writable<boolean | null>(null)

export const globalSuccess = writable<string | null>(null)

export function setError(msg: string | null) {
  globalError.set(msg)
  if (msg) setTimeout(() => globalError.set(null), 5000)
}

export function setSuccess(msg: string) {
  globalSuccess.set(msg)
  setTimeout(() => globalSuccess.set(null), 5000)
}

export function formatCurrency(n: number): string {
  return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n)
}

export function formatDate(s: string): string {
  return new Date(s).toLocaleDateString('it-IT')
}
