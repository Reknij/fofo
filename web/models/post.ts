import { ContentType } from "./util";

export enum PostStatus {
  Active,
  Archived,
  Banned,
}

export interface PostInfo {
  id: number;
  created_by_id: number;
  title: string;
  content?: string;
  content_type: ContentType;
  likes: number;
  dislikes: number;
  views: number;
  total_comment: number;
  total_comment_post: number;
  last_comment_at: number;
  last_comment_by_id: number;
  category_id: number;
  tags: string[];
  created_at: number;
  last_edit_at: number;
  last_edit_by_id: number;
  status: PostStatus;
  cover_url?: string;
  top_index?: number;
}

export interface PostToCreate {
  title: string;
  content: string;
  content_type: ContentType;
  category_id: number;
  tags: string[];
  cover_url?: string;
  top_index?: number;
}

export interface PostToUpdate {
  title: string;
  content: string;
  content_type: ContentType;
  tags: string[];
  cover_url?: string;
  top_index?: number;
}

export interface GetPostQuery {
  full?: boolean;
}

export interface GetPostsQuery {
  distinct?: boolean;
  created_by_id?: number;
  category_id?: number;
  sort: PostAlgorithmOrder;
  time_num?: number;
  time?: string;
  index: number;
  limit: number;
  extended: boolean;
  top_order_enable?: boolean;
}

export interface GetPostCountQuery {
  distinct?: boolean;
  sort: PostAlgorithmOrder;
  category_id?: number;
  created_by_id?: number;
  time_num?: number;
  time?: string;
}

export enum PostAlgorithmOrder {
  Hot,
  Views,
  Likes,
  Newest,
}

export interface SetPostBody {
  status: PostStatus;
}
