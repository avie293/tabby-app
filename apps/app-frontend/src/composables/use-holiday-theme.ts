import { computed, ref, watch } from 'vue'

export type Holiday = 'easter' | 'halloween' | 'christmas'
export type HolidayThemeSetting = 'auto' | 'off' | Holiday

export const HOLIDAY_THEME_SETTINGS: HolidayThemeSetting[] = [
	'auto',
	'off',
	'easter',
	'halloween',
	'christmas',
]

const STORAGE_KEY = 'tabbyapp:holiday-theme'
const DAY = 24 * 60 * 60 * 1000

function readSetting(): HolidayThemeSetting {
	try {
		const value = localStorage.getItem(STORAGE_KEY)
		if (value && (HOLIDAY_THEME_SETTINGS as string[]).includes(value)) {
			return value as HolidayThemeSetting
		}
	} catch {
		// Storage can be unavailable; the automatic theme still works.
	}
	return 'auto'
}

const setting = ref<HolidayThemeSetting>(readSetting())
watch(setting, (value) => {
	try {
		localStorage.setItem(STORAGE_KEY, value)
	} catch {
		// Not remembering the choice is fine.
	}
})

/** Easter Sunday of a year, using the anonymous Gregorian algorithm. */
export function easterSunday(year: number) {
	const a = year % 19
	const b = Math.floor(year / 100)
	const c = year % 100
	const d = Math.floor(b / 4)
	const e = b % 4
	const f = Math.floor((b + 8) / 25)
	const g = Math.floor((b - f + 1) / 3)
	const h = (19 * a + b - d - g + 15) % 30
	const i = Math.floor(c / 4)
	const k = c % 4
	const l = (32 + 2 * e + 2 * i - h - k) % 7
	const m = Math.floor((a + 11 * h + 22 * l) / 451)
	const month = Math.floor((h + l - 7 * m + 114) / 31)
	const day = ((h + l - 7 * m + 114) % 31) + 1
	return new Date(year, month - 1, day)
}

/**
 * The holiday whose theme is shown automatically on a date: Easter from Palm
 * Sunday to Easter Monday, Halloween from October 17 to November 1 and
 * Christmas from December 1 to December 26.
 */
export function holidayForDate(date: Date): Holiday | null {
	const day = new Date(date.getFullYear(), date.getMonth(), date.getDate())
	const easter = easterSunday(day.getFullYear()).getTime()
	if (day.getTime() >= easter - 7 * DAY && day.getTime() <= easter + DAY) return 'easter'

	const month = day.getMonth() + 1
	const dayOfMonth = day.getDate()
	if ((month === 10 && dayOfMonth >= 17) || (month === 11 && dayOfMonth === 1)) return 'halloween'
	if (month === 12 && dayOfMonth <= 26) return 'christmas'
	return null
}

// Re-checked every hour, so the theme also switches while the app stays open.
const today = ref(new Date())
setInterval(() => (today.value = new Date()), 60 * 60 * 1000)

export function useHolidayTheme() {
	const active = computed<Holiday | null>(() => {
		if (setting.value === 'off') return null
		if (setting.value === 'auto') return holidayForDate(today.value)
		return setting.value
	})
	return { setting, active }
}
