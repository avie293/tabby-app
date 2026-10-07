<script setup lang="ts">
import {
	CrownIcon,
	LeftArrowIcon,
	SaveIcon,
	TrashIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
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
import { dashboardKeys, imageExtension, useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()
const { userId } = useCurrentUser()

const messages = defineMessages({
	back: { id: 'app.dashboard.back-to-organizations', defaultMessage: 'Back to organizations' },
	name: { id: 'app.dashboard.organizations.name', defaultMessage: 'Name' },
	slug: { id: 'app.dashboard.organizations.slug', defaultMessage: 'URL' },
	summary: { id: 'app.dashboard.organizations.summary', defaultMessage: 'Summary' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
	members: { id: 'app.dashboard.organization.members', defaultMessage: 'Members' },
	invitePlaceholder: {
		id: 'app.dashboard.organization.invite-placeholder',
		defaultMessage: 'Modrinth username',
	},
	invite: { id: 'app.dashboard.organization.invite', defaultMessage: 'Invite' },
	invited: { id: 'app.dashboard.organization.invited', defaultMessage: 'Invited' },
	owner: { id: 'app.dashboard.organization.owner', defaultMessage: 'Owner' },
	role: { id: 'app.dashboard.organization.role', defaultMessage: 'Role' },
	saveRole: { id: 'app.dashboard.organization.save-role', defaultMessage: 'Save role' },
	removeMember: { id: 'app.dashboard.organization.remove-member', defaultMessage: 'Remove' },
	projects: { id: 'app.dashboard.organization.projects', defaultMessage: 'Projects' },
	delete: { id: 'app.dashboard.organization.delete', defaultMessage: 'Delete organization' },
	deleteDescription: {
		id: 'app.dashboard.organization.delete-description',
		defaultMessage:
			'This permanently deletes the organization. Its projects go back to their owners.',
	},
})

const id = computed(() => route.params.id as string)
const organizationQuery = useQuery({
	queryKey: computed(() => dashboardKeys.organization(id.value)),
	queryFn: () => client.labrinth.organizations_v3.get(id.value),
})
const organization = computed(() => organizationQuery.data.value)
const projectsQuery = useQuery({
	queryKey: computed(() => ['dashboard', 'organization-projects', id.value]),
	queryFn: () => client.labrinth.organizations_v3.getProjects(id.value),
})

const isOwner = computed(
	() =>
		!!organization.value?.members.some(
			(member) => member.is_owner && member.user.id === userId.value,
		),
)

const name = ref('')
const slug = ref('')
const summary = ref('')
const roles = ref<Record<string, string>>({})
watch(
	organization,
	(value) => {
		if (!value) return
		name.value = value.name
		slug.value = value.slug
		summary.value = value.description
		roles.value = Object.fromEntries(value.members.map((member) => [member.user.id, member.role]))
	},
	{ immediate: true },
)

async function refresh() {
	await queryClient.invalidateQueries({ queryKey: dashboardKeys.all })
}

const saveMutation = useMutation({
	mutationFn: () =>
		client.labrinth.organizations_v3.edit(id.value, {
			name: name.value.trim(),
			slug: slug.value.trim(),
			description: summary.value.trim(),
		}),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const iconMutation = useMutation({
	mutationFn: (file: File | null) =>
		file
			? client.labrinth.organizations_v3.editIcon(id.value, file, imageExtension(file))
			: client.labrinth.organizations_v3.deleteIcon(id.value),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const inviteName = ref('')
const inviteMutation = useMutation({
	mutationFn: async () => {
		const user = await client.labrinth.users_v2.get(inviteName.value.trim())
		await client.labrinth.teams_v2.addMember(organization.value!.team_id, { user_id: user.id })
	},
	onSuccess: () => {
		inviteName.value = ''
	},
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const roleMutation = useMutation({
	mutationFn: (memberId: string) =>
		client.labrinth.teams_v2.editMember(organization.value!.team_id, memberId, {
			role: roles.value[memberId]?.trim(),
		}),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const removeMemberMutation = useMutation({
	mutationFn: (memberId: string) =>
		client.labrinth.teams_v2.removeMember(organization.value!.team_id, memberId),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const deleteModal = ref<InstanceType<typeof ConfirmModal>>()
const deleteMutation = useMutation({
	mutationFn: () => client.labrinth.organizations_v3.delete(id.value),
	onSuccess: async () => {
		await refresh()
		await router.push('/dashboard/organizations')
	},
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div v-if="organization" class="flex flex-col gap-6">
		<ConfirmModal
			ref="deleteModal"
			:title="formatMessage(messages.delete)"
			:description="formatMessage(messages.deleteDescription)"
			:proceed-label="formatMessage(messages.delete)"
			@proceed="deleteMutation.mutate()"
		/>
		<RouterLink
			to="/dashboard/organizations"
			class="flex w-fit items-center gap-1 text-secondary no-underline hover:underline"
		>
			<LeftArrowIcon class="size-4" aria-hidden="true" />
			{{ formatMessage(messages.back) }}
		</RouterLink>

		<section
			class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
		>
			<IconEditor
				:src="organization.icon_url"
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
					<span class="text-secondary">modrinth.com/organization/</span>
					<Input v-model="slug" class="flex-1" :maxlength="64" />
				</div>
			</label>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.summary) }}</span>
				<Textarea v-model="summary" :maxlength="256" />
			</label>
			<div class="flex justify-between gap-2">
				<Button v-if="isOwner" type="outlined" color="red" @click="deleteModal?.show()">
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.delete) }}
				</Button>
				<span v-else />
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

		<section class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.members) }}</h2>
			<form class="flex gap-2" @submit.prevent="inviteName.trim() && inviteMutation.mutate()">
				<Input
					v-model="inviteName"
					class="flex-1"
					:placeholder="formatMessage(messages.invitePlaceholder)"
				/>
				<Button
					color="brand"
					type="colored"
					:disabled="!inviteName.trim() || inviteMutation.isPending.value"
					@click="inviteMutation.mutate()"
				>
					<UserPlusIcon aria-hidden="true" />
					{{ formatMessage(messages.invite) }}
				</Button>
			</form>
			<div
				v-for="member in organization.members"
				:key="member.user.id"
				class="flex items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3"
			>
				<Avatar :src="member.user.avatar_url" size="40px" circle />
				<div class="flex min-w-0 flex-1 flex-col">
					<span class="flex items-center gap-1 font-semibold text-contrast">
						{{ member.user.username }}
						<CrownIcon
							v-if="member.is_owner"
							v-tooltip="formatMessage(messages.owner)"
							class="size-4 text-orange"
							aria-hidden="true"
						/>
					</span>
					<span v-if="!member.accepted" class="text-sm text-secondary">
						{{ formatMessage(messages.invited) }}
					</span>
				</div>
				<Input
					v-model="roles[member.user.id]"
					class="w-48"
					:placeholder="formatMessage(messages.role)"
				/>
				<Button
					:disabled="roleMutation.isPending.value || roles[member.user.id] === member.role"
					@click="roleMutation.mutate(member.user.id)"
				>
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(messages.saveRole) }}
				</Button>
				<Button
					v-if="!member.is_owner"
					:disabled="removeMemberMutation.isPending.value"
					@click="removeMemberMutation.mutate(member.user.id)"
				>
					<XIcon aria-hidden="true" />
					{{ formatMessage(messages.removeMember) }}
				</Button>
			</div>
		</section>

		<section class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.projects) }}
			</h2>
			<RouterLink
				v-for="project in projectsQuery.data.value ?? []"
				:key="project.id"
				:to="`/dashboard/project/${project.id}`"
				class="flex items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3 text-primary no-underline hover:bg-surface-3"
			>
				<Avatar :src="project.icon_url" size="40px" />
				<span class="truncate font-semibold text-contrast">{{ project.name }}</span>
			</RouterLink>
		</section>
	</div>
</template>
