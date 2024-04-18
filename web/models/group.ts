import { ContentType } from "./util";

export enum GroupStatus {
    Active,
    OnlyComment,
    Observer,
}

export interface Group {
    id: number;
    title: string;
    description: string;
    description_content_type: ContentType
    status: GroupStatus;
}

export interface GroupToCreateUpdate {
    title: string;
    description: string;
    description_content_type: ContentType
    status: GroupStatus;
}

export interface GetGroupsQuery {
    index: number,
    limit: number,
    sort: GetGroupsSort,
    desc: boolean,
    extended: boolean,
}

export enum GetGroupsSort {
    Id,
    Title,
}

export interface SetGroupBody {
    status: GroupStatus,
}