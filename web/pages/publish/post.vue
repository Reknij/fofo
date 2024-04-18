<script setup lang="ts">
import { hasManagePermission } from '~/helper';
import { NCard, useMessage, useLoadingBar } from 'naive-ui';
import { getCategory } from '~/api/category';
import { getPost } from '~/api/post';
import type { Category } from '~/models/category';
import type { PostInfo } from '~/models/post';
import { useCurrentUser } from '~/states/auth';

const router = useRouter();
const loadingBar = useLoadingBar()
loadingBar.start()
const user = useCurrentUser();
const message = useMessage();

const query = router.currentRoute.value.query as any as {
    category_id?: string,
    edit_id?: string
};

const authorized = ref(query.edit_id == undefined);
const editMode = computed(() => query.edit_id !== undefined);
const post = ref<PostInfo>()
const category = ref<Category>();

if (query.edit_id) {
    const { data } = await getPost(Number.parseInt(query.edit_id))
    if (data.value) {
        post.value = data.value;
        const { data: c } = await getCategory(post.value.category_id);
        if (c.value) {
            category.value = c.value;
            if (data.value.created_by_id != user.value?.id && !hasManagePermission(user.value, c.value)) {
                message.error("You are not the original poster and don't have permission to edit.")
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

onMounted(() => loadingBar.finish());
</script>

<template>
    <ClientOnly>
        <n-card v-if="!category" size="small">Category not provided.</n-card>
        <n-card v-else-if="!user">Please login to continue!</n-card>
        <n-card v-else-if="!authorized">No permission to continue..</n-card>
        <n-card v-else-if="editMode && !post">Not found the post.</n-card>
        <PostEditor v-else :category="category" :edit="post"></PostEditor>
    </ClientOnly>
</template>