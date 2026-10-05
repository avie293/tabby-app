<script setup lang="ts">
import {
	FolderOpenIcon,
	PlayIcon,
	SaveIcon,
	ServerStackIcon,
	StopCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	Slider,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { localServerMessages, statusClasses } from '@/components/ui/local-servers/messages'
import {
	type ConsoleLine,
	local_server_console,
	local_server_delete,
	local_server_edit,
	local_server_get,
	local_server_send_command,
	local_server_start,
	local_server_stop,
	localServerKeys,
} from '@/helpers/local-servers'
import { openPath } from '@/helpers/utils.js'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	console: { id: 'app.local-server.console', defaultMessage: 'Console' },
	commandPlaceholder: {
		id: 'app.local-server.command-placeholder',
		defaultMessage: 'Type a command, e.g. op YourName',
	},
	consoleEmpty: {
		id: 'app.local-server.console-empty',
		defaultMessage: 'Start the server to see its console here.',
	},
	settings: { id: 'app.local-server.settings', defaultMessage: 'Settings' },
	memory: { id: 'app.local-server.memory', defaultMessage: 'Memory: {memory} MB' },
	port: { id: 'app.local-server.port', defaultMessage: 'Port' },
	save: { id: 'app.local-server.save', defaultMessage: 'Save' },
	stopToEdit: {
		id: 'app.local-server.stop-to-edit',
		defaultMessage: 'Changes take effect the next time the server starts.',
	},
	openFolder: { id: 'app.local-server.open-folder', defaultMessage: 'Open folder' },
	delete: { id: 'app.local-server.delete', defaultMessage: 'Delete server' },
	deleteConfirm: {
		id: 'app.local-server.delete-confirm',
		defaultMessage: 'Delete {name} and all of its worlds? This cannot be undone.',
	},
	joinTitle: { id: 'app.local-server.join-title', defaultMessage: 'How to join' },
	joinLocal: {
		id: 'app.local-server.join-local',
		defaultMessage: 'On this PC, join {address} in Minecraft.',
	},
	joinFriends: {
		id: 'app.local-server.join-friends',
		defaultMessage: 'Joining from other networks is coming soon.',
	},
})

const serverId = computed(() => route.params.id as string)

const serverQuery = useQuery({
	queryKey: computed(() => localServerKeys.detail(serverId.value)),
	queryFn: () => local_server_get(serverId.value),
	refetchInterval: 3000,
})
const server = computed(() => serverQuery.data.value)

useRootBreadcrumb({
	slot: 'root',
	id: 'local-server',
	label: 'Hosting',
	to: '/hosting/manage/',
	visual: { type: 'icon', component: ServerStackIcon },
})

const lines = ref<ConsoleLine[]>([])
const consoleStatus = ref<string>('stopped')
const consoleElement = ref<HTMLElement>()
const command = ref('')
let pollTimer: ReturnType<typeof setTimeout> | undefined

async function pollConsole() {
	try {
		const last = lines.value.at(-1)?.seq
		const result = await local_server_console(serverId.value, last)
		consoleStatus.value = result.status
		if (result.lines.length > 0) {
			const atBottom = isScrolledToBottom()
			const restarted = last !== undefined && result.lines[0].seq <= last
			lines.value = (restarted ? result.lines : [...lines.value, ...result.lines]).slice(-2000)
			if (atBottom) await scrollToBottom()
		}
	} catch {
		// The server may have been deleted; the next poll retries.
	}
	pollTimer = setTimeout(pollConsole, 1000)
}

function isScrolledToBottom() {
	const element = consoleElement.value
	return !element || element.scrollHeight - element.scrollTop - element.clientHeight < 40
}

async function scrollToBottom() {
	await nextTick()
	if (consoleElement.value) consoleElement.value.scrollTop = consoleElement.value.scrollHeight
}

watch(
	serverId,
	() => {
		if (pollTimer) clearTimeout(pollTimer)
		lines.value = []
		void pollConsole()
	},
	{ immediate: true },
)
onScopeDispose(() => {
	if (pollTimer) clearTimeout(pollTimer)
})

const status = computed(() => server.value?.status ?? 'stopped')

async function refresh() {
	await queryClient.invalidateQueries({ queryKey: localServerKeys.all })
}

const toggleMutation = useMutation({
	mutationFn: () =>
		status.value === 'stopped'
			? local_server_start(serverId.value)
			: local_server_stop(serverId.value),
	onSuccess: () => {
		if (status.value === 'stopped') lines.value = []
	},
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const commandMutation = useMutation({
	mutationFn: (value: string) => local_server_send_command(serverId.value, value),
	onSuccess: () => {
		command.value = ''
	},
	onError: (error) => handleError(error as Error),
})

const memory = ref(2048)
const port = ref('25565')
watch(
	server,
	(value, previous) => {
		if (value && value.id !== previous?.id) {
			memory.value = value.memory_mb
			port.value = String(value.port)
		}
	},
	{ immediate: true },
)

const editMutation = useMutation({
	mutationFn: () =>
		local_server_edit(serverId.value, {
			memory_mb: memory.value,
			port: Number.parseInt(port.value, 10) || undefined,
		}),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const deleteMutation = useMutation({
	mutationFn: () => local_server_delete(serverId.value),
	onSuccess: async () => {
		await refresh()
		await router.push('/hosting/manage/')
	},
	onError: (error) => handleError(error as Error),
})

function deleteServer() {
	if (!server.value) return
	if (window.confirm(formatMessage(messages.deleteConfirm, { name: server.value.name }))) {
		deleteMutation.mutate()
	}
}
</script>

<template>
	<div v-if="server" class="flex flex-col gap-6 p-6">
		<div class="flex items-center justify-between gap-4">
			<div class="flex min-w-0 items-center gap-4">
				<ServerStackIcon aria-hidden="true" class="size-12 shrink-0 text-brand" />
				<div class="flex min-w-0 flex-col gap-1">
					<h1 class="m-0 truncate text-2xl font-bold text-contrast">{{ server.name }}</h1>
					<div class="flex items-center gap-2 text-secondary">
						<span>
							{{ formatMessage(localServerMessages[server.software]) }} {{ server.game_version }}
						</span>
						<span
							class="rounded-full px-2 py-0.5 text-sm font-medium"
							:class="statusClasses[status]"
						>
							{{ formatMessage(localServerMessages[status]) }}
						</span>
					</div>
				</div>
			</div>
			<div class="flex shrink-0 items-center gap-2">
				<Button @click="openPath(server.path)">
					<FolderOpenIcon aria-hidden="true" />
					{{ formatMessage(messages.openFolder) }}
				</Button>
				<Button
					type="colored"
					size="lg"
					:color="status === 'stopped' ? 'brand' : 'red'"
					:disabled="status === 'stopping' || toggleMutation.isPending.value"
					@click="toggleMutation.mutate()"
				>
					<PlayIcon v-if="status === 'stopped'" aria-hidden="true" />
					<StopCircleIcon v-else aria-hidden="true" />
					{{
						formatMessage(
							status === 'stopped' ? localServerMessages.start : localServerMessages.stop,
						)
					}}
				</Button>
			</div>
		</div>

		<div class="rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4">
			<h2 class="m-0 mb-2 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.joinTitle) }}
			</h2>
			<p class="m-0 text-primary">
				{{ formatMessage(messages.joinLocal, { address: `localhost:${server.port}` }) }}
			</p>
			<p class="m-0 mt-1 text-secondary">{{ formatMessage(messages.joinFriends) }}</p>
		</div>

		<div class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">{{ formatMessage(messages.console) }}</h2>
			<div
				ref="consoleElement"
				class="h-96 overflow-y-auto rounded-2xl border border-solid border-surface-4 bg-surface-1 p-3 font-mono text-sm text-primary"
			>
				<p v-if="lines.length === 0" class="m-0 text-secondary">
					{{ formatMessage(messages.consoleEmpty) }}
				</p>
				<div v-for="line in lines" :key="line.seq" class="whitespace-pre-wrap break-all">
					{{ line.text }}
				</div>
			</div>
			<form class="flex gap-2" @submit.prevent="command.trim() && commandMutation.mutate(command)">
				<Input
					v-model="command"
					class="flex-1"
					:placeholder="formatMessage(messages.commandPlaceholder)"
					:disabled="consoleStatus === 'stopped'"
				/>
			</form>
		</div>

		<div
			class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
		>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.settings) }}
			</h2>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.memory, { memory }) }}
				</span>
				<Slider v-model="memory" :min="1024" :max="16384" :step="512" />
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.port) }}</span>
				<Input v-model="port" class="max-w-40" />
			</div>
			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.stopToEdit) }}</p>
			<div class="flex justify-between gap-2">
				<Button
					type="outlined"
					color="red"
					:disabled="status !== 'stopped' || deleteMutation.isPending.value"
					@click="deleteServer"
				>
					<TrashIcon aria-hidden="true" />
					{{ formatMessage(messages.delete) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="editMutation.isPending.value"
					@click="editMutation.mutate()"
				>
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(messages.save) }}
				</Button>
			</div>
		</div>
	</div>
</template>
