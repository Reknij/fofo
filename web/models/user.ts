export enum UserStatus {
    Active,
    Banned,
    OnlyComment,
    Observer,
}

export enum UserType {
    Guest,
    General,
    Administrator,
}

export interface UserInfo {
    id: number;
    email: string;
    username: string;
    alias: string;
    password: string;
    group_ids: number[];
    status: UserStatus;
    user_type: UserType;
    avatar_url?: string;
    signature: string,
    created_at: number,
    total_post: number,
    total_comment: number,
}

export interface SafeUserInfo {
    id: number;
    username: string;
    alias: string;
    group_ids: number[];
    status: UserStatus;
    user_type: UserType;
    avatar_url?: string;
    signature: string,
    created_at: number,
    total_post: number,
    total_comment: number,
}

export interface UserToCreate {
    email: string;
    username: string;
    password: string;
    alias: string;
}

export interface UserToUpdate {
    email: string;
    username: string;
    password: string;
    alias: string;
    signature: string,
    avatar_url?: string;
}

export interface GetUsersQuery {
    index: number,
    limit: number,
    sort: GetUsersSort,
    desc: boolean,
    extended: boolean,
}

export enum GetUsersSort {
    Id,
    Username,
    Alias,
    UserType,
}

export interface ToLoginUser {
    username: string,
    password: string,
}

export interface AuthAndUser {
    auth: string,
    user: UserInfo,
}

export enum UserTag {
    Null = 0,
    OP = 2,
    Moderator = 4,
}

export interface SetUserBody {
    status: UserStatus,
}