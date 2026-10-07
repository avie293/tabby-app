<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, EditIcon, PlusIcon, StarIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { formatNumber } from '@/composables/use-dashboard'

import VersionEditorModal from './VersionEditorModal.vue'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const formatDate = useFormatDateTime({ dateStyle: 'medium' })

const messages = defineMessages({
	create: { id: 'app.dashboard.versions.create', defaultMessage: 'Upload version' },
	edit: { id: 'app.dashboard.versions.edit', defaultMessage: 'Edit version' },
	empty: {
		id: 'app.dashboard.versions.empty',
		defaultMessage: 'Upload the first version of your project.',
	},
	delete: { id: 'app.dashboard.versions.delete', defaultMessage: 'Delete version' },
	deleteDescription: {
		id: 'app.dashboard.versions.delete-description',
		defaultMessage: 'This permanently deletes the version and its files.',
	},
})

const channelClasses: Record<string, string> = {
	release: 'bg-highlight-green text-green',
	beta: 'bg-highlight-orange text-orange',
	alpha: 'bg-highlight-red text-red',
}

const versionsKey = computed(() => ['dashboard', 'project-versions', props.project.id])
const versionsQuery = useQuery({
	queryKey: versionsKey,
	queryFn: () =>
		client.labrinth.versions_v3.getProjectVersions(props.project.id, { apiVersion: 3 }),
})
const versions = computed(() =>
	[...(versionsQuery.data.value ?? [])].sort(
		(a, b) => Date.parse(b.date_published) - Date.parse(a.date_published),
	),
)

async function refreshVersions() {
	await queryClient.invalidateQueries({ queryKey: versionsKey.value })
	await props.refresh()
}

const editor = ref<InstanceType<typeof VersionEditorModal>>()
const deleteModal = ref<InstanceType<typeof ConfirmModal>>()
const deleting = ref<string>()
const deleteMutation = useMutation({
	mutationFn: () => client.labrinth.versions_v3.deleteVersion(deleting.value!),
	onSettled: refreshVersions,
	onError: (error) => handleError(error as Error),
})

function confirmDelete(version: Labrinth.Versions.v3.Version) {
	deleting.value = version.id
	deleteModal.value?.show()
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<VersionEditorModal ref="editor" :project="project" @saved="refreshVersions" />
		<ConfirmModal
			ref="deleteModal"
			:title="formatMessage(messages.delete)"
			:description="formatMessage(messages.deleteDescription)"
			:proceed-label="formatMessage(messages.delete)"
			@proceed="deleteMutation.mutate()"
		/>
		<div>
			<Button color="brand" type="colored" @click="editor?.show()">
				<PlusIcon aria-hidden="true" />
				{{ formatMessage(messages.create) }}
			</Button>
		</div>
		<p v-if="!versionsQuery.isPending.value && versions.length === 0" class="m-0 text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<div
			v-for="version in versions"
			:key="version.id"
			class="flex items-center gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3"
		>
			<span
				class="w-16 shrink-0 rounded-full py-0.5 text-center text-sm font-semibold capitalize"
				:class="channelClasses[version.version_type]"
			>
				{{ version.version_type }}
			</span>
			<div class="flex min-w-0 flex-1 flex-col">
				<span class="flex items-center gap-1 truncate font-semibold text-contrast">
					<StarIcon
						v-if="version.featured"
						class="size-4 shrink-0 text-orange"
						fill="currentColor"
						aria-hidden="true"
					/>
					{{ version.name }}
					<span v-if="version.status !== 'listed'" class="text-sm font-normal text-secondary">
						· {{ version.status }}
					</span>
				</span>
				<span class="truncate text-sm text-secondary">
					{{ version.version_number }} ·
					{{
						(version.mrpack_loaders?.length ? version.mrpack_loaders : version.loaders).join(', ')
					}}
					· {{ version.game_versions.slice(0, 4).join(', ')
					}}{{ version.game_versions.length > 4 ? ' …' : '' }}
				</span>
			</div>
			<span class="flex w-20 items-center justify-end gap-1 text-secondary">
				<DownloadIcon class="size-4" aria-hidden="true" />
				{{ formatNumber(version.downloads) }}
			</span>
			<span class="w-28 text-right text-sm text-secondary">
				{{ formatDate(version.date_published) }}
			</span>
			<Button v-tooltip="formatMessage(messages.edit)" @click="editor?.show(version)">
				<EditIcon aria-hidden="true" />
			</Button>
			<Button
				v-tooltip="formatMessage(messages.delete)"
				type="outlined"
				color="red"
				@click="confirmDelete(version)"
			>
				<TrashIcon aria-hidden="true" />
			</Button>
		</div>
	</div>
</template>
