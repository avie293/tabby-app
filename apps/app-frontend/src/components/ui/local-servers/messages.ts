import { defineMessages } from '@modrinth/ui'

import type { LocalServerStatus } from '@/helpers/local-servers'

export const localServerMessages = defineMessages({
	vanilla: { id: 'app.local-servers.software.vanilla', defaultMessage: 'Vanilla' },
	fabric: { id: 'app.local-servers.software.fabric', defaultMessage: 'Fabric' },
	paper: { id: 'app.local-servers.software.paper', defaultMessage: 'Paper' },
	stopped: { id: 'app.local-servers.status.stopped', defaultMessage: 'Offline' },
	starting: { id: 'app.local-servers.status.starting', defaultMessage: 'Starting' },
	running: { id: 'app.local-servers.status.running', defaultMessage: 'Online' },
	stopping: { id: 'app.local-servers.status.stopping', defaultMessage: 'Stopping' },
	start: { id: 'app.local-servers.action.start', defaultMessage: 'Start' },
	stop: { id: 'app.local-servers.action.stop', defaultMessage: 'Stop' },
})

export const statusClasses: Record<LocalServerStatus, string> = {
	stopped: 'bg-surface-4 text-secondary',
	starting: 'bg-highlight-orange text-orange',
	running: 'bg-highlight-green text-green',
	stopping: 'bg-highlight-orange text-orange',
}
