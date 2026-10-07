<script setup lang="ts">
import { NewspaperIcon } from '@modrinth/assets'
import {
	ButtonLink,
	defineMessages,
	NewsArticleCard,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { onMounted, ref } from 'vue'

import { fetchNews, formatGameVersions, loaderName, type NewsItem } from '@/helpers/news'

import ProjectUpdateModal from './ProjectUpdateModal.vue'

const NEWS_LIMIT = 5

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'long' })

const messages = defineMessages({
	news: { id: 'app.news.title', defaultMessage: 'News' },
	viewAllNews: { id: 'app.news.view-all', defaultMessage: 'View all news' },
	updateSummary: {
		id: 'app.news.project-update.summary',
		defaultMessage: 'Minecraft {versions} · {loaders}',
	},
})

const news = ref<NewsItem[]>([])
const updateModal = ref<InstanceType<typeof ProjectUpdateModal>>()

onMounted(async () => {
	try {
		news.value = await fetchNews(NEWS_LIMIT)
	} catch (error) {
		console.error('Failed to fetch news', error)
	}
})
</script>

<template>
	<div v-if="news.length > 0" class="p-4 flex flex-col items-center">
		<ProjectUpdateModal ref="updateModal" />
		<h3 class="text-base mb-4 text-primary font-medium m-0 text-left w-full">
			{{ formatMessage(messages.news) }}
		</h3>
		<div class="space-y-4 flex flex-col items-center w-full">
			<template v-for="item in news" :key="item.key">
				<NewsArticleCard v-if="item.type === 'article'" :article="item" />
				<!-- Same look as NewsArticleCard, but opens the changelog instead of a link. -->
				<button
					v-else
					class="active:scale-[0.99] group flex w-full flex-col border-0 bg-transparent p-0 text-left text-contrast transition-all ease-in-out hover:brightness-125 cursor-pointer"
					@click="updateModal?.show(item)"
				>
					<article class="flex h-full grow flex-col gap-4">
						<img
							:src="item.thumbnail"
							alt=""
							class="aspect-video w-full rounded-xl border-[1px] border-solid border-button-border object-cover"
						/>
						<div class="flex grow flex-col gap-2">
							<h3 class="m-0 text-base leading-tight group-hover:underline">
								{{ [item.projectTitle, item.version].filter(Boolean).join(' ') }}
							</h3>
							<p class="m-0 text-sm leading-tight text-primary">
								{{
									formatMessage(messages.updateSummary, {
										versions: formatGameVersions(item.gameVersions),
										loaders: item.loaders.map(loaderName).join(', '),
									})
								}}
							</p>
							<div class="mt-auto text-sm text-secondary">
								{{ formatDate(item.date) }}
							</div>
						</div>
					</article>
				</button>
			</template>
			<ButtonLink
				type="colored"
				color="brand"
				size="xl"
				href="https://github.com/avie293/tabby-app/releases"
				target="_blank"
				class="my-4"
			>
				<NewspaperIcon />
				{{ formatMessage(messages.viewAllNews) }}
			</ButtonLink>
		</div>
	</div>
</template>
