<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, SaveIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	Combobox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { get_donation_platforms } from '@/helpers/tags'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const messages = defineMessages({
	issues: { id: 'app.dashboard.project.links.issues', defaultMessage: 'Issue tracker' },
	source: { id: 'app.dashboard.project.links.source', defaultMessage: 'Source code' },
	wiki: { id: 'app.dashboard.project.links.wiki', defaultMessage: 'Wiki' },
	discord: { id: 'app.dashboard.project.links.discord', defaultMessage: 'Discord invite' },
	donations: { id: 'app.dashboard.project.links.donations', defaultMessage: 'Donation links' },
	platform: { id: 'app.dashboard.project.links.platform', defaultMessage: 'Platform' },
	addDonation: { id: 'app.dashboard.project.links.add-donation', defaultMessage: 'Add link' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
})

const MAIN_LINKS = ['issues', 'source', 'wiki', 'discord'] as const

const platformsQuery = useQuery({
	queryKey: ['tags', 'donation-platforms'],
	queryFn: async () => (await get_donation_platforms()) as { short: string; name: string }[],
})
const platformOptions = computed(() =>
	(platformsQuery.data.value ?? []).map((platform) => ({
		value: platform.short,
		label: platform.name,
	})),
)

const mainLinks = ref<Record<string, string>>({})
const donations = ref<{ platform: string; url: string }[]>([])
watch(
	() => props.project.link_urls,
	(links) => {
		mainLinks.value = Object.fromEntries(MAIN_LINKS.map((key) => [key, links[key]?.url ?? '']))
		donations.value = Object.values(links)
			.filter((link) => link.donation)
			.map((link) => ({ platform: link.platform, url: link.url }))
	},
	{ immediate: true },
)

const saveMutation = useMutation({
	mutationFn: () => {
		const link_urls: Record<string, string | null> = {}
		// Every existing link is cleared first, so removed rows disappear.
		for (const key of Object.keys(props.project.link_urls)) link_urls[key] = null
		for (const key of MAIN_LINKS) link_urls[key] = mainLinks.value[key]?.trim() || null
		for (const donation of donations.value) {
			if (donation.platform && donation.url.trim())
				link_urls[donation.platform] = donation.url.trim()
		}
		return client.labrinth.projects_v3.edit(props.project.id, { link_urls })
	},
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<section
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
	>
		<label v-for="key in MAIN_LINKS" :key="key" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages[key]) }}</span>
			<Input v-model="mainLinks[key]" type="url" placeholder="https://" />
		</label>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.donations) }}</span>
			<div v-for="(donation, index) in donations" :key="index" class="flex gap-2">
				<Combobox
					v-model="donation.platform"
					class="w-56"
					:options="platformOptions"
					:placeholder="formatMessage(messages.platform)"
				/>
				<Input v-model="donation.url" class="flex-1" type="url" placeholder="https://" />
				<Button @click="donations.splice(index, 1)">
					<XIcon aria-hidden="true" />
				</Button>
			</div>
			<div>
				<Button @click="donations.push({ platform: '', url: '' })">
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.addDonation) }}
				</Button>
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
