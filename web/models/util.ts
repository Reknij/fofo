import type { Category } from "./category";
import type { CommentInfo } from "./comment";
import type { Group } from "./group";
import type { LikeStatus } from "./like";
import type { PostInfo } from "./post";
import type { SafeUserInfo } from "./user";
import type { VerificationKey } from "./verification";

export interface VerificationTargetWrapper<T> {
  target: T;
  verification?: VerificationKey;
}

export interface ListSlice<T> {
  items: T[];
  total: number;
}

export interface GetDatasExtended<T> {
  data: ListSlice<T>;
  posts?: any;
  comments?: any;
  categories?: any;
  groups?: any;
  users?: any;
  posts_like_status: any;
  comments_like_status: any;
}

export enum ContentType {
  Markdown,
}

export interface ApiDetailError {
  msg: string;
  code: number;
}

export function getPostFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): PostInfo | undefined {
  if (data.posts && id in data.posts) {
    return data.posts[id] as PostInfo;
  }
  return undefined;
}

export function getCommentFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): CommentInfo | undefined {
  if (data.comments && id in data.comments) {
    return data.comments[id] as CommentInfo;
  }
  return undefined;
}

export function getUserFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): SafeUserInfo | undefined {
  if (data.users && id in data.users) {
    return data.users[id] as SafeUserInfo;
  }
  return undefined;
}

export function getCategoryFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): Category | undefined {
  if (data.categories && id in data.categories) {
    return data.categories[id] as Category;
  }
  return undefined;
}

export function getGroupFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): Group | undefined {
  if (data.groups && id in data.groups) {
    return data.groups[id] as Group;
  }
  return undefined;
}

export function getCommentLikeStatusFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): LikeStatus | undefined {
  if (data.comments_like_status && id in data.comments_like_status) {
    return data.comments_like_status[id] as LikeStatus;
  }
  return undefined;
}

export function getPostLikeStatusFromExtended<T>(
  data: GetDatasExtended<T>,
  id: number
): LikeStatus | undefined {
  if (data.posts_like_status && id in data.posts_like_status) {
    return data.posts_like_status[id] as LikeStatus;
  }
  return undefined;
}
