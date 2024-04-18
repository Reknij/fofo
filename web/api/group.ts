import type { GetGroupsQuery, Group, GroupToCreateUpdate, SetGroupBody } from "~/models/group";
import type { GetDatasExtended, VerificationTargetWrapper } from "~/models/util";
import { useApiFetch } from "./customFetch";

export function createGroup(body: VerificationTargetWrapper<GroupToCreateUpdate>) {
    return useApiFetch<Group>('/group', {
        method: 'post',
        body,
    })
}

export function updateGroup(id: number, body: VerificationTargetWrapper<GroupToCreateUpdate>) {
    return useApiFetch<Group>(`/group/${id}`, {
        method: 'put',
        body,
    })
}

export function getGroup(id: number) {
    return useApiFetch<Group>(`/group/${id}`);
}

export function getGroups(query: GetGroupsQuery) {
    return useApiFetch<GetDatasExtended<Group>>(`/groups`, {
        query,
    });
}

export function setGroupStatus(id: number, body: SetGroupBody) {
    return useApiFetch(`/group_status/${id}`, {
        method: 'put',
        body,
    });
}