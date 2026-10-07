<script setup lang="ts">
import { ExternalIcon, LeftArrowIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	injectModrinthClient,
	ProjectStatusBadge,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { type Component, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DescriptionSettings from '@/components/ui/dashboard/project/DescriptionSettings.vue'
import GallerySettings from '@/components/ui/dashboard/project/GallerySettings.vue'
import GeneralSettings from '@/components/ui/dashboard/project/GeneralSettings.vue'
import LicenseSettings from '@/components/ui/dashboard/project/LicenseSettings.vue'
import LinksSettings from '@/components/ui/dashboard/project/LinksSettings.vue'
import MembersSettings from '@/components/ui/dashboard/project/MembersSettings.vue'
import TagsSettings from '@/components/ui/dashboard/project/TagsSettings.vue'
import VersionsSettings from '@/components/ui/dashboard/project/VersionsSettings.vue'
import { dashboardKeys } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	back: { id: 'app.dashboard.back-to-projects', defaultMessage: 'Back to projects' },
	view: { id: 'app.dashboard.project.view', defaultMessage: 'View project' },
	general: { id: 'app.dashboard.project.tab.general', defaultMessage: 'General' },
	description: { id: 'app.dashboard.project.tab.description', defaultMessage: 'Description' },
	versions: { id: 'app.dashboard.project.tab.versions', defaultMessage: 'Versions' },
	gallery: { id: 'app.dashboard.project.tab.gallery', defaultMessage: 'Gallery' },
	tags: { id: 'app.dashboard.project.tab.tags', defaultMessage: 'Tags' },
	links: { id: 'app.dashboard.project.tab.links', defaultMessage: 'Links' },
	license: { id: 'app.dashboard.project.tab.license', defaultMessage: 'License' },
	members: { id: 'app.dashboard.project.tab.members', defaultMessage: 'Members' },
})

const tabs: { id: string; label: string; component: Component }[] = [
	{ id: 'general', label: formatMessage(messages.general), component: GeneralSettings },
	{ id: 'description', label: formatMessage(messages.description), component: DescriptionSettings },
	{ id: 'versions', label: formatMessage(messages.versions), component: VersionsSettings },
	{ id: 'gallery', label: formatMessage(messages.gallery), component: GallerySettings },
	{ id: 'tags', label: formatMessage(messages.tags), component: TagsSettings },
	{ id: 'links', label: formatMessage(messages.links), component: LinksSettings },
	{ id: 'license', label: formatMessage(messages.license), component: LicenseSettings },
	{ id: 'members', label: formatMessage(messages.members), component: MembersSettings },
]

const id = computed(() => route.params.id as string)
const activeTab = computed(() => tabs.find((tab) => tab.id === route.query.tab) ?? tabs[0])

const projectQuery = useQuery({
	queryKey: computed(() => ['dashboard', 'project', id.value]),
	queryFn: () => client.labrinth.projects_v3.get(id.value),
})
const project = computed(() => projectQuery.data.value)

async function refresh() {
	await queryClient.invalidateQueries({ queryKey: dashboardKeys.all })
	await queryClient.invalidateQueries({ queryKey: ['project'] })
}

function selectTab(tab: string) {
	router.replace({ query: { ...route.query, tab } })
}
</script>

<template>
	<div v-if="project" class="flex flex-col gap-4">
		<RouterLink
			to="/dashboard/projects"
			class="flex w-fit items-center gap-1 text-secondary no-underline hover:underline"
		>
			<LeftArrowIcon class="size-4" aria-hidden="true" />
			{{ formatMessage(messages.back) }}
		</RouterLink>
		<div class="flex items-center gap-4">
			<Avatar :src="project.icon_url" size="64px" />
			<div class="flex min-w-0 flex-1 flex-col gap-1">
				<h1 class="m-0 truncate text-2xl font-bold text-contrast">{{ project.name }}</h1>
				<ProjectStatusBadge :status="project.status" />
			</div>
			<Button @click="router.push(`/project/${project.id}`)">
				<ExternalIcon aria-hidden="true" />
				{{ formatMessage(messages.view) }}
			</Button>
		</div>
		<div class="flex flex-wrap gap-1 rounded-2xl bg-bg-raised p-1">
			<button
				v-for="tab in tabs"
				:key="tab.id"
				class="cursor-pointer rounded-xl border-0 px-4 py-2 font-semibold"
				:class="
					tab.id === activeTab.id
						? 'bg-brand text-brand-inverted'
						: 'bg-transparent text-primary hover:bg-surface-3'
				"
				@click="selectTab(tab.id)"
			>
				{{ tab.label }}
			</button>
		</div>
		<component :is="activeTab.component" :project="project" :refresh="refresh" />
	</div>
</template>
