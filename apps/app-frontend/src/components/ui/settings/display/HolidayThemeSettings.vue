<script setup lang="ts">
import { Chips, defineMessages, useVIntl } from '@modrinth/ui'

import {
	HOLIDAY_THEME_SETTINGS,
	type HolidayThemeSetting,
	useHolidayTheme,
} from '@/composables/use-holiday-theme'

const { formatMessage } = useVIntl()
const { setting } = useHolidayTheme()

const messages = defineMessages({
	title: { id: 'app.settings.holiday-theme.title', defaultMessage: 'Holiday themes' },
	description: {
		id: 'app.settings.holiday-theme.description',
		defaultMessage:
			'Decorations for Easter, Halloween and Christmas. Automatic shows them around each holiday.',
	},
	auto: { id: 'app.settings.holiday-theme.auto', defaultMessage: 'Automatic' },
	off: { id: 'app.settings.holiday-theme.off', defaultMessage: 'Off' },
	easter: { id: 'app.settings.holiday-theme.easter', defaultMessage: 'Easter' },
	halloween: { id: 'app.settings.holiday-theme.halloween', defaultMessage: 'Halloween' },
	christmas: { id: 'app.settings.holiday-theme.christmas', defaultMessage: 'Christmas' },
})

function label(value: HolidayThemeSetting) {
	return formatMessage(messages[value])
}
</script>

<template>
	<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h3 class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.title) }}</h3>
		<p class="m-0 mt-1 mb-3 text-secondary">{{ formatMessage(messages.description) }}</p>
		<Chips
			v-model="setting"
			:items="HOLIDAY_THEME_SETTINGS"
			:format-label="label"
			:capitalize="false"
		/>
	</section>
</template>
