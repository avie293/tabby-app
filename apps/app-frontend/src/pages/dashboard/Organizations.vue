<script setup lang="ts">
import { PlusIcon, UsersIcon } from '@modrinth/assets'
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
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import { dashboardKeys, slugify, useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const router = useRouter()
const { userId } = useCurrentUser()

const messages = defineMessages({
	title: { id: 'app.dashboard.organizations.title', defaultMessage: 'Organizations' },
	create: { id: 'app.dashboard.organizations.create', defaultMessage: 'Create organization' },
	empty: {
		id: 'app.dashboard.organizations.empty',
		defaultMessage: "You aren't a member of any organization yet.",
	},
	members: {
		id: 'app.dashboard.organizations.members',
		defaultMessage: '{count, plural, one {# member} other {# members}}',
	},
	name: { id: 'app.dashboard.organizations.name', defaultMessage: 'Name' },
	slug: { id: 'app.dashboard.organizations.slug', defaultMessage: 'URL' },
	summary: { id: 'app.dashboard.organizations.summary', defaultMessage: 'Summary' },
})

const organizationsQuery = useQuery({
	queryKey: computed(() => dashboardKeys.organizations(userId.value ?? '')),
	queryFn: () => client.labrinth.users_v2.getOrganizations(userId.value!),
	enabled: () => !!userId.value,
})
const organizations = computed(() =>
	[...(organizationsQuery.data.value ?? [])].sort((a, b) => a.name.localeCompare(b.name)),
)

const createModal = ref<InstanceType<typeof NewModal>>()
const name = ref('')
const slug = ref('')
const summary = ref('')
const slugEdited = ref(false)
watch(name, (value) => {
	if (!slugEdited.value) slug.value = slugify(value)
})

const createMutation = useMutation({
	mutationFn: () =>
		client.labrinth.organizations_v3.create({
			name: name.value.trim(),
			slug: slug.value.trim(),
			description: summary.value.trim(),
		}),
	onSuccess: async (organization) => {
		createModal.value?.hide()
		name.value = ''
		slug.value = ''
		summary.value = ''
		slugEdited.value = false
		await queryClient.invalidateQueries({ queryKey: dashboardKeys.all })
		await router.push(`/dashboard/organization/${organization.id}`)
	},
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<NewModal ref="createModal" :header="formatMessage(messages.create)" max-width="32rem">
			<form
				class="flex flex-col gap-4"
				@submit.prevent="name.trim() && slug.trim() && summary.trim() && createMutation.mutate()"
			>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.name) }}</span>
					<Input v-model="name" :maxlength="64" />
				</label>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.slug) }}</span>
					<div class="flex items-center gap-1">
						<span class="text-secondary">modrinth.com/organization/</span>
						<Input v-model="slug" class="flex-1" :maxlength="64" @input="slugEdited = true" />
					</div>
				</label>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.summary) }}</span>
					<Textarea v-model="summary" :maxlength="256" />
				</label>
			</form>
			<template #actions>
				<div class="flex justify-end">
					<Button
						color="brand"
						type="colored"
						:disabled="
							!name.trim() || !slug.trim() || !summary.trim() || createMutation.isPending.value
						"
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
			v-if="!organizationsQuery.isPending.value && organizations.length === 0"
			class="m-0 text-secondary"
		>
			{{ formatMessage(messages.empty) }}
		</p>
		<div class="grid grid-cols-2 gap-4 xl:grid-cols-3">
			<RouterLink
				v-for="organization in organizations"
				:key="organization.id"
				:to="`/dashboard/organization/${organization.id}`"
				class="flex gap-3 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4 text-primary no-underline hover:bg-surface-3"
			>
				<Avatar :src="organization.icon_url" size="56px" />
				<div class="flex min-w-0 flex-col gap-1">
					<span class="truncate font-semibold text-contrast">{{ organization.name }}</span>
					<span class="flex items-center gap-1 text-sm text-secondary">
						<UsersIcon class="size-4" aria-hidden="true" />
						{{
							formatMessage(messages.members, {
								count: organization.members.filter((member) => member.accepted).length,
							})
						}}
					</span>
					<span class="line-clamp-2 text-sm">{{ organization.description }}</span>
				</div>
			</RouterLink>
		</div>
	</div>
</template>
