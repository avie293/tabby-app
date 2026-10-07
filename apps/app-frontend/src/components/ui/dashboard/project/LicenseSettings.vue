<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SaveIcon } from '@modrinth/assets'
import {
	Button,
	Combobox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const messages = defineMessages({
	license: { id: 'app.dashboard.project.license', defaultMessage: 'License' },
	licenseDescription: {
		id: 'app.dashboard.project.license-description',
		defaultMessage:
			'Choose an SPDX license or enter its identifier. The license tells others what they may do with your project.',
	},
	custom: { id: 'app.dashboard.project.license-custom', defaultMessage: 'Other SPDX identifier' },
	url: { id: 'app.dashboard.project.license-url', defaultMessage: 'License URL (optional)' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
})

const CUSTOM = '__custom__'
const LICENSES: [string, string][] = [
	['LicenseRef-All-Rights-Reserved', 'All Rights Reserved'],
	['MIT', 'MIT License'],
	['Apache-2.0', 'Apache License 2.0'],
	['GPL-3.0-only', 'GNU GPL v3.0 only'],
	['GPL-3.0-or-later', 'GNU GPL v3.0 or later'],
	['LGPL-3.0-only', 'GNU LGPL v3.0 only'],
	['LGPL-3.0-or-later', 'GNU LGPL v3.0 or later'],
	['AGPL-3.0-only', 'GNU AGPL v3.0 only'],
	['MPL-2.0', 'Mozilla Public License 2.0'],
	['BSD-2-Clause', 'BSD 2-Clause'],
	['BSD-3-Clause', 'BSD 3-Clause'],
	['ISC', 'ISC License'],
	['Unlicense', 'The Unlicense'],
	['CC0-1.0', 'CC0 1.0 (public domain)'],
	['CC-BY-4.0', 'CC BY 4.0'],
	['CC-BY-SA-4.0', 'CC BY-SA 4.0'],
	['CC-BY-NC-4.0', 'CC BY-NC 4.0'],
	['CC-BY-NC-SA-4.0', 'CC BY-NC-SA 4.0'],
	['CC-BY-ND-4.0', 'CC BY-ND 4.0'],
]

const options = computed(() => [
	...LICENSES.map(([value, label]) => ({ value, label, subLabel: value })),
	{ value: CUSTOM, label: formatMessage(messages.custom) },
])

const selected = ref('')
const customId = ref('')
const url = ref('')
watch(
	() => props.project.license,
	(license) => {
		const known = LICENSES.some(([value]) => value === license.id)
		selected.value = known ? license.id : CUSTOM
		customId.value = known ? '' : license.id
		url.value = license.url ?? ''
	},
	{ immediate: true },
)

const licenseId = computed(() =>
	selected.value === CUSTOM ? customId.value.trim() : selected.value,
)

const saveMutation = useMutation({
	mutationFn: () =>
		client.labrinth.projects_v3.edit(props.project.id, {
			license_id: licenseId.value,
			license_url: url.value.trim() || null,
		}),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<section
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
	>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.license) }}</span>
			<span class="text-sm text-secondary">{{ formatMessage(messages.licenseDescription) }}</span>
			<Combobox v-model="selected" class="max-w-md" :options="options" searchable />
			<Input
				v-if="selected === CUSTOM"
				v-model="customId"
				class="max-w-md"
				placeholder="e.g. EPL-2.0"
			/>
		</div>
		<label class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.url) }}</span>
			<Input v-model="url" type="url" placeholder="https://" />
		</label>
		<div class="flex justify-end">
			<Button
				color="brand"
				type="colored"
				:disabled="!licenseId || saveMutation.isPending.value"
				@click="saveMutation.mutate()"
			>
				<SaveIcon aria-hidden="true" />
				{{ formatMessage(messages.save) }}
			</Button>
		</div>
	</section>
</template>
