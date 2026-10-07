<script setup lang="ts">
import { DownloadIcon, HeartIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	injectModrinthClient,
	Input,
	ProjectStatusBadge,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import CreateProjectModal from '@/components/ui/dashboard/CreateProjectModal.vue'
import { dashboardKeys, formatNumber, useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { userId } = useCurrentUser()

const messages = defineMessages({
	title: { id: 'app.dashboard.projects.title', defaultMessage: 'Projects' },
	search: { id: 'app.dashboard.projects.search', defaultMessage: 'Search projects' },
	empty: {
		id: 'app.dashboard.projects.empty',
		defaultMessage: "You haven't published any projects yet.",
	},
	name: { id: 'app.dashboard.projects.column.name', defaultMessage: 'Name' },
	type: { id: 'app.dashboard.projects.column.type', defaultMessage: 'Type' },
	status: { id: 'app.dashboard.projects.column.status', defaultMessage: 'Status' },
	create: { id: 'app.dashboard.projects.create', defaultMessage: 'Create project' },
})

const createModal = ref<InstanceType<typeof CreateProjectModal>>()

const search = ref('')
const projectsQuery = useQuery({
	queryKey: computed(() => dashboardKeys.projects(userId.value ?? '')),
	queryFn: () => client.labrinth.users_v3.getProjects(userId.value!),
	enabled: () => !!userId.value,
})
const projects = computed(() =>
	[...(projectsQuery.data.value ?? [])]
		.filter((project) => project.name.toLowerCase().includes(search.value.trim().toLowerCase()))
		.sort((a, b) => a.name.localeCompare(b.name)),
)
</script>

<template>
	<div class="flex flex-col gap-4">
		<CreateProjectModal ref="createModal" />
		<div class="flex items-center justify-between gap-4">
			<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>
			<div class="flex items-center gap-2">
				<Input
					v-model="search"
					:icon="SearchIcon"
					class="w-64"
					:placeholder="formatMessage(messages.search)"
				/>
				<Button color="brand" type="colored" @click="createModal?.show()">
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.create) }}
				</Button>
			</div>
		</div>
		<p v-if="!projectsQuery.isPending.value && projects.length === 0" class="m-0 text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-4">
			<div
				class="grid grid-cols-[1fr_8rem_9rem_6rem_6rem] items-center gap-4 bg-surface-3 px-4 py-2 text-sm font-semibold text-secondary"
			>
				<span>{{ formatMessage(messages.name) }}</span>
				<span>{{ formatMessage(messages.type) }}</span>
				<span>{{ formatMessage(messages.status) }}</span>
				<DownloadIcon class="ml-auto size-4" aria-hidden="true" />
				<HeartIcon class="ml-auto size-4" aria-hidden="true" />
			</div>
			<RouterLink
				v-for="project in projects"
				:key="project.id"
				:to="`/dashboard/project/${project.id}`"
				class="grid grid-cols-[1fr_8rem_9rem_6rem_6rem] items-center gap-4 border-0 border-t border-solid border-surface-4 bg-bg-raised px-4 py-3 text-primary no-underline hover:bg-surface-3"
			>
				<span class="flex min-w-0 items-center gap-3">
					<Avatar :src="project.icon_url" size="40px" />
					<span class="truncate font-semibold text-contrast">{{ project.name }}</span>
				</span>
				<span class="truncate capitalize">{{ project.project_types.join(', ') }}</span>
				<ProjectStatusBadge :status="project.status" />
				<span class="text-right">{{ formatNumber(project.downloads) }}</span>
				<span class="text-right">{{ formatNumber(project.followers) }}</span>
			</RouterLink>
		</div>
	</div>
</template>
