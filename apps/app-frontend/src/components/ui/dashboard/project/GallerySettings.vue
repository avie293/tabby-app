<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { EditIcon, PlusIcon, SaveIcon, StarIcon, TrashIcon, UploadIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	Textarea,
	useVIntl,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { imageExtension } from '@/composables/use-dashboard'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()

const messages = defineMessages({
	upload: { id: 'app.dashboard.project.gallery.upload', defaultMessage: 'Upload image' },
	edit: { id: 'app.dashboard.project.gallery.edit', defaultMessage: 'Edit image' },
	empty: {
		id: 'app.dashboard.project.gallery.empty',
		defaultMessage: 'Show off your project with screenshots.',
	},
	title: { id: 'app.dashboard.project.gallery.title', defaultMessage: 'Title' },
	description: { id: 'app.dashboard.project.gallery.description', defaultMessage: 'Description' },
	featured: {
		id: 'app.dashboard.project.gallery.featured',
		defaultMessage: 'Featured image (shown on search results)',
	},
	chooseFile: { id: 'app.dashboard.project.gallery.choose-file', defaultMessage: 'Choose image' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	delete: { id: 'app.dashboard.project.gallery.delete', defaultMessage: 'Delete' },
})

const images = computed(() => [...props.project.gallery].sort((a, b) => a.ordering - b.ordering))

const modal = ref<InstanceType<typeof NewModal>>()
const editing = ref<Labrinth.Projects.v3.GalleryItem | null>(null)
const file = ref<File | null>(null)
const preview = ref<string | null>(null)
const title = ref('')
const description = ref('')
const featured = ref(false)
const fileInput = ref<HTMLInputElement>()

function openUpload() {
	editing.value = null
	file.value = null
	preview.value = null
	title.value = ''
	description.value = ''
	featured.value = props.project.gallery.length === 0
	modal.value?.show()
}

function openEdit(image: Labrinth.Projects.v3.GalleryItem) {
	editing.value = image
	file.value = null
	preview.value = image.url
	title.value = image.name ?? ''
	description.value = image.description ?? ''
	featured.value = image.featured
	modal.value?.show()
}

function onFile() {
	const chosen = fileInput.value?.files?.[0]
	if (!chosen) return
	file.value = chosen
	preview.value = URL.createObjectURL(chosen)
	if (fileInput.value) fileInput.value.value = ''
}

const saveMutation = useMutation({
	mutationFn: async () => {
		if (editing.value) {
			await client.labrinth.projects_v2.editGalleryImage(props.project.id, editing.value.url, {
				featured: featured.value,
				title: title.value.trim() || undefined,
				description: description.value.trim() || undefined,
			})
		} else if (file.value) {
			await client.labrinth.projects_v3.createGalleryImage(props.project.id, file.value, {
				ext: imageExtension(file.value),
				featured: featured.value,
				name: title.value.trim() || undefined,
				description: description.value.trim() || undefined,
				ordering: props.project.gallery.length,
			})
		}
	},
	onSuccess: () => modal.value?.hide(),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})

const deleteMutation = useMutation({
	mutationFn: (url: string) =>
		client.labrinth.projects_v3.deleteGalleryImage(props.project.id, url),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<NewModal
			ref="modal"
			:header="formatMessage(editing ? messages.edit : messages.upload)"
			max-width="36rem"
		>
			<div class="flex flex-col gap-4">
				<input
					ref="fileInput"
					type="file"
					accept="image/png,image/jpeg,image/gif,image/webp"
					class="hidden"
					@change="onFile"
				/>
				<img
					v-if="preview"
					:src="preview"
					alt=""
					class="aspect-video w-full rounded-xl object-cover"
				/>
				<div v-if="!editing">
					<Button @click="fileInput?.click()">
						<UploadIcon aria-hidden="true" />
						{{ formatMessage(messages.chooseFile) }}
					</Button>
				</div>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.title) }}</span>
					<Input v-model="title" :maxlength="255" />
				</label>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.description) }}</span>
					<Textarea v-model="description" :maxlength="2048" />
				</label>
				<Checkbox v-model="featured" :label="formatMessage(messages.featured)" />
			</div>
			<template #actions>
				<div class="flex justify-end">
					<Button
						color="brand"
						type="colored"
						:disabled="(!editing && !file) || saveMutation.isPending.value"
						@click="saveMutation.mutate()"
					>
						<SaveIcon aria-hidden="true" />
						{{ formatMessage(editing ? messages.save : messages.upload) }}
					</Button>
				</div>
			</template>
		</NewModal>
		<div>
			<Button color="brand" type="colored" @click="openUpload">
				<PlusIcon aria-hidden="true" />
				{{ formatMessage(messages.upload) }}
			</Button>
		</div>
		<p v-if="images.length === 0" class="m-0 text-secondary">{{ formatMessage(messages.empty) }}</p>
		<div class="grid grid-cols-2 gap-4 xl:grid-cols-3">
			<div
				v-for="image in images"
				:key="image.url"
				class="flex flex-col overflow-hidden rounded-2xl border border-solid border-surface-4 bg-bg-raised"
			>
				<img :src="image.url" alt="" class="aspect-video w-full object-cover" />
				<div class="flex flex-col gap-2 p-3">
					<span class="flex items-center gap-1 truncate font-semibold text-contrast">
						<StarIcon
							v-if="image.featured"
							class="size-4 shrink-0 text-orange"
							fill="currentColor"
							aria-hidden="true"
						/>
						{{ image.name || '—' }}
					</span>
					<div class="flex gap-2">
						<Button @click="openEdit(image)">
							<EditIcon aria-hidden="true" />
							{{ formatMessage(messages.edit) }}
						</Button>
						<Button
							type="outlined"
							color="red"
							:disabled="deleteMutation.isPending.value"
							@click="deleteMutation.mutate(image.url)"
						>
							<TrashIcon aria-hidden="true" />
							{{ formatMessage(messages.delete) }}
						</Button>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
