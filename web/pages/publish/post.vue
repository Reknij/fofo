<script setup lang="ts">
import { hasManagePermission } from '~/helper';
import { getCategory } from '~/api/category';
import { getPost } from '~/api/post';
import type { Category } from '~/models/category';
import type { PostInfo } from '~/models/post';
import { useCurrentUser } from '~/states/auth';

const router = useRouter();
const user = useCurrentUser();
const toast = useToast();

const query = router.currentRoute.value.query as any as {
    category_id?: string,
    edit_id?: string
};

const authorized = ref(query.edit_id == undefined);
const editMode = computed(() => query.edit_id !== undefined);
const post = ref<PostInfo>()
const category = ref<Category>();

if (query.edit_id) {
    const { data } = await getPost(Number.parseInt(query.edit_id), {
        full: true
    })
    if (data.value) {
        post.value = data.value;
        const { data: c } = await getCategory(post.value.category_id);
        if (c.value) {
            category.value = c.value;
            if (data.value.created_by_id != user.value?.id && !hasManagePermission(user.value, c.value)) {
                toast.add({
                    color: 'yellow',
                    description: "You are not the original poster and don't have permission to edit."
                })
                authorized.value = false;
            }
            authorized.value = true;
        }
    }
}

if (!editMode.value && query.category_id) {
    const { data } = await getCategory(Number.parseInt(query.category_id));
    if (data.value) category.value = data.value;
}

const links = [
    {
        label: 'Categories',
        to: '/categories',
    },
    {
        label: category.value?.title ?? "Unknown",
        to: category.value ? `/category/${category.value.id}` : '/categories'
    },
    {
        label: query.edit_id ? "Edit post" : "Create post",
    }
]
</script>

<template>
    <ClientOnly>
        <div class="space-y-2">
            <FofoBreadcrumb :links="links" />
            <UAlert title="Category not provided." v-if="!category" />
            <UAlert title="Please login to continue!" v-else-if="!user" />
            <UAlert title="No permission to continue.." v-else-if="!authorized" />
            <UAlert title="Not found the post.." v-else-if="editMode && !post" />
            <PostEditor v-else :category="category" :edit="post" />
        </div>
    </ClientOnly>
</template>