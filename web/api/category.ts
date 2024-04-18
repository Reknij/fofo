import type { Category, CategoryToCreate, CategoryToUpdate, GetCategoriesQuery, SetCategoryBody } from "~/models/category";
import type { GetDatasExtended, VerificationTargetWrapper } from "~/models/util";
import { useApiFetch } from "./customFetch";

export function getCategories(query: GetCategoriesQuery) {
    return useApiFetch<GetDatasExtended<Category>>(`/categories`, {
        query,
    });
}

export function getCategory(id: number) {
    return useApiFetch<Category>(`/category/${id}`);
}

export function createCategory(body: VerificationTargetWrapper<CategoryToCreate>) {
    return useApiFetch<Category>('/category', {
        method: 'post',
        body,
    })
}

export function updateCategory(category_id: number, body: VerificationTargetWrapper<CategoryToUpdate>) {
    return useApiFetch<Category>(`/category/${category_id}`, {
        method: 'put',
        body,
    })
}

export function setCategoryStatus(id: number, body: SetCategoryBody) {
    return useApiFetch<Category>(`/category_status/${id}`, {
        method: 'put',
        body,
    });
}