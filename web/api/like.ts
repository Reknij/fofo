import type { GetLikeStatusQuery, LikeActionBody, LikeStatus } from "~/models/like";
import { useApiFetch } from "./customFetch";

export function getLikeStatus(query: GetLikeStatusQuery) {
    return useApiFetch<LikeStatus>(`/like_status`, {
        query,
    });
}

export function likeAction(id: number, body: LikeActionBody) {
    return useApiFetch<LikeStatus>(`/like_action/${id}`, {
        method: 'PUT',
        body,
    })
}