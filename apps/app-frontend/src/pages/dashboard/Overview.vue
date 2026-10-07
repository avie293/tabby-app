<script setup lang="ts">
import { DownloadIcon, HeartIcon, ListIcon } from '@modrinth/assets'
import {
	Avatar,
	defineMessages,
	injectModrinthClient,
	ProjectStatusBadge,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { dashboardKeys, formatNumber, useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { user, userId } = useCurrentUser()

const messages = defineMessages({
	welcome: { id: 'app.dashboard.overview.welcome', defaultMessage: 'Welcome back, {name}' },
	projects: { id: 'app.dashboard.overview.projects', defaultMessage: 'Projects' },
	downloads: { id: 'app.dashboard.overview.downloads', defaultMessage: 'Downloads' },
	followers: { id: 'app.dashboard.overview.followers', defaultMessage: 'Followers' },
	recent: { id: 'app.dashboard.overview.recent', defaultMessage: 'Recently updated projects' },
	noProjects: {
		id: 'app.dashboard.overview.no-projects',
		defaultMessage: "You haven't published any projects yet.",
	},
})

const projectsQuery = useQuery({
	queryKey: computed(() => dashboardKeys.projects(userId.value ?? '')),
	queryFn: () => client.labrinth.users_v3.getProjects(userId.value!),
	enabled: () => !!userId.value,
})
const projects = computed(() => projectsQuery.data.value ?? [])

const stats = computed(() => [
	{ label: formatMessage(messages.projects), value: projects.value.length, icon: ListIcon },
	{
		label: formatMessage(messages.downloads),
		value: projects.value.reduce((total, project) => total + project.downloads, 0),
		icon: DownloadIcon,
	},
	{
		label: formatMessage(messages.followers),
		value: projects.value.reduce((total, project) => total + project.followers, 0),
		icon: HeartIcon,
	},
])

const recent = computed(() =>
	[...projects.value].sort((a, b) => Date.parse(b.updated) - Date.parse(a.updated)).slice(0, 5),
)
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex items-center gap-4">
			<Avatar :src="user?.avatar_url" size="64px" circle />
			<h1 class="m-0 text-2xl font-bold text-contrast">
				{{ formatMessage(messages.welcome, { name: user?.username ?? '' }) }}
			</h1>
		</div>
		<div class="grid grid-cols-3 gap-4">
			<div
				v-for="stat in stats"
				:key="stat.label"
				class="flex flex-col gap-2 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
			>
				<span class="flex items-center gap-2 text-secondary">
					<component :is="stat.icon" class="size-5" aria-hidden="true" />
					{{ stat.label }}
				</span>
				<span class="text-3xl font-bold text-contrast">{{ formatNumber(stat.value) }}</span>
			</div>
		</div>
		<div class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.recent) }}</h2>
			<p v-if="!projectsQuery.isPending.value && recent.length === 0" class="m-0 text-secondary">
				{{ formatMessage(messages.noProjects) }}
			</p>
			<RouterLink
				v-for="project in recent"
				:key="project.id"
				:to="`/project/${project.id}`"
				class="flex items-center gap-3 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3 text-primary no-underline hover:bg-surface-3"
			>
				<Avatar :src="project.icon_url" size="40px" />
				<span class="flex-1 truncate font-semibold text-contrast">{{ project.name }}</span>
				<ProjectStatusBadge :status="project.status" />
				<span class="flex w-24 items-center justify-end gap-1 text-secondary">
					<DownloadIcon class="size-4" aria-hidden="true" />
					{{ formatNumber(project.downloads) }}
				</span>
			</RouterLink>
		</div>
	</div>
</template>
