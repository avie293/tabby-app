import { invoke } from '@tauri-apps/api/core'

export type ServerSoftware = 'vanilla' | 'fabric' | 'paper'

export type LocalServerStatus = 'stopped' | 'starting' | 'running' | 'stopping'

export type LocalServer = {
	id: string
	name: string
	software: ServerSoftware
	game_version: string
	loader_version: string | null
	java_major: number
	memory_mb: number
	port: number
	created: string
}

export type LocalServerInfo = LocalServer & {
	status: LocalServerStatus
	path: string
}

export type ConsoleLine = { seq: number; text: string }

export type LocalServerConsole = {
	status: LocalServerStatus
	lines: ConsoleLine[]
}

export type CreateLocalServer = {
	name: string
	software: ServerSoftware
	game_version: string
	memory_mb: number
	accept_eula: boolean
}

export type EditLocalServer = {
	name?: string
	memory_mb?: number
	port?: number
}

export const localServerKeys = {
	all: ['local-servers'] as const,
	list: ['local-servers', 'list'] as const,
	detail: (id: string) => ['local-servers', 'detail', id] as const,
}

export async function local_server_list(): Promise<LocalServerInfo[]> {
	return await invoke('plugin:local-servers|local_server_list')
}

export async function local_server_get(id: string): Promise<LocalServerInfo> {
	return await invoke('plugin:local-servers|local_server_get', { id })
}

export async function local_server_create(request: CreateLocalServer): Promise<LocalServer> {
	return await invoke('plugin:local-servers|local_server_create', { request })
}

export async function local_server_edit(id: string, patch: EditLocalServer): Promise<LocalServer> {
	return await invoke('plugin:local-servers|local_server_edit', { id, patch })
}

export async function local_server_delete(id: string): Promise<void> {
	return await invoke('plugin:local-servers|local_server_delete', { id })
}

export async function local_server_start(id: string): Promise<void> {
	return await invoke('plugin:local-servers|local_server_start', { id })
}

export async function local_server_stop(id: string): Promise<void> {
	return await invoke('plugin:local-servers|local_server_stop', { id })
}

export async function local_server_send_command(id: string, command: string): Promise<void> {
	return await invoke('plugin:local-servers|local_server_send_command', { id, command })
}

export async function local_server_console(
	id: string,
	after?: number,
): Promise<LocalServerConsole> {
	return await invoke('plugin:local-servers|local_server_console', { id, after })
}
