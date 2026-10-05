<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	Chips,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	Input,
	NewModal,
	Slider,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { local_server_create, localServerKeys, type ServerSoftware } from '@/helpers/local-servers'
import { get_game_versions } from '@/helpers/tags.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const router = useRouter()

const messages = defineMessages({
	header: { id: 'app.local-servers.create.header', defaultMessage: 'Create a server' },
	nameLabel: { id: 'app.local-servers.create.name', defaultMessage: 'Name' },
	namePlaceholder: { id: 'app.local-servers.create.name-placeholder', defaultMessage: 'My server' },
	softwareLabel: { id: 'app.local-servers.create.software', defaultMessage: 'Server software' },
	versionLabel: { id: 'app.local-servers.create.version', defaultMessage: 'Minecraft version' },
	versionPlaceholder: {
		id: 'app.local-servers.create.version-placeholder',
		defaultMessage: 'Choose a version',
	},
	memoryLabel: {
		id: 'app.local-servers.create.memory',
		defaultMessage: 'Memory: {memory} MB',
	},
	eula: {
		id: 'app.local-servers.create.eula',
		defaultMessage: 'I agree to the Minecraft End User License Agreement (EULA)',
	},
	eulaLink: { id: 'app.local-servers.create.eula-link', defaultMessage: 'Read the EULA' },
	createButton: { id: 'app.local-servers.create.button', defaultMessage: 'Create server' },
	creating: {
		id: 'app.local-servers.create.creating',
		defaultMessage: 'Downloading server...',
	},
})

const softwareLabels: Record<ServerSoftware, string> = {
	vanilla: 'Vanilla',
	fabric: 'Fabric',
	paper: 'Paper',
}

const modal = ref<InstanceType<typeof NewModal>>()
const name = ref('')
const software = ref<ServerSoftware>('vanilla')
const gameVersion = ref<string>()
const memory = ref(2048)
const acceptEula = ref(false)

const gameVersionsQuery = useQuery({
	queryKey: ['tags', 'game-versions'],
	queryFn: get_game_versions,
})
const versionOptions = computed<ComboboxOption<string>[]>(() =>
	((gameVersionsQuery.data.value ?? []) as Array<{ version: string; version_type: string }>)
		.filter((version) => version.version_type === 'release')
		.map((version) => ({ value: version.version, label: version.version })),
)

const canCreate = computed(
	() => name.value.trim().length > 0 && !!gameVersion.value && acceptEula.value,
)

const createMutation = useMutation({
	mutationFn: () =>
		local_server_create({
			name: name.value,
			software: software.value,
			game_version: gameVersion.value!,
			memory_mb: memory.value,
			accept_eula: acceptEula.value,
		}),
	onSuccess: async (server) => {
		await queryClient.invalidateQueries({ queryKey: localServerKeys.all })
		modal.value?.hide()
		await router.push(`/hosting/local/${server.id}`)
	},
	onError: (error) => handleError(error as Error),
})

function show() {
	name.value = ''
	software.value = 'vanilla'
	gameVersion.value = versionOptions.value[0]?.value
	memory.value = 2048
	acceptEula.value = false
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="560px">
		<div class="flex flex-col gap-5">
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.nameLabel) }}</span>
				<Input v-model="name" :placeholder="formatMessage(messages.namePlaceholder)" />
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.softwareLabel) }}</span>
				<Chips
					v-model="software"
					:items="Object.keys(softwareLabels) as ServerSoftware[]"
					:format-label="(item: ServerSoftware) => softwareLabels[item]"
					:capitalize="false"
				/>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.versionLabel) }}</span>
				<Combobox
					v-model="gameVersion"
					:options="versionOptions"
					:placeholder="formatMessage(messages.versionPlaceholder)"
					searchable
				/>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.memoryLabel, { memory }) }}
				</span>
				<Slider v-model="memory" :min="1024" :max="16384" :step="512" />
			</div>
			<div class="flex flex-col gap-1">
				<Checkbox v-model="acceptEula" :label="formatMessage(messages.eula)" />
				<a
					href="https://aka.ms/MinecraftEULA"
					target="_blank"
					class="ml-8 text-sm text-brand hover:underline"
				>
					{{ formatMessage(messages.eulaLink) }}
				</a>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="!canCreate || createMutation.isPending.value"
					@click="createMutation.mutate()"
				>
					<PlusIcon aria-hidden="true" />
					{{
						formatMessage(
							createMutation.isPending.value ? messages.creating : messages.createButton,
						)
					}}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
