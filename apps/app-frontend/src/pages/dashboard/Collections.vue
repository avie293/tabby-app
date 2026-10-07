<script setup lang="ts">
import { LockIcon, PlusIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { dashboardKeys, useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const router = useRouter()
const { userId } = useCurrentUser()

const messages = defineMessages({
	title: { id: 'app.dashboard.collections.title', defaultMessage: 'Collections' },
	create: { id: 'app.dashboard.collections.create', defaultMessage: 'Create collection' },
	empty: {
		id: 'app.dashboard.collections.empty',
		defaultMessage: "You don't have any collections yet.",
	},
	projectCount: {
		id: 'app.dashboard.collections.project-count',
		defaultMessage: '{count, plural, one {# project} other {# projects}}',
	},
	name: { id: 'app.dashboard.collections.name', defaultMessage: 'Name' },
	description: { id: 'app.dashboard.collections.description', defaultMessage: 'Description' },
})

const collectionsQuery = useQuery({
	queryKey: computed(() => dashboardKeys.collections(userId.value ?? '')),
	queryFn: () => client.labrinth.users_v2.getCollections(userId.value!),
	enabled: () => !!userId.value,
})
const collections = computed(() =>
	[...(collectionsQuery.data.value ?? [])].sort((a, b) => a.name.localeCompare(b.name)),
)

const createModal = ref<InstanceType<typeof NewModal>>()
const name = ref('')
const description = ref('')

const createMutation = useMutation({
	mutationFn: () =>
		client.labrinth.collections.create({
			name: name.value.trim(),
			description: description.value.trim() || null,
			projects: [],
		}),
	onSuccess: async (collection) => {
		createModal.value?.hide()
		name.value = ''
		description.value = ''
		await queryClient.invalidateQueries({ queryKey: dashboardKeys.all })
		await router.push(`/dashboard/collection/${collection.id}`)
	},
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<NewModal ref="createModal" :header="formatMessage(messages.create)" max-width="32rem">
			<form class="flex flex-col gap-4" @submit.prevent="name.trim() && createMutation.mutate()">
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.name) }}</span>
					<Input v-model="name" :maxlength="64" />
				</label>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.description) }}</span>
					<Textarea v-model="description" :maxlength="255" />
				</label>
			</form>
			<template #actions>
				<div class="flex justify-end">
					<Button
						color="brand"
						type="colored"
						:disabled="!name.trim() || createMutation.isPending.value"
						@click="createMutation.mutate()"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.create) }}
					</Button>
				</div>
			</template>
		</NewModal>
		<div class="flex items-center justify-between gap-4">
			<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>
			<Button color="brand" type="colored" @click="createModal?.show()">
				<PlusIcon aria-hidden="true" />
				{{ formatMessage(messages.create) }}
			</Button>
		</div>
		<p
			v-if="!collectionsQuery.isPending.value && collections.length === 0"
			class="m-0 text-secondary"
		>
			{{ formatMessage(messages.empty) }}
		</p>
		<div class="grid grid-cols-2 gap-4 xl:grid-cols-3">
			<RouterLink
				v-for="collection in collections"
				:key="collection.id"
				:to="`/dashboard/collection/${collection.id}`"
				class="flex gap-3 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4 text-primary no-underline hover:bg-surface-3"
			>
				<Avatar :src="collection.icon_url" size="56px" />
				<div class="flex min-w-0 flex-col gap-1">
					<span class="flex items-center gap-1 truncate font-semibold text-contrast">
						<LockIcon
							v-if="collection.status === 'private'"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
						{{ collection.name }}
					</span>
					<span class="text-sm text-secondary">
						{{ formatMessage(messages.projectCount, { count: collection.projects.length }) }}
					</span>
					<span v-if="collection.description" class="line-clamp-2 text-sm">
						{{ collection.description }}
					</span>
				</div>
			</RouterLink>
		</div>
	</div>
</template>
