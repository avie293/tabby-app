<script setup lang="ts">
import { PlayIcon, PlusIcon, ServerStackIcon, StopCircleIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import {
	local_server_list,
	local_server_start,
	local_server_stop,
	type LocalServerInfo,
	localServerKeys,
} from '@/helpers/local-servers'

import CreateLocalServerModal from './CreateLocalServerModal.vue'
import { localServerMessages, statusClasses } from './messages'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const router = useRouter()

const messages = defineMessages({
	title: { id: 'app.local-servers.title', defaultMessage: 'Servers on this PC' },
	description: {
		id: 'app.local-servers.description',
		defaultMessage: 'Host a Minecraft server right on your computer and play with friends.',
	},
	create: { id: 'app.local-servers.create-button', defaultMessage: 'Create server' },
	empty: {
		id: 'app.local-servers.empty',
		defaultMessage: "You haven't created a server on this PC yet.",
	},
})

const createModal = ref<InstanceType<typeof CreateLocalServerModal>>()

const serversQuery = useQuery({
	queryKey: localServerKeys.list,
	queryFn: local_server_list,
	refetchInterval: 3000,
})
const servers = computed(() => serversQuery.data.value ?? [])

const toggleMutation = useMutation({
	mutationFn: (server: LocalServerInfo) =>
		server.status === 'stopped' ? local_server_start(server.id) : local_server_stop(server.id),
	onSettled: () => queryClient.invalidateQueries({ queryKey: localServerKeys.all }),
	onError: (error) => handleError(error as Error),
})
</script>

<template>
	<section class="flex flex-col gap-3 px-6 pt-6">
		<CreateLocalServerModal ref="createModal" />
		<div class="flex items-center justify-between gap-4">
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</h2>
				<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
			</div>
			<Button color="brand" type="colored" @click="createModal?.show()">
				<PlusIcon aria-hidden="true" />
				{{ formatMessage(messages.create) }}
			</Button>
		</div>
		<p v-if="!serversQuery.isPending.value && servers.length === 0" class="m-0 text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<div
			v-for="server in servers"
			:key="server.id"
			class="flex cursor-pointer items-center justify-between gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3 hover:bg-surface-3"
			@click="router.push(`/hosting/local/${server.id}`)"
		>
			<div class="flex min-w-0 items-center gap-3">
				<ServerStackIcon aria-hidden="true" class="size-8 shrink-0 text-brand" />
				<div class="flex min-w-0 flex-col">
					<span class="truncate font-semibold text-contrast">{{ server.name }}</span>
					<span class="text-sm text-secondary">
						{{ formatMessage(localServerMessages[server.software]) }} {{ server.game_version }} ·
						localhost:{{ server.port }}
					</span>
				</div>
			</div>
			<div class="flex shrink-0 items-center gap-3">
				<span
					class="rounded-full px-2 py-0.5 text-sm font-medium"
					:class="statusClasses[server.status]"
				>
					{{ formatMessage(localServerMessages[server.status]) }}
				</span>
				<Button
					:color="server.status === 'stopped' ? 'brand' : 'red'"
					type="colored"
					:disabled="server.status === 'stopping' || toggleMutation.isPending.value"
					@click.stop="toggleMutation.mutate(server)"
				>
					<PlayIcon v-if="server.status === 'stopped'" aria-hidden="true" />
					<StopCircleIcon v-else aria-hidden="true" />
					{{
						formatMessage(
							server.status === 'stopped' ? localServerMessages.start : localServerMessages.stop,
						)
					}}
				</Button>
			</div>
		</div>
	</section>
</template>
