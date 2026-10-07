<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, ExternalIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	Checkbox,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref, watch } from 'vue'

const props = defineProps<{ balance: Labrinth.Payout.v3.PayoutBalance; defaultEmail?: string | null }>()
const emit = defineEmits<{ withdrawn: [] }>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()

/** Without a tax form, Modrinth allows withdrawing just under this much per year. */
const TAX_THRESHOLD = 600
const WEBSITE_REVENUE_URL = 'https://modrinth.com/dashboard/revenue'

const messages = defineMessages({
	header: { id: 'app.dashboard.withdraw.header', defaultMessage: 'Withdraw' },
	country: { id: 'app.dashboard.withdraw.country', defaultMessage: 'Country code' },
	method: { id: 'app.dashboard.withdraw.method', defaultMessage: 'Payout method' },
	amount: { id: 'app.dashboard.withdraw.amount', defaultMessage: 'Amount (USD)' },
	limits: {
		id: 'app.dashboard.withdraw.limits',
		defaultMessage: 'Between {min} and {max}',
	},
	email: { id: 'app.dashboard.withdraw.email', defaultMessage: 'Delivery email' },
	fee: { id: 'app.dashboard.withdraw.fee', defaultMessage: 'Fee' },
	net: { id: 'app.dashboard.withdraw.net', defaultMessage: 'You receive' },
	terms: {
		id: 'app.dashboard.withdraw.terms',
		defaultMessage: 'I agree to the Modrinth Rewards Program terms.',
	},
	submit: { id: 'app.dashboard.withdraw.submit', defaultMessage: 'Withdraw {amount}' },
	done: {
		id: 'app.dashboard.withdraw.done',
		defaultMessage: 'Your withdrawal of {amount} was requested. It shows up in your transactions.',
	},
	noMethods: {
		id: 'app.dashboard.withdraw.no-methods',
		defaultMessage: 'No payout methods are available for this country.',
	},
	websiteOnly: {
		id: 'app.dashboard.withdraw.website-only',
		defaultMessage:
			'Bank transfers and crypto need an identity check that only the Modrinth website can do.',
	},
	taxForm: {
		id: 'app.dashboard.withdraw.tax-form',
		defaultMessage:
			'You can withdraw up to {amount} more this year without a tax form. To withdraw more, fill out the tax form on the Modrinth website.',
	},
	openWebsite: {
		id: 'app.dashboard.withdraw.open-website',
		defaultMessage: 'Open Modrinth website',
	},
})

function usd(value: number) {
	return new Intl.NumberFormat(undefined, { style: 'currency', currency: 'USD' }).format(value)
}

const modal = ref<InstanceType<typeof NewModal>>()
const country = ref('US')
const methodId = ref<string>()
const amount = ref('')
const email = ref('')
const agreed = ref(false)
const submitting = ref(false)
const result = ref<number | null>(null)

const formComplete = computed(() => props.balance.form_completion_status === 'complete')
const maxAllowed = computed(() =>
	formComplete.value
		? props.balance.available
		: Math.max(
				0,
				Math.min(props.balance.available, TAX_THRESHOLD - 0.01 - props.balance.withdrawn_ytd),
			),
)

const methodsQuery = useQuery({
	queryKey: computed(() => ['dashboard', 'payout-methods', country.value]),
	queryFn: () => client.labrinth.payout_v3.getMethods(country.value),
	enabled: () => /^[A-Z]{2}$/.test(country.value),
})
/** MuralPay needs a KYC flow that only exists on the website. */
const methods = computed(() =>
	(methodsQuery.data.value ?? []).filter((method) => method.type !== 'muralpay'),
)
const hasWebsiteOnlyMethods = computed(() =>
	(methodsQuery.data.value ?? []).some((method) => method.type === 'muralpay'),
)
const method = computed(() => methods.value.find((value) => value.id === methodId.value))

const limits = computed(() => {
	const standard = method.value?.interval.standard
	return {
		min: standard?.min ?? 0.01,
		max: Math.min(standard?.max ?? Infinity, maxAllowed.value),
	}
})
const fixedValues = computed(() => method.value?.interval.fixed?.values ?? null)
const amountValue = computed(() => Math.round(Number(amount.value) * 100) / 100)
const amountValid = computed(() =>
	fixedValues.value
		? fixedValues.value.includes(amountValue.value) && amountValue.value <= maxAllowed.value
		: amountValue.value >= limits.value.min && amountValue.value <= limits.value.max,
)

function request(): Labrinth.Payout.v3.WithdrawRequest {
	return {
		amount: amountValue.value,
		method: method.value!.type,
		method_id: method.value!.id,
		...(method.value!.type === 'tremendous'
			? { method_details: { delivery_email: email.value.trim() } }
			: {}),
	}
}

const feesQuery = useQuery({
	queryKey: computed(() => [
		'dashboard',
		'payout-fees',
		methodId.value,
		amountValue.value,
		email.value,
	]),
	queryFn: () => client.labrinth.payout_v3.calculateFees(request()),
	enabled: () =>
		!!method.value &&
		amountValid.value &&
		(method.value.type !== 'tremendous' || email.value.includes('@')),
	retry: false,
})

watch(methodId, () => {
	if (fixedValues.value?.length) amount.value = String(fixedValues.value[0])
})

async function submit() {
	submitting.value = true
	try {
		await client.labrinth.payout_v3.withdraw(request())
		result.value = amountValue.value
		emit('withdrawn')
	} catch (error) {
		handleError(error as Error)
	} finally {
		submitting.value = false
	}
}

async function show() {
	result.value = null
	methodId.value = undefined
	amount.value = ''
	agreed.value = false
	email.value = props.defaultEmail ?? ''
	modal.value?.show()
	country.value = (await client.labrinth.geoip.getCountry().catch(() => undefined)) ?? 'US'
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="38rem">
		<div v-if="result !== null" class="flex flex-col items-center gap-3 py-6 text-center">
			<CheckIcon class="size-10 text-green" aria-hidden="true" />
			<p class="m-0 text-primary">{{ formatMessage(messages.done, { amount: usd(result) }) }}</p>
		</div>
		<div v-else class="flex flex-col gap-4">
			<Admonition v-if="!formComplete" type="info">
				{{ formatMessage(messages.taxForm, { amount: usd(maxAllowed) }) }}
				<template #actions>
					<Button @click="openUrl(WEBSITE_REVENUE_URL)">
						<ExternalIcon aria-hidden="true" />
						{{ formatMessage(messages.openWebsite) }}
					</Button>
				</template>
			</Admonition>
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.country) }}</span>
				<Input
					:model-value="country"
					class="w-24"
					:maxlength="2"
					@update:model-value="country = String($event).toUpperCase()"
				/>
			</label>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.method) }}</span>
				<div v-if="methodsQuery.isFetching.value" class="flex justify-center py-4">
					<SpinnerIcon class="size-6 animate-spin" aria-hidden="true" />
				</div>
				<p v-else-if="methods.length === 0" class="m-0 text-secondary">
					{{ formatMessage(messages.noMethods) }}
				</p>
				<div v-else class="grid max-h-64 grid-cols-2 gap-2 overflow-y-auto">
					<button
						v-for="option in methods"
						:key="option.id"
						class="flex cursor-pointer items-center gap-2 rounded-xl border border-solid px-3 py-2 text-left"
						:class="
							methodId === option.id
								? 'border-brand bg-brand-highlight text-contrast'
								: 'border-surface-5 bg-surface-2 text-primary'
						"
						@click="methodId = option.id"
					>
						<img
							v-if="option.image_logo_url || option.image_url"
							:src="option.image_logo_url ?? option.image_url ?? ''"
							alt=""
							class="size-8 rounded object-contain"
						/>
						<span class="truncate font-semibold">{{ option.name }}</span>
					</button>
				</div>
				<p v-if="hasWebsiteOnlyMethods" class="m-0 text-sm text-secondary">
					{{ formatMessage(messages.websiteOnly) }}
				</p>
			</div>
			<template v-if="method">
				<label class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.amount) }}</span>
					<div v-if="fixedValues" class="flex flex-wrap gap-2">
						<button
							v-for="value in fixedValues"
							:key="value"
							class="cursor-pointer rounded-full border border-solid px-3 py-1 font-semibold"
							:class="
								amountValue === value
									? 'border-brand bg-brand-highlight text-brand'
									: 'border-surface-5 bg-surface-2 text-primary'
							"
							:disabled="value > maxAllowed"
							@click="amount = String(value)"
						>
							{{ usd(value) }}
						</button>
					</div>
					<template v-else>
						<Input v-model="amount" type="number" class="w-40" placeholder="0.00" />
						<span class="text-sm text-secondary">
							{{
								formatMessage(messages.limits, {
									min: usd(limits.min),
									max: usd(limits.max),
								})
							}}
						</span>
					</template>
				</label>
				<label v-if="method.type === 'tremendous'" class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.email) }}</span>
					<Input v-model="email" type="email" />
				</label>
				<div v-if="feesQuery.data.value" class="flex flex-col gap-1 rounded-xl bg-surface-2 p-3">
					<div class="flex justify-between">
						<span>{{ formatMessage(messages.fee) }}</span>
						<span>{{ usd(feesQuery.data.value.fee) }}</span>
					</div>
					<div class="flex justify-between font-semibold text-contrast">
						<span>{{ formatMessage(messages.net) }}</span>
						<span>{{ usd(feesQuery.data.value.net_usd) }}</span>
					</div>
				</div>
				<Checkbox v-model="agreed" :label="formatMessage(messages.terms)" />
			</template>
		</div>
		<template #actions>
			<div v-if="result === null" class="flex justify-end">
				<Button
					color="brand"
					type="colored"
					:disabled="!method || !amountValid || !agreed || !feesQuery.data.value || submitting"
					@click="submit"
				>
					{{ formatMessage(messages.submit, { amount: usd(amountValue || 0) }) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
