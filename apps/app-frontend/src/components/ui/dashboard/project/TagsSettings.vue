<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SaveIcon, StarIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { get_categories } from '@/helpers/tags'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const MAX_FEATURED = 3

const messages = defineMessages({
	description: {
		id: 'app.dashboard.project.tags-description',
		defaultMessage:
			'Pick every tag that fits. Star up to three of them to show them on search results.',
	},
	feature: { id: 'app.dashboard.project.tags-feature', defaultMessage: 'Show on search results' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
})

type Category = { name: string; project_type: string; header: string; icon: string }

const categoriesQuery = useQuery({
	queryKey: ['tags', 'categories'],
	queryFn: async () => (await get_categories()) as Category[],
})

const groups = computed(() => {
	const types = props.project.project_types as string[]
	const grouped = new Map<string, Category[]>()
	for (const category of categoriesQuery.data.value ?? []) {
		if (!types.includes(category.project_type)) continue
		if (!grouped.has(category.header)) grouped.set(category.header, [])
		if (!grouped.get(category.header)!.some((item) => item.name === category.name)) {
			grouped.get(category.header)!.push(category)
		}
	}
	return [...grouped.entries()]
})

const selected = ref<Set<string>>(new Set())
const featured = ref<Set<string>>(new Set())
watch(
	() => props.project,
	(project) => {
		selected.value = new Set([...project.categories, ...project.additional_categories])
		featured.value = new Set(project.categories)
	},
	{ immediate: true },
)

function toggle(name: string) {
	const next = new Set(selected.value)
	if (next.has(name)) {
		next.delete(name)
		const nextFeatured = new Set(featured.value)
		nextFeatured.delete(name)
		featured.value = nextFeatured
	} else {
		next.add(name)
	}
	selected.value = next
}

function toggleFeatured(name: string) {
	const next = new Set(featured.value)
	if (next.has(name)) next.delete(name)
	else if (next.size < MAX_FEATURED && selected.value.has(name)) next.add(name)
	featured.value = next
}

const saveMutation = useMutation({
	mutationFn: () =>
		client.labrinth.projects_v3.edit(props.project.id, {
			categories: [...featured.value],
			additional_categories: [...selected.value].filter((name) => !featured.value.has(name)),
		}),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<section
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
	>
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		<div v-for="[header, categories] in groups" :key="header" class="flex flex-col gap-2">
			<h3 class="m-0 text-base font-semibold capitalize text-contrast">{{ header }}</h3>
			<div class="grid grid-cols-2 gap-1 lg:grid-cols-3">
				<div
					v-for="category in categories"
					:key="category.name"
					class="flex items-center gap-2 rounded-xl px-2 py-1 hover:bg-surface-3"
				>
					<Checkbox
						:model-value="selected.has(category.name)"
						:label="category.name.replaceAll('-', ' ')"
						label-class="capitalize"
						class="flex-1"
						@update:model-value="toggle(category.name)"
					/>
					<button
						v-if="selected.has(category.name)"
						v-tooltip="formatMessage(messages.feature)"
						class="flex cursor-pointer border-0 bg-transparent p-1"
						:class="featured.has(category.name) ? 'text-orange' : 'text-secondary'"
						:disabled="!featured.has(category.name) && featured.size >= MAX_FEATURED"
						@click="toggleFeatured(category.name)"
					>
						<StarIcon
							class="size-4"
							:fill="featured.has(category.name) ? 'currentColor' : 'none'"
							aria-hidden="true"
						/>
					</button>
				</div>
			</div>
		</div>
		<div class="flex justify-end">
			<Button
				color="brand"
				type="colored"
				:disabled="saveMutation.isPending.value"
				@click="saveMutation.mutate()"
			>
				<SaveIcon aria-hidden="true" />
				{{ formatMessage(messages.save) }}
			</Button>
		</div>
	</section>
</template>
