import { ref, watch } from 'vue'

import { get_project } from '@/helpers/cache.js'
import { get_content_items, install_project_with_dependencies } from '@/helpers/instance'
import type { InstanceLoader } from '@/helpers/types'

export const ESSENTIAL_PROJECT_SLUG = 'essential'

const ESSENTIAL_LOADERS: InstanceLoader[] = ['fabric', 'forge', 'neoforge']
const AUTO_INSTALL_STORAGE_KEY = 'essential-auto-install'

export function supportsEssential(loader: InstanceLoader | string | null | undefined): boolean {
	return ESSENTIAL_LOADERS.includes(loader as InstanceLoader)
}

async function getEssentialProjectId(): Promise<string> {
	const project = (await get_project(ESSENTIAL_PROJECT_SLUG, 'must_revalidate')) as {
		id: string
	} | null
	if (!project?.id) {
		throw new Error('Essential could not be found on Modrinth.')
	}
	return project.id
}

export async function hasEssential(instanceId: string): Promise<boolean> {
	const [projectId, items] = await Promise.all([
		getEssentialProjectId(),
		get_content_items(instanceId),
	])
	return items.some((item) => item.project?.id === projectId)
}

/**
 * Installs the newest Essential version compatible with the instance, including
 * required dependencies such as Fabric API. Returns `false` when Essential is
 * already installed.
 */
export async function installEssential(instanceId: string): Promise<boolean> {
	const projectId = await getEssentialProjectId()
	const items = await get_content_items(instanceId)
	if (items.some((item) => item.project?.id === projectId)) {
		return false
	}
	await install_project_with_dependencies(instanceId, {
		project_id: projectId,
		content_type: 'mod',
	})
	return true
}

function readAutoInstall(): boolean {
	try {
		return localStorage.getItem(AUTO_INSTALL_STORAGE_KEY) === 'true'
	} catch {
		return false
	}
}

const autoInstallEssential = ref(readAutoInstall())

watch(autoInstallEssential, (enabled) => {
	try {
		localStorage.setItem(AUTO_INSTALL_STORAGE_KEY, String(enabled))
	} catch {
		// Storage can be unavailable; the toggle then only lasts for this session.
	}
})

export function useEssentialAutoInstall() {
	return autoInstallEssential
}
