import type { PostInfo, PostAlgorithmOrder, GetPostsQuery, GetPostCountQuery, PostToUpdate, PostToCreate, SetPostBody, GetPostQuery } from "~/models/post";
import type { GetDatasExtended, VerificationTargetWrapper } from "~/models/util";
import { useApiFetch } from "./customFetch";

export function createPost(body: VerificationTargetWrapper<PostToCreate>) {
    return useApiFetch<PostInfo>(`/post`, {
        method: 'post',
        body,
    });

}

export function updatePost(id: number, body: VerificationTargetWrapper<PostToUpdate>) {
    return useApiFetch<PostInfo>(`/post/${id}`, {
        method: 'put',
        body,
    });
}

export function deletePost(id: number) {
    return useApiFetch(`/post/${id}`, {
        method: 'delete',
    })
}

export function getPostsNoContent(query: GetPostsQuery) {
    return useApiFetch<GetDatasExtended<PostInfo>>(`/posts`, {
        query,
    });
}

export function getPost(id: number, query: GetPostQuery) {
    return useApiFetch<PostInfo>(`/post/${id}`, {
        query,
    });
}

export function setPostStatus(id: number, body: SetPostBody) {
    return useApiFetch(`/post_status/${id}`, {
        method: 'put',
        body,
    });
}