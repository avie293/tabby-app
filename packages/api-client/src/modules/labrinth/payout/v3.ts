import { AbstractModule } from '../../../core/abstract-module'
import type { Override, RawDecimal } from '../../../utils/types'
import type { Labrinth } from '../types'

type RawPayoutBalance = Override<
	Labrinth.Payout.v3.PayoutBalance,
	{
		available: RawDecimal
		withdrawn_lifetime: RawDecimal
		withdrawn_ytd: RawDecimal
		pending: RawDecimal
		dates: Record<string, RawDecimal>
	}
>

type RawTransactionItem =
	| Override<
			Extract<Labrinth.Payout.v3.TransactionItem, { type: 'withdrawal' }>,
			{
				amount: RawDecimal
				fee: RawDecimal | null
			}
	  >
	| Override<
			Extract<Labrinth.Payout.v3.TransactionItem, { type: 'payout_available' }>,
			{
				amount: RawDecimal
			}
	  >

export class LabrinthPayoutV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_payout_v3'
	}

	/**
	 * Get the authenticated user's payout balance
	 *
	 * @returns Promise resolving to the user's payout balance
	 */
	public async getBalance(): Promise<Labrinth.Payout.v3.PayoutBalance> {
		const balance = await this.client.request<RawPayoutBalance>('/payout/balance', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})

		return {
			...balance,
			available: Number(balance.available),
			withdrawn_lifetime: Number(balance.withdrawn_lifetime),
			withdrawn_ytd: Number(balance.withdrawn_ytd),
			pending: Number(balance.pending),
			dates: Object.fromEntries(
				Object.entries(balance.dates).map(([date, amount]) => [date, Number(amount)]),
			),
		}
	}

	/**
	 * Get the authenticated user's transaction history (withdrawals and payouts)
	 *
	 * @returns Promise resolving to an array of transaction items
	 */
	public async getHistory(): Promise<Labrinth.Payout.v3.TransactionItem[]> {
		const history = await this.client.request<RawTransactionItem[]>('/payout/history', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})

		return history.map((transaction) => {
			if (transaction.type === 'withdrawal') {
				return {
					...transaction,
					amount: Number(transaction.amount),
					fee: transaction.fee === null ? null : Number(transaction.fee),
				}
			}

			return {
				...transaction,
				amount: Number(transaction.amount),
			}
		})
	}

	/**
	 * Get available payout methods, optionally filtered by country
	 *
	 * @param country - Optional ISO country code to filter methods by supported countries
	 * @returns Promise resolving to an array of payout methods
	 */
	public async getMethods(country?: string): Promise<Labrinth.Payout.v3.PayoutMethod[]> {
		return this.client.request<Labrinth.Payout.v3.PayoutMethod[]>('/payout/methods', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: country ? { country } : undefined,
		})
	}

	/**
	 * Cancel a pending payout
	 *
	 * @param id - The payout ID to cancel
	 */
	/**
	 * Calculate the fee and net amount of a withdrawal before requesting it
	 *
	 * @example
	 * ```typescript
	 * const fees = await client.labrinth.payout_v3.calculateFees({ amount: 10, method: 'paypal', method_id: 'paypal_us' })
	 * ```
	 */
	public async calculateFees(
		request: Labrinth.Payout.v3.WithdrawRequest,
	): Promise<Labrinth.Payout.v3.WithdrawalFees> {
		const fees = await this.client.request<{
			net_usd: RawDecimal | null
			fee: RawDecimal | null
			exchange_rate: RawDecimal | null
		}>('/payout/fees', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: request,
		})
		return {
			net_usd: Number(fees.net_usd ?? 0),
			fee: Number(fees.fee ?? 0),
			exchange_rate: fees.exchange_rate === null ? null : Number(fees.exchange_rate),
		}
	}

	/**
	 * Request a withdrawal of the available balance
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.payout_v3.withdraw({ amount: 10, method: 'paypal', method_id: 'paypal_us' })
	 * ```
	 */
	public async withdraw(request: Labrinth.Payout.v3.WithdrawRequest): Promise<void> {
		return this.client.request<void>('/payout', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: request,
		})
	}

	public async cancel(id: string): Promise<void> {
		return this.client.request<void>(`/payout/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
