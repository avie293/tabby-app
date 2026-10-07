<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'
import {
	Button,
	Chips,
	Combobox,
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
const { user, userId } = useCurrentUser()

const messages = defineMessages({
	header: { id: 'app.dashboard.projects.create', defaultMessage: 'Create project' },
	name: { id: 'app.dashboard.project.name', defaultMessage: 'Name' },
	slug: { id: 'app.dashboard.project.slug', defaultMessage: 'URL' },
	summary: { id: 'app.dashboard.project.summary', defaultMessage: 'Summary' },
	owner: { id: 'app.dashboard.projects.owner', defaultMessage: 'Owner' },
	visibility: { id: 'app.dashboard.project.visibility', defaultMessage: 'Visibility' },
	approved: { id: 'app.dashboard.project.visibility.public', defaultMessage: 'Public' },
	unlisted: { id: 'app.dashboard.project.visibility.unlisted', defaultMessage: 'Unlisted' },
	private: { id: 'app.dashboard.project.visibility.private', defaultMessage: 'Private' },
	hint: {
		id: 'app.dashboard.projects.create-hint',
		defaultMessage:
			'The project starts as a draft. Its type follows from the files of its first version.',
	},
})

type Visibility = 'approved' | 'unlisted' | 'private'
const visibilities: Visibility[] = ['approved', 'unlisted', 'private']

const modal = ref<InstanceType<typeof NewModal>>()
const name = ref('')
const slug = ref('')
const summary = ref('')
const owner = ref('self')
const visibility = ref<Visibility>('approved')
const slugEdited = ref(false)
watch(name, (value) => {
	if (!slugEdited.value) slug.value = slugify(value)
})

const organizationsQuery = useQuery({
	queryKey: computed(() => dashboardKeys.organizations(userId.value ?? '')),
	queryFn: () => client.labrinth.users_v2.getOrganizations(userId.value!),
	enabled: () => !!userId.value,
})
const ownerOptions = computed(() => [
	{ value: 'self', label: user.value?.username ?? '' },
	...(organizationsQuery.data.value ?? []).map((organization) => ({
		value: organization.id,
		label: organization.name,
	})),
])

const createMutation = useMutation({
	mutationFn: () =>
		client.labrinth.projects_v2.create({
			title: name.value.trim(),
			project_type: 'mod',
			slug: slug.value.trim(),
			description: summary.value.trim(),
			body: '',
			requested_status: visibility.value,
			initial_versions: [],
			team_members: [{ user_id: userId.value, name: user.value?.username, role: 'Owner' }],
			categories: [],
			client_side: 'required',
			server_side: 'required',
			license_id: 'LicenseRef-Unknown',
			is_draft: true,
			organization_id: owner.value !== 'self' ? owner.value : undefined,
		}).promise,
	onSuccess: async (project) => {
		modal.value?.hide()
		await queryClient.invalidateQueries({ queryKey: dashboardKeys.all })
		await router.push(`/dashboard/project/${project.id}`)
	},
	onError: (error) => handleError(error as Error),
})

function show() {
	name.value = ''
	slug.value = ''
	summary.value = ''
	owner.value = 'self'
	visibility.value = 'approved'
	slugEdited.value = false
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="34rem">
		<div class="flex flex-col gap-4">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.name) }}</span>
				<Input v-model="name" :maxlength="64" />
			</label>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.slug) }}</span>
				<div class="flex items-center gap-1">
					<span class="text-secondary">modrinth.com/project/</span>
					<Input v-model="slug" class="flex-1" :maxlength="64" @input="slugEdited = true" />
				</div>
			</label>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.summary) }}</span>
				<Textarea v-model="summary" :maxlength="256" />
			</label>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.owner) }}</span>
				<Combobox v-model="owner" class="max-w-xs" :options="ownerOptions" />
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.visibility) }}</span>
				<Chips
					v-model="visibility"
					:items="visibilities"
					:format-label="(item: Visibility) => formatMessage(messages[item])"
					:capitalize="false"
				/>
			</div>
			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.hint) }}</p>
		</div>
		<template #actions>
			<div class="flex justify-end">
				<Button
					color="brand"
					type="colored"
					:disabled="
						name.trim().length < 3 ||
						slug.trim().length < 3 ||
						summary.trim().length < 3 ||
						createMutation.isPending.value
					"
					@click="createMutation.mutate()"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.header) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
