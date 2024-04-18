import { UserType } from "./user";
import { ContentType } from "./util";

export interface Category {
    id: number;
    title: string;
    description?: string;
    description_content_type: ContentType
    status: CategoryStatus;
    read_level: UserType;
    write_level: UserType;
    comment_level: UserType;
    moderator_ids: number[];
    group_ids: number[];
    total_post: number,
}

export interface CategoryToCreate {
    title: string;
    description: string;
    description_content_type: ContentType;
    status: CategoryStatus;
    read_level: UserType;
    write_level: UserType;
    comment_level: UserType;
    moderator_ids: number[];
    group_ids: number[];
}

export interface CategoryToUpdate {
    title: string;
    description: string;
    description_content_type: ContentType;
    status: CategoryStatus;
    read_level: UserType;
    write_level: UserType;
    comment_level: UserType;
    moderator_ids: number[];
    group_ids: number[];
    total_post: number;
}

export enum CategoryStatus {
    Active,
    Archived,
    Stopped,
}

export interface GetCategoriesQuery {
    index: number,
    limit: number,
    sort: GetCategoriesSort,
    desc: boolean,
    extended: boolean,
}

export enum GetCategoriesSort {
    Id,
    Title,
    TotalPost,
}

export interface SetCategoryBody {
    status: CategoryStatus,
}