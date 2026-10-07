<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SaveIcon, SendIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	Chips,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import IconEditor from '@/components/ui/dashboard/IconEditor.vue'
import { imageExtension } from '@/composables/use-dashboard'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const router = useRouter()

const messages = defineMessages({
	name: { id: 'app.dashboard.project.name', defaultMessage: 'Name' },
	slug: { id: 'app.dashboard.project.slug', defaultMessage: 'URL' },
	summary: { id: 'app.dashboard.project.summary', defaultMessage: 'Summary' },
	visibility: { id: 'app.dashboard.project.visibility', defaultMessage: 'Visibility' },
	visibilityPending: {
		id: 'app.dashboard.project.visibility-pending',
		defaultMessage: 'The project gets this visibility once it has been approved.',
	},
	approved: { id: 'app.dashboard.project.visibility.public', defaultMessage: 'Public' },
	unlisted: { id: 'app.dashboard.project.visibility.unlisted', defaultMessage: 'Unlisted' },
	private: { id: 'app.dashboard.project.visibility.private', defaultMessage: 'Private' },
	archived: { id: 'app.dashboard.project.visibility.archived', defaultMessage: 'Archived' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
	submit: { id: 'app.dashboard.project.submit', defaultMessage: 'Submit for review' },
	submitDescription: {
		id: 'app.dashboard.project.submit-description',
		defaultMessage:
			'Your project is a draft. Add a description, a license and at least one version, then submit it so Modrinth can review it.',
	},
	submitted: { id: 'app.dashboard.project.submitted', defaultMessage: 'Submitted for review' },
	delete: { id: 'app.dashboard.project.delete', defaultMessage: 'Delete project' },
	deleteDescription: {
		id: 'app.dashboard.project.delete-description',
		defaultMessage:
			'This permanently deletes the project with all of its versions, files and statistics.',
	},
})

type Visibility = 'approved' | 'unlisted' | 'private' | 'archived'
const visibilities: Visibility[] = ['approved', 'unlisted', 'private', 'archived']

/** Projects that went through review can change their status directly. */
const reviewed = computed(() => (visibilities as string[]).includes(props.project.status as string))

const name = ref('')
const slug = ref('')
const summary = ref('')
const visibility = ref<Visibility>('approved')
watch(
	() => props.project,
	(project) => {
		name.value = project.name
		slug.value = project.slug ?? ''
		summary.value = project.summary
		const current = reviewed.value ? project.status : project.requested_status
		visibility.value = (visibilities as string[]).includes(current as string)
			? (current as Visibility)
			: 'approved'
	},
	{ immediate: true },
)

const saveMutation = useMutation({
	mutationFn: () =>
		client.labrinth.projects_v3.edit(props.project.id, {
			name: name.value.trim(),
			slug: slug.value.trim(),
			summary: summary.value.trim(),
			...(reviewed.value ? { status: visibility.value } : { requested_status: visibility.value }),
		}),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})

const submitMutation = useMutation({
	mutationFn: () => client.labrinth.projects_v3.edit(props.project.id, { status: 'processing' }),
	onSuccess: () => addNotification({ title: formatMessage(messages.submitted), type: 'success' }),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})

const iconMutation = useMutation({
	mutationFn: (file: File | null) =>
		file
			? client.labrinth.projects_v3.changeIcon(props.project.id, file, imageExtension(file))
			: client.labrinth.projects_v3.deleteIcon(props.project.id),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})

const deleteModal = ref<InstanceType<typeof ConfirmModal>>()
const deleteMutation = useMutation({
	mutationFn: () => client.labrinth.projects_v3.deleteProject(props.project.id),
	onSuccess: async () => {
		await props.refresh()
		await router.push('/dashboard/projects')
	},
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<ConfirmModal
			ref="deleteModal"
			:title="formatMessage(messages.delete)"
			:description="formatMessage(messages.deleteDescription)"
			:proceed-label="formatMessage(messages.delete)"
			@proceed="deleteMutation.mutate()"
		/>
		<section
			v-if="project.status === 'draft' || project.status === 'rejected'"
			class="flex items-center justify-between gap-4 rounded-2xl border border-solid border-brand bg-bg-raised p-4"
		>
			<p class="m-0 text-primary">{{ formatMessage(messages.submitDescription) }}</p>
			<Button
				color="brand"
				type="colored"
				:disabled="submitMutation.isPending.value"
				@click="submitMutation.mutate()"
			>
				<SendIcon aria-hidden="true" />
				{{ formatMessage(messages.submit) }}
			</Button>
		</section>
		<section
			class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
		>
			<IconEditor
				:src="project.icon_url"
				:disabled="iconMutation.isPending.value"
				@upload="iconMutation.mutate($event)"
				@remove="iconMutation.mutate(null)"
			/>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.name) }}</span>
				<Input v-model="name" :maxlength="64" />
			</label>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.slug) }}</span>
				<div class="flex items-center gap-1">
					<span class="text-secondary">modrinth.com/project/</span>
					<Input v-model="slug" class="flex-1" :maxlength="64" />
				</div>
			</label>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.summary) }}</span>
				<Textarea v-model="summary" :maxlength="256" />
			</label>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.visibility) }}</span>
				<Chips
					v-model="visibility"
					:items="visibilities"
					:format-label="(item: Visibility) => formatMessage(messages[item])"
					:capitalize="false"
				/>
				<span v-if="!reviewed" class="text-sm text-secondary">
					{{ formatMessage(messages.visibilityPending) }}
				</span>
			</div>
			<div class="flex justify-between gap-2">
				<Button type="outlined" color="red" @click="deleteModal?.show()">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.delete) }}
				</Button>
				<Button
					color="brand"
					type="colored"
					:disabled="!name.trim() || !slug.trim() || saveMutation.isPending.value"
					@click="saveMutation.mutate()"
				>
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(messages.save) }}
				</Button>
			</div>
		</section>
	</div>
</template>
