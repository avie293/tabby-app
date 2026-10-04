import { invoke } from '@tauri-apps/api/core'

import type { InstallJobSnapshot } from '@/helpers/install'

export type ModrinthAppInstance = {
	id: string
	name: string
	folder: string
	game_version: string | null
	loader: string
	loader_version: string | null
	playtime_seconds: number
	last_played: number | null
	icon_path: string | null
}

export type ModrinthAppImportOptions = {
	content: boolean
	worlds: boolean
	screenshots: boolean
	settings: boolean
	playtime: boolean
}

export type ModrinthAppImportHistory = {
	imports: Record<string, { instance_id: string; imported_at: string }>
	playtime_transfers: Record<
		string,
		{ instance_id: string; seconds: number; transferred_at: string }
	>
}

export const modrinthAppImportKeys = {
	all: ['modrinth-app-import'] as const,
	instances: ['modrinth-app-import', 'instances'] as const,
	history: ['modrinth-app-import', 'history'] as const,
}

export async function modrinth_app_list_instances(): Promise<ModrinthAppInstance[]> {
	return await invoke('plugin:import|modrinth_app_list_instances')
}

export async function modrinth_app_import_instances(
	instanceIds: string[],
	options: ModrinthAppImportOptions,
): Promise<InstallJobSnapshot[]> {
	return await invoke('plugin:import|modrinth_app_import_instances', { instanceIds, options })
}

export async function modrinth_app_transfer_playtime(
	sourceInstanceId: string,
	targetInstanceId: string,
): Promise<number> {
	return await invoke('plugin:import|modrinth_app_transfer_playtime', {
		sourceInstanceId,
		targetInstanceId,
	})
}

export async function modrinth_app_get_history(): Promise<ModrinthAppImportHistory> {
	return await invoke('plugin:import|modrinth_app_get_history')
}
