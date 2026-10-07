<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CurrencyIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import WithdrawModal from '@/components/ui/dashboard/WithdrawModal.vue'
import { useCurrentUser } from '@/composables/use-dashboard'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const formatDate = useFormatDateTime({ dateStyle: 'medium' })
const { user } = useCurrentUser()

const messages = defineMessages({
	title: { id: 'app.dashboard.revenue.title', defaultMessage: 'Revenue' },
	available: { id: 'app.dashboard.revenue.available', defaultMessage: 'Available' },
	pending: { id: 'app.dashboard.revenue.pending', defaultMessage: 'Pending' },
	withdrawnYear: {
		id: 'app.dashboard.revenue.withdrawn-year',
		defaultMessage: 'Withdrawn this year',
	},
	withdrawnTotal: {
		id: 'app.dashboard.revenue.withdrawn-total',
		defaultMessage: 'Withdrawn in total',
	},
	withdraw: { id: 'app.dashboard.revenue.withdraw', defaultMessage: 'Withdraw' },
	upcoming: { id: 'app.dashboard.revenue.upcoming', defaultMessage: 'Upcoming payouts' },
	upcomingNote: {
		id: 'app.dashboard.revenue.upcoming-note',
		defaultMessage: 'Revenue becomes available about 60 days after the end of its month.',
	},
	transactions: { id: 'app.dashboard.revenue.transactions', defaultMessage: 'Transactions' },
	noTransactions: {
		id: 'app.dashboard.revenue.no-transactions',
		defaultMessage: 'No transactions yet.',
	},
	withdrawal: { id: 'app.dashboard.revenue.withdrawal', defaultMessage: 'Withdrawal via {method}' },
	income: { id: 'app.dashboard.revenue.income', defaultMessage: 'Creator rewards' },
	fee: { id: 'app.dashboard.revenue.fee', defaultMessage: 'Fee {fee}' },
	cancel: { id: 'app.dashboard.revenue.cancel', defaultMessage: 'Cancel' },
})

const statusClasses: Record<string, string> = {
	success: 'bg-highlight-green text-green',
	'in-transit': 'bg-highlight-orange text-orange',
	cancelling: 'bg-highlight-orange text-orange',
	cancelled: 'bg-surface-4 text-secondary',
	failed: 'bg-highlight-red text-red',
	unknown: 'bg-surface-4 text-secondary',
}

function usd(value: number) {
	return new Intl.NumberFormat(undefined, { style: 'currency', currency: 'USD' }).format(value)
}

const balanceQuery = useQuery({
	queryKey: ['dashboard', 'payout-balance'],
	queryFn: () => client.labrinth.payout_v3.getBalance(),
})
const historyQuery = useQuery({
	queryKey: ['dashboard', 'payout-history'],
	queryFn: () => client.labrinth.payout_v3.getHistory(),
})
const balance = computed(() => balanceQuery.data.value)

const cards = computed(() =>
	balance.value
		? [
				{ label: formatMessage(messages.available), value: balance.value.available },
				{ label: formatMessage(messages.pending), value: balance.value.pending },
				{ label: formatMessage(messages.withdrawnYear), value: balance.value.withdrawn_ytd },
				{ label: formatMessage(messages.withdrawnTotal), value: balance.value.withdrawn_lifetime },
			]
		: [],
)

const upcoming = computed(() =>
	Object.entries(balance.value?.dates ?? {})
		.filter(([date]) => Date.parse(date) > Date.now())
		.sort(([a], [b]) => Date.parse(a) - Date.parse(b)),
)

const transactions = computed(() =>
	[...(historyQuery.data.value ?? [])].sort(
		(a, b) => Date.parse(b.created) - Date.parse(a.created),
	),
)

function methodName(
	transaction: Extract<Labrinth.Payout.v3.TransactionItem, { type: 'withdrawal' }>,
) {
	return transaction.method_address ?? transaction.method_type ?? '—'
}

async function refresh() {
	await queryClient.invalidateQueries({ queryKey: ['dashboard', 'payout-balance'] })
	await queryClient.invalidateQueries({ queryKey: ['dashboard', 'payout-history'] })
}

const cancelMutation = useMutation({
	mutationFn: (id: string) => client.labrinth.payout_v3.cancel(id),
	onSettled: refresh,
	onError: (error) => handleError(error as Error),
})

const withdrawModal = ref<InstanceType<typeof WithdrawModal>>()
</script>

<template>
	<div class="flex flex-col gap-6">
		<WithdrawModal
			v-if="balance"
			ref="withdrawModal"
			:balance="balance"
			:default-email="user?.email"
			@withdrawn="refresh"
		/>
		<div class="flex items-center justify-between gap-4">
			<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>
			<Button
				color="brand"
				type="colored"
				:disabled="!balance || balance.available <= 0"
				@click="withdrawModal?.show()"
			>
				<CurrencyIcon aria-hidden="true" />
				{{ formatMessage(messages.withdraw) }}
			</Button>
		</div>
		<div class="grid grid-cols-2 gap-4 xl:grid-cols-4">
			<div
				v-for="card in cards"
				:key="card.label"
				class="flex flex-col gap-2 rounded-2xl border border-solid border-surface-4 bg-bg-raised p-4"
			>
				<span class="text-secondary">{{ card.label }}</span>
				<span class="text-2xl font-bold text-contrast">{{ usd(card.value) }}</span>
			</div>
		</div>
		<section v-if="upcoming.length > 0" class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.upcoming) }}
			</h2>
			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.upcomingNote) }}</p>
			<div
				v-for="[date, amount] in upcoming"
				:key="date"
				class="flex justify-between rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3"
			>
				<span>{{ formatDate(date) }}</span>
				<span class="font-semibold text-contrast">{{ usd(amount) }}</span>
			</div>
		</section>
		<section class="flex flex-col gap-2">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.transactions) }}
			</h2>
			<p
				v-if="!historyQuery.isPending.value && transactions.length === 0"
				class="m-0 text-secondary"
			>
				{{ formatMessage(messages.noTransactions) }}
			</p>
			<div
				v-for="(transaction, index) in transactions"
				:key="transaction.type === 'withdrawal' ? transaction.id : `income-${index}`"
				class="flex items-center gap-4 rounded-2xl border border-solid border-surface-4 bg-bg-raised px-4 py-3"
			>
				<div class="flex min-w-0 flex-1 flex-col">
					<span class="truncate font-semibold text-contrast">
						{{
							transaction.type === 'withdrawal'
								? formatMessage(messages.withdrawal, { method: methodName(transaction) })
								: formatMessage(messages.income)
						}}
					</span>
					<span class="text-sm text-secondary">
						{{ formatDate(transaction.created) }}
						<template v-if="transaction.type === 'withdrawal' && transaction.fee">
							· {{ formatMessage(messages.fee, { fee: usd(transaction.fee) }) }}
						</template>
					</span>
				</div>
				<span
					v-if="transaction.type === 'withdrawal'"
					class="rounded-full px-2 py-0.5 text-sm font-medium capitalize"
					:class="statusClasses[transaction.status]"
				>
					{{ transaction.status.replace('-', ' ') }}
				</span>
				<span
					class="w-28 text-right font-semibold"
					:class="transaction.type === 'withdrawal' ? 'text-contrast' : 'text-green'"
				>
					{{ transaction.type === 'withdrawal' ? '−' : '+' }}{{ usd(transaction.amount) }}
				</span>
				<Button
					v-if="transaction.type === 'withdrawal' && transaction.status === 'in-transit'"
					:disabled="cancelMutation.isPending.value"
					@click="cancelMutation.mutate(transaction.id)"
				>
					<XIcon aria-hidden="true" />
					{{ formatMessage(messages.cancel) }}
				</Button>
			</div>
		</section>
	</div>
</template>
