<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SaveIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	MarkdownEditor,
	useVIntl,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { ref, watch } from 'vue'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	refresh: () => Promise<void>
}>()

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const MAX_LENGTH = 65536

const messages = defineMessages({
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
	saved: { id: 'app.dashboard.saved', defaultMessage: 'Changes saved' },
	imageType: {
		id: 'app.dashboard.project.image-type',
		defaultMessage: 'Images have to be PNG, JPEG, GIF or WebP files.',
	},
	imageSize: {
		id: 'app.dashboard.project.image-size',
		defaultMessage: 'Images can be at most 1 MB.',
	},
})

const description = ref('')
watch(
	() => props.project.description,
	(value) => (description.value = value),
	{ immediate: true },
)

const IMAGE_EXTENSIONS: Partial<Record<string, Labrinth.Images.v3.ImageExtension>> = {
	'image/gif': 'gif',
	'image/jpeg': 'jpeg',
	'image/png': 'png',
	'image/webp': 'webp',
}

async function onImageUpload(file: File) {
	const extension = IMAGE_EXTENSIONS[file.type]
	if (!extension) throw new Error(formatMessage(messages.imageType))
	if (file.size > 1024 * 1024) throw new Error(formatMessage(messages.imageSize))
	const image = await client.labrinth.images_v3.uploadImage(file, extension, {
		context: 'project',
		project_id: props.project.id,
	}).promise
	return image.url
}

const saveMutation = useMutation({
	mutationFn: () =>
		client.labrinth.projects_v3.edit(props.project.id, { description: description.value }),
	onSuccess: () => addNotification({ title: formatMessage(messages.saved), type: 'success' }),
	onSettled: props.refresh,
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<div class="flex flex-col gap-4">
		<MarkdownEditor
			v-model="description"
			:on-image-upload="onImageUpload"
			:max-length="MAX_LENGTH"
			:min-height="400"
		/>
		<div class="flex justify-end">
			<Button
				color="brand"
				type="colored"
				:disabled="saveMutation.isPending.value || description === project.description"
				@click="saveMutation.mutate()"
			>
				<SaveIcon aria-hidden="true" />
				{{ formatMessage(messages.save) }}
			</Button>
		</div>
	</div>
</template>
