<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { Button, defineMessages, NewModal, useFormatDateTime, useVIntl } from '@modrinth/ui'
import { renderString } from '@modrinth/utils/parse'
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { formatGameVersions, loaderName, type ProjectUpdate } from '@/helpers/news'

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'long' })
const router = useRouter()

const messages = defineMessages({
	minecraft: { id: 'app.news.project-update.minecraft', defaultMessage: 'Minecraft {versions}' },
	changelog: { id: 'app.news.project-update.changelog', defaultMessage: 'Changelog' },
	noChangelog: {
		id: 'app.news.project-update.no-changelog',
		defaultMessage: 'No changelog was provided for this update.',
	},
	openProject: { id: 'app.news.project-update.open-project', defaultMessage: 'Open project' },
})

const modal = ref<InstanceType<typeof NewModal>>()
const update = ref<ProjectUpdate>()

function show(value: ProjectUpdate) {
	update.value = value
	modal.value?.show()
}

function openProject() {
	if (!update.value) return
	modal.value?.hide()
	router.push(`/project/${update.value.projectId}`)
}

defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		:header="update ? [update.projectTitle, update.version].filter(Boolean).join(' ') : ''"
		max-width="44rem"
		scrollable
	>
		<div v-if="update" class="flex flex-col gap-4">
			<img
				:src="update.thumbnail"
				alt=""
				class="aspect-video w-full rounded-xl border-[1px] border-solid border-button-border object-cover"
			/>
			<div class="flex flex-wrap items-center gap-2 text-sm">
				<span class="rounded-full bg-surface-4 px-3 py-1 font-medium text-contrast">
					{{
						formatMessage(messages.minecraft, {
							versions: formatGameVersions(update.gameVersions),
						})
					}}
				</span>
				<span
					v-for="loader in update.loaders"
					:key="loader"
					class="rounded-full bg-surface-4 px-3 py-1 font-medium text-contrast"
				>
					{{ loaderName(loader) }}
				</span>
				<span class="ml-auto text-secondary">{{ formatDate(update.date) }}</span>
			</div>
			<div class="flex flex-col gap-2">
				<h3 class="m-0 text-base font-semibold text-contrast">
					{{ formatMessage(messages.changelog) }}
				</h3>
				<template v-if="update.changelogs.length > 0">
					<div
						v-for="(changelog, index) in update.changelogs"
						:key="index"
						class="markdown-body rounded-xl bg-surface-2 p-4 text-primary"
						v-html="renderString(changelog)"
					/>
				</template>
				<p v-else class="m-0 text-secondary">{{ formatMessage(messages.noChangelog) }}</p>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end">
				<Button color="brand" type="colored" @click="openProject">
					<ExternalIcon aria-hidden="true" />
					{{ formatMessage(messages.openProject) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
