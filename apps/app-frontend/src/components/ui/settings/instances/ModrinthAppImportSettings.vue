<script setup lang="ts">
import { ImportIcon, RightArrowIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	Checkbox,
	Combobox,
	type ComboboxOption,
	defineMessages,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, ref } from 'vue'

import {
	modrinth_app_get_history,
	modrinth_app_import_instances,
	modrinth_app_list_instances,
	modrinth_app_transfer_playtime,
	modrinthAppImportKeys,
	type ModrinthAppImportOptions,
	type ModrinthAppInstance,
} from '@/helpers/modrinth-app-import'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
const queryClient = useQueryClient()

const messages = defineMessages({
	description: {
		id: 'app.settings.modrinth-app-import.description',
		defaultMessage:
			'Bring instances, worlds and playtime over from the Modrinth App installed on this computer. The Modrinth App itself is not changed.',
	},
	notFound: {
		id: 'app.settings.modrinth-app-import.not-found',
		defaultMessage: 'No Modrinth App installation was found on this computer.',
	},
	loading: {
		id: 'app.settings.modrinth-app-import.loading',
		defaultMessage: 'Reading your Modrinth App instances...',
	},
	instancesTitle: {
		id: 'app.settings.modrinth-app-import.instances.title',
		defaultMessage: 'Instances',
	},
	selectAll: {
		id: 'app.settings.modrinth-app-import.instances.select-all',
		defaultMessage: 'Select all',
	},
	noInstances: {
		id: 'app.settings.modrinth-app-import.instances.empty',
		defaultMessage: "Your Modrinth App doesn't have any installed instances.",
	},
	alreadyImported: {
		id: 'app.settings.modrinth-app-import.instances.already-imported',
		defaultMessage: 'Already imported',
	},
	dataTitle: {
		id: 'app.settings.modrinth-app-import.data.title',
		defaultMessage: 'What to import',
	},
	dataContent: {
		id: 'app.settings.modrinth-app-import.data.content',
		defaultMessage: 'Mods, resource packs, shaders and data packs',
	},
	dataWorlds: {
		id: 'app.settings.modrinth-app-import.data.worlds',
		defaultMessage: 'Worlds',
	},
	dataScreenshots: {
		id: 'app.settings.modrinth-app-import.data.screenshots',
		defaultMessage: 'Screenshots',
	},
	dataSettings: {
		id: 'app.settings.modrinth-app-import.data.settings',
		defaultMessage: 'Game settings, configs and other files',
	},
	dataPlaytime: {
		id: 'app.settings.modrinth-app-import.data.playtime',
		defaultMessage: 'Playtime',
	},
	importButton: {
		id: 'app.settings.modrinth-app-import.import-button',
		defaultMessage:
			'{count, plural, =0 {Select instances to import} one {Import # instance} other {Import # instances}}',
	},
	importStarted: {
		id: 'app.settings.modrinth-app-import.import-started',
		defaultMessage: '{count, plural, one {Importing # instance} other {Importing # instances}}',
	},
	importStartedText: {
		id: 'app.settings.modrinth-app-import.import-started-text',
		defaultMessage: 'You can follow the progress in the downloads panel.',
	},
	playtimeTitle: {
		id: 'app.settings.modrinth-app-import.playtime.title',
		defaultMessage: 'Transfer playtime',
	},
	playtimeDescription: {
		id: 'app.settings.modrinth-app-import.playtime.description',
		defaultMessage:
			'Add the playtime of a Modrinth App instance to any instance here, for example after you rebuilt it.',
	},
	playtimeTarget: {
		id: 'app.settings.modrinth-app-import.playtime.target',
		defaultMessage: 'Choose an instance',
	},
	playtimeTransfer: {
		id: 'app.settings.modrinth-app-import.playtime.transfer',
		defaultMessage: 'Transfer',
	},
	playtimeTransferred: {
		id: 'app.settings.modrinth-app-import.playtime.transferred',
		defaultMessage: 'Transferred to {instance}',
	},
	playtimeTransferredNotification: {
		id: 'app.settings.modrinth-app-import.playtime.transferred-notification',
		defaultMessage: 'Added {playtime} to {instance}',
	},
	unknownInstance: {
		id: 'app.settings.modrinth-app-import.playtime.unknown-instance',
		defaultMessage: 'a removed instance',
	},
	noPlaytime: {
		id: 'app.settings.modrinth-app-import.playtime.empty',
		defaultMessage: 'None of your Modrinth App instances have playtime yet.',
	},
	hours: {
		id: 'app.settings.modrinth-app-import.playtime.hours',
		defaultMessage: '{hours} h {minutes} min',
	},
	minutes: {
		id: 'app.settings.modrinth-app-import.playtime.minutes',
		defaultMessage: '{minutes} min',
	},
})

const instancesQuery = useQuery({
	queryKey: modrinthAppImportKeys.instances,
	queryFn: modrinth_app_list_instances,
	retry: false,
})
const historyQuery = useQuery({
	queryKey: modrinthAppImportKeys.history,
	queryFn: modrinth_app_get_history,
})
const tabbyappInstancesQuery = useQuery(instanceListQueryOptions())

const modrinthInstances = computed(() => instancesQuery.data.value ?? [])
const history = computed(() => historyQuery.data.value ?? { imports: {}, playtime_transfers: {} })

const selectedIds = ref<string[]>([])
const options = reactive<ModrinthAppImportOptions>({
	content: true,
	worlds: true,
	screenshots: true,
	settings: true,
	playtime: true,
})
const dataOptions: Array<{ key: keyof ModrinthAppImportOptions; label: keyof typeof messages }> = [
	{ key: 'content', label: 'dataContent' },
	{ key: 'worlds', label: 'dataWorlds' },
	{ key: 'screenshots', label: 'dataScreenshots' },
	{ key: 'settings', label: 'dataSettings' },
	{ key: 'playtime', label: 'dataPlaytime' },
]

const allSelected = computed(
	() =>
		modrinthInstances.value.length > 0 &&
		selectedIds.value.length === modrinthInstances.value.length,
)

function toggleAll(selected: boolean) {
	selectedIds.value = selected ? modrinthInstances.value.map((instance) => instance.id) : []
}

function toggleInstance(id: string, selected: boolean) {
	selectedIds.value = selected
		? [...selectedIds.value, id]
		: selectedIds.value.filter((selectedId) => selectedId !== id)
}

function formatPlaytime(seconds: number) {
	const hours = Math.floor(seconds / 3600)
	const minutes = Math.floor((seconds % 3600) / 60)
	return hours > 0
		? formatMessage(messages.hours, { hours, minutes })
		: formatMessage(messages.minutes, { minutes })
}

function instanceSummary(instance: ModrinthAppInstance) {
	const loader = instance.loader.charAt(0).toUpperCase() + instance.loader.slice(1)
	return [loader, instance.game_version, formatPlaytime(instance.playtime_seconds)]
		.filter(Boolean)
		.join(' · ')
}

function tabbyappInstanceName(id: string) {
	return (
		tabbyappInstancesQuery.data.value?.find((instance) => instance.id === id)?.name ??
		formatMessage(messages.unknownInstance)
	)
}

async function refreshAfterImport() {
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: modrinthAppImportKeys.history }),
		queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
	])
}

const importMutation = useMutation({
	mutationFn: () => modrinth_app_import_instances(selectedIds.value, { ...options }),
	onSuccess: async (jobs) => {
		addNotification({
			type: 'success',
			title: formatMessage(messages.importStarted, { count: jobs.length }),
			text: formatMessage(messages.importStartedText),
		})
		selectedIds.value = []
		await refreshAfterImport()
	},
	onError: (error) => handleError(error as Error),
})

const playtimeTargets = reactive<Record<string, string | undefined>>({})
const playtimeSources = computed(() =>
	modrinthInstances.value.filter((instance) => instance.playtime_seconds > 0),
)
const targetOptions = computed<ComboboxOption<string>[]>(() =>
	(tabbyappInstancesQuery.data.value ?? []).map((instance) => ({
		value: instance.id,
		label: instance.name,
	})),
)

const transferMutation = useMutation({
	mutationFn: ({ sourceId, targetId }: { sourceId: string; targetId: string }) =>
		modrinth_app_transfer_playtime(sourceId, targetId),
	onSuccess: async (seconds, { targetId }) => {
		addNotification({
			type: 'success',
			title: formatMessage(messages.playtimeTransferredNotification, {
				playtime: formatPlaytime(seconds),
				instance: tabbyappInstanceName(targetId),
			}),
		})
		await refreshAfterImport()
	},
	onError: (error) => handleError(error as Error),
})

function transferPlaytime(sourceId: string) {
	const targetId = playtimeTargets[sourceId]
	if (!targetId) return
	transferMutation.mutate({ sourceId, targetId })
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>

		<p v-if="instancesQuery.isPending.value" class="m-0 text-secondary">
			{{ formatMessage(messages.loading) }}
		</p>
		<Admonition v-else-if="instancesQuery.isError.value" type="info">
			{{ formatMessage(messages.notFound) }}
		</Admonition>

		<template v-else>
			<section class="flex flex-col gap-3">
				<div class="flex items-center justify-between gap-4">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.instancesTitle) }}
					</h2>
					<Checkbox
						v-if="modrinthInstances.length > 0"
						:model-value="allSelected"
						:indeterminate="selectedIds.length > 0 && !allSelected"
						:label="formatMessage(messages.selectAll)"
						@update:model-value="toggleAll"
					/>
				</div>
				<p v-if="modrinthInstances.length === 0" class="m-0 text-secondary">
					{{ formatMessage(messages.noInstances) }}
				</p>
				<div
					v-for="instance in modrinthInstances"
					:key="instance.id"
					class="flex items-center justify-between gap-4 rounded-xl border border-solid border-surface-4 bg-surface-2 px-4 py-3"
				>
					<Checkbox
						:model-value="selectedIds.includes(instance.id)"
						:label="instance.name"
						label-class="font-semibold text-contrast"
						@update:model-value="(selected) => toggleInstance(instance.id, selected)"
					/>
					<div class="flex shrink-0 items-center gap-2 text-sm text-secondary">
						<span
							v-if="history.imports[instance.id]"
							class="rounded-full bg-brand-highlight px-2 py-0.5 font-medium text-brand"
						>
							{{ formatMessage(messages.alreadyImported) }}
						</span>
						{{ instanceSummary(instance) }}
					</div>
				</div>
			</section>

			<section class="flex flex-col gap-3">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.dataTitle) }}
				</h2>
				<div class="grid grid-cols-2 gap-3">
					<Checkbox
						v-for="option in dataOptions"
						:key="option.key"
						v-model="options[option.key]"
						:label="formatMessage(messages[option.label])"
					/>
				</div>
				<div>
					<Button
						color="brand"
						:disabled="selectedIds.length === 0 || importMutation.isPending.value"
						@click="importMutation.mutate()"
					>
						<ImportIcon aria-hidden="true" />
						{{ formatMessage(messages.importButton, { count: selectedIds.length }) }}
					</Button>
				</div>
			</section>

			<section class="flex flex-col gap-3 border-0 border-t border-solid border-surface-4 pt-6">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.playtimeTitle) }}
				</h2>
				<p class="m-0 text-secondary">{{ formatMessage(messages.playtimeDescription) }}</p>
				<p v-if="playtimeSources.length === 0" class="m-0 text-secondary">
					{{ formatMessage(messages.noPlaytime) }}
				</p>
				<div
					v-for="instance in playtimeSources"
					:key="instance.id"
					class="flex items-center justify-between gap-4 rounded-xl border border-solid border-surface-4 bg-surface-2 px-4 py-3"
				>
					<div class="flex min-w-0 flex-col">
						<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
						<span class="text-sm text-secondary">
							{{ formatPlaytime(instance.playtime_seconds) }}
						</span>
					</div>
					<span
						v-if="history.playtime_transfers[instance.id]"
						class="shrink-0 text-sm font-medium text-brand"
					>
						{{
							formatMessage(messages.playtimeTransferred, {
								instance: tabbyappInstanceName(history.playtime_transfers[instance.id].instance_id),
							})
						}}
					</span>
					<div v-else class="flex shrink-0 items-center gap-2">
						<RightArrowIcon aria-hidden="true" class="text-secondary" />
						<Combobox
							v-model="playtimeTargets[instance.id]"
							class="w-56"
							:options="targetOptions"
							:placeholder="formatMessage(messages.playtimeTarget)"
							searchable
						/>
						<Button
							:disabled="!playtimeTargets[instance.id] || transferMutation.isPending.value"
							@click="transferPlaytime(instance.id)"
						>
							{{ formatMessage(messages.playtimeTransfer) }}
						</Button>
					</div>
				</div>
			</section>
		</template>
	</div>
</template>
