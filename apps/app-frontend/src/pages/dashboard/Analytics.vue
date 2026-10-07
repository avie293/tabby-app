<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SpinnerIcon } from '@modrinth/assets'
import {
	Avatar,
	Chart,
	Checkbox,
	Chips,
	defineMessages,
	injectModrinthClient,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { dashboardKeys, formatNumber, useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { userId } = useCurrentUser()

const messages = defineMessages({
	title: { id: 'app.dashboard.analytics.title', defaultMessage: 'Analytics' },
	downloads: { id: 'app.dashboard.analytics.downloads', defaultMessage: 'Downloads' },
	views: { id: 'app.dashboard.analytics.views', defaultMessage: 'Views' },
	playtime: { id: 'app.dashboard.analytics.playtime', defaultMessage: 'Playtime (hours)' },
	revenue: { id: 'app.dashboard.analytics.revenue', defaultMessage: 'Revenue' },
	week: { id: 'app.dashboard.analytics.range.week', defaultMessage: '7 days' },
	month: { id: 'app.dashboard.analytics.range.month', defaultMessage: '30 days' },
	quarter: { id: 'app.dashboard.analytics.range.quarter', defaultMessage: '90 days' },
	year: { id: 'app.dashboard.analytics.range.year', defaultMessage: '1 year' },
	projects: { id: 'app.dashboard.analytics.projects', defaultMessage: 'Projects' },
	total: { id: 'app.dashboard.analytics.total', defaultMessage: 'Total' },
	empty: {
		id: 'app.dashboard.analytics.empty',
		defaultMessage: 'Publish a project to see its analytics here.',
	},
})

type Metric = 'downloads' | 'views' | 'playtime' | 'revenue'
type Range = 'week' | 'month' | 'quarter' | 'year'
const metrics: Metric[] = ['downloads', 'views', 'playtime', 'revenue']
const ranges: Range[] = ['week', 'month', 'quarter', 'year']
const RANGE_DAYS: Record<Range, number> = { week: 7, month: 30, quarter: 90, year: 365 }

const metric = ref<Metric>('downloads')
const range = ref<Range>('month')

const projectsQuery = useQuery({
	queryKey: computed(() => dashboardKeys.projects(userId.value ?? '')),
	queryFn: () => client.labrinth.users_v3.getProjects(userId.value!),
	enabled: () => !!userId.value,
})
const projects = computed(() =>
	[...(projectsQuery.data.value ?? [])].sort((a, b) => b.downloads - a.downloads),
)
const selected = ref<string[]>([])
watch(
	projects,
	(value) => {
		if (selected.value.length === 0) selected.value = value.slice(0, 5).map((p) => p.id)
	},
	{ immediate: true },
)

function toggleProject(id: string) {
	selected.value = selected.value.includes(id)
		? selected.value.filter((value) => value !== id)
		: [...selected.value, id]
}

/** One slice per day, or per week for a whole year. */
const slices = computed(() => (range.value === 'year' ? 52 : RANGE_DAYS[range.value]))
const timeRange = computed(() => {
	const end = new Date()
	const start = new Date(end.getTime() - RANGE_DAYS[range.value] * 24 * 60 * 60 * 1000)
	return { start, end }
})

const METRIC_KEYS: Record<Metric, keyof Labrinth.Analytics.v3.ReturnMetrics> = {
	downloads: 'project_downloads',
	views: 'project_views',
	playtime: 'project_playtime',
	revenue: 'project_revenue',
}

const analyticsQuery = useQuery({
	queryKey: computed(() => [
		'dashboard',
		'analytics',
		metric.value,
		range.value,
		[...selected.value].sort(),
	]),
	queryFn: () =>
		client.labrinth.analytics_v3.fetch({
			time_range: {
				start: timeRange.value.start.toISOString(),
				end: timeRange.value.end.toISOString(),
				resolution: { slices: slices.value },
			},
			return_metrics: { [METRIC_KEYS[metric.value]]: { bucket_by: ['project_id'] } },
			project_ids: selected.value,
		}),
	enabled: () => selected.value.length > 0,
})

function valueOf(entry: Labrinth.Analytics.v3.AnalyticsData) {
	if ('downloads' in entry) return entry.downloads
	if ('views' in entry) return entry.views
	if ('seconds' in entry) return entry.seconds / 3600
	if ('revenue' in entry) return Number(entry.revenue)
	return 0
}

const labels = computed(() => {
	const { start, end } = timeRange.value
	const step = (end.getTime() - start.getTime()) / slices.value
	return Array.from({ length: slices.value }, (_, index) =>
		new Date(start.getTime() + step * index).toISOString(),
	)
})

const series = computed(() => {
	const slicesData = analyticsQuery.data.value?.metrics ?? []
	return selected.value.map((projectId) => ({
		name: projects.value.find((project) => project.id === projectId)?.name ?? projectId,
		data: labels.value.map(
			(_, index) =>
				Math.round(
					(slicesData[index] ?? [])
						.filter((entry) => 'source_project' in entry && entry.source_project === projectId)
						.reduce((total, entry) => total + valueOf(entry), 0) * 100,
				) / 100,
		),
	}))
})

const totals = computed(() =>
	series.value
		.map((entry, index) => ({
			id: selected.value[index],
			name: entry.name,
			total: entry.data.reduce((total, value) => total + value, 0),
		}))
		.sort((a, b) => b.total - a.total),
)

function formatValue(value: number) {
	return metric.value === 'revenue' ? `$${value.toFixed(2)}` : formatNumber(Math.round(value))
}

const chartKey = computed(
	() =>
		`${metric.value}-${range.value}-${selected.value.join(',')}-${analyticsQuery.dataUpdatedAt.value}`,
)
</script>

<template>
	<div class="flex flex-col gap-4">
		<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>
		<p v-if="!projectsQuery.isPending.value && projects.length === 0" class="m-0 text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<template v-else>
			<div class="flex flex-wrap items-center justify-between gap-4">
				<Chips
					v-model="metric"
					:items="metrics"
					:format-label="(item: Metric) => formatMessage(messages[item])"
					:capitalize="false"
				/>
				<Chips
					v-model="range"
					:items="ranges"
					:format-label="(item: Range) => formatMessage(messages[item])"
					:capitalize="false"
				/>
			</div>
			<div class="rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4">
				<div v-if="analyticsQuery.isFetching.value" class="flex justify-center py-24">
					<SpinnerIcon class="size-8 animate-spin" aria-hidden="true" />
				</div>
				<Chart
					v-else-if="series.length > 0"
					:key="chartKey"
					:name="formatMessage(messages[metric])"
					:labels="labels"
					:data="series"
					:prefix="metric === 'revenue' ? '$' : ''"
					type="area"
					hide-toolbar
				/>
			</div>
			<div class="grid grid-cols-[2fr_1fr] gap-4">
				<div
					class="flex flex-col gap-1 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
				>
					<h2 class="m-0 mb-2 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.projects) }}
					</h2>
					<div
						v-for="project in projects"
						:key="project.id"
						class="flex items-center gap-3 rounded-xl px-2 py-1 hover:bg-surface-3"
					>
						<Checkbox
							:model-value="selected.includes(project.id)"
							@update:model-value="toggleProject(project.id)"
						/>
						<Avatar :src="project.icon_url" size="32px" />
						<span class="flex-1 truncate">{{ project.name }}</span>
					</div>
				</div>
				<div
					class="flex flex-col gap-2 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
				>
					<h2 class="m-0 mb-2 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.total) }}
					</h2>
					<div v-for="entry in totals" :key="entry.id" class="flex justify-between gap-2">
						<span class="truncate">{{ entry.name }}</span>
						<span class="font-semibold text-contrast">{{ formatValue(entry.total) }}</span>
					</div>
				</div>
			</div>
		</template>
	</div>
</template>
