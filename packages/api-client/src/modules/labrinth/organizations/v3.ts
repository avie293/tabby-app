import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthOrganizationsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_organizations_v3'
	}

	/**
	 * Get an organization by ID or slug
	 *
	 * @param idOrSlug - Organization ID or slug
	 * @returns Promise resolving to the organization data
	 *
	 * @example
	 * ```typescript
	 * const org = await client.labrinth.organizations_v3.get('my-org')
	 * ```
	 */
	public async get(idOrSlug: string): Promise<Labrinth.Organizations.v3.Organization> {
		return this.client.request<Labrinth.Organizations.v3.Organization>(
			`/organization/${idOrSlug}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get an organization's projects
	 *
	 * @param idOrSlug - Organization ID or slug
	 * @returns Promise resolving to the organization's projects
	 *
	 * @example
	 * ```typescript
	 * const projects = await client.labrinth.organizations_v3.getProjects('my-org')
	 * ```
	 */
	public async getProjects(idOrSlug: string): Promise<Labrinth.Projects.v3.Project[]> {
		return this.client.request<Labrinth.Projects.v3.Project[]>(
			`/organization/${idOrSlug}/projects`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get multiple organizations by their IDs
	 *
	 * @param ids - Array of organization IDs
	 * @returns Promise resolving to an array of organizations
	 *
	 * @example
	 * ```typescript
	 * const orgs = await client.labrinth.organizations_v3.getMultiple(['id1', 'id2'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Organizations.v3.Organization[]> {
		return this.client.request<Labrinth.Organizations.v3.Organization[]>(
			`/organizations?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Add a project to an organization
	 *
	 * @param idOrSlug - Organization ID or slug
	 * @param request - The project to add
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.addProject('my-org', { project_id: 'AABBCCDD' })
	 * ```
	 */
	public async addProject(
		idOrSlug: string,
		request: Labrinth.Organizations.v3.AddProjectRequest,
	): Promise<void> {
		return this.client.request(`/organization/${idOrSlug}/projects`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: request,
		})
	}

	/**
	 * Remove a project from an organization
	 *
	 * @param idOrSlug - Organization ID or slug
	 * @param projectId - Project ID to remove
	 * @param data - Request body containing the new_owner user ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.removeProject('my-org', 'proj123', { new_owner: 'user456' })
	 * ```
	 */
	public async removeProject(
		idOrSlug: string,
		projectId: string,
		data: Labrinth.Organizations.v3.RemoveProjectRequest,
	): Promise<void> {
		return this.client.request(`/organization/${idOrSlug}/projects/${projectId}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
			body: data,
		})
	}

	/**
	 * Create an organization owned by the signed in user
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.create({ name: 'My org', slug: 'my-org', description: '' })
	 * ```
	 */
	public async create(
		data: Labrinth.Organizations.v3.CreateOrganizationRequest,
	): Promise<Labrinth.Organizations.v3.Organization> {
		return this.client.request<Labrinth.Organizations.v3.Organization>(`/organization`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Edit an organization's name, slug or description
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.edit('my-org', { description: 'New description' })
	 * ```
	 */
	public async edit(
		idOrSlug: string,
		data: Labrinth.Organizations.v3.EditOrganizationRequest,
	): Promise<void> {
		return this.client.request(`/organization/${idOrSlug}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete an organization
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.delete('my-org')
	 * ```
	 */
	public async delete(idOrSlug: string): Promise<void> {
		return this.client.request(`/organization/${idOrSlug}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}

	/**
	 * Change an organization's icon
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.editIcon('my-org', iconFile, 'png')
	 * ```
	 */
	public async editIcon(idOrSlug: string, icon: Blob, ext: string): Promise<void> {
		return this.client.request(`/organization/${idOrSlug}/icon?ext=${ext}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: icon,
		})
	}

	/**
	 * Remove an organization's icon
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.organizations_v3.deleteIcon('my-org')
	 * ```
	 */
	public async deleteIcon(idOrSlug: string): Promise<void> {
		return this.client.request(`/organization/${idOrSlug}/icon`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
