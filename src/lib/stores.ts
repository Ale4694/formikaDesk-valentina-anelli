import { writable, derived } from 'svelte/store'
import type { Cliente, Fornitore, Ricambio, Documento, DashboardStats, View, DocumentoCompleto, LicenseInfo, AppConfig, ReportMensile } from './types'

try { localStorage.removeItem('autoparts_view') } catch {}

export const currentView = writable<View>('dashboard')
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
export const editDocumentoId = writable<number | null>(null)
export const schedaClienteId = writable<number | null>(null)
export const printData = writable<DocumentoCompleto | null>(null)
export const printReportData = writable<ReportMensile | null>(null)
export const licenseValid = writable<boolean | null>(null)
export const licenseInfo = writable<LicenseInfo | null>(null)
export const appConfig = writable<AppConfig | null>(null)

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

// "Quanto tempo fa" arrotondato, in italiano. Un prezzo di due anni fa
// e uno di due mesi fa hanno peso diverso: qui evitiamo che l'utente
// debba farsi il calcolo a mente ogni volta che guarda una data.
export function formatRelativeTime(s: string): string {
  const diffMs = Date.now() - new Date(s).getTime()
  const diffDays = Math.floor(diffMs / 86_400_000)

  if (diffDays <= 0) return 'oggi'
  if (diffDays === 1) return 'ieri'
  if (diffDays < 30) return `${diffDays} giorni fa`

  const diffMonths = Math.round(diffDays / 30.44)
  if (diffMonths < 12) return `${diffMonths} mes${diffMonths === 1 ? 'e' : 'i'} fa`

  const diffYears = Math.round(diffDays / 365.25)
  return `${diffYears} ann${diffYears === 1 ? 'o' : 'i'} fa`
}
