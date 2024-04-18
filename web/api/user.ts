import type { AuthAndUser, GetUsersQuery, GetUsersSort, SafeUserInfo, SetUserBody, ToLoginUser, UserInfo, UserToCreate, UserToUpdate } from "~/models/user";
import type { ApiDetailError, GetDatasExtended, VerificationTargetWrapper } from "~/models/util";
import { useApiFetch } from "./customFetch";

export function createUser(body: VerificationTargetWrapper<UserToCreate>) {
    return useApiFetch<AuthAndUser>(`/user`, {
        method: 'post',
        body,
    });
}

export function updateUser(id: number, body: VerificationTargetWrapper<UserToUpdate>) {
    return useApiFetch<UserInfo>(`/user/${id}`, {
        method: 'put',
        body,
    });
}

export function getUsers(query: GetUsersQuery) {
    return useApiFetch<GetDatasExtended<SafeUserInfo>>(`/users`, {
        query,
    });
}

export function getUser(id: number) {
    return useApiFetch<SafeUserInfo>(`/user/${id}`);
}

export function loginUser(body: VerificationTargetWrapper<ToLoginUser>) {
    return useApiFetch<AuthAndUser>(`/login_user`, {
        method: 'post',
        body,
    });
}

export function logoutUser(auth: string) {
    return useApiFetch(`/logout_user?auth=${auth}`);
}

export function revertUser(auth: string) {
    return useApiFetch<UserInfo>(`/revert_user?auth=${auth}`);
}

export function setUserStatus(id: number, body: SetUserBody) {
    return useApiFetch(`/user_status/${id}`, {
        method: 'put',
        body,
    });
}