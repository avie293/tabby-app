<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

const props = defineProps<{ userId: string | null }>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const messages = defineMessages({
	header: { id: 'app.project.save.header', defaultMessage: 'Save to collection' },
	empty: {
		id: 'app.project.save.empty',
		defaultMessage: "You don't have any collections yet. Create one below.",
	},
	projectCount: {
		id: 'app.project.save.project-count',
		defaultMessage: '{count, plural, one {# project} other {# projects}}',
	},
	newCollection: { id: 'app.project.save.new-collection', defaultMessage: 'New collection' },
	namePlaceholder: { id: 'app.project.save.name-placeholder', defaultMessage: 'Collection name' },
	create: { id: 'app.project.save.create', defaultMessage: 'Create' },
})

const modal = ref<InstanceType<typeof NewModal>>()
const projectId = ref<string>()
const newName = ref('')

const collectionsKey = computed(() => ['user-collections', props.userId] as const)
const collectionsQuery = useQuery({
	queryKey: collectionsKey,
	queryFn: () => client.labrinth.users_v2.getCollections(props.userId!),
	enabled: () => !!props.userId && !!projectId.value,
})
const collections = computed(() => collectionsQuery.data.value ?? [])

function contains(collection: Labrinth.Collections.Collection) {
	return !!projectId.value && collection.projects.includes(projectId.value)
}

const toggleMutation = useMutation({
	mutationFn: async (collection: Labrinth.Collections.Collection) => {
		const id = projectId.value!
		const projects = contains(collection)
			? collection.projects.filter((project) => project !== id)
			: [...collection.projects, id]
		await client.labrinth.collections.edit(collection.id, { new_projects: projects })
	},
	onSettled: () => queryClient.invalidateQueries({ queryKey: collectionsKey.value }),
	onError: (error) => handleError(error as Error),
})

const createMutation = useMutation({
	mutationFn: () =>
		client.labrinth.collections.create({
			name: newName.value.trim(),
			projects: [projectId.value!],
		}),
	onSuccess: () => {
		newName.value = ''
	},
	onSettled: () => queryClient.invalidateQueries({ queryKey: collectionsKey.value }),
	onError: (error) => handleError(error as Error),
})

function show(id: string) {
	projectId.value = id
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="30rem">
		<div class="flex flex-col gap-4">
			<div v-if="collectionsQuery.isPending.value" class="flex justify-center py-4">
				<SpinnerIcon class="size-6 animate-spin" aria-hidden="true" />
			</div>
			<p v-else-if="collections.length === 0" class="m-0 text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
			<div v-else class="flex max-h-80 flex-col gap-1 overflow-y-auto">
				<div
					v-for="collection in collections"
					:key="collection.id"
					class="flex items-center gap-3 rounded-xl px-2 py-2 hover:bg-surface-3"
				>
					<Checkbox
						:model-value="contains(collection)"
						:label="collection.name"
						:disabled="toggleMutation.isPending.value"
						class="flex-1"
						@update:model-value="toggleMutation.mutate(collection)"
					/>
					<span class="shrink-0 text-sm text-secondary">
						{{ formatMessage(messages.projectCount, { count: collection.projects.length }) }}
					</span>
				</div>
			</div>
			<form
				class="flex flex-col gap-2 border-0 border-t border-solid border-surface-4 pt-4"
				@submit.prevent="newName.trim() && createMutation.mutate()"
			>
				<span class="font-semibold text-contrast">{{ formatMessage(messages.newCollection) }}</span>
				<div class="flex gap-2">
					<Input
						v-model="newName"
						class="flex-1"
						:placeholder="formatMessage(messages.namePlaceholder)"
					/>
					<Button
						type="colored"
						color="brand"
						:disabled="!newName.trim() || createMutation.isPending.value"
						@click="createMutation.mutate()"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.create) }}
					</Button>
				</div>
			</form>
		</div>
	</NewModal>
</template>
