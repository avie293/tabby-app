<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { LeftArrowIcon, SaveIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
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
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import IconEditor from '@/components/ui/dashboard/IconEditor.vue'
import { dashboardKeys, imageExtension } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	back: { id: 'app.dashboard.back-to-collections', defaultMessage: 'Back to collections' },
	name: { id: 'app.dashboard.collections.name', defaultMessage: 'Name' },
	description: { id: 'app.dashboard.collections.description', defaultMessage: 'Description' },
	visibility: { id: 'app.dashboard.collection.visibility', defaultMessage: 'Visibility' },
	listed: { id: 'app.dashboard.collection.listed', defaultMessage: 'Public' },
	unlisted: { id: 'app.dashboard.collection.unlisted', defaultMessage: 'Unlisted' },
	private: { id: 'app.dashboard.collection.private', defaultMessage: 'Private' },
	projects: { id: 'app.dashboard.collection.projects', defaultMessage: 'Projects' },
	noProjects: {
		id: 'app.dashboard.collection.no-projects',
		defaultMessage: 'Save projects to this collection from their project page.',
	},
	removeProject: { id: 'app.dashboard.collection.remove-project', defaultMessage: 'Remove' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
	delete: { id: 'app.dashboard.collection.delete', defaultMessage: 'Delete collection' },
	deleteDescription: {
		id: 'app.dashboard.collection.delete-description',
		defaultMessage: 'This permanently deletes the collection. The projects in it are not affected.',
	},
})

type Visibility = 'listed' | 'unlisted' | 'private'
const visibilities: Visibility[] = ['listed', 'unlisted', 'private']

const id = computed(() => route.params.id as string)
const collectionQuery = useQuery({
	queryKey: computed(() => dashboardKeys.collection(id.value)),
	queryFn: () => client.labrinth.collections.get(id.value),
})
const collection = computed(() => collectionQuery.data.value)

const projectsQuery = useQuery({
	queryKey: computed(() => ['dashboard', 'collection-projects', collection.value?.projects ?? []]),
	queryFn: () =>
		collection.value!.projects.length > 0
			? client.labrinth.projects_v2.getMultiple(collection.value!.projects)
			: Promise.resolve([] as Labrinth.Projects.v2.Project[]),
	enabled: () => !!collection.value,
})

const name = ref('')
const description = ref('')
const visibility = ref<Visibility>('listed')
watch(
	collection,
	(value) => {
		if (!value) return
		name.value = value.name
		description.value = value.description ?? ''
		visibility.value = (visibilities as string[]).includes(value.status)
			? (value.status as Visibility)
			: 'private'
	},
	{ immediate: true },
)

async function refresh() {
	await queryClient.invalidateQueries({ queryKey: dashboardKeys.all })
	await queryClient.invalidateQueries({ queryKey: ['user-collections'] })
}

const saveMutation = useMutation({
	mutationFn: () =>
		client.labrinth.collections.edit(id.value, {
			name: name.value.trim(),
			description: description.value.trim() || null,
			status: visibility.value,
		}),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const removeProjectMutation = useMutation({
	mutationFn: (projectId: string) =>
		client.labrinth.collections.edit(id.value, {
			new_projects: collection.value!.projects.filter((project) => project !== projectId),
		}),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const iconMutation = useMutation({
	mutationFn: (file: File | null) =>
		file
			? client.labrinth.collections.editIcon(id.value, file, imageExtension(file))
			: client.labrinth.collections.deleteIcon(id.value),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const deleteModal = ref<InstanceType<typeof ConfirmModal>>()
const deleteMutation = useMutation({
	mutationFn: () => client.labrinth.collections.delete(id.value),
	onSuccess: async () => {
		await refresh()
		await router.push('/dashboard/collections')
	},
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div v-if="collection" class="flex flex-col gap-6">
		<ConfirmModal
			ref="deleteModal"
			:title="formatMessage(messages.delete)"
			:description="formatMessage(messages.deleteDescription)"
			:proceed-label="formatMessage(messages.delete)"
			@proceed="deleteMutation.mutate()"
		/>
		<RouterLink
			to="/dashboard/collections"
			class="flex w-fit items-center gap-1 text-secondary no-underline hover:underline"
		>
			<LeftArrowIcon class="size-4" aria-hidden="true" />
			{{ formatMessage(messages.back) }}
		</RouterLink>

		<section
			class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
		>
			<IconEditor
				:src="collection.icon_url"
				:disabled="iconMutation.isPending.value"
				@upload="iconMutation.mutate($event)"
				@remove="iconMutation.mutate(null)"
			/>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.name) }}</span>
				<Input v-model="name" :maxlength="64" />
			</label>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.description) }}</span>
				<Textarea v-model="description" :maxlength="255" />
			</label>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.visibility) }}</span>
				<Chips
					v-model="visibility"
					:items="visibilities"
					:format-label="(item: Visibility) => formatMessage(messages[item])"
					:capitalize="false"
				/>
			</div>
			<div class="flex justify-between gap-2">
				<Button type="outlined" color="red" @click="deleteModal?.show()">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.delete) }}
				</Button>
				<Button
					color="brand"
					type="colored"
					:disabled="!name.trim() || saveMutation.isPending.value"
					@click="saveMutation.mutate()"
				>
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(messages.save) }}
				</Button>
			</div>
		</section>

		<section class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.projects) }}
			</h2>
			<p v-if="collection.projects.length === 0" class="m-0 text-secondary">
				{{ formatMessage(messages.noProjects) }}
			</p>
			<div
				v-for="project in projectsQuery.data.value ?? []"
				:key="project.id"
				class="flex items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3"
			>
				<RouterLink
					:to="`/project/${project.id}`"
					class="flex min-w-0 flex-1 items-center gap-3 text-primary no-underline hover:underline"
				>
					<Avatar :src="project.icon_url" size="40px" />
					<span class="truncate font-semibold text-contrast">{{ project.title }}</span>
				</RouterLink>
				<Button
					:disabled="removeProjectMutation.isPending.value"
					@click="removeProjectMutation.mutate(project.id)"
				>
					<XIcon aria-hidden="true" />
					{{ formatMessage(messages.removeProject) }}
				</Button>
			</div>
		</section>
	</div>
</template>
