const FEED_URL = 'https://raw.githubusercontent.com/avie293/tabby-app/main/news/articles.json'
const MODRINTH_API = 'https://api.modrinth.com/v2'

/** Versions uploaded closer together than this belong to the same update. */
const UPDATE_WINDOW_MS = 6 * 60 * 60 * 1000
const UPDATES_PER_PROJECT = 3

export type NewsArticle = {
	type: 'article'
	key: string
	title: string
	summary: string
	thumbnail: string
	date: string
	path: string
}

export type ProjectUpdate = {
	type: 'project_update'
	key: string
	projectId: string
	projectTitle: string
	/** The version shared by every file of the update, e.g. 1.1.0. */
	version: string | null
	thumbnail: string
	date: string
	gameVersions: string[]
	loaders: string[]
	changelogs: string[]
}

export type NewsItem = NewsArticle | ProjectUpdate

type Feed = {
	articles?: { title: string; summary: string; thumbnail: string; date: string; link: string }[]
	modrinth?: {
		user: string
		default_thumbnail: string
		thumbnails?: Record<string, string>
	}
}

type ModrinthProject = { id: string; title: string; loaders: string[] }

type ModrinthVersion = {
	id: string
	version_number: string
	date_published: string
	game_versions: string[]
	loaders: string[]
	changelog: string | null
}

const LOADER_NAMES: Record<string, string> = {
	fabric: 'Fabric',
	forge: 'Forge',
	neoforge: 'NeoForge',
	quilt: 'Quilt',
	datapack: 'Datapack',
}

export function loaderName(loader: string) {
	return LOADER_NAMES[loader] ?? loader.charAt(0).toUpperCase() + loader.slice(1)
}

/** Sorts Minecraft versions like 1.21.1 and 26.2 numerically. */
function compareGameVersions(a: string, b: string) {
	const left = a.split(/[.-]/).map(Number)
	const right = b.split(/[.-]/).map(Number)
	for (let index = 0; index < Math.max(left.length, right.length); index++) {
		const difference = (left[index] ?? 0) - (right[index] ?? 0)
		if (Number.isNaN(difference)) return a.localeCompare(b)
		if (difference !== 0) return difference
	}
	return 0
}

/** Shows many versions as a range, e.g. 26.1 – 26.3. */
export function formatGameVersions(versions: string[]) {
	if (versions.length <= 3) return versions.join(', ')
	return `${versions[0]} – ${versions[versions.length - 1]}`
}

/** The version every file of an update shares, if there is one. */
function sharedVersion(versions: ModrinthVersion[]) {
	for (const separator of ['+', '-']) {
		const bases = new Set(versions.map((version) => version.version_number.split(separator)[0]))
		if (bases.size === 1) return [...bases][0]
	}
	return null
}

/**
 * Groups the versions of a project into updates. A release usually uploads
 * one file per Minecraft version and loader, which should show up as a
 * single update.
 */
function groupUpdates(project: ModrinthProject, versions: ModrinthVersion[], thumbnail: string) {
	const sorted = [...versions].sort(
		(a, b) => Date.parse(b.date_published) - Date.parse(a.date_published),
	)
	const groups: ModrinthVersion[][] = []
	for (const version of sorted) {
		const group = groups[groups.length - 1]
		const oldest = group?.[group.length - 1]
		if (
			group &&
			oldest &&
			Date.parse(oldest.date_published) - Date.parse(version.date_published) <= UPDATE_WINDOW_MS
		) {
			group.push(version)
		} else {
			groups.push([version])
		}
	}
	return groups.slice(0, UPDATES_PER_PROJECT).map(
		(group): ProjectUpdate => ({
			type: 'project_update',
			key: `${project.id}-${group[0].id}`,
			projectId: project.id,
			projectTitle: project.title,
			version: sharedVersion(group),
			thumbnail,
			date: group[0].date_published,
			gameVersions: [...new Set(group.flatMap((version) => version.game_versions))].sort(
				compareGameVersions,
			),
			loaders: [...new Set(group.flatMap((version) => version.loaders))],
			changelogs: [
				...new Set(
					group.map((version) => version.changelog?.trim() ?? '').filter((text) => text !== ''),
				),
			],
		}),
	)
}

async function fetchJson<T>(url: string): Promise<T> {
	const response = await fetch(url)
	if (!response.ok) throw new Error(`${url} returned ${response.status}`)
	return (await response.json()) as T
}

async function fetchProjectUpdates(config: NonNullable<Feed['modrinth']>) {
	const projects = await fetchJson<ModrinthProject[]>(
		`${MODRINTH_API}/user/${encodeURIComponent(config.user)}/projects`,
	)
	const titles = projects.map((project) => project.title)
	const updates = await Promise.all(
		projects.map(async (project) => {
			const versions = await fetchJson<ModrinthVersion[]>(
				`${MODRINTH_API}/project/${project.id}/version`,
			)
			// Two projects with the same name, like a mod and its datapack, need telling apart.
			const duplicate = titles.filter((title) => title === project.title).length > 1
			const named =
				duplicate && project.loaders.includes('datapack')
					? { ...project, title: `${project.title} (Datapack)` }
					: project
			return groupUpdates(
				named,
				versions,
				config.thumbnails?.[project.id] ?? config.default_thumbnail,
			)
		}),
	)
	return updates.flat()
}

/** Tabbyapp news and updates of our Modrinth projects, newest first. */
export async function fetchNews(limit: number): Promise<NewsItem[]> {
	const feed = await fetchJson<Feed>(FEED_URL)
	const articles: NewsItem[] = (feed.articles ?? []).map((article) => ({
		type: 'article',
		key: article.link,
		title: article.title,
		summary: article.summary,
		thumbnail: article.thumbnail,
		date: article.date,
		path: article.link,
	}))
	let updates: NewsItem[] = []
	if (feed.modrinth) {
		try {
			updates = await fetchProjectUpdates(feed.modrinth)
		} catch (error) {
			console.error('Failed to fetch project updates', error)
		}
	}
	return [...articles, ...updates]
		.sort((a, b) => Date.parse(b.date) - Date.parse(a.date))
		.slice(0, limit)
}
