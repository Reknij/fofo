import { ContentType } from "./util";

export interface CommentInfo {
  id: number;
  parent_id: number;
  created_at: number;
  last_comment_at: number;
  created_by_id: number;
  last_comment_by_id: number;
  last_edit_at: number;
  last_edit_by_id: number;
  content: string;
  content_type: ContentType;
  category_id: number;
  post_id: number;
  reply_user_id: number;
  reply_comment_id: number;
  likes: number;
  dislikes: number;
  status: CommentStatus;
  total_comment: number;
  top_index?: number;
}

export enum CommentStatus {
  Active,
  Banned,
}

export interface CommentToCreate {
  content: string;
  content_type: ContentType;
  post_id: number;
  reply_comment_id: number;
  top_index?: number;
}

export interface CommentToUpdate {
  content: string;
  content_type: ContentType;
  top_index?: number;
}

export interface GetCommentsQuery {
  post_id: number;
  parent_id: number;
  index: number;
  limit: number;
  sort: GetCommentsSort;
  desc: boolean;
  extended: boolean;
  top_order_enable?: boolean;
}

export enum GetCommentsSort {
  Id,
  Likes,
  Dislikes,
  TotalPost,
}

export interface SetCommentBody {
  status: CommentStatus;
}
