<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CrownIcon, SaveIcon, TransferIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	Checkbox,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { useCurrentUser } from '@/composables/use-dashboard'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { userId } = useCurrentUser()

const messages = defineMessages({
	invitePlaceholder: {
		id: 'app.dashboard.organization.invite-placeholder',
		defaultMessage: 'Modrinth username',
	},
	invite: { id: 'app.dashboard.organization.invite', defaultMessage: 'Invite' },
	invited: { id: 'app.dashboard.organization.invited', defaultMessage: 'Invited' },
	owner: { id: 'app.dashboard.organization.owner', defaultMessage: 'Owner' },
	role: { id: 'app.dashboard.organization.role', defaultMessage: 'Role' },
	payoutSplit: {
		id: 'app.dashboard.project.members.payout-split',
		defaultMessage: 'Revenue share',
	},
	permissions: { id: 'app.dashboard.project.members.permissions', defaultMessage: 'Permissions' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
	remove: { id: 'app.dashboard.organization.remove-member', defaultMessage: 'Remove' },
	transfer: { id: 'app.dashboard.project.members.transfer', defaultMessage: 'Make owner' },
	transferDescription: {
		id: 'app.dashboard.project.members.transfer-description',
		defaultMessage:
			'You will no longer be the owner of this project. This cannot be undone by you.',
	},
	organizationNote: {
		id: 'app.dashboard.project.members.organization-note',
		defaultMessage: 'This project belongs to an organization. Its members can also manage it.',
	},
	uploadVersion: {
		id: 'app.dashboard.permission.upload-version',
		defaultMessage: 'Upload versions',
	},
	deleteVersion: {
		id: 'app.dashboard.permission.delete-version',
		defaultMessage: 'Delete versions',
	},
	editDetails: { id: 'app.dashboard.permission.edit-details', defaultMessage: 'Edit details' },
	editBody: { id: 'app.dashboard.permission.edit-body', defaultMessage: 'Edit description' },
	manageInvites: {
		id: 'app.dashboard.permission.manage-invites',
		defaultMessage: 'Manage invites',
	},
	removeMember: { id: 'app.dashboard.permission.remove-member', defaultMessage: 'Remove members' },
	editMember: { id: 'app.dashboard.permission.edit-member', defaultMessage: 'Edit members' },
	deleteProject: {
		id: 'app.dashboard.permission.delete-project',
		defaultMessage: 'Delete project',
	},
	viewAnalytics: {
		id: 'app.dashboard.permission.view-analytics',
		defaultMessage: 'View analytics',
	},
	viewPayouts: { id: 'app.dashboard.permission.view-payouts', defaultMessage: 'View revenue' },
})

/** Project permission bits, in the order Modrinth defines them. */
const PERMISSIONS = [
	'uploadVersion',
	'deleteVersion',
	'editDetails',
	'editBody',
	'manageInvites',
	'removeMember',
	'editMember',
	'deleteProject',
	'viewAnalytics',
	'viewPayouts',
] as const

const membersKey = computed(() => ['dashboard', 'project-members', props.project.id])
const membersQuery = useQuery({
	queryKey: membersKey,
	queryFn: () => client.labrinth.projects_v3.getMembers(props.project.id),
})
const members = computed(() =>
	[...(membersQuery.data.value ?? [])].sort((a, b) => a.ordering - b.ordering),
)
const isOwner = computed(() =>
	members.value.some((member) => member.is_owner && member.user.id === userId.value),
)

type Draft = { role: string; permissions: number; payoutsSplit: number }
const drafts = ref<Record<string, Draft>>({})
watch(
	members,
	(value) => {
		drafts.value = Object.fromEntries(
			value.map((member) => [
				member.user.id,
				{
					role: member.role,
					permissions: member.permissions ?? 0,
					payoutsSplit: member.payouts_split ?? 0,
				},
			]),
		)
	},
	{ immediate: true },
)

function hasPermission(userIdValue: string, bit: number) {
	return ((drafts.value[userIdValue]?.permissions ?? 0) & (1 << bit)) !== 0
}

function togglePermission(userIdValue: string, bit: number) {
	const draft = drafts.value[userIdValue]
	if (draft) draft.permissions ^= 1 << bit
}

async function refreshMembers() {
	await queryClient.invalidateQueries({ queryKey: membersKey.value })
	await props.refresh()
}

const inviteName = ref('')
const inviteMutation = useMutation({
	mutationFn: async () => {
		const user = await client.labrinth.users_v2.get(inviteName.value.trim())
		await client.labrinth.teams_v2.addMember(props.project.team_id, { user_id: user.id })
	},
	onSuccess: () => {
		inviteName.value = ''
	},
	onSettled: refreshMembers,
	onError: (error) => handleError(error as Error),
})

const saveMutation = useMutation({
	mutationFn: (member: Labrinth.Projects.v3.TeamMember) => {
		const draft = drafts.value[member.user.id]
		return client.labrinth.teams_v2.editMember(props.project.team_id, member.user.id, {
			role: draft.role.trim(),
			payouts_split: draft.payoutsSplit,
			...(member.is_owner ? {} : { permissions: draft.permissions }),
		})
	},
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: refreshMembers,
	onError: (error) => handleError(error as Error),
})

const removeMutation = useMutation({
	mutationFn: (memberId: string) =>
		client.labrinth.teams_v2.removeMember(props.project.team_id, memberId),
	onSettled: refreshMembers,
	onError: (error) => handleError(error as Error),
})

const transferModal = ref<InstanceType<typeof ConfirmModal>>()
const transferTarget = ref<string>()
const transferMutation = useMutation({
	mutationFn: () =>
		client.labrinth.teams_v2.transferOwnership(props.project.team_id, {
			user_id: transferTarget.value!,
		}),
	onSettled: refreshMembers,
	onError: (error) => handleError(error as Error),
})

function confirmTransfer(memberId: string) {
	transferTarget.value = memberId
	transferModal.value?.show()
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<ConfirmModal
			ref="transferModal"
			:title="formatMessage(messages.transfer)"
			:description="formatMessage(messages.transferDescription)"
			:proceed-label="formatMessage(messages.transfer)"
			@proceed="transferMutation.mutate()"
		/>
		<p v-if="project.organization" class="m-0 text-secondary">
			{{ formatMessage(messages.organizationNote) }}
		</p>
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
		<section
			v-for="member in members"
			:key="member.user.id"
			class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
		>
			<div class="flex items-center gap-3">
				<Avatar :src="member.user.avatar_url" size="48px" circle />
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
				<Button
					v-if="isOwner && !member.is_owner && member.accepted"
					@click="confirmTransfer(member.user.id)"
				>
					<TransferIcon aria-hidden="true" />
					{{ formatMessage(messages.transfer) }}
				</Button>
				<Button
					v-if="!member.is_owner"
					type="outlined"
					color="red"
					:disabled="removeMutation.isPending.value"
					@click="removeMutation.mutate(member.user.id)"
				>
					<XIcon aria-hidden="true" />
					{{ formatMessage(messages.remove) }}
				</Button>
			</div>
			<div v-if="drafts[member.user.id]" class="grid grid-cols-2 gap-4">
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.role) }}</span>
					<Input v-model="drafts[member.user.id].role" :maxlength="64" />
				</label>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.payoutSplit) }}</span>
					<Input
						:model-value="String(drafts[member.user.id].payoutsSplit)"
						type="number"
						@update:model-value="
							drafts[member.user.id].payoutsSplit = Math.max(0, Number($event) || 0)
						"
					/>
				</label>
			</div>
			<div v-if="!member.is_owner" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.permissions) }}</span>
				<div class="grid grid-cols-2 gap-1 lg:grid-cols-3">
					<Checkbox
						v-for="(permission, bit) in PERMISSIONS"
						:key="permission"
						:model-value="hasPermission(member.user.id, bit)"
						:label="formatMessage(messages[permission])"
						@update:model-value="togglePermission(member.user.id, bit)"
					/>
				</div>
			</div>
			<div class="flex justify-end">
				<Button
					color="brand"
					type="colored"
					:disabled="saveMutation.isPending.value"
					@click="saveMutation.mutate(member)"
				>
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(messages.save) }}
				</Button>
			</div>
		</section>
	</div>
</template>
