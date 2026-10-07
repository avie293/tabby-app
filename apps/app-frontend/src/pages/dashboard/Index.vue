<script setup lang="ts">
import {
	ChartIcon,
	CurrencyIcon,
	DashboardIcon,
	LibraryIcon,
	ListIcon,
	OrganizationIcon,
} from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import type { Component } from 'vue'

import { useCurrentUser } from '@/composables/use-dashboard'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

const { formatMessage } = useVIntl()
const { query } = useCurrentUser()

const messages = defineMessages({
	dashboard: { id: 'app.dashboard.title', defaultMessage: 'Dashboard' },
	overview: { id: 'app.dashboard.nav.overview', defaultMessage: 'Overview' },
	projects: { id: 'app.dashboard.nav.projects', defaultMessage: 'Projects' },
	organizations: { id: 'app.dashboard.nav.organizations', defaultMessage: 'Organizations' },
	collections: { id: 'app.dashboard.nav.collections', defaultMessage: 'Collections' },
	analytics: { id: 'app.dashboard.nav.analytics', defaultMessage: 'Analytics' },
	revenue: { id: 'app.dashboard.nav.revenue', defaultMessage: 'Revenue' },
	creators: { id: 'app.dashboard.nav.creators', defaultMessage: 'Creators' },
	signIn: {
		id: 'app.dashboard.sign-in',
		defaultMessage: 'Sign in with your Modrinth account to see your dashboard.',
	},
})

useRootBreadcrumb({
	slot: 'root',
	id: 'dashboard',
	label: formatMessage(messages.dashboard),
	to: '/dashboard',
	visual: { type: 'icon', component: DashboardIcon },
})

type NavItem = { to: string; label: string; icon: Component; exact?: boolean } | { heading: string }

const items: NavItem[] = [
	{ to: '/dashboard', label: formatMessage(messages.overview), icon: DashboardIcon, exact: true },
	{ to: '/dashboard/collections', label: formatMessage(messages.collections), icon: LibraryIcon },
	{ heading: formatMessage(messages.creators) },
	{ to: '/dashboard/projects', label: formatMessage(messages.projects), icon: ListIcon },
	{
		to: '/dashboard/organizations',
		label: formatMessage(messages.organizations),
		icon: OrganizationIcon,
	},
	{ to: '/dashboard/analytics', label: formatMessage(messages.analytics), icon: ChartIcon },
	{ to: '/dashboard/revenue', label: formatMessage(messages.revenue), icon: CurrencyIcon },
]
</script>

<template>
	<div class="flex gap-6 p-6">
		<nav class="flex w-56 shrink-0 flex-col gap-1">
			<h2 class="m-0 mb-2 px-3 text-lg font-bold text-contrast">
				{{ formatMessage(messages.dashboard) }}
			</h2>
			<template v-for="item in items" :key="'heading' in item ? item.heading : item.to">
				<span
					v-if="'heading' in item"
					class="mt-4 px-3 text-xs font-semibold uppercase tracking-wide text-secondary"
				>
					{{ item.heading }}
				</span>
				<RouterLink
					v-else
					:to="item.to"
					class="flex items-center gap-2 rounded-xl px-3 py-2 font-semibold text-primary no-underline hover:bg-surface-3"
					:active-class="item.exact ? '' : '!bg-surface-4 !text-contrast'"
					:exact-active-class="'!bg-surface-4 !text-contrast'"
				>
					<component :is="item.icon" class="size-5" aria-hidden="true" />
					{{ item.label }}
				</RouterLink>
			</template>
		</nav>
		<main class="min-w-0 flex-1">
			<p v-if="query.isError.value" class="m-0 text-secondary">
				{{ formatMessage(messages.signIn) }}
			</p>
			<RouterView v-else-if="query.data.value" />
		</main>
	</div>
</template>
