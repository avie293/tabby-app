<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { FileIcon, PlusIcon, SaveIcon, UploadIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	Chips,
	Combobox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	MarkdownEditor,
	NewModal,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'

const props = defineProps<{ project: Labrinth.Projects.v3.Project }>()
const emit = defineEmits<{ saved: [] }>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()

const messages = defineMessages({
	create: { id: 'app.dashboard.versions.create', defaultMessage: 'Upload version' },
	edit: { id: 'app.dashboard.versions.edit', defaultMessage: 'Edit version' },
	files: { id: 'app.dashboard.versions.files', defaultMessage: 'Files' },
	addFiles: { id: 'app.dashboard.versions.add-files', defaultMessage: 'Add files' },
	primary: { id: 'app.dashboard.versions.primary', defaultMessage: 'Primary' },
	newFile: { id: 'app.dashboard.versions.new-file', defaultMessage: 'New' },
	versionNumber: { id: 'app.dashboard.versions.number', defaultMessage: 'Version number' },
	name: { id: 'app.dashboard.versions.name', defaultMessage: 'Name (optional)' },
	channel: { id: 'app.dashboard.versions.channel', defaultMessage: 'Release channel' },
	release: { id: 'app.dashboard.versions.release', defaultMessage: 'Release' },
	beta: { id: 'app.dashboard.versions.beta', defaultMessage: 'Beta' },
	alpha: { id: 'app.dashboard.versions.alpha', defaultMessage: 'Alpha' },
	status: { id: 'app.dashboard.versions.status', defaultMessage: 'Status' },
	listed: { id: 'app.dashboard.versions.listed', defaultMessage: 'Listed' },
	unlisted: { id: 'app.dashboard.versions.unlisted', defaultMessage: 'Unlisted' },
	draft: { id: 'app.dashboard.versions.draft', defaultMessage: 'Draft' },
	archived: { id: 'app.dashboard.versions.archived', defaultMessage: 'Archived' },
	featured: { id: 'app.dashboard.versions.featured', defaultMessage: 'Featured version' },
	loaders: { id: 'app.dashboard.versions.loaders', defaultMessage: 'Loaders' },
	gameVersions: {
		id: 'app.dashboard.versions.game-versions',
		defaultMessage: 'Minecraft versions',
	},
	searchVersions: { id: 'app.dashboard.versions.search', defaultMessage: 'Filter versions' },
	snapshots: { id: 'app.dashboard.versions.snapshots', defaultMessage: 'Show snapshots' },
	changelog: { id: 'app.dashboard.versions.changelog', defaultMessage: 'Changelog' },
	dependencies: { id: 'app.dashboard.versions.dependencies', defaultMessage: 'Dependencies' },
	dependencyPlaceholder: {
		id: 'app.dashboard.versions.dependency-placeholder',
		defaultMessage: 'Project slug or ID',
	},
	addDependency: { id: 'app.dashboard.versions.add-dependency', defaultMessage: 'Add dependency' },
	required: { id: 'app.dashboard.versions.dependency.required', defaultMessage: 'Required' },
	optional: { id: 'app.dashboard.versions.dependency.optional', defaultMessage: 'Optional' },
	incompatible: {
		id: 'app.dashboard.versions.dependency.incompatible',
		defaultMessage: 'Incompatible',
	},
	embedded: { id: 'app.dashboard.versions.dependency.embedded', defaultMessage: 'Embedded' },
	uploading: { id: 'app.dashboard.versions.uploading', defaultMessage: 'Uploading…' },
	save: { id: 'app.dashboard.save', defaultMessage: 'Save changes' },
})

type Channel = Labrinth.Versions.v3.VersionChannel
type Status = 'listed' | 'unlisted' | 'draft' | 'archived'
type DependencyType = Labrinth.Versions.v2.DependencyType
type DependencyRow = {
	project: string
	projectId?: string
	versionId?: string
	fileName?: string
	type: DependencyType
}

const channels: Channel[] = ['release', 'beta', 'alpha']
const statuses: Status[] = ['listed', 'unlisted', 'draft', 'archived']
const dependencyTypes: DependencyType[] = ['required', 'optional', 'incompatible', 'embedded']

const modal = ref<InstanceType<typeof NewModal>>()
const editing = ref<Labrinth.Versions.v3.Version | null>(null)
const files = ref<File[]>([])
const versionNumber = ref('')
const name = ref('')
const channel = ref<Channel>('release')
const status = ref<Status>('listed')
const featured = ref(false)
const changelog = ref('')
const loaders = ref<string[]>([])
const gameVersions = ref<string[]>([])
const dependencies = ref<DependencyRow[]>([])
const versionFilter = ref('')
const showSnapshots = ref(false)
const saving = ref(false)
const fileInput = ref<HTMLInputElement>()

type LoaderTag = { name: string; supported_project_types: string[] }
type GameVersionTag = { version: string; version_type: string }

const loadersQuery = useQuery({
	queryKey: ['tags', 'loaders'],
	queryFn: async () => (await get_loaders()) as LoaderTag[],
})
const gameVersionsQuery = useQuery({
	queryKey: ['tags', 'game-versions'],
	queryFn: async () => (await get_game_versions()) as GameVersionTag[],
})

const isModpack = computed(
	() =>
		props.project.project_types.includes('modpack') ||
		files.value[0]?.name.toLowerCase().endsWith('.mrpack') ||
		editing.value?.loaders.includes('mrpack'),
)

const loaderOptions = computed(() => {
	const types = isModpack.value ? ['mod'] : (props.project.project_types as string[])
	return (loadersQuery.data.value ?? [])
		.filter(
			(loader) =>
				loader.name !== 'mrpack' &&
				(types.length === 0 || loader.supported_project_types.some((type) => types.includes(type))),
		)
		.map((loader) => loader.name)
})

const gameVersionOptions = computed(() =>
	(gameVersionsQuery.data.value ?? [])
		.filter((version) => showSnapshots.value || version.version_type === 'release')
		.map((version) => version.version)
		.filter((version) => version.includes(versionFilter.value.trim())),
)

function toggle(list: string[], value: string) {
	const index = list.indexOf(value)
	if (index === -1) list.push(value)
	else list.splice(index, 1)
}

function show(version?: Labrinth.Versions.v3.Version) {
	editing.value = version ?? null
	files.value = []
	versionNumber.value = version?.version_number ?? ''
	name.value = version && version.name !== version.version_number ? version.name : ''
	channel.value = version?.version_type ?? 'release'
	status.value = (statuses as string[]).includes(version?.status ?? '')
		? (version!.status as Status)
		: 'listed'
	featured.value = version?.featured ?? false
	changelog.value = version?.changelog ?? ''
	loaders.value = version
		? [...(version.loaders.includes('mrpack') ? (version.mrpack_loaders ?? []) : version.loaders)]
		: []
	gameVersions.value = [...(version?.game_versions ?? [])]
	dependencies.value = (version?.dependencies ?? []).map((dependency) => ({
		project: dependency.project_id ?? dependency.file_name ?? '',
		projectId: dependency.project_id,
		versionId: dependency.version_id,
		fileName: dependency.file_name,
		type: dependency.dependency_type,
	}))
	versionFilter.value = ''
	modal.value?.show()
}

function onFiles() {
	files.value.push(...Array.from(fileInput.value?.files ?? []))
	if (fileInput.value) fileInput.value.value = ''
	if (!versionNumber.value && files.value[0]) {
		const match = files.value[0].name.match(
			/(\d+\.\d+(?:\.\d+)?(?:[-+][\w.+-]*)?)\.(?:jar|zip|mrpack)$/i,
		)
		if (match) versionNumber.value = match[1]
	}
}

/** Turns typed slugs into project ids, which the API requires. */
async function resolveDependencies(): Promise<Labrinth.Versions.v3.Dependency[]> {
	return await Promise.all(
		dependencies.value
			.filter((dependency) => dependency.project.trim() || dependency.versionId)
			.map(async (dependency) => {
				if (dependency.fileName && !dependency.projectId) {
					return { file_name: dependency.fileName, dependency_type: dependency.type }
				}
				const projectId =
					dependency.projectId && dependency.project === dependency.projectId
						? dependency.projectId
						: (await client.labrinth.projects_v2.get(dependency.project.trim())).id
				return {
					project_id: projectId,
					version_id: dependency.versionId,
					dependency_type: dependency.type,
				}
			}),
	)
}

const canSave = computed(
	() =>
		!saving.value &&
		versionNumber.value.trim() !== '' &&
		gameVersions.value.length > 0 &&
		loaders.value.length > 0 &&
		(editing.value !== null || files.value.length > 0),
)

async function save() {
	saving.value = true
	try {
		const resolved = await resolveDependencies()
		const projectType = isModpack.value ? 'modpack' : null
		if (editing.value) {
			await client.labrinth.versions_v3.modifyVersion(editing.value.id, {
				version_number: versionNumber.value.trim(),
				name: name.value.trim() || versionNumber.value.trim(),
				changelog: changelog.value,
				version_type: channel.value,
				status: status.value,
				featured: featured.value,
				game_versions: gameVersions.value,
				dependencies: resolved,
				...(projectType === 'modpack'
					? { loaders: ['mrpack'], mrpack_loaders: loaders.value }
					: { loaders: loaders.value }),
			})
			if (files.value.length > 0) {
				await client.labrinth.versions_v3.addFilesToVersion(
					editing.value.id,
					files.value.map((file) => ({ file })),
				).promise
			}
		} else {
			await client.labrinth.versions_v3.createVersion(
				{
					project_id: props.project.id,
					version_number: versionNumber.value.trim(),
					name: name.value.trim() || versionNumber.value.trim(),
					changelog: changelog.value,
					version_type: channel.value,
					featured: featured.value,
					game_versions: gameVersions.value,
					loaders: loaders.value,
					dependencies: resolved,
				},
				files.value.map((file) => ({ file })),
				projectType,
			).promise
		}
		modal.value?.hide()
		emit('saved')
	} catch (error) {
		handleError(error as Error)
	} finally {
		saving.value = false
	}
}

defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(editing ? messages.edit : messages.create)"
		max-width="56rem"
		scrollable
	>
		<div class="flex flex-col gap-5">
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.files) }}</span>
				<input ref="fileInput" type="file" multiple class="hidden" @change="onFiles" />
				<div
					v-for="file in editing?.files ?? []"
					:key="file.url"
					class="flex items-center gap-2 rounded-xl bg-surface-2 px-3 py-2"
				>
					<FileIcon class="size-4 shrink-0" aria-hidden="true" />
					<span class="flex-1 truncate">{{ file.filename }}</span>
					<span v-if="file.primary" class="text-sm text-secondary">
						{{ formatMessage(messages.primary) }}
					</span>
				</div>
				<div
					v-for="(file, index) in files"
					:key="`${file.name}-${index}`"
					class="flex items-center gap-2 rounded-xl bg-surface-2 px-3 py-2"
				>
					<FileIcon class="size-4 shrink-0" aria-hidden="true" />
					<span class="flex-1 truncate">{{ file.name }}</span>
					<span class="text-sm text-secondary">
						{{ formatMessage(index === 0 && !editing ? messages.primary : messages.newFile) }}
					</span>
					<Button @click="files.splice(index, 1)">
						<XIcon aria-hidden="true" />
					</Button>
				</div>
				<div>
					<Button @click="fileInput?.click()">
						<UploadIcon aria-hidden="true" />
						{{ formatMessage(messages.addFiles) }}
					</Button>
				</div>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{
						formatMessage(messages.versionNumber)
					}}</span>
					<Input v-model="versionNumber" :maxlength="32" placeholder="1.0.0" />
				</label>
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.name) }}</span>
					<Input v-model="name" :maxlength="64" />
				</label>
			</div>

			<div class="flex flex-wrap gap-6">
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.channel) }}</span>
					<Chips
						v-model="channel"
						:items="channels"
						:format-label="(item: Channel) => formatMessage(messages[item])"
						:capitalize="false"
					/>
				</div>
				<div v-if="editing" class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.status) }}</span>
					<Chips
						v-model="status"
						:items="statuses"
						:format-label="(item: Status) => formatMessage(messages[item])"
						:capitalize="false"
					/>
				</div>
			</div>
			<Checkbox v-model="featured" :label="formatMessage(messages.featured)" />

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.loaders) }}</span>
				<div class="flex flex-wrap gap-2">
					<button
						v-for="loader in loaderOptions"
						:key="loader"
						class="cursor-pointer rounded-full border border-solid px-3 py-1 font-semibold capitalize"
						:class="
							loaders.includes(loader)
								? 'border-brand bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-2 text-primary'
						"
						@click="toggle(loaders, loader)"
					>
						{{ loader }}
					</button>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-4">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.gameVersions) }}
						<span class="font-normal text-secondary">({{ gameVersions.length }})</span>
					</span>
					<label class="flex items-center gap-2 text-sm text-secondary">
						{{ formatMessage(messages.snapshots) }}
						<Toggle v-model="showSnapshots" />
					</label>
				</div>
				<Input v-model="versionFilter" :placeholder="formatMessage(messages.searchVersions)" />
				<div class="flex max-h-48 flex-wrap gap-2 overflow-y-auto rounded-xl bg-surface-2 p-2">
					<button
						v-for="version in gameVersionOptions"
						:key="version"
						class="cursor-pointer rounded-full border border-solid px-3 py-1 text-sm font-semibold"
						:class="
							gameVersions.includes(version)
								? 'border-brand bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-3 text-primary'
						"
						@click="toggle(gameVersions, version)"
					>
						{{ version }}
					</button>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.changelog) }}</span>
				<MarkdownEditor v-model="changelog" :min-height="160" />
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.dependencies) }}</span>
				<div v-for="(dependency, index) in dependencies" :key="index" class="flex gap-2">
					<Input
						v-model="dependency.project"
						class="flex-1"
						:placeholder="formatMessage(messages.dependencyPlaceholder)"
					/>
					<Combobox
						v-model="dependency.type"
						class="w-44"
						:options="
							dependencyTypes.map((type) => ({ value: type, label: formatMessage(messages[type]) }))
						"
					/>
					<Button @click="dependencies.splice(index, 1)">
						<XIcon aria-hidden="true" />
					</Button>
				</div>
				<div>
					<Button @click="dependencies.push({ project: '', type: 'required' })">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.addDependency) }}
					</Button>
				</div>
			</div>
		</div>
		<template #actions>
			<div class="flex justify-end">
				<Button color="brand" type="colored" :disabled="!canSave" @click="save">
					<SaveIcon v-if="!saving" aria-hidden="true" />
					{{
						formatMessage(saving ? messages.uploading : editing ? messages.save : messages.create)
					}}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
