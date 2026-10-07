import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

export const dashboardKeys = {
	all: ['dashboard'] as const,
	me: ['dashboard', 'me'] as const,
	projects: (userId: string) => ['dashboard', 'projects', userId] as const,
	organizations: (userId: string) => ['dashboard', 'organizations', userId] as const,
	organization: (id: string) => ['dashboard', 'organization', id] as const,
	collections: (userId: string) => ['dashboard', 'collections', userId] as const,
	collection: (id: string) => ['dashboard', 'collection', id] as const,
}

/** The signed in Modrinth user, or an error when nobody is signed in. */
export function useCurrentUser() {
	const client = injectModrinthClient()
	const query = useQuery({
		queryKey: dashboardKeys.me,
		queryFn: () => client.labrinth.users_v3.getAuthenticated(),
		retry: false,
	})
	return {
		query,
		user: computed(() => query.data.value),
		userId: computed(() => query.data.value?.id),
	}
}

/** The file extension Modrinth expects for an uploaded icon. */
export function imageExtension(file: File) {
	const extension = file.name.split('.').pop()?.toLowerCase()
	return extension === 'jpeg' ? 'jpg' : (extension ?? 'png')
}

/** Turns a name into a URL slug, e.g. "My Org" into "my-org". */
export function slugify(value: string) {
	return value
		.toLowerCase()
		.normalize('NFKD')
		.replace(/[̀-ͯ]/g, '')
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
		.slice(0, 64)
}

export function formatNumber(value: number) {
	return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(
		value,
	)
}
