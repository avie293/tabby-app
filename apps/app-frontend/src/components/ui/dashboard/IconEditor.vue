<script setup lang="ts">
import { TrashIcon, UploadIcon } from '@modrinth/assets'
import { Avatar, Button, defineMessages, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

defineProps<{ src: string | null | undefined; disabled?: boolean; circle?: boolean }>()
const emit = defineEmits<{ upload: [file: File]; remove: [] }>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	upload: { id: 'app.dashboard.icon.upload', defaultMessage: 'Upload icon' },
	remove: { id: 'app.dashboard.icon.remove', defaultMessage: 'Remove icon' },
})

const input = ref<HTMLInputElement>()

function onChange() {
	const file = input.value?.files?.[0]
	if (file) emit('upload', file)
	if (input.value) input.value.value = ''
}
</script>

<template>
	<div class="flex items-center gap-4">
		<Avatar :src="src" size="96px" :circle="circle" />
		<div class="flex flex-col gap-2">
			<input
				ref="input"
				type="file"
				accept="image/png,image/jpeg,image/gif,image/webp"
				class="hidden"
				@change="onChange"
			/>
			<Button :disabled="disabled" @click="input?.click()">
				<UploadIcon aria-hidden="true" />
				{{ formatMessage(messages.upload) }}
			</Button>
			<Button v-if="src" :disabled="disabled" @click="emit('remove')">
				<TrashIcon aria-hidden="true" />
				{{ formatMessage(messages.remove) }}
			</Button>
		</div>
	</div>
</template>
