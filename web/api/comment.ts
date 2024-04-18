import type { CommentInfo, CommentToCreate, CommentToUpdate, GetCommentsQuery, SetCommentBody } from "~/models/comment";
import type { GetDatasExtended, VerificationTargetWrapper } from "~/models/util";
import { useApiFetch } from "./customFetch";

export function createComment(body: VerificationTargetWrapper<CommentToCreate>) {
    return useApiFetch<CommentInfo>(`/comment`, {
        method: 'POST',
        body,
    });
}

export function updateComment(id: number, body: VerificationTargetWrapper<CommentToUpdate>) {
    return useApiFetch<CommentInfo>(`/comment/${id}`, {
        method: 'PUT',
        body,
    })
}

export function deleteComment(id: number) {
    return useApiFetch(`/comment/${id}`, {
        method: 'DELETE',
    })
}

export function getComment(id: number) {
    return useApiFetch<CommentInfo>(`/comment/${id}`)
}

export function getComments(query: GetCommentsQuery) {
    return useApiFetch<GetDatasExtended<CommentInfo>>(`/comments`, {
        query: query,
    })
}

export function setCommentStatus(id: number, body: SetCommentBody) {
    return useApiFetch(`/comment_status/${id}`, {
        method: 'put',
        body,
    });
}